-- CI / Actions

CREATE TYPE workflow_run_status AS ENUM (
    'queued', 'in_progress', 'completed',
    'waiting', 'requested', 'pending'
);
CREATE TYPE workflow_conclusion AS ENUM (
    'success', 'failure', 'neutral', 'cancelled',
    'skipped', 'timed_out', 'action_required', 'startup_failure'
);
CREATE TYPE runner_status AS ENUM ('online', 'offline', 'busy', 'disabled');

CREATE TABLE workflows (
                           id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                           repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                           name            VARCHAR(255) NOT NULL,
                           path            TEXT NOT NULL,
                           state           VARCHAR(50) NOT NULL DEFAULT 'active',
                           created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                           updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE workflow_runs (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               workflow_id     UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
                               repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                               actor_id        UUID REFERENCES users(id) ON DELETE SET NULL,
                               run_number      INT NOT NULL,
                               event           VARCHAR(50) NOT NULL,
                               status          workflow_run_status NOT NULL DEFAULT 'queued',
                               conclusion      workflow_conclusion,
                               head_sha        VARCHAR(40) NOT NULL,
                               head_branch     VARCHAR(255),
                               run_attempt     INT NOT NULL DEFAULT 1,
                               started_at      TIMESTAMPTZ,
                               completed_at    TIMESTAMPTZ,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE workflow_jobs (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               run_id          UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
                               runner_id       UUID,
                               name            VARCHAR(255) NOT NULL,
                               status          workflow_run_status NOT NULL DEFAULT 'queued',
                               conclusion      workflow_conclusion,
                               head_sha        VARCHAR(40) NOT NULL,
                               labels          TEXT[] NOT NULL DEFAULT '{}',
                               started_at      TIMESTAMPTZ,
                               completed_at    TIMESTAMPTZ,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE workflow_steps (
                                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                job_id          UUID NOT NULL REFERENCES workflow_jobs(id) ON DELETE CASCADE,
                                name            VARCHAR(255) NOT NULL,
                                status          workflow_run_status NOT NULL DEFAULT 'queued',
                                conclusion      workflow_conclusion,
                                number          INT NOT NULL,
                                started_at      TIMESTAMPTZ,
                                completed_at    TIMESTAMPTZ
);

CREATE TABLE artifacts (
                           id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                           run_id          UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
                           repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                           name            VARCHAR(255) NOT NULL,
                           size_bytes      BIGINT NOT NULL DEFAULT 0,
                           storage_key     TEXT NOT NULL,
                           expires_at      TIMESTAMPTZ,
                           created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE runner_groups (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
                               repo_id         UUID REFERENCES repositories(id) ON DELETE CASCADE,
                               name            VARCHAR(255) NOT NULL,
                               visibility      VARCHAR(20) NOT NULL DEFAULT 'all',
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE runners (
                         id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                         group_id        UUID REFERENCES runner_groups(id) ON DELETE SET NULL,
                         name            VARCHAR(255) NOT NULL,
                         os              VARCHAR(50),
                         architecture    VARCHAR(50),
                         labels          TEXT[] NOT NULL DEFAULT '{}',
                         status          runner_status NOT NULL DEFAULT 'offline',
                         token_hash      TEXT NOT NULL,
                         last_seen_at    TIMESTAMPTZ,
                         created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workflow_runs_repo ON workflow_runs(repo_id);
CREATE INDEX idx_workflow_runs_workflow ON workflow_runs(workflow_id);
CREATE INDEX idx_workflow_jobs_run ON workflow_jobs(run_id);
CREATE INDEX idx_workflow_steps_job ON workflow_steps(job_id);
CREATE INDEX idx_artifacts_run ON artifacts(run_id);