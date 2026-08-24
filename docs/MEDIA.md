<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Media & S3 Storage

This document describes the image upload/retrieval subsystem and the local S3-compatible object storage (SeaweedFS) used to persist media files.

## Overview

Users upload image files which are stored in an S3-compatible bucket. The application records the object location and image metadata in PostgreSQL and streams the object back when the image is requested. Binary payloads live in object storage; only references and metadata live in the database.

## Object Storage (SeaweedFS)

A SeaweedFS container (see [`docker-compose.yaml`](../docker-compose.yaml)) exposes an S3-compatible API. The application talks to it exclusively through the AWS S3 SDK (`aws-sdk-s3`).

| Port   | Purpose                         |
| ------ | ------------------------------- |
| `8333` | S3 access point (`S3_BASE_URL`) |
| `9333` | SeaweedFS admin panel           |
| `8888` | SeaweedFS filer                 |

The application connects with the credentials and bucket configured in `.env`:

- `AWS_ACCESS_KEY_ID` — access key (SeaweedFS credentials)
- `AWS_SECRET_ACCESS_KEY` — secret key (SeaweedFS credentials)
- `S3_BUCKET` — bucket name; created automatically on startup if it does not exist
- `S3_BASE_URL` — endpoint URL (local: `http://localhost:8333`)

On startup ([`src/config.rs`](../src/config.rs)) the app checks whether the bucket exists and creates it if needed.

### Object Key Layout

Objects are stored under a date-partitioned, content-addressed key. The first two path segments are the UTC upload date (`images/yyyy/mm`), and the file name is derived from the file's SHA-256 digest:

```text
images/yyyy/mm/{sha256_hex}.{extension}
```

For example: `images/2026/08/a1b2c3…9f0e.png`. The `yyyy/mm` prefix is the upload year and month (in UTC, see [`src/handlers/images.rs`](../src/handlers/images.rs)) and groups objects into monthly folders. Deduplication is handled at the database level: `image_sources` is keyed on the SHA-256 hash, so re-uploading identical bytes reuses the existing row and does not write a new object — identical files therefore map to the same object and are shared across uploads.

## Database

Two tables store media references (see [`migrations/2026-08-14-035230-0000_create_images`](../migrations/2026-08-14-035230-0000_create_images/up.sql)):

- **`image_sources`** — deduplicated image content, keyed by the `file_sha256_hash` (the object's SHA-256 digest, the primary key). Stores `s3_path` (`images/yyyy/mm`), `extension`, `file_size`, `mime_type`, `bucket_name`, `width`, and `height` (all positive, enforced by CHECK constraints). The full S3 object key is derived as `s3_path/{file_sha256_hash}.{extension}`.
- **`user_images`** — a user-uploaded image referencing an `image_sources` entry via `file_sha256_hash` (foreign key), and records the uploader (`created_by`) and `original_file_name`.

Each row in `user_images` has an `id` that is used as the public image name.

## API Endpoints

All media endpoints are under the `/api/v1` prefix. See [`src/router.rs`](../src/router.rs) for the route table.

### Upload Image

```text
POST /api/v1/media/images
```

- **Content-Type**: `multipart/form-data`
- **Field**: `file` — the image binary (the first field named `file` is used; other fields are ignored)

The uploaded bytes are validated as an image (JPEG, PNG, or WebP). On success the object is written to the bucket, rows are inserted into `image_sources` (with `ON CONFLICT ... DO NOTHING` for deduplication) and `user_images` inside a transaction, and the created image is returned.

**Response `200 OK`** — JSON:

```json
{
  "id": 1,
  "original_file_name": "photo.png",
  "file_size": 20480,
  "width": 800,
  "height": 600,
  "created_by": 1,
  "created_at": "2026-08-16 12:00:00"
}
```

> **Note**: Authentication is not yet implemented. Uploads currently record a hard-coded `created_by = 1` placeholder (see [`src/handlers/images.rs`](../src/handlers/images.rs)) until JWT/user context is wired in.

### Retrieve Image

```text
GET /api/v1/media/images/{image_name}
```

`{image_name}` is the `user_images.id`, optionally followed by an extension (e.g. `/media/images/1.png`). The ID portion is parsed from the name; the extension is ignored.

The object is streamed back with:

- `Content-Type` — the stored `mime_type`
- `Cache-Control: public, max-age=2592000` (30 days)

**Response** — the raw image bytes.

## Error Codes

Media operations report the following `ApiErrorCode` values (see [`src/handlers/mod.rs`](../src/handlers/mod.rs)):

| Code                         | Meaning                                                                         |
| ---------------------------- | ------------------------------------------------------------------------------- |
| `FILE_UPLOAD_ERROR`          | Malformed multipart request, unknown/corrupt image format, or S3 upload failure |
| `PATH_PARAMETER_PARSE_ERROR` | Image name does not contain a valid numeric ID                                  |
| `S3_STORAGE_ERROR`           | Failure loading an object from the bucket                                       |
| `DB_QUERY_ERROR`             | Image not found or database insert/query failure                                |

## See Also

- [Development Guide](./DEVELOPMENT.md) — setup, environment variables, and workflow
- [Troubleshooting](./TROUBLESHOOTING.md) — S3/SeaweedFS troubleshooting
- [Dependency Management](./DEPENDENCIES.md) — adding/updating Rust dependencies
