-- Security

CREATE TYPE advisory_severity AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE alert_state AS ENUM ('open', 'dismissed', 'fixed', 'auto_dismissed');

CREATE TABLE security_advisories (
                                     id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                     repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                     cve_id          VARCHAR(50),
                                     ghsa_id         VARCHAR(50),
                                     summary         TEXT NOT NULL,
                                     description     TEXT,
                                     severity        advisory_severity NOT NULL DEFAULT 'medium',
                                     cvss_score      DECIMAL(4,2),
                                     published_at    TIMESTAMPTZ,
                                     created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                     updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE secret_scanning_alerts (
                                        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                        repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                        number          INT NOT NULL,
                                        secret_type     VARCHAR(100) NOT NULL,
                                        secret          TEXT NOT NULL,
                                        state           alert_state NOT NULL DEFAULT 'open',
                                        resolved_by_id  UUID REFERENCES users(id) ON DELETE SET NULL,
                                        resolved_at     TIMESTAMPTZ,
                                        resolution      VARCHAR(50),
                                        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                        UNIQUE(repo_id, number)
);

CREATE TABLE code_scanning_alerts (
                                      id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                      repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                                      number          INT NOT NULL,
                                      rule_id         VARCHAR(255),
                                      rule_severity   advisory_severity,
                                      rule_description TEXT,
                                      state           alert_state NOT NULL DEFAULT 'open',
                                      path            TEXT,
                                      start_line      INT,
                                      end_line        INT,
                                      dismissed_by_id UUID REFERENCES users(id) ON DELETE SET NULL,
                                      dismissed_at    TIMESTAMPTZ,
                                      created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                      UNIQUE(repo_id, number)
);

CREATE INDEX idx_security_advisories_repo ON security_advisories(repo_id);
CREATE INDEX idx_secret_alerts_repo ON secret_scanning_alerts(repo_id);
CREATE INDEX idx_code_alerts_repo ON code_scanning_alerts(repo_id);