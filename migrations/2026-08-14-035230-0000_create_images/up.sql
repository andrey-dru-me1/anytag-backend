CREATE TABLE image_sources (
    file_sha256_hash CHAR(64) PRIMARY KEY,
    s3_path VARCHAR(512) NOT NULL,
    extension VARCHAR(15) NOT NULL,
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
    file_sha256_hash CHAR(64) NOT NULL REFERENCES image_sources(file_sha256_hash) ON DELETE RESTRICT,
    original_file_name VARCHAR(255) NOT NULL,
    created_by INT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
