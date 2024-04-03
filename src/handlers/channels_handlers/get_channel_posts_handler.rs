use crate::{
    models::{author_model::Author, components::channel_enums::Visibility, post_model::Post},
    utils::websocket_helpers::send_response,
    AppState,
};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Extension,
};
use bson::{doc, oid::ObjectId};
use chrono::TimeDelta;
use futures::{StreamExt, TryStreamExt};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketResponse {
    success: bool,
    data: Option<BTreeMap<String, Vec<Vec<Post>>>>,
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
        send_response(
            &mut sender,
            WebSocketResponse {
                success: false,
                data: None,
                error_message: Some("Unauthorized access".to_string()),
            },
        )
        .await;
        return;
    }

    // Retrieve initial posts
    let initial_posts = fetch_posts(state.clone(), channel_id).await;

    let initial_response = match initial_posts {
        Some(posts) => match transform_posts(posts) {
            Ok(transformed_posts) => WebSocketResponse {
                success: true,
                data: Some(transformed_posts),
                error_message: None,
            },
            Err(err) => {
                eprintln!("Error transforming posts: {:?}", err);
                return;
            }
        },
        None => WebSocketResponse {
            success: true,
            data: Some(BTreeMap::new()),
            error_message: None,
        },
    };

    send_response(&mut sender, initial_response).await;

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
        while change_stream.is_alive() {
            match change_stream.try_next().await {
                Ok(Some(_)) => {
                    let posts = fetch_posts(state.clone(), channel_id).await;

                    if let Some(posts) = posts {
                        match transform_posts(posts) {
                            Ok(transformed_posts) => {
                                send_response(
                                    &mut sender,
                                    WebSocketResponse {
                                        success: true,
                                        data: Some(transformed_posts),
                                        error_message: None,
                                    },
                                )
                                .await;
                            }
                            Err(err) => {
                                eprintln!("Error transforming posts: {:?}", err);
                                break;
                            }
                        }
                    }
                }
                Ok(None) => break,
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

    let options = FindOptions::builder().limit(20).build();

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
        match channel.visibility {
            Visibility::Public => true,
            Visibility::Private => false,
        }
    } else {
        false
    }
}

fn transform_posts(posts: Vec<Post>) -> Result<BTreeMap<String, Vec<Vec<Post>>>, String> {
    let mut result: BTreeMap<String, Vec<Vec<Post>>> = BTreeMap::new();

    let interval_limit = match TimeDelta::try_minutes(5) {
        Some(time) => Some(time),
        None => return Err(format!("Failed to calculate time interval")),
    };

    for post in posts {
        let created_date_str = post.created_at.date_naive().to_string();

        // Check if there's an existing array for the created date
        let entry = result
            .entry(created_date_str.clone())
            .or_insert_with(Vec::new);

        if let Some(interval_limit) = interval_limit {
            // Check if there's an existing array for the time interval
            let mut interval_found = false;
            for interval_posts in entry.iter_mut() {
                if let Some(last_post) = interval_posts.last() {
                    let time_difference =
                        post.created_at.signed_duration_since(last_post.created_at);
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
        } else {
            return Err("Failed to calculate time interval".to_string());
        }
    }

    Ok(result)
}
