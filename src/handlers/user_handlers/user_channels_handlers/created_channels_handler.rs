use std::sync::Arc;

use crate::{
    models::{author_model::Author, channel_model::Channel},
    AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, State,
    },
    response::IntoResponse,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};

pub async fn created_channels(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), author.id))
}

async fn websocket(mut _socket: WebSocket, state: State<Arc<AppState>>, user_id: ObjectId) {
    let (mut sender, _receiver) = _socket.split();

    // Retrieve initial channels
    let channels = fetch_channels(state.clone(), user_id).await;

    if let Some(channels) = channels {
        let json = serde_json::to_string(&channels).map_err(|err| {
            eprintln!("Error serializing channels: {:?}", err);
            "Failed to serialize channels".to_string()
        });

        if let Ok(json) = json {
            if let Err(err) = sender.send(Message::Text(json)).await {
                eprintln!("Error sending message to websocket client: {:?}", err);
                return;
            }
        }
    }

    // Listen for changes in channels
    let change_stream = state
        .db
        .channels_collection
        .watch(
            /* vec![doc! {"$match": {"owner_id": user_id}}] */ None, None,
        )
        .await
        .map_err(|err| {
            eprintln!("Error creating change stream: {:?}", err);
            "Failed to create change stream".to_string()
        });

    if let Ok(mut change_stream) = change_stream {
        while change_stream.is_alive() {
            match change_stream.try_next().await {
                Ok(Some(_)) => {
                    let channels = fetch_channels(state.clone(), user_id).await;

                    if let Some(channels) = channels {
                        let json = serde_json::to_string(&channels).map_err(|err| {
                            eprintln!("Error serializing channels: {:?}", err);
                            "Failed to serialize channels".to_string()
                        });

                        if let Ok(json) = json {
                            if let Err(err) = sender.send(Message::Text(json)).await {
                                eprintln!("Error sending message to websocket client: {:?}", err);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                Ok(None) => {
                    break;
                }
                Err(err) => {
                    eprintln!("Error reading change stream: {:?}", err);
                    break;
                }
            }
        }
    }
}

async fn fetch_channels(state: State<Arc<AppState>>, user_id: ObjectId) -> Option<Vec<Channel>> {
    let filter = doc! {"author.id": user_id};

    let channels_result = state
        .db
        .channels_collection
        .find(filter.to_owned(), None)
        .await
        .map_err(|err| {
            eprintln!("Error finding channels: {:?}", err);
            "Failed to fetch channels".to_string()
        });

    if let Ok(cursor) = channels_result {
        if let Ok(channels) = cursor.try_collect().await.map_err(|err| {
            eprintln!("Error collecting channels: {:?}", err);
            "Failed to fetch channels".to_string()
        }) {
            return Some(channels);
        }
    }

    None
}
