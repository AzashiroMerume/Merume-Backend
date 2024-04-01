use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};
use mongodb::{change_stream::event::OperationType, options::ChangeStreamOptions};
use serde::{Deserialize, Serialize};

use crate::{models::post_model::Post, AppState};

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketResponse {
    operation_type: OperationType,
    post: Option<Post>,
    post_id: Option<ObjectId>,
}

pub async fn all_channels_updates(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), user_id))
}

async fn websocket(socket: WebSocket, state: State<AppState>, user_id: ObjectId) {
    let (mut sender, _receiver) = socket.split();

    let pipeline = vec![doc! {
        "$match": {
            "author.id": user_id
        }
    }];

    let change_stream = state
        .db
        .posts_collection
        .watch(pipeline, None)
        .await
        .map_err(|err| {
            eprintln!("Error creating change stream: {:?}", err);
            "Failed to create change stream".to_string()
        });

    if let Ok(mut change_stream) = change_stream {
        while change_stream.is_alive() {
            match change_stream.try_next().await {
                Ok(Some(change_event)) => match change_event.operation_type {
                    OperationType::Insert => {
                        let response = WebSocketResponse {
                            operation_type: OperationType::Insert,
                            post: change_event.full_document,
                            post_id: None,
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            if let Err(err) = sender.send(Message::Text(json)).await {
                                eprintln!("Error sending message to websocket client: {:?}", err);
                                break;
                            }
                        } else {
                            eprintln!("Error serializing response to JSON");
                            break;
                        }
                    }
                    OperationType::Delete => {
                        let post_id = change_event
                            .document_key
                            .unwrap()
                            .get_object_id("_id")
                            .unwrap();
                        let response = WebSocketResponse {
                            operation_type: OperationType::Delete,
                            post: None,
                            post_id: Some(post_id),
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            if let Err(err) = sender.send(Message::Text(json)).await {
                                eprintln!("Error sending message to websocket client: {:?}", err);
                                break;
                            }
                        } else {
                            eprintln!("Error serializing response to JSON");
                            break;
                        }
                    }
                    _ => {}
                },
                Ok(None) => {
                    eprintln!("Broke in none");
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
