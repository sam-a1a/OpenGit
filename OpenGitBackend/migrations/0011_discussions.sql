-- Discussions

CREATE TABLE discussion_categories (
                                       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                       repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                       name            VARCHAR(100) NOT NULL,
                                       description     TEXT,
                                       emoji           VARCHAR(10),
                                       is_answerable   BOOLEAN NOT NULL DEFAULT false,
                                       created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                       UNIQUE(repo_id, name)
);

CREATE TABLE discussions (
                             id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                             repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                             author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                             category_id     UUID NOT NULL REFERENCES discussion_categories(id) ON DELETE CASCADE,
                             number          INT NOT NULL,
                             title           VARCHAR(255) NOT NULL,
                             body            TEXT,
                             is_locked       BOOLEAN NOT NULL DEFAULT false,
                             is_answered     BOOLEAN NOT NULL DEFAULT false,
                             answer_id       UUID,
                             comment_count   INT NOT NULL DEFAULT 0,
                             created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                             updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                             UNIQUE(repo_id, number)
);

CREATE TABLE discussion_comments (
                                     id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                     discussion_id   UUID NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
                                     author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                                     reply_to_id     UUID REFERENCES discussion_comments(id) ON DELETE SET NULL,
                                     body            TEXT NOT NULL,
                                     is_answer       BOOLEAN NOT NULL DEFAULT false,
                                     created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                     updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE discussion_reactions (
                                      id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                      user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                      discussion_id   UUID REFERENCES discussions(id) ON DELETE CASCADE,
                                      comment_id      UUID REFERENCES discussion_comments(id) ON DELETE CASCADE,
                                      reaction        reaction_type NOT NULL,
                                      created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE discussions
    ADD CONSTRAINT fk_answer
        FOREIGN KEY (answer_id)
            REFERENCES discussion_comments(id)
            ON DELETE SET NULL;

CREATE INDEX idx_discussions_repo ON discussions(repo_id);
CREATE INDEX idx_discussion_comments_discussion ON discussion_comments(discussion_id);