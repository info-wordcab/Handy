use axum::extract::ws::{Message, WebSocket};
use futures_util::{sink::SinkExt, stream::StreamExt};

use super::{AppState, TranscriptionMessage};

pub async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let client_id = uuid::Uuid::new_v4().to_string();
    log::info!("WebSocket client connected: {}", client_id);

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json_msg = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    log::error!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json_msg)).await.is_err() {
                log::info!("Client disconnected during send");
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(_)) => {
                    // We don't expect any messages from the client for now
                    // But we can handle them here if needed in the future
                }
                Ok(Message::Binary(_)) => {
                    // Handle binary messages if needed
                }
                Ok(Message::Close(_)) => {
                    log::info!("Client sent close message");
                    break;
                }
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {
            log::info!("Send task completed for client: {}", client_id);
        }
        _ = recv_task => {
            log::info!("Receive task completed for client: {}", client_id);
        }
    }

    log::info!("WebSocket client disconnected: {}", client_id);
}