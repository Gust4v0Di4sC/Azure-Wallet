use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum_extra::extract::cookie::{Key, PrivateCookieJar};
use jwt_simple::{
    claims::Claims,
    prelude::{Duration, HS256Key, MACLike},
};
use serde::{Deserialize, Serialize};

use crate::{app::AppState, error::AppError};

pub const AUTH_COOKIE_NAME: &str = "auth_token";

#[derive(Debug, Clone)]
pub struct User {
    id: i64,
    email: String,
}

impl User {
    pub fn new(id: i64, email: String) -> Self {
        Self { id, email }
    }

    pub const fn email(&self) -> &String {
        &self.email
    }

    pub const fn id(&self) -> i64 {
        self.id
    }

    pub fn auth_token(&self, state: &AppState) -> Result<String, AppError> {
        let key = HS256Key::from_bytes(&state.auth.jwt_secret);
        let claims = Claims::with_custom_claims(UserClaims::from(self), Duration::from_hours(8));
        Ok(key.authenticate(claims)?)
    }

    pub fn from_auth_token(token: &str, state: &AppState) -> Result<Self, AppError> {
        let key = HS256Key::from_bytes(&state.auth.jwt_secret);
        let claims: UserClaims = key.verify_token(token, None)?.custom;
        Ok(Self::new(claims.id, claims.email))
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::MissingAuthorization)?;
        let token = jar
            .get(AUTH_COOKIE_NAME)
            .ok_or(AppError::MissingAuthorization)?;

        User::from_auth_token(token.value(), state).map_err(|_| AppError::InvalidCredentials)
    }
}

impl FromRequestParts<AppState> for Option<User> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(User::from_request_parts(parts, state).await.ok())
    }
}

#[derive(Serialize, Deserialize)]
struct UserClaims {
    id: i64,
    email: String,
}

impl From<&User> for UserClaims {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
        }
    }
}
