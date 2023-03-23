use crate::models::channels_model::Channel;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt, TryStreamExt};
use mongodb::Client;

pub async fn channels_handler(
    ws: WebSocketUpgrade,
    State(client): State<Client>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, State(client)))
}

async fn websocket(mut socket: WebSocket, state: State<Client>) {
    // By splitting we can send and receive at the same time.
    let (mut sender, receiver) = socket.split();

    // Retrieve all channels from the "channels" collection
    let collection = state.database("Merume").collection("channels");

    // Send the initial data over the WebSocket as a JSON string
    let channels: Vec<Channel> = collection
        .find(None, None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();
    let json = serde_json::to_string(&channels).unwrap();
    sender.send(Message::Text(json)).await.unwrap();

    // Watch for changes in the collection
    let mut change_stream = collection.watch(None, None).await.unwrap();

    loop {
        match change_stream.try_next().await {
            Ok(Some(change_event)) => {
                // A change event occurred in the collection
                let channels: Vec<Channel> = collection
                    .find(None, None)
                    .await
                    .unwrap()
                    .try_collect()
                    .await
                    .unwrap();

                // Send all channels over the WebSocket as a JSON string
                let json = serde_json::to_string(&channels).unwrap();
                sender.send(Message::Text(json)).await.unwrap();
            }
            Ok(None) => {
                // The change stream has been closed
                break;
            }
            Err(_) => {
                // An error occurred in the change stream
                break;
            }
        }
    }
}
