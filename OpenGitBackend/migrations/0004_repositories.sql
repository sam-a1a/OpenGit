-- Repositories

CREATE TYPE repo_visibility AS ENUM ('public', 'private', 'internal');
CREATE TYPE merge_strategy AS ENUM ('merge', 'squash', 'rebase');
CREATE TYPE collaborator_permission AS ENUM ('read', 'triage', 'write', 'maintain', 'admin');

CREATE TABLE repositories (
                              id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                              owner_id                    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                              org_id                      UUID REFERENCES organizations(id) ON DELETE CASCADE,
                              name                        VARCHAR(100) NOT NULL,
                              description                 TEXT,
                              visibility                  repo_visibility NOT NULL DEFAULT 'public',
                              default_branch              VARCHAR(255) NOT NULL DEFAULT 'main',
                              is_fork                     BOOLEAN NOT NULL DEFAULT false,
                              forked_from_id              UUID REFERENCES repositories(id) ON DELETE SET NULL,
                              is_template                 BOOLEAN NOT NULL DEFAULT false,
                              template_from_id            UUID REFERENCES repositories(id) ON DELETE SET NULL,
                              is_archived                 BOOLEAN NOT NULL DEFAULT false,
                              is_disabled                 BOOLEAN NOT NULL DEFAULT false,
                              is_empty                    BOOLEAN NOT NULL DEFAULT true,

    -- features
                              has_issues                  BOOLEAN NOT NULL DEFAULT true,
                              has_wiki                    BOOLEAN NOT NULL DEFAULT true,
                              has_projects                BOOLEAN NOT NULL DEFAULT true,
                              has_discussions             BOOLEAN NOT NULL DEFAULT false,
                              has_packages                BOOLEAN NOT NULL DEFAULT false,
                              has_pages                   BOOLEAN NOT NULL DEFAULT false,

    -- merge settings
                              allow_merge_commit          BOOLEAN NOT NULL DEFAULT true,
                              allow_squash_merge          BOOLEAN NOT NULL DEFAULT true,
                              allow_rebase_merge          BOOLEAN NOT NULL DEFAULT true,
                              delete_branch_on_merge      BOOLEAN NOT NULL DEFAULT false,

    -- stats (denormalized for perf)
                              star_count                  INT NOT NULL DEFAULT 0,
                              fork_count                  INT NOT NULL DEFAULT 0,
                              watcher_count               INT NOT NULL DEFAULT 0,
                              open_issue_count            INT NOT NULL DEFAULT 0,

    -- storage path
                              git_path                    TEXT NOT NULL,

                              created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
                              updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
                              pushed_at                   TIMESTAMPTZ,

                              UNIQUE(owner_id, name)
);

CREATE TABLE repo_topics (
                             repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                             topic           VARCHAR(50) NOT NULL,
                             PRIMARY KEY (repo_id, topic)
);

CREATE TABLE repo_stars (
                            user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                            repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            PRIMARY KEY (user_id, repo_id)
);

CREATE TABLE repo_watches (
                              user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                              repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                              level           VARCHAR(20) NOT NULL DEFAULT 'watching',
                              created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                              PRIMARY KEY (user_id, repo_id)
);

CREATE TABLE repo_collaborators (
                                    repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                    permission      collaborator_permission NOT NULL DEFAULT 'read',
                                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                    PRIMARY KEY (repo_id, user_id)
);

CREATE TABLE repo_deploy_keys (
                                  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                  repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                  title           VARCHAR(255) NOT NULL,
                                  key_data        TEXT NOT NULL,
                                  fingerprint     VARCHAR(255) NOT NULL,
                                  read_only       BOOLEAN NOT NULL DEFAULT true,
                                  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE repo_branch_protections (
                                         id                              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                         repo_id                         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                         pattern                         VARCHAR(255) NOT NULL,
                                         require_pull_request            BOOLEAN NOT NULL DEFAULT false,
                                         required_approving_review_count INT NOT NULL DEFAULT 0,
                                         dismiss_stale_reviews           BOOLEAN NOT NULL DEFAULT false,
                                         require_code_owner_reviews      BOOLEAN NOT NULL DEFAULT false,
                                         require_status_checks           BOOLEAN NOT NULL DEFAULT false,
                                         required_status_checks          TEXT[] NOT NULL DEFAULT '{}',
                                         require_up_to_date_branch       BOOLEAN NOT NULL DEFAULT false,
                                         restrict_pushes                 BOOLEAN NOT NULL DEFAULT false,
                                         allow_force_pushes              BOOLEAN NOT NULL DEFAULT false,
                                         allow_deletions                 BOOLEAN NOT NULL DEFAULT false,
                                         created_at                      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                         updated_at                      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_repos_owner ON repositories(owner_id);
CREATE INDEX idx_repos_org ON repositories(org_id);
CREATE INDEX idx_repos_visibility ON repositories(visibility);
CREATE INDEX idx_repo_stars_repo ON repo_stars(repo_id);
CREATE INDEX idx_repo_stars_user ON repo_stars(user_id);