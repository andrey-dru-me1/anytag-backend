-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
-- SPDX-License-Identifier: AGPL-3.0-only

CREATE TABLE user_user_subscriptions (
    follower_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followed_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (follower_id, followed_id),
    CHECK (follower_id != followed_id)
);

CREATE INDEX user_user_subscriptions_follower_id_idx ON user_user_subscriptions(follower_id);
CREATE INDEX user_user_subscriptions_followed_id_idx ON user_user_subscriptions(followed_id);
