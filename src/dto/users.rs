// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::models::UserId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserCreatedResponse {
    pub message: String,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub user_id: UserId,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Serialize)]
pub struct CurrentUserResponse {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub message: String,
}
