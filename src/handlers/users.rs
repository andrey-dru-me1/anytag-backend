// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{Json, extract::State, http::HeaderMap, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;
use email_address::EmailAddress;
use zxcvbn::{Score, zxcvbn};

use crate::db::{DbPool, get_db_conn};
use crate::dto::{
    CreateUserRequest, CurrentUserResponse, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, RefreshTokenRequest, TokenPairResponse, UserCreatedResponse,
};
use crate::handlers::{ErrCode, HandlerErr};
use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;

use crate::jwt::{TokenType, create_access_token, create_refresh_token, hash_token, verify_token};

use rand_core::OsRng;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("argon2 password hashing failed: {}", e))
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, HandlerErr> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| {
            HandlerErr::builder()
                .http_status(StatusCode::UNAUTHORIZED)
                .code(ErrCode::InvalidToken)
                .context("authorization header is missing")
                .message("Missing authorization token")
                .build()
        })?;

    let auth_header = auth_header.to_str().map_err(|e| {
        HandlerErr::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ErrCode::InvalidToken)
            .context(format!("authorization header is not valid UTF-8: {}", e))
            .message("Invalid authorization token")
            .build()
    })?;

    auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        HandlerErr::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ErrCode::InvalidToken)
            .context("authorization header does not start with 'Bearer '")
            .message("Invalid authorization token")
            .build()
    })
}

/// Handler for creating a new user
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, HandlerErr> {
    if !EmailAddress::is_valid(&payload.email) {
        return Err(HandlerErr::builder()
            .http_status(StatusCode::UNPROCESSABLE_ENTITY)
            .code(ErrCode::InvalidEmail)
            .context("email format validation failed")
            .message("Invalid email")
            .build());
    }

    let estimate = zxcvbn(&payload.password, &[&payload.name, &payload.email]);
    if estimate.score() < Score::Three {
        let mut message = "The password is weak.".to_string();
        if let Some(feedback) = estimate.feedback() {
            message = format!("{} {}", message, feedback);
        }
        return Err(HandlerErr::builder()
            .http_status(StatusCode::UNPROCESSABLE_ENTITY)
            .code(ErrCode::WeakPassword)
            .context(format!(
                "password complexity check failed: zxcvbn score is {}",
                estimate.score()
            ))
            .message(message)
            .build());
    }

    let mut conn = get_db_conn(&pool).map_err(HandlerErr::from_db_conn_err)?;

    let password_hashed =
        hash_password(&payload.password).map_err(|e| (ErrCode::PasswordHashError, e))?;

    let new_user = NewUser {
        name: payload.name,
        email: payload.email,
        password_hash: password_hashed,
    };

    let created = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<crate::models::User>(&mut conn)
        .map_err(|e| {
            (
                ErrCode::DbQueryError,
                format!("failed to create new user: {}", e),
            )
        })?;

    Ok(Json(UserCreatedResponse {
        message: "user created".to_string(),
        name: created.name,
        email: created.email,
    }))
}

pub async fn login_user(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, HandlerErr> {
    let mut conn = get_db_conn(&pool).map_err(HandlerErr::from_db_conn_err)?;

    let err_builder = HandlerErr::builder()
        .http_status(StatusCode::UNAUTHORIZED)
        .code(ErrCode::InvalidCredentials)
        .message("Invalid email or password");

    let user: User = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .map_err(|e| {
            err_builder
                .clone()
                .context(format!(
                    "failed to find user by email '{}' in database: {}",
                    payload.email, e
                ))
                .build()
        })?;

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
        err_builder
            .clone()
            .context(format!("password hash parsing failed: {}", e))
            .build()
    })?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|e| {
            err_builder
                .clone()
                .context(format!("argon2 password verification failed: {}", e))
                .build()
        })?;

    let access_token = create_access_token(user.id).map_err(|_| {
        HandlerErr::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ErrCode::JwtCreationError)
            .context("failed to create access token")
            .message("Failed to create access token")
            .build()
    })?;

    let refresh_token = create_refresh_token(user.id).map_err(|_| {
        HandlerErr::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ErrCode::JwtCreationError)
            .context("failed to create refresh token")
            .message("Failed to create refresh token")
            .build()
    })?;

    let refresh_token_hash = crate::jwt::hash_token(&refresh_token);

    let refresh_expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);

    diesel::insert_into(crate::schema::refresh_tokens::table)
        .values(&crate::models::NewRefreshToken {
            user_id: user.id,
            token_hash: refresh_token_hash,
            expires_at: refresh_expires_at,
        })
        .execute(&mut conn)
        .map_err(|e| {
            (
                ErrCode::DbQueryError,
                format!("failed to save refresh token: {}", e),
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
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HandlerErr> {
    let token = extract_bearer_token(&headers)?;

    let claims = verify_token(token, TokenType::Access).map_err(|_| {
        HandlerErr::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ErrCode::InvalidToken)
            .context("access token verification failed")
            .message("Invalid authorization token")
            .build()
    })?;

    let mut conn = get_db_conn(&pool).map_err(HandlerErr::from_db_conn_err)?;

    let user = users
        .find(claims.sub)
        .first::<User>(&mut conn)
        .map_err(|e| {
            HandlerErr::builder()
                .http_status(StatusCode::UNAUTHORIZED)
                .code(ErrCode::InvalidToken)
                .context(format!(
                    "failed to find user from token subject '{}': {}",
                    claims.sub, e
                ))
                .message("Invalid authorization token")
                .build()
        })?;

    Ok(Json(CurrentUserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
    }))
}

pub async fn refresh_token(
    State(pool): State<DbPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, HandlerErr> {
    let claims = verify_token(&payload.refresh_token, TokenType::Refresh).map_err(|_| {
        HandlerErr::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ErrCode::InvalidToken)
            .context("refresh token verification failed")
            .message("Invalid refresh token")
            .build()
    })?;

    let mut conn = get_db_conn(&pool).map_err(HandlerErr::from_db_conn_err)?;

    let refresh_token_hash = hash_token(&payload.refresh_token);

    let stored_token = crate::schema::refresh_tokens::table
        .filter(crate::schema::refresh_tokens::token_hash.eq(&refresh_token_hash))
        .first::<crate::models::RefreshToken>(&mut conn)
        .map_err(|e| {
            HandlerErr::builder()
                .http_status(StatusCode::UNAUTHORIZED)
                .code(ErrCode::InvalidToken)
                .context(format!("refresh token not found in database: {}", e))
                .message("Invalid refresh token")
                .build()
        })?;

    if stored_token.revoked_at.is_some() {
        return Err(HandlerErr::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ErrCode::InvalidToken)
            .context("refresh token is revoked")
            .message("Invalid refresh token")
            .build());
    }

    if stored_token.expires_at < chrono::Utc::now().naive_utc() {
        return Err(HandlerErr::builder()
            .http_status(StatusCode::UNAUTHORIZED)
            .code(ErrCode::InvalidToken)
            .context("refresh token is expired in database")
            .message("Invalid refresh token")
            .build());
    }

    diesel::update(crate::schema::refresh_tokens::table.find(stored_token.id))
        .set(crate::schema::refresh_tokens::revoked_at.eq(chrono::Utc::now().naive_utc()))
        .execute(&mut conn)
        .map_err(|e| {
            (
                ErrCode::DbQueryError,
                format!("failed to revoke old refresh token: {}", e),
            )
        })?;

    let access_token = create_access_token(claims.sub).map_err(|_| {
        HandlerErr::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ErrCode::JwtCreationError)
            .context("failed to create access token")
            .message("Failed to create access token")
            .build()
    })?;

    let refresh_token = create_refresh_token(claims.sub).map_err(|_| {
        HandlerErr::builder()
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .code(ErrCode::JwtCreationError)
            .context("failed to create refresh token")
            .message("Failed to create refresh token")
            .build()
    })?;

    let refresh_token_hash = hash_token(&refresh_token);
    let refresh_expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(30);

    diesel::insert_into(crate::schema::refresh_tokens::table)
        .values(&crate::models::NewRefreshToken {
            user_id: claims.sub,
            token_hash: refresh_token_hash,
            expires_at: refresh_expires_at,
        })
        .execute(&mut conn)
        .map_err(|e| {
            (
                ErrCode::DbQueryError,
                format!("failed to save new refresh token: {}", e),
            )
        })?;

    Ok(Json(TokenPairResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
    }))
}

pub async fn logout_user(
    State(pool): State<DbPool>,
    Json(payload): Json<LogoutRequest>,
) -> Result<impl IntoResponse, HandlerErr> {
    let mut conn = get_db_conn(&pool).map_err(HandlerErr::from_db_conn_err)?;

    let refresh_token_hash = hash_token(&payload.refresh_token);

    diesel::update(
        crate::schema::refresh_tokens::table
            .filter(crate::schema::refresh_tokens::token_hash.eq(refresh_token_hash))
            .filter(crate::schema::refresh_tokens::revoked_at.is_null()),
    )
    .set(crate::schema::refresh_tokens::revoked_at.eq(chrono::Utc::now().naive_utc()))
    .execute(&mut conn)
    .map_err(|e| {
        (
            ErrCode::DbQueryError,
            format!("failed to revoke refresh token on logout: {}", e),
        )
    })?;

    Ok(Json(LogoutResponse {
        message: "logout successful".to_string(),
    }))
}
