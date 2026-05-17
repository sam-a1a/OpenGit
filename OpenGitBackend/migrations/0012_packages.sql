-- Packages (PLACEHOLDER - implement in v2)

CREATE TABLE packages (
                          id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          repo_id         UUID REFERENCES repositories(id) ON DELETE CASCADE,
                          org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
                          owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                          name            VARCHAR(255) NOT NULL,
                          package_type    VARCHAR(50) NOT NULL, -- npm, docker, cargo, maven, etc.
                          visibility      repo_visibility NOT NULL DEFAULT 'public',
                          download_count  BIGINT NOT NULL DEFAULT 0,
                          created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                          updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE package_versions (
                                  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                  package_id      UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
                                  version         VARCHAR(255) NOT NULL,
                                  description     TEXT,
                                  metadata        JSONB NOT NULL DEFAULT '{}',
                                  download_count  BIGINT NOT NULL DEFAULT 0,
                                  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                  UNIQUE(package_id, version)
);

CREATE TABLE package_files (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               version_id      UUID NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE,
                               name            VARCHAR(255) NOT NULL,
                               size_bytes      BIGINT NOT NULL DEFAULT 0,
                               storage_key     TEXT NOT NULL,
                               sha256          VARCHAR(64),
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);