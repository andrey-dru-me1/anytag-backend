-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
-- SPDX-License-Identifier: AGPL-3.0-only

CREATE TABLE image_sources (
    id SERIAL PRIMARY KEY,
    s3_key VARCHAR(512) UNIQUE NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    bucket_name VARCHAR(63) NOT NULL,
    width INT NOT NULL,
    height INT NOT NULL,

    CONSTRAINT check_file_size_positive CHECK (file_size > 0),
    CONSTRAINT check_dimensions_positive CHECK (width > 0 AND height > 0)
);

CREATE TABLE user_images (
    id SERIAL PRIMARY KEY,
    image_source_id int NOT NULL REFERENCES image_sources(id) ON DELETE RESTRICT,
    original_file_name VARCHAR(255) NOT NULL,
    created_by INT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
