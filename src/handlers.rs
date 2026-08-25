// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

pub mod health;
pub mod images;
pub mod posts;
pub mod tags;
pub mod users;

use diesel_async::pooled_connection::deadpool;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::borrow::Cow;
use strum::Display;

#[derive(Debug, strum::AsRefStr, Clone, Display, PartialEq, Eq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiErrorCode {
    WeakPassword,
    DbConnectionError,
    DbQueryError,
    PasswordHashError,
    InvalidCredentials,
    InvalidEmail,
    FileUploadError,
    PathParameterParseError,
    S3StorageError,
    ImageNotFound,
}

#[derive(bon::Builder, Debug)]
#[builder(derive(Clone))]
pub struct ApiError {
    #[builder(default = StatusCode::INTERNAL_SERVER_ERROR)]
    http_status: StatusCode,
    code: ApiErrorCode,
    #[builder(into)]
    context: Cow<'static, str>,
    #[builder(into)]
    message: Option<Cow<'static, str>>,
}

impl ApiError {
    /// Returns the HTTP status code for this error.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn status(&self) -> StatusCode {
        self.http_status
    }

    /// Returns a reference to the error code.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn error_code(&self) -> &ApiErrorCode {
        &self.code
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(
            code = %self.code,
            context = %&self.context
        );

        #[derive(Serialize, Debug)]
        struct ApiErrorBody<'a> {
            code: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            message: Option<Cow<'static, str>>,
        }

        let body = ApiErrorBody {
            code: self.code.as_ref(),
            message: self.message,
        };
        let body = Json(body);

        (self.http_status, body).into_response()
    }
}

impl From<(ApiErrorCode, String)> for ApiError {
    fn from((code, message): (ApiErrorCode, String)) -> Self {
        Self::builder().code(code).context(message).build()
    }
}

impl From<(StatusCode, ApiErrorCode, String)> for ApiError {
    fn from((status, code, message): (StatusCode, ApiErrorCode, String)) -> Self {
        Self::builder()
            .http_status(status)
            .code(code)
            .context(message)
            .build()
    }
}

impl From<deadpool::PoolError> for ApiError {
    fn from(err: deadpool::PoolError) -> Self {
        Self::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ApiErrorCode::DbConnectionError)
            .context(format!("database connection failed: {err}"))
            .build()
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(err: diesel::result::Error) -> Self {
        Self::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ApiErrorCode::DbQueryError)
            .context(format!("database query failed: {err}"))
            .build()
    }
}
