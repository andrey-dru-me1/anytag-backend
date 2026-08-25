// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use serde::{Deserialize, Serialize};

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use std::env;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, errors::ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    ExpiredToken,
    WrongTokenType,
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

fn access_token_ttl_minutes() -> i64 {
    env::var("ACCESS_TOKEN_TTL_MINUTES")
        .unwrap_or_else(|_| "15".to_string())
        .parse()
        .expect("ACCESS_TOKEN_TTL_MINUTES must be a valid integer")
}

pub fn refresh_token_ttl_days() -> i64 {
    env::var("REFRESH_TOKEN_TTL_DAYS")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .expect("REFRESH_TOKEN_TTL_DAYS must be a valid integer")
}

fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

fn expiration_timestamp(duration: Duration) -> i64 {
    (Utc::now() + duration).timestamp()
}

pub fn create_access_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(
        user_id,
        TokenType::Access,
        Duration::minutes(access_token_ttl_minutes()),
    )
}

pub fn create_refresh_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(
        user_id,
        TokenType::Refresh,
        Duration::days(refresh_token_ttl_days()),
    )
}

fn create_token(
    user_id: i32,
    token_type: TokenType,
    ttl: Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id,
        iat: now_timestamp(),
        exp: expiration_timestamp(ttl),
        token_type,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
}

pub fn verify_token(token: &str, expected_token_type: TokenType) -> Result<Claims, JwtError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| match err.kind() {
        ErrorKind::ExpiredSignature => JwtError::ExpiredToken,
        _ => JwtError::InvalidToken,
    })?;

    if token_data.claims.token_type != expected_token_type {
        return Err(JwtError::WrongTokenType);
    }

    Ok(token_data.claims)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());

    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}
