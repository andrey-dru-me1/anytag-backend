// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod health;
mod posts;
mod tags;
mod users;

pub use health::*;
pub use posts::*;
pub use tags::*;
pub use users::*;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::borrow::Cow;
use strum::Display;

#[derive(strum::AsRefStr, Clone, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrCode {
    WeakPassword,
    DbConnectionError,
    DbQueryError,
    PasswordHashError,
    InvalidCredentials,
}

#[derive(bon::Builder)]
#[builder(derive(Clone))]
pub struct HandlerErr {
    #[builder(default = StatusCode::INTERNAL_SERVER_ERROR)]
    http_status: StatusCode,
    code: ErrCode,
    #[builder(into)]
    context: Cow<'static, str>,
    #[builder(into)]
    message: Option<Cow<'static, str>>,
}

impl HandlerErr {
    pub fn from_db_conn_err((status, msg): (StatusCode, String)) -> Self {
        Self::builder()
            .http_status(status)
            .code(ErrCode::DbConnectionError)
            .context(format!("database connection failed: {}", msg))
            .build()
    }
}

impl IntoResponse for HandlerErr {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(
            code = %self.code,
            context = %&self.context
        );

        #[derive(Serialize, Debug)]
        struct HandlerErrBody<'a> {
            code: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            message: Option<Cow<'static, str>>,
        }

        let body = HandlerErrBody {
            code: self.code.as_ref(),
            message: self.message,
        };
        let body = Json(body);

        (self.http_status, body).into_response()
    }
}

impl From<(ErrCode, String)> for HandlerErr {
    fn from((code, message): (ErrCode, String)) -> Self {
        Self::builder().code(code).context(message).build()
    }
}

impl From<(StatusCode, ErrCode, String)> for HandlerErr {
    fn from((status, code, message): (StatusCode, ErrCode, String)) -> Self {
        Self::builder()
            .http_status(status)
            .code(code)
            .context(message)
            .build()
    }
}
