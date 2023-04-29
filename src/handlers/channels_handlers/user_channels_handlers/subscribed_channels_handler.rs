use crate::{
    models::{channel_model::Channel, user_channel_model::UserChannel},
    AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    Extension,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};

pub async fn subscribed_channels(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), user_id))
}

async fn websocket(mut _socket: WebSocket, state: State<AppState>, user_id: ObjectId) {
    // By splitting we can send and receive at the same time.
    let (mut sender, _receiver) = _socket.split();

    let user_channels: Vec<UserChannel> = state
        .db
        .user_channels_collection
        .find(doc! {"user_id": user_id, "is_owner": false}, None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    // Retrieve the channels corresponding to the user's subscribed channels
    let channel_ids: Vec<ObjectId> = user_channels
        .iter()
        .map(|uc| uc.channel_id.clone())
        .collect();

    let filter = doc! {"_id": {"$in": channel_ids}};
    let channels: Vec<Channel> = state
        .db
        .channels_collection
        .find(filter.to_owned(), None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    let json = serde_json::to_string(&channels).unwrap();
    sender.send(Message::Text(json)).await.unwrap();

    // Watch for changes in the collection
    let mut change_stream = state
        .db
        .user_channels_collection
        .watch(None, None)
        .await
        .unwrap();

    loop {
        match change_stream.try_next().await {
            Ok(Some(_)) => {
                // A change event occurred in the collection
                let channels: Vec<Channel> = state
                    .db
                    .channels_collection
                    .find(filter.clone(), None)
                    .await
                    .unwrap()
                    .try_collect()
                    .await
                    .unwrap();

                // Send all channels over the WebSocket as a JSON string
                let json = serde_json::to_string(&channels).unwrap();
                sender.send(Message::Text(json)).await.unwrap();
            }
            Ok(None) | Err(_) => {
                break;
            }
        }
    }
}
