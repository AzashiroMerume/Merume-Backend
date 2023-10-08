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
use mongodb::{
    change_stream::event::ChangeStreamEvent,
    options::{ChangeStreamOptions, FindOptions, FullDocumentType},
};

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
    let initial_posts = fetch_posts(state.clone(), channel_id).await;

    if let Ok(json) = serde_json::to_string(&initial_posts) {
        if let Err(err) = sender.send(Message::Text(json)).await {
            eprintln!("Error sending message to websocket client: {:?}", err);
            return;
        }
    } else {
        eprintln!("Error serializing posts to JSON");
        return;
    }

    // if !is_channel_public(channel_id, &state).await
    //     && !is_user_subscribed(user_id, channel_id, &state).await
    // {
    //     return;
    // }

    let pipeline = [doc! {"$match": {"channel_id": channel_id}}];

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
        println!("Changed");
        loop {
            match change_stream.try_next().await {
                Ok(Some(_)) => {
                    let posts = fetch_posts(state.clone(), channel_id).await;

                    if let Some(posts) = posts {
                        let json = serde_json::to_string(&posts).map_err(|err| {
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
        .sort(doc! {"timestamp": -1})
        .limit(20)
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
        return channel.channel_type == "Public";
    }

    false
}
