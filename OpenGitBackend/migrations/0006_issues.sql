-- Issues

CREATE TYPE issue_state AS ENUM ('open', 'closed');
CREATE TYPE issue_state_reason AS ENUM ('completed', 'not_planned', 'reopened');
CREATE TYPE reaction_type AS ENUM (
    'thumbs_up', 'thumbs_down', 'laugh',
    'hooray', 'confused', 'heart', 'rocket', 'eyes'
);

CREATE TABLE milestones (
                            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                            title           VARCHAR(255) NOT NULL,
                            description     TEXT,
                            state           issue_state NOT NULL DEFAULT 'open',
                            due_on          TIMESTAMPTZ,
                            closed_at       TIMESTAMPTZ,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE labels (
                        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                        name            VARCHAR(50) NOT NULL,
                        color           CHAR(6) NOT NULL,
                        description     VARCHAR(100),
                        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                        UNIQUE(repo_id, name)
);

CREATE TABLE issues (
                        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                        author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                        milestone_id    UUID REFERENCES milestones(id) ON DELETE SET NULL,
                        number          INT NOT NULL,
                        title           VARCHAR(255) NOT NULL,
                        body            TEXT,
                        state           issue_state NOT NULL DEFAULT 'open',
                        state_reason    issue_state_reason,
                        locked          BOOLEAN NOT NULL DEFAULT false,
                        lock_reason     VARCHAR(50),
                        is_pinned       BOOLEAN NOT NULL DEFAULT false,
                        comment_count   INT NOT NULL DEFAULT 0,
                        closed_at       TIMESTAMPTZ,
                        closed_by_id    UUID REFERENCES users(id) ON DELETE SET NULL,
                        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                        updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                        UNIQUE(repo_id, number)
);

CREATE TABLE issue_assignees (
                                 issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                                 user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                 PRIMARY KEY (issue_id, user_id)
);

CREATE TABLE issue_labels (
                              issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                              label_id        UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
                              PRIMARY KEY (issue_id, label_id)
);

CREATE TABLE issue_comments (
                                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                                author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                                body            TEXT NOT NULL,
                                is_edited       BOOLEAN NOT NULL DEFAULT false,
                                created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE issue_reactions (
                                 id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                 user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                 issue_id        UUID REFERENCES issues(id) ON DELETE CASCADE,
                                 comment_id      UUID REFERENCES issue_comments(id) ON DELETE CASCADE,
                                 reaction        reaction_type NOT NULL,
                                 created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                 UNIQUE(user_id, issue_id, reaction),
                                 UNIQUE(user_id, comment_id, reaction)
);

CREATE TABLE issue_subscriptions (
                                     user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                     issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
                                     subscribed      BOOLEAN NOT NULL DEFAULT true,
                                     created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                     PRIMARY KEY (user_id, issue_id)
);

CREATE INDEX idx_issues_repo ON issues(repo_id);
CREATE INDEX idx_issues_author ON issues(author_id);
CREATE INDEX idx_issues_state ON issues(state);
CREATE INDEX idx_issues_number ON issues(repo_id, number);
CREATE INDEX idx_issue_comments_issue ON issue_comments(issue_id);