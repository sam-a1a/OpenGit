-- Projects (Kanban)

CREATE TYPE project_visibility AS ENUM ('private', 'public');

CREATE TABLE projects (
                          id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                          repo_id         UUID REFERENCES repositories(id) ON DELETE CASCADE,
                          org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
                          name            VARCHAR(255) NOT NULL,
                          description     TEXT,
                          visibility      project_visibility NOT NULL DEFAULT 'private',
                          closed          BOOLEAN NOT NULL DEFAULT false,
                          created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                          updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE project_columns (
                                 id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                 project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                                 name            VARCHAR(255) NOT NULL,
                                 position        INT NOT NULL DEFAULT 0,
                                 created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                 updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE project_cards (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               column_id       UUID NOT NULL REFERENCES project_columns(id) ON DELETE CASCADE,
                               creator_id      UUID REFERENCES users(id) ON DELETE SET NULL,
                               note            TEXT,
                               issue_id        UUID REFERENCES issues(id) ON DELETE CASCADE,
                               pr_id           UUID REFERENCES pull_requests(id) ON DELETE CASCADE,
                               position        INT NOT NULL DEFAULT 0,
                               archived        BOOLEAN NOT NULL DEFAULT false,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                               updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_projects_repo ON projects(repo_id);
CREATE INDEX idx_projects_org ON projects(org_id);
CREATE INDEX idx_project_columns_project ON project_columns(project_id);
CREATE INDEX idx_project_cards_column ON project_cards(column_id);