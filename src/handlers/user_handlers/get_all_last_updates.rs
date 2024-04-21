use crate::{models::post_model::Post, utils::websocket_helpers::send_response, AppState};
use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
    Extension,
};
use bson::{doc, oid::ObjectId};
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    change_stream::event::OperationType,
    options::{
        ChangeStreamOptions,
        FullDocumentType::{self},
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketResponse {
    operation_type: OperationType,
    post: Option<Post>,
    post_id: Option<ObjectId>,
}

pub async fn all_last_updates(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<ObjectId>,
) -> Response {
    ws.on_upgrade(move |socket| websocket(socket, State(state), user_id))
}

async fn websocket(socket: WebSocket, state: State<Arc<AppState>>, user_id: ObjectId) {
    let (mut sender, _receiver) = socket.split();

    let pipeline = vec![doc! {
        "$match": {
            "$or": [
                {"fullDocument.author.id": user_id},
                {"fullDocumentBeforeChange.author.id": user_id},
                {"updateDescription.updatedFields.author.id": user_id},
            ]
        }
    }];

    // Define the options to include all operation types and full document for updates
    let options = ChangeStreamOptions::builder()
        .full_document(Some(FullDocumentType::UpdateLookup))
        .full_document_before_change(Some(
            mongodb::options::FullDocumentBeforeChangeType::WhenAvailable,
        ))
        .build();

    let change_stream = state
        .db
        .posts_collection
        .watch(pipeline, Some(options))
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
                        send_response(&mut sender, response).await;
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
                        send_response(&mut sender, response).await;
                    }
                    OperationType::Update => {
                        let response = WebSocketResponse {
                            operation_type: OperationType::Update,
                            post: change_event.full_document,
                            post_id: None,
                        };
                        send_response(&mut sender, response).await;
                    }
                    _ => {}
                },
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
