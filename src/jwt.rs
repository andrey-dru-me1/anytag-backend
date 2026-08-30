// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use serde::{Deserialize, Serialize};

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, errors::ErrorKind};

use crate::config::JwtConfig;

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

#[derive(Debug, PartialEq, Eq)]
pub enum JwtError {
    InvalidToken,
    ExpiredToken,
    WrongTokenType,
}

fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

fn expiration_timestamp(duration: Duration) -> i64 {
    (Utc::now() + duration).timestamp()
}

pub fn create_access_token(
    user_id: i32,
    config: &JwtConfig,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(
        user_id,
        TokenType::Access,
        Duration::minutes(config.access_token_ttl_minutes),
        &config.secret,
    )
}

pub fn create_refresh_token(
    user_id: i32,
    config: &JwtConfig,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(
        user_id,
        TokenType::Refresh,
        Duration::days(config.refresh_token_ttl_days),
        &config.secret,
    )
}

fn create_token(
    user_id: i32,
    token_type: TokenType,
    ttl: Duration,
    secret: &str,
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
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(
    token: &str,
    expected_token_type: TokenType,
    config: &JwtConfig,
) -> Result<Claims, JwtError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn jwt_config(secret: &str) -> JwtConfig {
        JwtConfig {
            secret: secret.to_string(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 30,
        }
    }

    // -----------------------------------------------------------------------
    // create and verify tokens
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_and_verify_access_token() {
        let config = jwt_config("test-access-secret");

        let token = create_access_token(42, &config).expect("access token creation should succeed");
        let claims = verify_token(&token, TokenType::Access, &config)
            .expect("access token verification should succeed");

        assert_eq!(claims.sub, 42);
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_create_and_verify_refresh_token() {
        let config = jwt_config("test-refresh-secret");

        let token =
            create_refresh_token(7, &config).expect("refresh token creation should succeed");
        let claims = verify_token(&token, TokenType::Refresh, &config)
            .expect("refresh token verification should succeed");

        assert_eq!(claims.sub, 7);
        assert_eq!(claims.token_type, TokenType::Refresh);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_verify_token_rejects_wrong_type() {
        let config = jwt_config("test-token-type-secret");
        let token =
            create_refresh_token(7, &config).expect("refresh token creation should succeed");

        let result = verify_token(&token, TokenType::Access, &config);

        assert_eq!(result.unwrap_err(), JwtError::WrongTokenType);
    }

    #[test]
    fn test_verify_token_rejects_different_secret() {
        let signing_config = jwt_config("signing-secret");
        let verification_config = jwt_config("different-secret");
        let token =
            create_access_token(42, &signing_config).expect("access token creation should succeed");

        let result = verify_token(&token, TokenType::Access, &verification_config);

        assert_eq!(result.unwrap_err(), JwtError::InvalidToken);
    }

    #[test]
    fn test_verify_token_rejects_expired_token() {
        let config = jwt_config("test-expired-secret");
        let token = create_token(42, TokenType::Access, Duration::minutes(-5), &config.secret)
            .expect("expired token creation should succeed");

        let result = verify_token(&token, TokenType::Access, &config);

        assert_eq!(result.unwrap_err(), JwtError::ExpiredToken);
    }

    // -----------------------------------------------------------------------
    // hash_token
    // -----------------------------------------------------------------------

    #[test]
    fn test_hash_token_known_vector() {
        assert_eq!(
            hash_token("refresh-token"),
            "0eb17643d4e9261163783a420859c92c7d212fa9624106a12b510afbec266120"
        );
    }

    #[test]
    fn test_hash_token_changes_with_input() {
        assert_ne!(hash_token("first-token"), hash_token("second-token"));
    }
}
