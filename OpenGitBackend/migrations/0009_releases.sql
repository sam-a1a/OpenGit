-- Releases

CREATE TABLE releases (
                          id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                          author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                          tag_name        VARCHAR(255) NOT NULL,
                          target_sha      VARCHAR(40),
                          name            VARCHAR(255),
                          body            TEXT,
                          is_draft        BOOLEAN NOT NULL DEFAULT false,
                          is_prerelease   BOOLEAN NOT NULL DEFAULT false,
                          is_latest       BOOLEAN NOT NULL DEFAULT false,
                          published_at    TIMESTAMPTZ,
                          created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                          updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                          UNIQUE(repo_id, tag_name)
);

CREATE TABLE release_assets (
                                id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                release_id          UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
                                uploader_id         UUID REFERENCES users(id) ON DELETE SET NULL,
                                name                VARCHAR(255) NOT NULL,
                                label               VARCHAR(255),
                                content_type        VARCHAR(100) NOT NULL,
                                size_bytes          BIGINT NOT NULL DEFAULT 0,
                                download_count      INT NOT NULL DEFAULT 0,
                                storage_key         TEXT NOT NULL,
                                state               VARCHAR(20) NOT NULL DEFAULT 'uploaded',
                                created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
                                updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_releases_repo ON releases(repo_id);
CREATE INDEX idx_release_assets_release ON release_assets(release_id);