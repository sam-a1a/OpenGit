-- Admin & Moderation

CREATE TYPE audit_action AS ENUM (
    'user_created', 'user_deleted', 'user_banned', 'user_promoted',
    'repo_created', 'repo_deleted', 'repo_transferred', 'repo_archived',
    'org_created', 'org_deleted',
    'member_added', 'member_removed',
    'settings_updated', 'webhook_created', 'webhook_deleted',
    'token_created', 'token_revoked',
    'login_success', 'login_failed', 'two_factor_enabled'
);

CREATE TABLE audit_log (
                           id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                           actor_id        UUID REFERENCES users(id) ON DELETE SET NULL,
                           actor_ip        INET,
                           action          audit_action NOT NULL,
                           target_type     VARCHAR(50),
                           target_id       UUID,
                           metadata        JSONB NOT NULL DEFAULT '{}',
                           created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE site_settings (
                               key             VARCHAR(100) PRIMARY KEY,
                               value           JSONB NOT NULL,
                               updated_by_id   UUID REFERENCES users(id) ON DELETE SET NULL,
                               updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE banned_users (
                              id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                              user_id         UUID REFERENCES users(id) ON DELETE CASCADE,
                              email           VARCHAR(254),
                              ip_address      INET,
                              reason          TEXT,
                              banned_by_id    UUID REFERENCES users(id) ON DELETE SET NULL,
                              expires_at      TIMESTAMPTZ,
                              created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE abuse_reports (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               reporter_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               target_type     VARCHAR(50) NOT NULL,
                               target_id       UUID NOT NULL,
                               reason          VARCHAR(100) NOT NULL,
                               description     TEXT,
                               resolved        BOOLEAN NOT NULL DEFAULT false,
                               resolved_by_id  UUID REFERENCES users(id) ON DELETE SET NULL,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_log_actor ON audit_log(actor_id);
CREATE INDEX idx_audit_log_created ON audit_log(created_at);
CREATE INDEX idx_audit_log_action ON audit_log(action);

INSERT INTO site_settings (key, value) VALUES
                                           ('registration_enabled', 'true'),
                                           ('require_email_verification', 'true'),
                                           ('max_repo_size_mb', '1024'),
                                           ('max_file_size_mb', '100'),
                                           ('allow_org_creation', 'true'),
                                           ('instance_name', '"OpenGit"'),
                                           ('instance_description', '"A self-hosted Git platform"');