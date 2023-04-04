use crate::models::{channel_model::Channel, user_channel_model::UserChannel};
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
use mongodb::{Client, Collection};

pub async fn subscribed_channels(
    ws: WebSocketUpgrade,
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(client), user_id))
}

async fn websocket(mut socket: WebSocket, client: State<Client>, user_id: ObjectId) {
    // By splitting we can send and receive at the same time.
    let (mut sender, receiver) = socket.split();

    let user_channels_col: Collection<UserChannel> =
        client.database("Merume").collection("user_channels");

    let user_channels: Vec<UserChannel> = user_channels_col
        .find(doc! {"user_id": user_id, "is_owner": false}, None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    // Retrieve the channels corresponding to the user's subscribed channels
    let channel_ids: Vec<ObjectId> = user_channels
        .iter()
        .map(|uc| uc.channel_id.clone().unwrap())
        .collect();
    let channels_col: Collection<Channel> = client.database("Merume").collection("channels");
    let filter = doc! {"_id": {"$in": channel_ids}};
    let channels: Vec<Channel> = channels_col
        .find(filter.to_owned(), None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    let json = serde_json::to_string(&channels).unwrap();
    sender.send(Message::Text(json)).await.unwrap();

    // Watch for changes in the collection
    let mut change_stream = channels_col.watch(None, None).await.unwrap();

    loop {
        match change_stream.try_next().await {
            Ok(Some(change_event)) => {
                // A change event occurred in the collection
                let channels: Vec<Channel> = channels_col
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
