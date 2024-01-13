use crate::AppState;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Extension, State,
    },
    response::IntoResponse,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;

pub async fn heartbeat(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), user_id))
}

async fn websocket(mut socket: WebSocket, state: State<AppState>, user_id: ObjectId) {
    set_user_online(user_id, &state).await;

    loop {
        if let Some(msg) = socket.recv().await {
            let msg = if let Ok(msg) = msg {
                msg
            } else {
                break;
            };

            if socket.send(msg).await.is_err() {
                break;
            }
        } else {
            break;
        }
    }

    set_user_offline(user_id, &state).await;
}

async fn set_user_online(user_id: ObjectId, state: &State<AppState>) {
    let now = Utc::now();
    let update_result = state
        .db
        .users_collection_bson
        .update_one(
            doc! {"_id": user_id},
            doc! {
                "$set": {
                    "last_time_online": bson::DateTime::from_chrono(now),
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

async fn set_user_offline(user_id: ObjectId, state: &State<AppState>) {
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
