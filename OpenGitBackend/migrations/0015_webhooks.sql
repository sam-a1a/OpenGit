-- Webhooks

CREATE TABLE webhooks (
                          id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          repo_id         UUID REFERENCES repositories(id) ON DELETE CASCADE,
                          org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
                          creator_id      UUID REFERENCES users(id) ON DELETE SET NULL,
                          url             TEXT NOT NULL,
                          content_type    VARCHAR(20) NOT NULL DEFAULT 'json',
                          secret_hash     TEXT,
                          events          TEXT[] NOT NULL DEFAULT '{"push"}',
                          is_active       BOOLEAN NOT NULL DEFAULT true,
                          insecure_ssl    BOOLEAN NOT NULL DEFAULT false,
                          created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                          updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE webhook_deliveries (
                                    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                    webhook_id      UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
                                    event           VARCHAR(50) NOT NULL,
                                    request_headers JSONB NOT NULL DEFAULT '{}',
                                    request_body    JSONB NOT NULL DEFAULT '{}',
                                    response_status INT,
                                    response_headers JSONB,
                                    response_body   TEXT,
                                    duration_ms     INT,
                                    redelivery      BOOLEAN NOT NULL DEFAULT false,
                                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_webhooks_repo ON webhooks(repo_id);
CREATE INDEX idx_webhooks_org ON webhooks(org_id);
CREATE INDEX idx_webhook_deliveries_webhook ON webhook_deliveries(webhook_id);