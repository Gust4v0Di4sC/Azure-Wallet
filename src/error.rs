use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication cookie is missing")]
    MissingAuthorization,
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error("Asset does not exist")]
    AssetDoesNotExist,
    #[error("User does not exist")]
    UserDoesNotExist,
    #[error("This email is already registered")]
    EmailTaken,
    #[error("Invalid request")]
    Validation(Vec<FieldError>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
    #[error(transparent)]
    Bcrypt(#[from] bcrypt::BcryptError),
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    code: &'static str,
    message: String,
    field_errors: Vec<FieldError>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            code: self.code(),
            message: self.to_string(),
            field_errors: self.field_errors(),
        };

        (status, Json(error_response)).into_response()
    }
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::EmailTaken | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::MissingAuthorization | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AssetDoesNotExist | Self::UserDoesNotExist => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Template(_) | Self::Jwt(_) | Self::Bcrypt(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingAuthorization => "missing_authorization",
            Self::InvalidCredentials => "invalid_credentials",
            Self::AssetDoesNotExist => "asset_not_found",
            Self::UserDoesNotExist => "user_not_found",
            Self::EmailTaken => "email_taken",
            Self::Validation(_) => "validation_error",
            Self::Database(_) => "database_error",
            Self::Template(_) => "template_error",
            Self::Jwt(_) => "token_error",
            Self::Bcrypt(_) => "password_hash_error",
        }
    }

    pub fn field_errors(&self) -> Vec<FieldError> {
        match self {
            Self::Validation(errors) => errors.clone(),
            Self::EmailTaken => vec![FieldError::new("email", "This email is already registered")],
            Self::InvalidCredentials => vec![FieldError::new(
                "password",
                "Check your email and password, then try again",
            )],
            _ => Vec::new(),
        }
    }
}
