use rust_decimal::Decimal;

use crate::{
    auth::user::User,
    error::{AppError, FieldError},
    models::{CreateAssetRequest, PortfolioSummary, UpdateAssetRequest},
    repository::Repository,
};

pub struct PortfolioService {
    repository: Repository,
}

impl PortfolioService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub async fn summary(&self, user: &User) -> Result<PortfolioSummary, AppError> {
        let assets = self.repository.list_assets(user.id()).await?;
        let total_value = assets
            .iter()
            .map(|asset| asset.total_value)
            .sum::<Decimal>();

        Ok(PortfolioSummary {
            assets,
            total_value,
        })
    }

    pub async fn get_asset(
        &self,
        user: &User,
        asset_id: i64,
    ) -> Result<crate::models::Asset, AppError> {
        self.repository
            .get_asset(user.id(), asset_id)
            .await?
            .ok_or(AppError::AssetDoesNotExist)
    }

    pub async fn create_asset(
        &self,
        user: &User,
        request: CreateAssetRequest,
    ) -> Result<crate::models::Asset, AppError> {
        validate_create_asset(&request)?;

        match self.repository.create_asset(user.id(), &request).await {
            Ok(asset) => Ok(asset),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(AppError::Validation(vec![FieldError::new(
                    "ticker",
                    "You already have an asset with this ticker",
                )]))
            }
            Err(err) => Err(AppError::Database(err)),
        }
    }

    pub async fn update_asset(
        &self,
        user: &User,
        asset_id: i64,
        request: UpdateAssetRequest,
    ) -> Result<crate::models::Asset, AppError> {
        validate_update_asset(&request)?;

        match self
            .repository
            .update_asset(user.id(), asset_id, &request)
            .await
        {
            Ok(Some(asset)) => Ok(asset),
            Ok(None) => Err(AppError::AssetDoesNotExist),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(AppError::Validation(vec![FieldError::new(
                    "ticker",
                    "You already have an asset with this ticker",
                )]))
            }
            Err(err) => Err(AppError::Database(err)),
        }
    }
}

fn validate_create_asset(request: &CreateAssetRequest) -> Result<(), AppError> {
    let mut errors = Vec::new();

    validate_required("name", &request.name, "Enter the asset name", &mut errors);
    validate_required("ticker", &request.ticker, "Enter the ticker", &mut errors);
    validate_required(
        "asset_class",
        &request.asset_class,
        "Enter the asset class",
        &mut errors,
    );
    validate_required(
        "currency",
        &request.currency,
        "Enter the currency",
        &mut errors,
    );
    validate_positive(
        "quantity",
        request.quantity,
        "Quantity must be greater than zero",
        &mut errors,
    );
    validate_positive(
        "average_price",
        request.average_price,
        "Average price must be greater than zero",
        &mut errors,
    );
    validate_positive(
        "current_price",
        request.current_price,
        "Current price must be greater than zero",
        &mut errors,
    );

    finish_validation(errors)
}

fn validate_update_asset(request: &UpdateAssetRequest) -> Result<(), AppError> {
    let mut errors = Vec::new();
    let mut has_change = false;

    validate_optional_text(
        "name",
        request.name.as_deref(),
        "Enter the asset name",
        &mut has_change,
        &mut errors,
    );
    validate_optional_text(
        "ticker",
        request.ticker.as_deref(),
        "Enter the ticker",
        &mut has_change,
        &mut errors,
    );
    validate_optional_text(
        "asset_class",
        request.asset_class.as_deref(),
        "Enter the asset class",
        &mut has_change,
        &mut errors,
    );
    validate_optional_text(
        "currency",
        request.currency.as_deref(),
        "Enter the currency",
        &mut has_change,
        &mut errors,
    );
    validate_optional_decimal(
        "quantity",
        request.quantity,
        "Quantity must be greater than zero",
        &mut has_change,
        &mut errors,
    );
    validate_optional_decimal(
        "average_price",
        request.average_price,
        "Average price must be greater than zero",
        &mut has_change,
        &mut errors,
    );
    validate_optional_decimal(
        "current_price",
        request.current_price,
        "Current price must be greater than zero",
        &mut has_change,
        &mut errors,
    );

    if !has_change {
        errors.push(FieldError::new(
            "asset",
            "Send at least one field to update",
        ));
    }

    finish_validation(errors)
}

fn validate_required(field: &str, value: &str, message: &str, errors: &mut Vec<FieldError>) {
    if value.trim().is_empty() {
        errors.push(FieldError::new(field, message));
    }
}

fn validate_positive(field: &str, value: Decimal, message: &str, errors: &mut Vec<FieldError>) {
    if value <= Decimal::new(0, 0) {
        errors.push(FieldError::new(field, message));
    }
}

fn validate_optional_text(
    field: &str,
    value: Option<&str>,
    message: &str,
    has_change: &mut bool,
    errors: &mut Vec<FieldError>,
) {
    if let Some(value) = value {
        *has_change = true;
        validate_required(field, value, message, errors);
    }
}

fn validate_optional_decimal(
    field: &str,
    value: Option<Decimal>,
    message: &str,
    has_change: &mut bool,
    errors: &mut Vec<FieldError>,
) {
    if let Some(value) = value {
        *has_change = true;
        validate_positive(field, value, message, errors);
    }
}

fn finish_validation(errors: Vec<FieldError>) -> Result<(), AppError> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors))
    }
}
