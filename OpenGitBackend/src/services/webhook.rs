use crate::{error::AppError, models::webhook::Webhook, state::AppState};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use serde_json::Value;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_payload(secret: &str, body: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(body.as_bytes());
    let result = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(result))
}

#[derive(Debug, Clone)]
pub struct WebhookDispatch {
    pub webhook_id: Uuid,
    pub url:        String,
    pub secret:     Option<String>,
    pub event:      String,
    pub payload:    Value,
}

pub async fn dispatch(state: AppState, dispatch: WebhookDispatch) {
    tokio::spawn(async move {
        let _ = try_dispatch(&state, &dispatch, 0).await;
    });
}

async fn try_dispatch(
    state:    &AppState,
    d:        &WebhookDispatch,
    attempt:  u32,
) -> Result<(), AppError> {
    let body    = serde_json::to_string(&d.payload)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("X-OpenGit-Event", d.event.parse().unwrap_or_else(|_| "push".parse().unwrap()));
    headers.insert("X-OpenGit-Delivery", Uuid::new_v4().to_string().parse().unwrap());

    if let Some(secret) = &d.secret {
        let sig = sign_payload(secret, &body);
        headers.insert("X-Hub-Signature-256", sig.parse().unwrap());
    }

    let client   = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    let started = std::time::Instant::now();
    let result  = client
        .post(&d.url)
        .headers(headers.clone())
        .body(body.clone())
        .send()
        .await;

    let duration_ms = started.elapsed().as_millis() as i32;

    let (status_code, response_body) = match result {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let body   = resp.text().await.unwrap_or_default();
            (Some(status), Some(body))
        }
        Err(e) => (None, Some(format!("Request failed: {}", e))),
    };

    let success = status_code.map(|s| s >= 200 && s < 300).unwrap_or(false);

    // log delivery
    let _ = sqlx::query(
        "INSERT INTO webhook_deliveries
            (webhook_id, event, request_headers, request_body,
             response_status, response_body, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
        .bind(d.webhook_id)
        .bind(&d.event)
        .bind(serde_json::to_value(&headers.iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect::<std::collections::HashMap<_, _>>()
        ).unwrap_or_default())
        .bind(serde_json::from_str::<Value>(&body).unwrap_or(Value::String(body.clone())))
        .bind(status_code)
        .bind(&response_body)
        .bind(duration_ms)
        .execute(&state.db)
        .await;

    // retry up to 3 times with exponential backoff
    if !success && attempt < 3 {
        let delay = std::time::Duration::from_secs(2u64.pow(attempt) * 10);
        tokio::time::sleep(delay).await;
        Box::pin(try_dispatch(state, d, attempt + 1)).await?;
    }

    Ok(())
}

pub async fn dispatch_event(
    state:   &AppState,
    repo_id: Uuid,
    event:   &str,
    payload: Value,
) {
    let webhooks: Vec<Webhook> = match sqlx::query_as(
        "SELECT * FROM webhooks
         WHERE repo_id = $1 AND is_active = true AND $2 = ANY(events)"
    )
        .bind(repo_id)
        .bind(event)
        .fetch_all(&state.db)
        .await {
        Ok(w) => w,
        Err(_) => return,
    };

    for webhook in webhooks {
        dispatch(state.clone(), WebhookDispatch {
            webhook_id: webhook.id,
            url:        webhook.url.clone(),
            secret:     webhook.secret_hash.clone(),
            event:      event.to_string(),
            payload:    payload.clone(),
        }).await;
    }
}