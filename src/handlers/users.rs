// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use email_address::EmailAddress;
use zxcvbn::{Score, zxcvbn};

use crate::config::AppState;
use crate::dto::{CreateUserRequest, LoginRequest, LoginResponse, UserCreatedResponse};
use crate::handlers::{ApiError, ApiErrorCode};
use crate::models::{NewUser, User};
use crate::schema::users::dsl::*;

use argon2::{
    Argon2,
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
};

fn hash_password(password: &str) -> Result<String, String> {
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| format!("argon2 password hashing failed: {}", e))
}

/// Validate email format.
///
/// Returns `Ok(())` if the email is valid, or an `ApiError` with [`ApiErrorCode::InvalidEmail`]
/// and status [`StatusCode::UNPROCESSABLE_ENTITY`].
fn validate_email(input: &str) -> Result<(), ApiError> {
    if !EmailAddress::is_valid(input) {
        return Err(ApiError::builder()
            .http_status(StatusCode::UNPROCESSABLE_ENTITY)
            .code(ApiErrorCode::InvalidEmail)
            .context("email format validation failed")
            .message("Invalid email")
            .build());
    }
    Ok(())
}

/// Validate password strength using zxcvbn.
///
/// Returns `Ok(())` if the password scores at least [`Score::Three`],
/// or an `ApiError` with [`ApiErrorCode::WeakPassword`] and
/// status [`StatusCode::UNPROCESSABLE_ENTITY`].
fn validate_password_strength(
    password: &str,
    user_name: &str,
    user_email: &str,
) -> Result<(), ApiError> {
    let estimate = zxcvbn(password, &[user_name, user_email]);
    if estimate.score() < Score::Three {
        let mut message = "The password is weak.".to_string();
        if let Some(feedback) = estimate.feedback() {
            message = format!("{} {}", message, feedback);
        }
        return Err(ApiError::builder()
            .http_status(StatusCode::UNPROCESSABLE_ENTITY)
            .code(ApiErrorCode::WeakPassword)
            .context(format!(
                "password complexity check failed: zxcvbn score is {}",
                estimate.score()
            ))
            .message(message)
            .build());
    }
    Ok(())
}

/// Handler for creating a new user
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_email(&payload.email)?;
    validate_password_strength(&payload.password, &payload.name, &payload.email)?;

    let mut conn = state.db_pool.get().await?;

    let password_hashed =
        hash_password(&payload.password).map_err(|e| (ApiErrorCode::PasswordHashError, e))?;

    let new_user = NewUser {
        name: payload.name,
        email: payload.email,
        password_hash: password_hashed,
    };

    let created = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<crate::models::User>(&mut conn)
        .await
        .map_err(|e| {
            (
                ApiErrorCode::DbQueryError,
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
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut conn = state.db_pool.get().await?;

    let err_builder = ApiError::builder()
        .http_status(StatusCode::UNAUTHORIZED)
        .code(ApiErrorCode::InvalidCredentials)
        .message("Invalid email or password");

    let user: User = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .await
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

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::phc::PasswordHash;
    use axum::http::StatusCode;

    // -----------------------------------------------------------------------
    // hash_password
    // -----------------------------------------------------------------------

    #[test]
    fn test_hash_password_produces_valid_hash() {
        let hash = hash_password("strongpassword123").expect("hashing should succeed");
        assert!(
            hash.starts_with("$argon2"),
            "hash should start with '$argon2', got: {hash}"
        );
        // Verify the hash can be parsed back
        PasswordHash::new(&hash).expect("hash should be parseable");
    }

    #[test]
    fn test_hash_password_produces_unique_salts() {
        let hash1 = hash_password("password").expect("hashing should succeed");
        let hash2 = hash_password("password").expect("hashing should succeed");
        assert_ne!(
            hash1, hash2,
            "same password should produce different hashes"
        );
    }

    #[test]
    fn test_hash_password_unicode() {
        let hash = hash_password("пароль你好🔒").expect("unicode password should hash");
        assert!(hash.starts_with("$argon2"));
    }

    // -----------------------------------------------------------------------
    // validate_email
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_email_valid() {
        let result = validate_email("user@example.com");
        assert!(
            result.is_ok(),
            "valid email should pass: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_validate_email_invalid() {
        let err = validate_email("not-an-email").unwrap_err();
        assert_eq!(err.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.error_code().as_ref(), "INVALID_EMAIL");
    }

    #[test]
    fn test_validate_email_empty() {
        let err = validate_email("").unwrap_err();
        // An empty string is not a valid email
        assert_eq!(err.error_code().as_ref(), "INVALID_EMAIL");
    }

    // -----------------------------------------------------------------------
    // validate_password_strength
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_password_strength_strong() {
        let result = validate_password_strength(
            "CorrectHorseBatteryStaple99!",
            "alice",
            "alice@example.com",
        );
        assert!(
            result.is_ok(),
            "strong password should pass: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_validate_password_strength_weak() {
        let err = validate_password_strength("12345678", "bob", "bob@example.com").unwrap_err();
        assert_eq!(err.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.error_code().as_ref(), "WEAK_PASSWORD");
    }

    #[test]
    fn test_validate_password_strength_with_user_info() {
        // Password containing the user's name should still execute without
        // panicking. zxcvbn penalises passwords that contain user info.
        let result = validate_password_strength("Alice123!", "Alice", "alice@example.com");
        // zxcvbn may consider this weak or strong; we just verify no panic.
        let _ = result;
    }
}
