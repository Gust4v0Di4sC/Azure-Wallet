use axum::{Router, extract::FromRef};
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cookie_key: Key,
    pub auth: AuthConfig,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: Vec<u8>,
    pub cookie_secure: bool,
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        let database_url = required_env("DATABASE_URL")?;
        let cookie_key = required_env("AUTH_COOKIE_KEY")?;
        let jwt_secret = required_env("JWT_SECRET")?;
        let cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|value| value == "true" || value == "1")
            .unwrap_or(true);

        if cookie_key.as_bytes().len() < 64 {
            color_eyre::eyre::bail!("AUTH_COOKIE_KEY must have at least 64 bytes");
        }

        let db = PgPool::connect(&database_url).await?;
        let mut migrator = sqlx::migrate!();
        migrator.set_ignore_missing(true);
        migrator.run(&db).await?;

        Ok(Self {
            db,
            cookie_key: Key::from(cookie_key.as_bytes()),
            auth: AuthConfig {
                jwt_secret: jwt_secret.into_bytes(),
                cookie_secure,
            },
        })
    }
}

fn required_env(key: &str) -> color_eyre::Result<String> {
    std::env::var(key).map_err(|_| {
        color_eyre::eyre::eyre!("missing required environment variable {key}; check your .env file")
    })
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!("failed to listen for shutdown signal: {err}");
        return;
    }

    tracing::info!("Shutdown signal received; stopping service");
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        dotenvy::dotenv()?;
        let state = AppState::new().await?;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .merge(routes::frontend::router())
            .with_state(state);

        info!("Starting service at http://localhost:3000");
        info!("Open the login page at http://localhost:3000/login");

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}
