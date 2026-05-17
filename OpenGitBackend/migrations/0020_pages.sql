-- Pages (PLACEHOLDER - implement in v2)

CREATE TYPE pages_source AS ENUM ('branch', 'workflow');
CREATE TYPE pages_status AS ENUM ('building', 'built', 'errored', 'disabled');

CREATE TABLE pages (
                       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE UNIQUE,
                       source          pages_source NOT NULL DEFAULT 'branch',
                       branch          VARCHAR(255),
                       path            VARCHAR(10) NOT NULL DEFAULT '/',
                       custom_domain   VARCHAR(255),
                       https_enforced  BOOLEAN NOT NULL DEFAULT true,
                       status          pages_status NOT NULL DEFAULT 'disabled',
                       url             TEXT,
                       created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);