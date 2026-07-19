use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct Asset {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub ticker: String,
    pub asset_class: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub current_price: Decimal,
    pub currency: String,
    pub total_value: Decimal,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateAssetRequest {
    pub name: String,
    pub ticker: String,
    pub asset_class: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub current_price: Decimal,
    pub currency: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub asset_class: Option<String>,
    pub quantity: Option<Decimal>,
    pub average_price: Option<Decimal>,
    pub current_price: Option<Decimal>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PortfolioSummary {
    pub assets: Vec<Asset>,
    pub total_value: Decimal,
}

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct AssetMovement {
    pub id: i64,
    pub user_id: i64,
    pub asset_id: i64,
    pub ticker: String,
    pub asset_name: String,
    pub movement_type: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub currency: String,
    pub total_value: Decimal,
    pub occurred_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthForm {
    pub email: String,
    pub password: String,
}
