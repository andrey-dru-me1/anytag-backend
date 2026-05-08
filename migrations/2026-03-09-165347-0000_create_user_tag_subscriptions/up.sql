-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
-- SPDX-License-Identifier: AGPL-3.0-only

CREATE TABLE user_tag_subscriptions (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, tag_id)
);

CREATE INDEX user_tag_subscriptions_user_id_idx ON user_tag_subscriptions(user_id);
CREATE INDEX user_tag_subscriptions_tag_id_idx ON user_tag_subscriptions(tag_id);
