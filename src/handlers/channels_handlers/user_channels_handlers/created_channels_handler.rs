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

pub async fn created_channels(
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
        Ok(ws.on_upgrade(move |socket| websocket(socket, State(client), user_id)))
    } else {
        // Document with the specified ObjectId does not exist in the collection
        return Err(StatusCode::UNAUTHORIZED);
    }
}

async fn websocket(mut socket: WebSocket, client: State<Client>, user_id: ObjectId) {
    let (mut sender, receiver) = socket.split();

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
