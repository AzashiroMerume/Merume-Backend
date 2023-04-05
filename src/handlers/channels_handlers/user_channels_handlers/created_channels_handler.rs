use crate::models::channel_model::Channel;
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

pub async fn created_channels(
    ws: WebSocketUpgrade,
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(client), user_id))
}

async fn websocket(mut socket: WebSocket, client: State<Client>, user_id: ObjectId) {
    let (mut sender, _receiver) = socket.split();

    let channels_col: Collection<Channel> = client.database("Merume").collection("channels");

    let filter = doc! {"owner_id": user_id};

    let channels: Vec<Channel> = channels_col
        .find(filter.to_owned(), None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    let json = serde_json::to_string(&channels).unwrap();
    sender.send(Message::Text(json)).await.unwrap();

    let mut change_stream = channels_col.watch(None, None).await.unwrap();

    loop {
        match change_stream.try_next().await {
            Ok(Some(_)) => {
                let channels: Vec<Channel> = channels_col
                    .find(filter.clone(), None)
                    .await
                    .unwrap()
                    .try_collect()
                    .await
                    .unwrap();

                let json = serde_json::to_string(&channels).unwrap();
                sender.send(Message::Text(json)).await.unwrap();
            }
            Ok(None) | Err(_) => {
                break;
            }
        }
    }
}
