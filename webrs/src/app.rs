use crate::config;
use axum::Router;
use sqlx::PgPool;

pub mod auth;
pub mod common;
mod database;
pub mod enumeration;
pub mod error;
pub mod id;
pub mod json;
mod latency;
mod logger;
pub mod middleware;
pub mod path;
pub mod query;
pub mod response;
pub mod serde;
mod server;
pub mod utils;
pub mod valid;
pub mod validation;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

pub async fn run(router: Router<AppState>) -> anyhow::Result<()> {
    logger::init();
    id::init()?;
    tracing::info!("Starting app server...");

    let db = database::init().await?;
    let state = AppState::new(db);
    let server = server::Server::new(config::get().server());

    server.start(state, router).await
}
