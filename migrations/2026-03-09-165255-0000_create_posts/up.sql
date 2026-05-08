-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
-- SPDX-License-Identifier: AGPL-3.0-only

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX posts_user_id_idx ON posts(user_id);
