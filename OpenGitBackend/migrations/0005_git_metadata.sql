-- Git Metadata (indexed from git layer for search/display)

CREATE TABLE branches (
                          id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                          name            VARCHAR(255) NOT NULL,
                          head_sha        VARCHAR(40) NOT NULL,
                          is_default      BOOLEAN NOT NULL DEFAULT false,
                          is_protected    BOOLEAN NOT NULL DEFAULT false,
                          updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                          UNIQUE(repo_id, name)
);

CREATE TABLE tags (
                      id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                      repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                      name            VARCHAR(255) NOT NULL,
                      sha             VARCHAR(40) NOT NULL,
                      is_annotated    BOOLEAN NOT NULL DEFAULT false,
                      tagger_name     VARCHAR(255),
                      tagger_email    VARCHAR(254),
                      message         TEXT,
                      created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                      UNIQUE(repo_id, name)
);

CREATE TABLE commits (
                         id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                         repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                         sha             VARCHAR(40) NOT NULL,
                         message         TEXT NOT NULL,
                         author_name     VARCHAR(255) NOT NULL,
                         author_email    VARCHAR(254) NOT NULL,
                         authored_at     TIMESTAMPTZ NOT NULL,
                         committer_name  VARCHAR(255),
                         committer_email VARCHAR(254),
                         committed_at    TIMESTAMPTZ,
                         user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
                         verified        BOOLEAN NOT NULL DEFAULT false,
                         UNIQUE(repo_id, sha)
);

CREATE INDEX idx_branches_repo ON branches(repo_id);
CREATE INDEX idx_tags_repo ON tags(repo_id);
CREATE INDEX idx_commits_repo ON commits(repo_id);
CREATE INDEX idx_commits_sha ON commits(sha);
CREATE INDEX idx_commits_authored_at ON commits(authored_at);