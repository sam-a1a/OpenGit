-- Wiki

CREATE TABLE wiki_pages (
                            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                            author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                            title           VARCHAR(255) NOT NULL,
                            slug            VARCHAR(255) NOT NULL,
                            content         TEXT NOT NULL,
                            is_sidebar      BOOLEAN NOT NULL DEFAULT false,
                            is_footer       BOOLEAN NOT NULL DEFAULT false,
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            UNIQUE(repo_id, slug)
);

CREATE TABLE wiki_revisions (
                                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                page_id         UUID NOT NULL REFERENCES wiki_pages(id) ON DELETE CASCADE,
                                author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
                                content         TEXT NOT NULL,
                                message         VARCHAR(255),
                                created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_wiki_pages_repo ON wiki_pages(repo_id);
CREATE INDEX idx_wiki_revisions_page ON wiki_revisions(page_id);