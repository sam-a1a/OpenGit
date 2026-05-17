-- Labels & Milestones (org-level labels)

CREATE TABLE org_labels (
                            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                            name            VARCHAR(50) NOT NULL,
                            color           CHAR(6) NOT NULL,
                            description     VARCHAR(100),
                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                            UNIQUE(org_id, name)
);