-- Gists (PLACEHOLDER - implement in v2)

CREATE TYPE gist_visibility AS ENUM ('public', 'secret');

CREATE TABLE gists (
                       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                       description     TEXT,
                       visibility      gist_visibility NOT NULL DEFAULT 'public',
                       fork_of_id      UUID REFERENCES gists(id) ON DELETE SET NULL,
                       comment_count   INT NOT NULL DEFAULT 0,
                       star_count      INT NOT NULL DEFAULT 0,
                       created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE gist_files (
                            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            gist_id         UUID NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
                            filename        VARCHAR(255) NOT NULL,
                            language        VARCHAR(100),
                            content         TEXT NOT NULL,
                            size_bytes      INT NOT NULL DEFAULT 0,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE gist_revisions (
                                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                gist_id         UUID NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
                                version         VARCHAR(40) NOT NULL,
                                change_status   JSONB NOT NULL DEFAULT '{}',
                                created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE gist_stars (
                            user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                            gist_id         UUID NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            PRIMARY KEY (user_id, gist_id)
);

CREATE TABLE gist_comments (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               gist_id         UUID NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
                               author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                               body            TEXT NOT NULL,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                               updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);