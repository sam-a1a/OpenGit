-- Users & Identity

CREATE TYPE user_role AS ENUM ('user', 'admin', 'superadmin');
CREATE TYPE user_status_availability AS ENUM (
    'available', 'busy', 'away', 'do_not_disturb',
    'invisible', 'sick', 'on_vacation', 'in_a_meeting'
);

CREATE TABLE users (
                       id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       username                VARCHAR(39) NOT NULL UNIQUE,
                       display_name            VARCHAR(100),
                       bio                     TEXT,
                       avatar_url              TEXT,
                       website                 TEXT,
                       location                TEXT,
                       pronouns                VARCHAR(50),
                       company                 TEXT,
                       twitter_username        VARCHAR(50),
                       role                    user_role NOT NULL DEFAULT 'user',

    -- status
                       status_emoji            VARCHAR(10),
                       status_message          VARCHAR(255),
                       status_availability     user_status_availability NOT NULL DEFAULT 'available',
                       status_expires_at       TIMESTAMPTZ,

    -- settings
                       is_active               BOOLEAN NOT NULL DEFAULT true,
                       is_verified             BOOLEAN NOT NULL DEFAULT false,
                       two_factor_enabled      BOOLEAN NOT NULL DEFAULT false,
                       two_factor_secret       TEXT,
                       profile_private         BOOLEAN NOT NULL DEFAULT false,

                       created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_emails (
                             id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                             user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                             email           VARCHAR(254) NOT NULL UNIQUE,
                             is_primary      BOOLEAN NOT NULL DEFAULT false,
                             is_verified     BOOLEAN NOT NULL DEFAULT false,
                             verified_at     TIMESTAMPTZ,
                             created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_ssh_keys (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               title           VARCHAR(255) NOT NULL,
                               key_type        VARCHAR(20) NOT NULL,
                               key_data        TEXT NOT NULL,
                               fingerprint     VARCHAR(255) NOT NULL UNIQUE,
                               last_used_at    TIMESTAMPTZ,
                               expires_at      TIMESTAMPTZ,
                               read_only       BOOLEAN NOT NULL DEFAULT false,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_gpg_keys (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               key_id          VARCHAR(255) NOT NULL UNIQUE,
                               public_key      TEXT NOT NULL,
                               emails          TEXT[] NOT NULL DEFAULT '{}',
                               expires_at      TIMESTAMPTZ,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_oauth_connections (
                                        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                        user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                        provider        VARCHAR(50) NOT NULL,
                                        provider_user_id VARCHAR(255) NOT NULL,
                                        access_token    TEXT,
                                        refresh_token   TEXT,
                                        expires_at      TIMESTAMPTZ,
                                        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                        UNIQUE(provider, provider_user_id)
);

CREATE TABLE user_follows (
                              follower_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                              following_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                              created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                              PRIMARY KEY (follower_id, following_id)
);

CREATE TABLE user_blocks (
                             blocker_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                             blocked_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                             created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                             PRIMARY KEY (blocker_id, blocked_id)
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_user_emails_user_id ON user_emails(user_id);
CREATE INDEX idx_user_ssh_keys_user_id ON user_ssh_keys(user_id);
CREATE INDEX idx_user_follows_follower ON user_follows(follower_id);
CREATE INDEX idx_user_follows_following ON user_follows(following_id);