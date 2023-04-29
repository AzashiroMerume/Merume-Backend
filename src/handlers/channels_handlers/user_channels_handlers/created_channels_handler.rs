use crate::{models::channel_model::Channel, AppState};
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

pub async fn created_channels(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), user_id))
}

async fn websocket(mut _socket: WebSocket, state: State<AppState>, user_id: ObjectId) {
    let (mut sender, _receiver) = _socket.split();

    let filter = doc! {"owner_id": user_id};

    let channels_result = state
        .db
        .channels_collection
        .find(filter.to_owned(), None)
        .await
        .map_err(|err| {
            eprintln!("Error finding channels: {:?}", err);
            "Failed to fetch channels".to_string()
        });

    let channels: Vec<Channel> = match channels_result {
        Ok(channels) => channels.try_collect().await.map_err(|err| {
            eprintln!("Error collecting channels: {:?}", err);
            "Failed to fetch channels".to_string()
        }),
        Err(err) => Err(err),
    }
    .unwrap();

    let json = serde_json::to_string(&channels).map_err(|err| {
        eprintln!("Error serializing channels: {:?}", err);
        "Failed to serialize channels".to_string()
    });

    if let Ok(json) = json {
        if let Err(err) = sender.send(Message::Text(json)).await {
            eprintln!("Error sending message to websocket client: {:?}", err);
        }
    }

    let change_stream = state
        .db
        .channels_collection
        .watch(None, None)
        .await
        .map_err(|err| {
            eprintln!("Error creating change stream: {:?}", err);
            "Failed to create change stream".to_string()
        });

    if let Ok(mut change_stream) = change_stream {
        loop {
            match change_stream.try_next().await {
                Ok(Some(_)) => {
                    let channels_result = state
                        .db
                        .channels_collection
                        .find(filter.clone(), None)
                        .await
                        .map_err(|err| {
                            eprintln!("Error finding channels: {:?}", err);
                            "Failed to fetch channels".to_string()
                        });

                    let channels: Vec<Channel> = match channels_result {
                        Ok(channels) => channels.try_collect().await.map_err(|err| {
                            eprintln!("Error collecting channels: {:?}", err);
                            "Failed to fetch channels".to_string()
                        }),
                        Err(err) => Err(err),
                    }
                    .unwrap();

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
