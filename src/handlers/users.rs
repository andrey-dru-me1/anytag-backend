// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;
use zxcvbn::{Score, zxcvbn};

use crate::db::{DbPool, get_db_conn};
use crate::dto::{CreateUserRequest, LoginRequest, LoginResponse, UserCreatedResponse};
use crate::handlers::{ErrCode, HandlerErr};
use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;

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

/// Handler for creating a new user
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, HandlerErr> {
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

    Ok(Json(LoginResponse {
        message: "login successful".to_string(),
        user_id: user.id,
        email: user.email,
    }))
}
