use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, AssetMovement, CreateAssetRequest, UpdateAssetRequest, UserRecord},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self, user_id: i64) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as::<_, Asset>(
            "SELECT id,
                    user_id,
                    name,
                    ticker,
                    asset_class,
                    quantity,
                    average_price,
                    current_price,
                    currency,
                    quantity * current_price AS total_value
             FROM assets
             WHERE user_id = $1
             ORDER BY name;",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
    }

    pub async fn get_asset(&self, user_id: i64, asset_id: i64) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as::<_, Asset>(
            "SELECT id,
                    user_id,
                    name,
                    ticker,
                    asset_class,
                    quantity,
                    average_price,
                    current_price,
                    currency,
                    quantity * current_price AS total_value
             FROM assets
             WHERE user_id = $1 AND id = $2;",
        )
        .bind(user_id)
        .bind(asset_id)
        .fetch_optional(&self.db)
        .await
    }

    pub async fn list_asset_movements(
        &self,
        user_id: i64,
        limit: i64,
    ) -> sqlx::Result<Vec<AssetMovement>> {
        sqlx::query_as::<_, AssetMovement>(
            "SELECT id,
                    user_id,
                    asset_id,
                    ticker,
                    asset_name,
                    movement_type,
                    quantity,
                    unit_price,
                    currency,
                    quantity * unit_price AS total_value,
                    to_char(created_at, 'YYYY-MM-DD HH24:MI') AS occurred_at
             FROM asset_movements
             WHERE user_id = $1
             ORDER BY created_at DESC, id DESC
             LIMIT $2;",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(
        &self,
        user_id: i64,
        request: &CreateAssetRequest,
    ) -> sqlx::Result<Asset> {
        sqlx::query_as::<_, Asset>(
            "INSERT INTO assets (
                user_id,
                name,
                ticker,
                asset_class,
                quantity,
                average_price,
                current_price,
                currency
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id,
                       user_id,
                       name,
                       ticker,
                       asset_class,
                       quantity,
                       average_price,
                       current_price,
                       currency,
                       quantity * current_price AS total_value;",
        )
        .bind(user_id)
        .bind(request.name.trim())
        .bind(request.ticker.trim().to_uppercase())
        .bind(request.asset_class.trim())
        .bind(request.quantity)
        .bind(request.average_price)
        .bind(request.current_price)
        .bind(request.currency.trim().to_uppercase())
        .fetch_one(&self.db)
        .await
    }

    pub async fn create_asset_movement(
        &self,
        user_id: i64,
        asset_id: i64,
        ticker: &str,
        asset_name: &str,
        movement_type: &str,
        quantity: rust_decimal::Decimal,
        unit_price: rust_decimal::Decimal,
        currency: &str,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO asset_movements (
                user_id,
                asset_id,
                ticker,
                asset_name,
                movement_type,
                quantity,
                unit_price,
                currency
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8);",
        )
        .bind(user_id)
        .bind(asset_id)
        .bind(ticker)
        .bind(asset_name)
        .bind(movement_type)
        .bind(quantity)
        .bind(unit_price)
        .bind(currency)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        request: &UpdateAssetRequest,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as::<_, Asset>(
            "UPDATE assets
             SET name = COALESCE($3, name),
                 ticker = COALESCE($4, ticker),
                 asset_class = COALESCE($5, asset_class),
                 quantity = COALESCE($6, quantity),
                 average_price = COALESCE($7, average_price),
                 current_price = COALESCE($8, current_price),
                 currency = COALESCE($9, currency)
             WHERE id = $1 AND user_id = $2
             RETURNING id,
                       user_id,
                       name,
                       ticker,
                       asset_class,
                       quantity,
                       average_price,
                       current_price,
                       currency,
                       quantity * current_price AS total_value;",
        )
        .bind(asset_id)
        .bind(user_id)
        .bind(request.name.as_deref().map(str::trim))
        .bind(
            request
                .ticker
                .as_ref()
                .map(|ticker| ticker.trim().to_uppercase()),
        )
        .bind(request.asset_class.as_deref().map(str::trim))
        .bind(request.quantity)
        .bind(request.average_price)
        .bind(request.current_price)
        .bind(
            request
                .currency
                .as_ref()
                .map(|currency| currency.trim().to_uppercase()),
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn add_user(&self, email: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as::<_, UserRecord>(
            "INSERT INTO users (email, password_hash)
             VALUES ($1, $2)
             RETURNING id, email, password_hash;",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_email(&self, email: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as::<_, UserRecord>(
            "SELECT id, email, password_hash
             FROM users
             WHERE email = $1;",
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
