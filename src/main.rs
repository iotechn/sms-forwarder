mod config;
mod bark;
mod modem;
mod web;

use config::Config;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("loading config");
    let cfg = Config::load();
    tracing::info!("config loaded, emergency_keywords: {} items", cfg.emergency_keywords.len());

    let app = web::router(cfg.clone());

    tokio::spawn(modem::start(cfg));
    tracing::info!("SMS poll task spawned");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:10086").await.unwrap();
    tracing::info!("listening on 0.0.0.0:10086");
    axum::serve(listener, app).await.unwrap();
}
