use axum::{extract::State, routing::{get, post}, Json, Router};

use crate::config::Config;
use crate::modem;

pub fn router(cfg: Config) -> Router {
    Router::new()
        .route("/config", get(get_cfg))
        .route("/config", post(set_cfg))
        .route("/send", post(send_sms))
        .with_state(cfg)
}

async fn get_cfg(State(cfg): State<Config>) -> Json<Config> {
    tracing::debug!("GET /config");
    Json(cfg)
}

async fn set_cfg(State(_cfg): State<Config>, Json(new): Json<Config>) {
    tracing::info!("POST /config, saving");
    new.save();
}

async fn send_sms(State(cfg): State<Config>, Json((number, text)): Json<(String, String)>) {
    tracing::info!(number = %number, "接收到发送的命令");
    modem::send_sms(&cfg, &number, &text).await;
}
