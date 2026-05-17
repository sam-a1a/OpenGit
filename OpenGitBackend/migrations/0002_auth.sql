-- Auth & Tokens

CREATE TYPE token_scope AS ENUM (
    'repo', 'repo_read', 'repo_write',
    'user', 'user_read',
    'admin', 'gist',
    'notifications', 'delete_repo',
    'workflow', 'packages', 'write_org'
);

CREATE TABLE sessions (
                          id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                          ip_address      INET,
                          user_agent      TEXT,
                          last_active_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                          expires_at      TIMESTAMPTZ NOT NULL,
                          created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE refresh_tokens (
                                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                token_hash      VARCHAR(255) NOT NULL UNIQUE,
                                family_id       UUID NOT NULL,
                                session_id      UUID REFERENCES sessions(id) ON DELETE CASCADE,
                                used            BOOLEAN NOT NULL DEFAULT false,
                                ip_address      INET,
                                expires_at      TIMESTAMPTZ NOT NULL,
                                created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE personal_access_tokens (
                                        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                        user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                        name            VARCHAR(255) NOT NULL,
                                        token_hash      VARCHAR(255) NOT NULL UNIQUE,
                                        scopes          token_scope[] NOT NULL DEFAULT '{}',
                                        last_used_at    TIMESTAMPTZ,
                                        expires_at      TIMESTAMPTZ,
                                        created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE oauth_apps (
                            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                            name            VARCHAR(255) NOT NULL,
                            description     TEXT,
                            homepage_url    TEXT NOT NULL,
                            callback_url    TEXT NOT NULL,
                            client_id       VARCHAR(255) NOT NULL UNIQUE,
                            client_secret   VARCHAR(255) NOT NULL,
                            logo_url        TEXT,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE oauth_authorizations (
                                      id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                      user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                      app_id          UUID NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
                                      scopes          token_scope[] NOT NULL DEFAULT '{}',
                                      access_token    TEXT NOT NULL UNIQUE,
                                      created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                      UNIQUE(user_id, app_id)
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX idx_pat_user_id ON personal_access_tokens(user_id);