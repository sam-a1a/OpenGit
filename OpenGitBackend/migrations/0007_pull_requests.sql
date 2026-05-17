-- Pull Requests

CREATE TYPE pr_state AS ENUM ('open', 'closed', 'merged');
CREATE TYPE pr_review_state AS ENUM ('pending', 'approved', 'changes_requested', 'commented', 'dismissed');
CREATE TYPE check_status AS ENUM ('queued', 'in_progress', 'completed');
CREATE TYPE check_conclusion AS ENUM ('success', 'failure', 'neutral', 'cancelled', 'skipped', 'timed_out', 'action_required');

CREATE TABLE pull_requests (
                               id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               repo_id             UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                               author_id           UUID REFERENCES users(id) ON DELETE SET NULL,
                               milestone_id        UUID REFERENCES milestones(id) ON DELETE SET NULL,
                               number              INT NOT NULL,
                               title               VARCHAR(255) NOT NULL,
                               body                TEXT,
                               state               pr_state NOT NULL DEFAULT 'open',
                               is_draft            BOOLEAN NOT NULL DEFAULT false,
                               locked              BOOLEAN NOT NULL DEFAULT false,
                               head_branch         VARCHAR(255) NOT NULL,
                               head_sha            VARCHAR(40),
                               base_branch         VARCHAR(255) NOT NULL,
                               base_sha            VARCHAR(40),
                               head_repo_id        UUID REFERENCES repositories(id) ON DELETE SET NULL,
                               merge_commit_sha    VARCHAR(40),
                               merged_at           TIMESTAMPTZ,
                               merged_by_id        UUID REFERENCES users(id) ON DELETE SET NULL,
                               closed_at           TIMESTAMPTZ,
                               comment_count       INT NOT NULL DEFAULT 0,
                               commit_count        INT NOT NULL DEFAULT 0,
                               additions           INT NOT NULL DEFAULT 0,
                               deletions           INT NOT NULL DEFAULT 0,
                               changed_files       INT NOT NULL DEFAULT 0,
                               created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
                               updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
                               UNIQUE(repo_id, number)
);

CREATE TABLE pr_reviews (
                            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            pr_id           UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
                            reviewer_id     UUID REFERENCES users(id) ON DELETE SET NULL,
                            state           pr_review_state NOT NULL DEFAULT 'pending',
                            body            TEXT,
                            commit_sha      VARCHAR(40),
                            submitted_at    TIMESTAMPTZ,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE pr_review_comments (
                                    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                    pr_id           UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
                                    review_id       UUID REFERENCES pr_reviews(id) ON DELETE CASCADE,
                                    author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                                    reply_to_id     UUID REFERENCES pr_review_comments(id) ON DELETE SET NULL,
                                    body            TEXT NOT NULL,
                                    path            TEXT,
                                    commit_sha      VARCHAR(40),
                                    line            INT,
                                    start_line      INT,
                                    side            VARCHAR(5) DEFAULT 'RIGHT',
                                    is_edited       BOOLEAN NOT NULL DEFAULT false,
                                    resolved        BOOLEAN NOT NULL DEFAULT false,
                                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE pr_requested_reviewers (
                                        pr_id           UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
                                        user_id         UUID REFERENCES users(id) ON DELETE CASCADE,
                                        team_id         UUID REFERENCES org_teams(id) ON DELETE CASCADE,
                                        created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE pr_status_checks (
                                  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                  repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                  sha             VARCHAR(40) NOT NULL,
                                  name            VARCHAR(255) NOT NULL,
                                  context         VARCHAR(255),
                                  status          check_status NOT NULL DEFAULT 'queued',
                                  conclusion      check_conclusion,
                                  target_url      TEXT,
                                  description     VARCHAR(255),
                                  started_at      TIMESTAMPTZ,
                                  completed_at    TIMESTAMPTZ,
                                  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE pr_assignees (
                              pr_id           UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
                              user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                              PRIMARY KEY (pr_id, user_id)
);

CREATE TABLE pr_labels (
                           pr_id           UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
                           label_id        UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
                           PRIMARY KEY (pr_id, label_id)
);

CREATE INDEX idx_prs_repo ON pull_requests(repo_id);
CREATE INDEX idx_prs_author ON pull_requests(author_id);
CREATE INDEX idx_prs_state ON pull_requests(state);
CREATE INDEX idx_pr_reviews_pr ON pr_reviews(pr_id);
CREATE INDEX idx_pr_review_comments_pr ON pr_review_comments(pr_id);
CREATE INDEX idx_pr_status_checks_sha ON pr_status_checks(sha);