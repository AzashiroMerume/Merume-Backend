use crate::{models::post_model::Post, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Extension,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};
// use mongodb::options::{ChangeStreamOptions, FullDocumentType};

use serde_json::json;

pub async fn channel_posts(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user_id): Extension<ObjectId>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), channel_id, user_id))
}

async fn websocket(
    mut _socket: WebSocket,
    state: State<AppState>,
    channel_id: ObjectId,
    user_id: ObjectId,
) {
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

    let is_subscribed = is_user_subscribed(user_id, channel_id, &state).await;
    let is_public = is_channel_public(channel_id, &state).await;

    if !is_subscribed && !is_public {
        // User is not subscribed and channel is not public, close the websocket
        return;
    }

    // let pipeline = vec![doc! {
    //     "$match": {
    //         "channel_id": channel_id
    //     }
    // }];

    // let options = ChangeStreamOptions::builder()
    //     .full_document(Some(FullDocumentType::UpdateLookup))
    //     .build();

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
        return channel.channel_type == "Public";
    }

    false
}
