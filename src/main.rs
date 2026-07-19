use crate::app::App;

mod app;
pub mod auth;
pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod services;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    App::start().await
}
