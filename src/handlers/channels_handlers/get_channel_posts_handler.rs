use crate::{
    models::post_model::Post, responses::ChannelResponse, utils::pagination::Pagination, AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
};
use bson::{doc, oid::ObjectId};
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub async fn get_channel_posts(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(channel_id): Path<ObjectId>,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), channel_id, pagination))
}

async fn websocket(
    mut socket: WebSocket,
    state: State<AppState>,
    channel_id: ObjectId,
    pagination: Pagination,
) {
    let (mut sender, mut receiver) = socket.split();

    // Fetch initial posts based on pagination parameters
    let skip = pagination.page * pagination.limit;
    let filter = doc! {"channel_id": channel_id};
    let options = Some(doc! {"limit": pagination.limit, "skip": skip});

    let posts_result = state
        .db
        .posts_collection
        .find(filter.clone(), options)
        .await
        .map_err(|err| {
            eprintln!("Error finding posts: {:?}", err);
            "Failed to fetch posts".to_string()
        });

    let posts: Vec<Post> = match posts_result {
        Ok(posts) => posts.try_collect().await.map_err(|err| {
            eprintln!("Error collecting posts: {:?}", err);
            "Failed to fetch posts".to_string()
        }),
        Err(err) => Err(err),
    }
    .unwrap();

    // Send initial posts to the client
    let response = ChannelResponse {
        success: true,
        data: Some(posts),
        error_message: None,
    };

    if let Ok(response_json) = serde_json::to_string(&response) {
        if let Err(err) = sender.send(Message::Text(response_json)).await {
            eprintln!("Error sending initial posts to websocket client: {:?}", err);
            return;
        }
    }

    // Create a shared deque to store new posts
    let new_posts: Arc<Mutex<VecDeque<ChannelResponse>>> = Arc::new(Mutex::new(VecDeque::new()));

    // Spawn a task to listen for new posts
    let new_posts_clone = Arc::clone(&new_posts);
    tokio::spawn(async move {
        while let Some(message) = receiver.next().await {
            if let Ok(scroll_event) = message.to_str() {
                if scroll_event == "scroll_top" {
                    // Fetch new posts based on pagination parameters
                    let new_skip = (pagination.page + 1) * pagination.limit;
                    let new_options = Some(doc! {"limit": pagination.limit, "skip": new_skip});

                    let new_posts_result = state
                        .db
                        .posts_collection
                        .find(filter.clone(), new_options)
                        .await
                        .map_err(|err| {
                            eprintln!("Error finding new posts: {:?}", err);
                            "Failed to fetch new posts".to_string()
                        });

                    let new_posts: Vec<Post> = match new_posts_result {
                        Ok(posts) => posts.try_collect().await.map_err(|err| {
                            eprintln!("Error collecting new posts: {:?}", err);
                            "Failed to fetch new posts".to_string()
                        }),
                        Err(err) => Err(err),
                    }
                    .unwrap();

                    // Add new posts to the shared deque
                    new_posts_clone.lock().unwrap().extend(new_posts);
                }
            }
        }
    });

    // Send new posts to the client
    while let Some(new_post) = new_posts.lock().unwrap().pop_front() {
        let new_posts_response = ChannelResponse {
            success: true,
            data: Some(vec![new_post]),
            error_message: None,
        };

        if let Ok(response_json) = serde_json::to_string(&new_posts_response) {
            if let Err(err) = sender.send(Message::Text(response_json)).await {
                eprintln!("Error sending new posts to websocket client: {:?}", err);
                break;
            }
        }
    }
}
