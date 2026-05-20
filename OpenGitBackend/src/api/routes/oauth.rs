use crate::{
    api::middleware::auth::AuthUser,
    error::AppError,
    models::auth::{OauthApp, OauthAuthorization},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

// OAuth app management

#[derive(Debug, Deserialize)]
pub struct CreateAppInput {
    pub name:         String,
    pub description:  Option<String>,
    pub homepage_url: String,
    pub callback_url: String,
}

pub async fn create_app(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<CreateAppInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("App name is required".into()));
    }
    if !input.callback_url.starts_with("http") {
        return Err(AppError::BadRequest("callback_url must be a valid URL".into()));
    }

    let client_id     = format!("{}", Uuid::new_v4().simple());
    let client_secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

    let app: OauthApp = sqlx::query_as(
        "INSERT INTO oauth_apps
            (owner_id, name, description, homepage_url,
             callback_url, client_id, client_secret)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.homepage_url)
        .bind(&input.callback_url)
        .bind(&client_id)
        .bind(&client_secret)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(app)))
}

pub async fn list_apps(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let apps: Vec<OauthApp> = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE owner_id = $1 ORDER BY created_at DESC"
    )
        .bind(auth_user.user_id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(apps))
}

pub async fn get_app(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    axum::extract::Path(client_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let app: OauthApp = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE client_id = $1 AND owner_id = $2"
    )
        .bind(&client_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("App".into()))?;

    Ok(Json(app))
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppInput {
    pub name:         Option<String>,
    pub description:  Option<String>,
    pub homepage_url: Option<String>,
    pub callback_url: Option<String>,
}

pub async fn update_app(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    axum::extract::Path(client_id): axum::extract::Path<String>,
    Json(input):   Json<UpdateAppInput>,
) -> Result<impl IntoResponse, AppError> {
    let app: OauthApp = sqlx::query_as(
        "UPDATE oauth_apps SET
            name         = COALESCE($1, name),
            description  = COALESCE($2, description),
            homepage_url = COALESCE($3, homepage_url),
            callback_url = COALESCE($4, callback_url),
            updated_at   = now()
         WHERE client_id = $5 AND owner_id = $6
         RETURNING *"
    )
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.homepage_url)
        .bind(&input.callback_url)
        .bind(&client_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("App".into()))?;

    Ok(Json(app))
}

pub async fn delete_app(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    axum::extract::Path(client_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query(
        "DELETE FROM oauth_apps WHERE client_id = $1 AND owner_id = $2"
    )
        .bind(&client_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn reset_client_secret(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    axum::extract::Path(client_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let new_secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

    let app: OauthApp = sqlx::query_as(
        "UPDATE oauth_apps SET client_secret = $1, updated_at = now()
         WHERE client_id = $2 AND owner_id = $3
         RETURNING *"
    )
        .bind(&new_secret)
        .bind(&client_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("App".into()))?;

    Ok(Json(json!({
        "app":           app,
        "client_secret": new_secret,
        "warning":       "Store this secret immediately — it will not be shown again",
    })))
}

// Authorization flow

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub client_id:     String,
    pub redirect_uri:  String,
    pub scope:         Option<String>,
    pub state:         Option<String>,
    pub response_type: Option<String>,
}

pub async fn authorize(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(params): Query<AuthorizeQuery>,
) -> Result<impl IntoResponse, AppError> {
    if params.response_type.as_deref().unwrap_or("code") != "code" {
        return Err(AppError::BadRequest("Only response_type=code is supported".into()));
    }

    // look up app
    let app: OauthApp = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE client_id = $1"
    )
        .bind(&params.client_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("App".into()))?;

    // validate redirect_uri
    if app.callback_url != params.redirect_uri
        && !params.redirect_uri.starts_with(&app.callback_url)
    {
        return Err(AppError::BadRequest("redirect_uri does not match registered URL".into()));
    }

    let scopes: Vec<String> = params.scope
        .as_deref()
        .unwrap_or("repo user")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let valid_scopes = [
        "repo", "repo_read", "repo_write",
        "user", "user_read",
        "admin", "gist", "notifications",
        "delete_repo", "workflow", "packages",
        "write_org",
    ];

    for scope in &scopes {
        if !valid_scopes.contains(&scope.as_str()) {
            return Err(AppError::BadRequest(format!("Unknown scope: {}", scope)));
        }
    }

    // check if user already authorized this app with same scopes
    let existing: Option<OauthAuthorization> = sqlx::query_as(
        "SELECT * FROM oauth_authorizations WHERE user_id = $1 AND app_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(app.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?;

    // return app info and requested scopes for the UI to display
    Ok(Json(json!({
        "app": {
            "name":         app.name,
            "description":  app.description,
            "homepage_url": app.homepage_url,
            "logo_url":     app.logo_url,
        },
        "scopes":           scopes,
        "already_authorized": existing.is_some(),
        "redirect_uri":     params.redirect_uri,
        "state":            params.state,
        "client_id":        params.client_id,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ApproveInput {
    pub client_id:    String,
    pub redirect_uri: String,
    pub scopes:       Vec<String>,
    pub state:        Option<String>,
    pub approved:     bool,
}

pub async fn approve(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<ApproveInput>,
) -> Result<impl IntoResponse, AppError> {
    let app: OauthApp = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE client_id = $1"
    )
        .bind(&input.client_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("App".into()))?;

    if !input.approved {
        let redirect = format!(
            "{}?error=access_denied&error_description=User+denied+access{}",
            input.redirect_uri,
            input.state.as_ref()
                .map(|s| format!("&state={}", s))
                .unwrap_or_default()
        );
        return Ok(Json(json!({ "redirect_uri": redirect })));
    }

    // generate authorization code
    let code = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

    sqlx::query(
        "INSERT INTO oauth_authorization_codes
            (app_id, user_id, code, scopes, redirect_uri, expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + interval '10 minutes')"
    )
        .bind(app.id)
        .bind(auth_user.user_id)
        .bind(&code)
        .bind(&input.scopes)
        .bind(&input.redirect_uri)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    let redirect = format!(
        "{}?code={}{}",
        input.redirect_uri,
        code,
        input.state.as_ref()
            .map(|s| format!("&state={}", s))
            .unwrap_or_default()
    );

    Ok(Json(json!({ "redirect_uri": redirect })))
}

// Token exchange

#[derive(Debug, Deserialize)]
pub struct TokenInput {
    pub grant_type:    String,
    pub code:          Option<String>,
    pub redirect_uri:  Option<String>,
    pub client_id:     String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token:  String,
    pub token_type:    &'static str,
    pub scope:         String,
    pub expires_in:    Option<u64>,
}

pub async fn token_exchange(
    State(state): State<AppState>,
    Json(input):  Json<TokenInput>,
) -> Result<impl IntoResponse, AppError> {
    // look up app by client credentials
    let app: OauthApp = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE client_id = $1 AND client_secret = $2"
    )
        .bind(&input.client_id)
        .bind(&input.client_secret)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

    match input.grant_type.as_str() {
        "authorization_code" => {
            let code = input.code.as_deref()
                .ok_or(AppError::BadRequest("code is required".into()))?;

            // look up and validate code
            let auth_code = sqlx::query_as::<_, (Uuid, Vec<String>, String, bool, chrono::DateTime<chrono::Utc>)>(
                "SELECT user_id, scopes, redirect_uri, used, expires_at
                 FROM oauth_authorization_codes
                 WHERE code = $1 AND app_id = $2"
            )
                .bind(code)
                .bind(app.id)
                .fetch_optional(&state.db)
                .await
                .map_err(AppError::Database)?
                .ok_or(AppError::BadRequest("Invalid authorization code".into()))?;

            let (user_id, scopes, stored_redirect, used, expires_at) = auth_code;

            if used {
                return Err(AppError::BadRequest("Authorization code already used".into()));
            }
            if expires_at < chrono::Utc::now() {
                return Err(AppError::BadRequest("Authorization code expired".into()));
            }
            if let Some(ref uri) = input.redirect_uri {
                if uri != &stored_redirect {
                    return Err(AppError::BadRequest("redirect_uri mismatch".into()));
                }
            }

            // mark code as used
            sqlx::query(
                "UPDATE oauth_authorization_codes SET used = true WHERE code = $1"
            )
                .bind(code)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;

            // generate access token
            let access_token = format!(
                "ogat_{}",
                Uuid::new_v4().simple().to_string() + &Uuid::new_v4().simple().to_string()
            );
            let token_hash = blake3::hash(access_token.as_bytes()).to_hex().to_string();

            // store or update authorization
            sqlx::query(
                "INSERT INTO oauth_authorizations
                    (user_id, app_id, scopes, access_token)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (user_id, app_id)
                 DO UPDATE SET scopes = $3, access_token = $4"
            )
                .bind(user_id)
                .bind(app.id)
                .bind(&scopes)
                .bind(&token_hash)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;

            Ok(Json(TokenResponse {
                access_token,
                token_type: "bearer",
                scope:      scopes.join(" "),
                expires_in: None,
            }))
        }

        "client_credentials" => {
            // for machine-to-machine auth
            let access_token = format!(
                "ogct_{}",
                Uuid::new_v4().simple().to_string() + &Uuid::new_v4().simple().to_string()
            );

            Ok(Json(TokenResponse {
                access_token,
                token_type: "bearer",
                scope:      "".to_string(),
                expires_in: Some(3600),
            }))
        }

        _ => Err(AppError::BadRequest(
            "Unsupported grant_type. Use authorization_code or client_credentials".into()
        )),
    }
}

// ── Revoke token

#[derive(Debug, Deserialize)]
pub struct RevokeInput {
    pub token:         String,
    pub client_id:     String,
    pub client_secret: String,
}

pub async fn revoke_token(
    State(state): State<AppState>,
    Json(input):  Json<RevokeInput>,
) -> Result<impl IntoResponse, AppError> {
    let app: OauthApp = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE client_id = $1 AND client_secret = $2"
    )
        .bind(&input.client_id)
        .bind(&input.client_secret)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

    let token_hash = blake3::hash(input.token.as_bytes()).to_hex().to_string();

    sqlx::query(
        "DELETE FROM oauth_authorizations WHERE app_id = $1 AND access_token = $2"
    )
        .bind(app.id)
        .bind(&token_hash)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::OK)
}

// ── Userinfo endpoint

pub async fn userinfo(
    State(state): State<AppState>,
    headers:      axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();

    let auth_row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM oauth_authorizations WHERE access_token = $1"
    )
        .bind(&token_hash)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?;

    let (user_id,) = auth_row.ok_or(AppError::Unauthorized)?;

    let user = crate::db::queries::users::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let email: Option<(String,)> = sqlx::query_as(
        "SELECT email FROM user_emails WHERE user_id = $1 AND is_primary = true"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "sub":        user.id,
        "login":      user.username,
        "name":       user.display_name,
        "email":      email.map(|(e,)| e),
        "avatar_url": user.avatar_url,
        "bio":        user.bio,
        "location":   user.location,
        "website":    user.website,
    })))
}

// List authorizations (apps the user has authorized)

pub async fn list_authorizations(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let rows: Vec<(Uuid, Vec<String>, chrono::DateTime<chrono::Utc>, String, Option<String>, String, Option<String>)> = sqlx::query_as(
        "SELECT oa.id, oa.scopes, oa.created_at,
                app.name, app.description, app.client_id, app.logo_url
         FROM oauth_authorizations oa
         JOIN oauth_apps app ON app.id = oa.app_id
         WHERE oa.user_id = $1
         ORDER BY oa.created_at DESC"
    )
        .bind(auth_user.user_id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(
        |(id, scopes, created_at, name, description, client_id, logo_url)| {
            json!({
                "id":         id,
                "scopes":     scopes,
                "created_at": created_at,
                "app": {
                    "client_id":   client_id,
                    "name":        name,
                    "description": description,
                    "logo_url":    logo_url,
                }
            })
        }
    ).collect();

    Ok(Json(json!({ "authorizations": result })))
}

pub async fn revoke_authorization(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    axum::extract::Path(client_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let app: OauthApp = sqlx::query_as(
        "SELECT * FROM oauth_apps WHERE client_id = $1"
    )
        .bind(&client_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("App".into()))?;

    sqlx::query(
        "DELETE FROM oauth_authorizations WHERE user_id = $1 AND app_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(app.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}