use crate::AppState;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Extension, State,
    },
    response::Response,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use std::sync::Arc;

pub async fn heartbeat(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<ObjectId>,
) -> Response {
    ws.on_upgrade(move |socket| websocket(socket, State(state), user_id))
}

async fn websocket(mut socket: WebSocket, state: State<Arc<AppState>>, user_id: ObjectId) {
    set_user_online(user_id, &state).await;

    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            break;
        };

        if socket.send(msg).await.is_err() {
            break;
        }
    }

    set_user_offline(user_id, &state).await;
}

async fn set_user_online(user_id: ObjectId, state: &State<Arc<AppState>>) {
    let now = Utc::now().to_rfc3339();
    let update_result = state
        .db
        .users_collection
        .update_one(
            doc! {"_id": user_id},
            doc! {
                "$set": {
                    "last_time_online": now,
                    "is_online": true,
                }
            },
            None,
        )
        .await;

    if let Err(err) = update_result {
        eprintln!("Error updating user online status: {:?}", err);
        // Handle the error appropriately, e.g., log it or send a notification
    }
}

async fn set_user_offline(user_id: ObjectId, state: &State<Arc<AppState>>) {
    let update_result = state
        .db
        .users_collection
        .update_one(
            doc! {"_id": user_id},
            doc! {"$set": {"is_online": false}},
            None,
        )
        .await;

    if let Err(err) = update_result {
        eprintln!("Error updating user online status: {:?}", err);
        // Handle the error appropriately, e.g., log it or send a notification
    }
}
