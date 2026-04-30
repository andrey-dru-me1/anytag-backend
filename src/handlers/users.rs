// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use diesel::prelude::*;

use crate::db::{get_db_conn, DbPool};
use crate::dto::{CreateUserRequest, LoginRequest, LoginResponse, UserCreatedResponse};
use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;

use rand_core::OsRng;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// Handler for creating a new user
pub async fn create_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if payload.password.as_str().len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "password too short".to_string(),
        ));
    }

    let mut conn = get_db_conn(&pool)?;

    let password_hashed = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            e,
        )
    })?;

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
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error: {}", e),
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = get_db_conn(&pool)?;

    let user: User = users
        .filter(email.eq(payload.email))
        .first::<User>(&mut conn)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "invalid email or password".to_string(),
            )
        })?;
    
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid password hash format".to_string(),
        )
    })?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "invalid email or password".to_string(),
            )
        })?;
    
    Ok(Json(LoginResponse {
        message: "login successful".to_string(),
        user_id: user.id,
        email: user.email,
    }))
}