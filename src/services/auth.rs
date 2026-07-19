use bcrypt::{DEFAULT_COST, non_truncating_hash, non_truncating_verify};

use crate::{
    auth::user::User,
    error::{AppError, FieldError},
    models::AuthForm,
    repository::Repository,
};

pub struct AuthService {
    repository: Repository,
}

impl AuthService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub async fn register(&self, form: AuthForm) -> Result<User, AppError> {
        validate_auth_form(&form)?;

        let email = normalize_email(&form.email);
        let password_hash = non_truncating_hash(form.password, DEFAULT_COST)?;
        let user_record = match self.repository.add_user(&email, &password_hash).await {
            Ok(user_record) => user_record,
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                return Err(AppError::EmailTaken);
            }
            Err(err) => return Err(AppError::Database(err)),
        };

        Ok(User::new(user_record.id, user_record.email))
    }

    pub async fn login(&self, form: AuthForm) -> Result<User, AppError> {
        validate_auth_form(&form)?;

        let email = normalize_email(&form.email);
        let user_record = self
            .repository
            .get_user_by_email(&email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        if non_truncating_verify(form.password, &user_record.password_hash)? {
            Ok(User::new(user_record.id, user_record.email))
        } else {
            Err(AppError::InvalidCredentials)
        }
    }
}

fn validate_auth_form(form: &AuthForm) -> Result<(), AppError> {
    let mut errors = Vec::new();
    let email = form.email.trim();

    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        errors.push(FieldError::new("email", "Enter a valid email address"));
    }

    if form.password.chars().count() < 6 {
        errors.push(FieldError::new(
            "password",
            "Password must have at least 6 characters",
        ));
    }

    if form.password.as_bytes().len() > 72 {
        errors.push(FieldError::new(
            "password",
            "Password must have at most 72 bytes for bcrypt",
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors))
    }
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
