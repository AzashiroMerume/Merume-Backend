use crate::{models::post_model::Post, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};
use serde_json::json;

pub async fn channel_posts(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), channel_id))
}

async fn websocket(mut _socket: WebSocket, state: State<AppState>, channel_id: ObjectId) {
    let (mut sender, _receiver) = _socket.split();

    // Retrieve initial posts
    let initial_posts = fetch_initial_posts(state.clone(), channel_id).await;

    for post in initial_posts {
        let json = json!(post);

        if let Ok(json_str) = serde_json::to_string(&json) {
            if let Err(err) = sender.send(Message::Text(json_str)).await {
                eprintln!("Error sending message to websocket client: {:?}", err);
                return;
            }
        } else {
            eprintln!("Error serializing post to JSON");
            return;
        }
    }

    // Listen for changes in channel posts
    let change_stream = state
        .db
        .posts_collection
        .watch(Some(doc! {"channel_id": channel_id}), None)
        .await
        .map_err(|err| {
            eprintln!("Error creating change stream: {:?}", err);
            "Failed to create change stream".to_string()
        });

    if let Ok(mut change_stream) = change_stream {
        while let Some(change_event) = change_stream.next().await {
            if let Ok(change_event) = change_event {
                if let Some(post) = change_event.full_document {
                    if post.channel_id == channel_id {
                        let json = json!(post);

                        if let Ok(json_str) = serde_json::to_string(&json) {
                            if let Err(err) = sender.send(Message::Text(json_str)).await {
                                eprintln!("Error sending message to websocket client: {:?}", err);
                                break;
                            }
                        } else {
                            eprintln!("Error serializing post to JSON");
                            break;
                        }
                    }
                }
            } else {
                eprintln!("Error reading change stream");
                break;
            }
        }
    }
}

async fn fetch_initial_posts(state: State<AppState>, channel_id: ObjectId) -> Vec<Post> {
    let filter = doc! {"channel_id": channel_id};

    let options = mongodb::options::FindOptions::builder()
        .sort(doc! {"timestamp": -1})
        .limit(20)
        .build();

    if let Ok(cursor) = state.db.posts_collection.find(filter, options).await {
        if let Ok(posts) = cursor.try_collect().await {
            return posts;
        } else {
            eprintln!("Error collecting posts");
        }
    } else {
        eprintln!("Error finding posts");
    }

    vec![]
}
