use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use serde::Serialize;

pub async fn send_response<T>(sender: &mut SplitSink<WebSocket, Message>, response: T)
where
    T: Serialize,
{
    if let Ok(json) = serde_json::to_string(&response) {
        if let Err(err) = sender.send(Message::Text(json)).await {
            eprintln!("Error sending message to websocket client: {:?}", err);
        }
    } else {
        eprintln!("Error serializing response to JSON");
    }
}
