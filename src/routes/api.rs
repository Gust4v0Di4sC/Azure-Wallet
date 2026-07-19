use axum::{
    Json, Router,
    extract::Path,
    routing::{get, patch},
};

use crate::{
    app::AppState,
    auth::user::User,
    error::AppError,
    models::{Asset, CreateAssetRequest, PortfolioSummary, UpdateAssetRequest},
    repository::Repository,
    services::portfolio::PortfolioService,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/assets", get(list_assets).post(create_asset))
        .route("/assets/{asset_id}", patch(update_asset))
}

#[tracing::instrument(skip_all)]
async fn list_assets(
    user: User,
    repository: Repository,
) -> Result<Json<PortfolioSummary>, AppError> {
    let portfolio = PortfolioService::new(repository).summary(&user).await?;
    Ok(Json(portfolio))
}

#[tracing::instrument(skip_all)]
async fn create_asset(
    user: User,
    repository: Repository,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let new_asset = PortfolioService::new(repository)
        .create_asset(&user, request)
        .await?;

    Ok(Json(new_asset))
}

#[tracing::instrument(skip_all)]
async fn update_asset(
    user: User,
    repository: Repository,
    Path(asset_id): Path<i64>,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let updated_asset = PortfolioService::new(repository)
        .update_asset(&user, asset_id, request)
        .await?;

    Ok(Json(updated_asset))
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use sqlx::PgPool;

    use super::*;
    use crate::models::AuthForm;
    use crate::services::auth::AuthService;

    fn decimal(value: i64) -> Decimal {
        Decimal::new(value, 0)
    }

    async fn create_test_user(db: PgPool) -> User {
        let form = AuthForm {
            email: "user@example.com".to_string(),
            password: "secret1".to_string(),
        };

        AuthService::new(db.into())
            .register(form)
            .await
            .expect("user registered")
    }

    #[sqlx::test]
    async fn test_register_and_login(db: PgPool) {
        let repository: Repository = db.into();
        let form = AuthForm {
            email: "user@example.com".to_string(),
            password: "secret1".to_string(),
        };

        let service = AuthService::new(repository);
        let user = service.register(form.clone()).await.expect("registered");
        let logged_user = service.login(form).await.expect("logged in");

        assert_eq!(user.id(), logged_user.id());
        assert_eq!(logged_user.email(), "user@example.com");
    }

    #[sqlx::test]
    async fn test_reject_short_password(db: PgPool) {
        let form = AuthForm {
            email: "user@example.com".to_string(),
            password: "123".to_string(),
        };

        let err = AuthService::new(db.into())
            .register(form)
            .await
            .expect_err("validation failed");

        assert_eq!(err.code(), "validation_error");
    }

    #[sqlx::test]
    async fn test_create_asset(db: PgPool) {
        let user = create_test_user(db.clone()).await;
        let request = CreateAssetRequest {
            name: "Bitcoin".to_string(),
            ticker: "btc".to_string(),
            asset_class: "crypto".to_string(),
            quantity: decimal(2),
            average_price: decimal(5),
            current_price: decimal(10),
            currency: "usd".to_string(),
        };

        let Json(new_asset) = create_asset(user, db.into(), Json(request))
            .await
            .expect("success");

        assert_eq!(new_asset.id, 1);
        assert_eq!(new_asset.name, "Bitcoin");
        assert_eq!(new_asset.ticker, "BTC");
        assert_eq!(new_asset.total_value, decimal(20));

        insta::assert_json_snapshot!(new_asset);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_list_assets(db: PgPool) {
        let user = User::new(1, "user@example.com".to_string());
        let Json(summary) = list_assets(user, db.into()).await.expect("success");

        assert_eq!(summary.assets.len(), 1);
        assert_eq!(summary.assets[0].name, "Bitcoin");
        assert_eq!(summary.total_value, decimal(20));

        insta::assert_json_snapshot!(summary);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_update_asset(db: PgPool) {
        let user = User::new(1, "user@example.com".to_string());
        let request = UpdateAssetRequest {
            name: Some("Ethereum".to_string()),
            ticker: Some("eth".to_string()),
            asset_class: Some("crypto".to_string()),
            quantity: Some(decimal(3)),
            average_price: Some(decimal(7)),
            current_price: Some(decimal(20)),
            currency: Some("usd".to_string()),
        };

        let Json(updated_asset) = update_asset(user, db.into(), Path(1), Json(request))
            .await
            .expect("success");

        assert_eq!(updated_asset.id, 1);
        assert_eq!(updated_asset.name, "Ethereum");
        assert_eq!(updated_asset.ticker, "ETH");
        assert_eq!(updated_asset.total_value, decimal(60));

        insta::assert_json_snapshot!(updated_asset);
    }
}
