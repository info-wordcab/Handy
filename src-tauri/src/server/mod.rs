use axum::{
    extract::ws::{WebSocketUpgrade},
    extract::State,
    response::{Json, Response},
    routing::{get, Router},
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower::ServiceBuilder;

pub mod websocket;

#[derive(Clone, Debug, Serialize)]
pub struct TranscriptionMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    pub timestamp: u64,
    pub model: Option<String>,
    pub pii_redaction_applied: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct StatusMessage {
    pub status: String,
}

pub type AppState = Arc<ServerState>;

pub struct ServerState {
    pub tx: broadcast::Sender<TranscriptionMessage>,
}

impl ServerState {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast_transcription(&self, message: TranscriptionMessage) {
        let _ = self.tx.send(message);
    }
}

pub async fn start_server(port: u16) -> Result<(AppState, tokio::task::JoinHandle<()>), Box<dyn std::error::Error>> {
    let state = Arc::new(ServerState::new());

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/ws", get(websocket_handler))
        .with_state(state.clone())
        .layer(ServiceBuilder::new());

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    log::info!("API server listening on http://127.0.0.1:{}", port);

    let server_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            log::error!("Server error: {}", e);
        }
    });

    Ok((state, server_task))
}

async fn health_handler() -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "ok".to_string(),
    })
}

async fn status_handler() -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "idle".to_string(),
    })
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| websocket::handle_socket(socket, state))
}