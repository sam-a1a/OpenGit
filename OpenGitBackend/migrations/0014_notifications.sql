-- Notifications

CREATE TYPE notification_reason AS ENUM (
    'assign', 'author', 'comment', 'ci_activity',
    'invitation', 'manual', 'mention', 'review_requested',
    'security_alert', 'state_change', 'subscribed', 'team_mention'
);

CREATE TABLE notifications (
                               id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               repo_id         UUID REFERENCES repositories(id) ON DELETE CASCADE,
                               subject_type    VARCHAR(50) NOT NULL,
                               subject_id      UUID,
                               subject_title   TEXT NOT NULL,
                               reason          notification_reason NOT NULL,
                               is_read         BOOLEAN NOT NULL DEFAULT false,
                               is_saved        BOOLEAN NOT NULL DEFAULT false,
                               last_read_at    TIMESTAMPTZ,
                               created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                               updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE notification_subscriptions (
                                            user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                            repo_id         UUID REFERENCES repositories(id) ON DELETE CASCADE,
                                            org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
                                            subscribed      BOOLEAN NOT NULL DEFAULT true,
                                            ignored         BOOLEAN NOT NULL DEFAULT false,
                                            created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                            PRIMARY KEY (user_id, COALESCE(repo_id, '00000000-0000-0000-0000-000000000000'::UUID),
                                                         COALESCE(org_id,  '00000000-0000-0000-0000-000000000000'::UUID))
);

CREATE INDEX idx_notifications_user ON notifications(user_id);
CREATE INDEX idx_notifications_unread ON notifications(user_id, is_read);