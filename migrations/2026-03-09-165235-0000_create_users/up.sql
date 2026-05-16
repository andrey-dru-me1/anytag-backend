-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
-- SPDX-License-Identifier: AGPL-3.0-only

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
