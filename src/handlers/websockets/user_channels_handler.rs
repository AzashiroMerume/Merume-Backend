use crate::{
    models::{channel_model::Channel, user_model::User},
    utils::jwt::Claims,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};
use mongodb::{Client, Collection};

pub async fn channels_handler(
    ws: WebSocketUpgrade,
    State(client): State<Client>,
    Extension(token_info): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = match ObjectId::parse_str(&token_info.sub) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };
    let collection: Collection<User> = client.database("Merume").collection("users");

    if let Ok(Some(_)) = collection.find_one(doc! {"_id": user_id}, None).await {
        // Document with the specified ObjectId exists in the collection
        Ok(ws.on_upgrade(|socket| websocket(socket, State(client), Extension(token_info))))
    } else {
        // Document with the specified ObjectId does not exist in the collection
        return Err(StatusCode::UNAUTHORIZED);
    }
}

async fn websocket(
    mut socket: WebSocket,
    client: State<Client>,
    Extension(token_info): Extension<Claims>,
) {
    // By splitting we can send and receive at the same time.
    let (mut sender, receiver) = socket.split();

    // Retrieve all channels from the "channels" collection
    let collection: Collection<Channel> = client.database("Merume").collection("channels");

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
