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

async fn websocket(mut _socket: WebSocket, client: State<Client>, user_id: ObjectId) {
    let (mut sender, _receiver) = _socket.split();

    let channels_col: Collection<Channel> = client.database("Merume").collection("channels");

    let filter = doc! {"owner_id": user_id};

    let channels_result = channels_col
        .find(filter.to_owned(), None)
        .await
        .map_err(|e| {
            eprintln!("Error finding channels: {:?}", e);
            "Failed to fetch channels".to_string()
        });

    let channels: Vec<Channel> = match channels_result {
        Ok(channels) => channels.try_collect().await.map_err(|e| {
            eprintln!("Error collecting channels: {:?}", e);
            "Failed to fetch channels".to_string()
        }),
        Err(e) => Err(e),
    }
    .unwrap();

    let json = serde_json::to_string(&channels).map_err(|e| {
        eprintln!("Error serializing channels: {:?}", e);
        "Failed to serialize channels".to_string()
    });

    if let Ok(json) = json {
        if let Err(e) = sender.send(Message::Text(json)).await {
            eprintln!("Error sending message to websocket client: {:?}", e);
        }
    }

    let change_stream = channels_col.watch(None, None).await.map_err(|e| {
        eprintln!("Error creating change stream: {:?}", e);
        "Failed to create change stream".to_string()
    });

    if let Ok(mut change_stream) = change_stream {
        loop {
            match change_stream.try_next().await {
                Ok(Some(_)) => {
                    let channels_result =
                        channels_col.find(filter.clone(), None).await.map_err(|e| {
                            eprintln!("Error finding channels: {:?}", e);
                            "Failed to fetch channels".to_string()
                        });

                    let channels: Vec<Channel> = match channels_result {
                        Ok(channels) => channels.try_collect().await.map_err(|e| {
                            eprintln!("Error collecting channels: {:?}", e);
                            "Failed to fetch channels".to_string()
                        }),
                        Err(e) => Err(e),
                    }
                    .unwrap();

                    let json = serde_json::to_string(&channels).map_err(|e| {
                        eprintln!("Error serializing channels: {:?}", e);
                        "Failed to serialize channels".to_string()
                    });

                    if let Ok(json) = json {
                        if let Err(e) = sender.send(Message::Text(json)).await {
                            eprintln!("Error sending message to websocket client: {:?}", e);
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading change stream: {:?}", e);
                    break;
                }
            }
        }
    }
}
