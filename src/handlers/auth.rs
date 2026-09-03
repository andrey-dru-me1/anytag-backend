// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use argon2::{
    Argon2,
    password_hash::{PasswordVerifier, phc::PasswordHash},
};
use axum::{Json, extract::State, http::HeaderMap, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::config::AppState;
use crate::dto::{
    CurrentUserResponse, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse,
    RefreshTokenRequest, TokenPairResponse,
};
use crate::handlers::{ApiError, ApiErrorCode};
use crate::jwt::{TokenType, create_access_token, create_refresh_token, hash_token, verify_token};
use crate::models::{User, UserId};
use crate::schema::users::dsl::*;

const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$CTFhFdXPJO1aFaMaO6Mm5c8y7cJHAph8ArZWb2GRPPc";

fn verify_password(password: &str, stored_password_hash: &str) -> Result<(), String> {
    let parsed_hash = PasswordHash::new(stored_password_hash)
        .map_err(|e| format!("password hash parsing failed: {e}"))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| format!("argon2 password verification failed: {e}"))
}

fn password_task_error(error: tokio::task::JoinError) -> ApiError {
    ApiError::builder()
        .code(ApiErrorCode::PasswordHashError)
        .context(format!("password verification task failed: {error}"))
        .build()
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| {
            ApiError::builder()
                .http_status(StatusCode::UNAUTHORIZED)
                .code(ApiErrorCode::InvalidToken)
                .context("authorization header is missing")
                .message("Missing authorization token")
                .build()
        })?;

    let auth_header = auth_header.to_str().map_err(|e| {
        ApiError::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ApiErrorCode::InvalidToken)
            .context(format!("authorization header is not valid UTF-8: {e}"))
            .message("Invalid authorization token")
            .build()
    })?;

    auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        ApiError::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ApiErrorCode::InvalidToken)
            .context("authorization header does not start with 'Bearer '")
            .message("Invalid authorization token")
            .build()
    })
}

pub(super) fn get_current_user_id(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<UserId, ApiError> {
    let token = extract_bearer_token(headers)?;

    let claims = verify_token(token, TokenType::Access, &state.config.jwt).map_err(|err| {
        ApiError::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ApiErrorCode::InvalidToken)
            .context(format!("access token verification failed: {err:?}"))
            .message("Authentication failed")
            .build()
    })?;

    Ok(claims.sub)
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let err_builder = ApiError::builder()
        .http_status(StatusCode::UNAUTHORIZED)
        .code(ApiErrorCode::InvalidCredentials)
        .message("Invalid email or password");

    let user: Option<User> = {
        let mut conn = state.db_pool.get().await?;
        users
            .filter(email.eq(&payload.email))
            .first::<User>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                err_builder
                    .clone()
                    .context(format!("failed to find user by email in database: {e}"))
                    .build()
            })?
    };

    let stored_password_hash = user
        .as_ref()
        .map(|user| user.password_hash.clone())
        .unwrap_or_else(|| DUMMY_PASSWORD_HASH.to_string());
    let password_matches = tokio::task::spawn_blocking(move || {
        verify_password(&payload.password, &stored_password_hash).is_ok()
    })
    .await
    .map_err(password_task_error)?;

    let Some(user) = user.filter(|_| password_matches) else {
        return Err(err_builder.context("password verification failed").build());
    };

    let access_token = create_access_token(user.id, &state.config.jwt).map_err(|e| {
        ApiError::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ApiErrorCode::JwtCreationError)
            .context(format!("failed to create access token: {e}"))
            .message("Authentication service error")
            .build()
    })?;

    let refresh_token = create_refresh_token(user.id, &state.config.jwt).map_err(|e| {
        ApiError::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ApiErrorCode::JwtCreationError)
            .context(format!("failed to create refresh token: {e}"))
            .message("Authentication service error")
            .build()
    })?;

    let refresh_token_hash = hash_token(&refresh_token);
    let refresh_expires_at = chrono::Utc::now().naive_utc()
        + chrono::Duration::days(state.config.jwt.refresh_token_ttl_days);

    let mut conn = state.db_pool.get().await?;
    diesel::insert_into(crate::schema::refresh_tokens::table)
        .values(&crate::models::NewRefreshToken {
            user_id: user.id,
            token_hash: refresh_token_hash,
            expires_at: refresh_expires_at,
        })
        .execute(&mut conn)
        .await
        .map_err(|e| {
            (
                ApiErrorCode::DbQueryError,
                format!("failed to save refresh token: {e}"),
            )
        })?;

    Ok(Json(LoginResponse {
        message: "login successful".to_string(),
        user_id: user.id,
        email: user.email,
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
    }))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let current_user_id = get_current_user_id(&headers, &state)?;

    let mut conn = state.db_pool.get().await?;
    let user = users
        .find(current_user_id)
        .first::<User>(&mut conn)
        .await
        .map_err(|e| {
            ApiError::builder()
                .http_status(StatusCode::UNAUTHORIZED)
                .code(ApiErrorCode::InvalidToken)
                .context(format!(
                    "failed to find user from token subject '{}': {e}",
                    current_user_id
                ))
                .message("Authentication failed")
                .build()
        })?;

    Ok(Json(CurrentUserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
    }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let claims = verify_token(
        &payload.refresh_token,
        TokenType::Refresh,
        &state.config.jwt,
    )
    .map_err(|_| {
        ApiError::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ApiErrorCode::InvalidToken)
            .context("refresh token verification failed")
            .message("Authentication failed")
            .build()
    })?;

    let old_refresh_token_hash = hash_token(&payload.refresh_token);
    let err_builder = ApiError::builder()
        .http_status(StatusCode::UNAUTHORIZED)
        .code(ApiErrorCode::InvalidToken)
        .message("Authentication failed");

    let access_token = create_access_token(claims.sub, &state.config.jwt).map_err(|e| {
        ApiError::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ApiErrorCode::JwtCreationError)
            .context(format!("failed to create access token: {e}"))
            .message("Authentication service error")
            .build()
    })?;
    let refresh_token = create_refresh_token(claims.sub, &state.config.jwt).map_err(|e| {
        ApiError::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ApiErrorCode::JwtCreationError)
            .context(format!("failed to create refresh token: {e}"))
            .message("Authentication service error")
            .build()
    })?;

    let new_refresh_token_hash = hash_token(&refresh_token);
    let refresh_expires_at = chrono::Utc::now().naive_utc()
        + chrono::Duration::days(state.config.jwt.refresh_token_ttl_days);
    let new_refresh_token = crate::models::NewRefreshToken {
        user_id: claims.sub,
        token_hash: new_refresh_token_hash,
        expires_at: refresh_expires_at,
    };

    state
        .db_pool
        .get()
        .await?
        .transaction::<_, ApiError, _>(async |conn| {
            let stored_token = crate::schema::refresh_tokens::table
                .filter(crate::schema::refresh_tokens::token_hash.eq(&old_refresh_token_hash))
                .for_update()
                .first::<crate::models::RefreshToken>(conn)
                .await
                .map_err(|e| {
                    err_builder
                        .clone()
                        .context(format!("refresh token not found in database: {e}"))
                        .build()
                })?;

            if stored_token.user_id != claims.sub {
                return Err(err_builder
                    .clone()
                    .context("refresh token subject does not match stored token owner")
                    .build());
            }
            if stored_token.revoked_at.is_some() {
                return Err(err_builder
                    .clone()
                    .context("refresh token is revoked")
                    .build());
            }
            if stored_token.expires_at < chrono::Utc::now().naive_utc() {
                return Err(err_builder
                    .clone()
                    .context("refresh token is expired in database")
                    .build());
            }

            diesel::update(crate::schema::refresh_tokens::table.find(stored_token.id))
                .set(crate::schema::refresh_tokens::revoked_at.eq(chrono::Utc::now().naive_utc()))
                .execute(conn)
                .await
                .map_err(|e| {
                    ApiError::builder()
                        .code(ApiErrorCode::DbQueryError)
                        .context(format!("failed to revoke old refresh token: {e}"))
                        .build()
                })?;
            diesel::insert_into(crate::schema::refresh_tokens::table)
                .values(&new_refresh_token)
                .execute(conn)
                .await
                .map_err(|e| {
                    ApiError::builder()
                        .code(ApiErrorCode::DbQueryError)
                        .context(format!("failed to save new refresh token: {e}"))
                        .build()
                })?;

            Ok(())
        })
        .await?;

    Ok(Json(TokenPairResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
    }))
}

pub async fn logout_user(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut conn = state.db_pool.get().await?;
    let refresh_token_hash = hash_token(&payload.refresh_token);

    diesel::update(
        crate::schema::refresh_tokens::table
            .filter(crate::schema::refresh_tokens::token_hash.eq(refresh_token_hash))
            .filter(crate::schema::refresh_tokens::revoked_at.is_null()),
    )
    .set(crate::schema::refresh_tokens::revoked_at.eq(chrono::Utc::now().naive_utc()))
    .execute(&mut conn)
    .await
    .map_err(|e| {
        (
            ApiErrorCode::DbQueryError,
            format!("failed to revoke refresh token on logout: {e}"),
        )
    })?;

    Ok(Json(LogoutResponse {
        message: "logout successful".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::PasswordHasher;

    // -----------------------------------------------------------------------
    // verify_password
    // -----------------------------------------------------------------------

    #[test]
    fn test_verify_password_accepts_correct_password() {
        let password = "CorrectHorseBatteryStaple99!";
        let hash = Argon2::default()
            .hash_password(password.as_bytes())
            .expect("hashing should succeed")
            .to_string();

        assert!(verify_password(password, &hash).is_ok());
    }

    #[test]
    fn test_verify_password_rejects_wrong_password() {
        let hash = Argon2::default()
            .hash_password("CorrectHorseBatteryStaple99!".as_bytes())
            .expect("hashing should succeed")
            .to_string();

        assert!(verify_password("WrongPassword123!", &hash).is_err());
    }
}
