use std::collections::HashMap;

use crate::{
    models::{author_model::Author, post_model::Post},
    AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Extension,
};
use bson::{doc, oid::ObjectId};
use chrono::Duration;
use futures::{SinkExt, StreamExt, TryStreamExt};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketResponse {
    success: bool,
    data: Option<HashMap<String, Vec<Vec<Post>>>>,
    error_message: Option<String>,
}

pub async fn channel_posts(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), channel_id, author.id))
}

async fn websocket(
    mut _socket: WebSocket,
    state: State<AppState>,
    channel_id: ObjectId,
    user_id: ObjectId,
) {
    let (mut sender, _receiver) = _socket.split();

    if !is_channel_public(channel_id, &state).await
        && !is_user_subscribed(user_id, channel_id, &state).await
    {
        let response_json = match serde_json::to_string(&WebSocketResponse {
            success: false,
            data: None,
            error_message: Some("Unauthorized access".to_string()),
        }) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("Error sending message to websocket client: {:?}", err);
                return;
            }
        };

        if let Err(err) = sender.send(Message::Text(response_json)).await {
            eprintln!("Error sending message to websocket client: {:?}", err);
            return;
        }
    }

    // Retrieve initial posts
    let initial_posts = fetch_posts(state.clone(), channel_id).await;

    let initial_json = match initial_posts {
        Some(posts) => {
            let response = WebSocketResponse {
                success: true,
                data: Some(transform_posts(posts)),
                error_message: None,
            };
            if let Ok(json) = serde_json::to_string(&response) {
                json
            } else {
                eprintln!("Error serializing posts to JSON");
                return;
            }
        }
        None => "[]".to_string(), // Send an empty array if there are no initial posts
    };

    if let Err(err) = sender.send(Message::Text(initial_json)).await {
        eprintln!("Error sending message to websocket client: {:?}", err);
        return;
    }

    // let pipeline = [doc! {"$match": {"channel_id": channel_id}}];

    // Listen for changes in channel posts
    let change_stream = state
        .db
        .posts_collection
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
                    let posts = fetch_posts(state.clone(), channel_id).await;

                    if let Some(posts) = posts {
                        let response = WebSocketResponse {
                            success: true,
                            data: Some(transform_posts(posts)),
                            error_message: None,
                        };
                        let json = serde_json::to_string(&response).map_err(|err| {
                            eprintln!("Error serializing posts: {:?}", err);
                            "Failed to serialize posts".to_string()
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

async fn fetch_posts(state: State<AppState>, channel_id: ObjectId) -> Option<Vec<Post>> {
    let filter = doc! {"channel_id": channel_id};

    let options = FindOptions::builder()
        .limit(20)
        .sort(doc! {"timestamp": -1})
        .build();

    if let Ok(cursor) = state.db.posts_collection.find(filter, options).await {
        if let Ok(posts) = cursor.try_collect().await {
            return Some(posts);
        } else {
            eprintln!("Error collecting posts");
        }
    } else {
        eprintln!("Error finding posts");
    }

    None
}

async fn is_user_subscribed(user_id: ObjectId, channel_id: ObjectId, state: &AppState) -> bool {
    if let Ok(Some(user_channel)) = state
        .db
        .user_channels_collection
        .find_one(doc! {"user_id": user_id, "channel_id": channel_id}, None)
        .await
    {
        return user_channel.subscribed_at.is_some();
    }

    false
}

async fn is_channel_public(channel_id: ObjectId, state: &AppState) -> bool {
    if let Ok(Some(channel)) = state
        .db
        .channels_collection
        .find_one(doc! {"_id": channel_id}, None)
        .await
    {
        return channel.channel_visibility == "Public";
    }

    false
}

fn transform_posts(posts: Vec<Post>) -> HashMap<String, Vec<Vec<Post>>> {
    let mut result: HashMap<String, Vec<Vec<Post>>> = HashMap::new();

    for post in posts {
        let created_date_str = post.created_at.date_naive().to_string();

        // Calculate the interval limit (5 minutes)
        let interval_limit = Duration::minutes(5);

        // Check if there's an existing array for the created date
        let entry = result
            .entry(created_date_str.clone())
            .or_insert_with(Vec::new);

        // Check if there's an existing array for the time interval
        let mut interval_found = false;
        for interval_posts in entry.iter_mut() {
            if let Some(last_post) = interval_posts.last() {
                let time_difference = post.created_at.signed_duration_since(last_post.created_at);
                if time_difference <= interval_limit {
                    interval_posts.push(post.clone());
                    interval_found = true;
                    break;
                }
            }
        }

        if !interval_found {
            // Create a new interval array for this time
            entry.push(vec![post]);
        }
    }

    result
}
