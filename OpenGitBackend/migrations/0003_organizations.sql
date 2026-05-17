-- Organizations

CREATE TYPE org_member_role AS ENUM ('owner', 'member', 'billing_manager');
CREATE TYPE team_permission AS ENUM ('pull', 'triage', 'push', 'maintain', 'admin');
CREATE TYPE org_visibility AS ENUM ('public', 'private');

CREATE TABLE organizations (
                               id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               name                VARCHAR(39) NOT NULL UNIQUE,
                               display_name        VARCHAR(100),
                               description         TEXT,
                               avatar_url          TEXT,
                               website             TEXT,
                               location            TEXT,
                               email               VARCHAR(254),
                               twitter_username    VARCHAR(50),
                               visibility          org_visibility NOT NULL DEFAULT 'public',
                               verified            BOOLEAN NOT NULL DEFAULT false,
                               created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
                               updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE org_members (
                             org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                             user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                             role            org_member_role NOT NULL DEFAULT 'member',
                             joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
                             PRIMARY KEY (org_id, user_id)
);

CREATE TABLE org_invitations (
                                 id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                 org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                                 inviter_id      UUID NOT NULL REFERENCES users(id),
                                 invitee_email   VARCHAR(254) NOT NULL,
                                 role            org_member_role NOT NULL DEFAULT 'member',
                                 token           VARCHAR(255) NOT NULL UNIQUE,
                                 accepted        BOOLEAN,
                                 expires_at      TIMESTAMPTZ NOT NULL,
                                 created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE org_teams (
                           id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                           org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                           parent_id       UUID REFERENCES org_teams(id) ON DELETE SET NULL,
                           name            VARCHAR(255) NOT NULL,
                           slug            VARCHAR(255) NOT NULL,
                           description     TEXT,
                           privacy         VARCHAR(20) NOT NULL DEFAULT 'secret',
                           created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                           updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                           UNIQUE(org_id, slug)
);

CREATE TABLE team_members (
                              team_id         UUID NOT NULL REFERENCES org_teams(id) ON DELETE CASCADE,
                              user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                              role            VARCHAR(20) NOT NULL DEFAULT 'member',
                              created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                              PRIMARY KEY (team_id, user_id)
);

CREATE TABLE team_repo_permissions (
                                       team_id         UUID NOT NULL REFERENCES org_teams(id) ON DELETE CASCADE,
                                       repo_id         UUID NOT NULL,
                                       permission      team_permission NOT NULL DEFAULT 'pull',
                                       created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                       PRIMARY KEY (team_id, repo_id)
);

CREATE INDEX idx_org_members_user ON org_members(user_id);
CREATE INDEX idx_org_members_org ON org_members(org_id);
CREATE INDEX idx_org_teams_org ON org_teams(org_id);