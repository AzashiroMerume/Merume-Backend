use crate::{models::post_model::Post, utils::pagination::Pagination, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use bson::{doc, oid::ObjectId};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt, TryStreamExt,
};

pub async fn get_channel_posts(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(channel_id): Path<ObjectId>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, State(state), channel_id))
}

async fn websocket(mut socket: WebSocket, state: State<AppState>, channel_id: ObjectId) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(write(sender, state, channel_id));
    tokio::spawn(read(receiver, state, channel_id));
}

async fn read(mut receiver: SplitStream<WebSocket>, state: State<AppState>, channel_id: ObjectId) {
    while let Some(message) = receiver.next().await {
        if let Ok(pagination) = message.to_str() {
            let pagination = Pagination {
                page: 1, // Update with the appropriate page value
                limit: 20,
            };

            let posts = fetch_posts(state.clone(), channel_id, pagination).await;
            if let Some(posts) = posts {
                // Send the posts to the client
                let response_json = serde_json::to_string(&posts).unwrap();
                if let Err(err) = sender.send(Message::Text(response_json)).await {
                    eprintln!("Error sending posts to websocket client: {:?}", err);
                    break;
                }
            }
        }
    }
}

async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    state: State<AppState>,
    channel_id: ObjectId,
) {
    let pagination = Pagination { page: 0, limit: 20 };

    let posts = fetch_posts(state.clone(), channel_id, pagination).await;
    if let Some(posts) = posts {
        // Send the initial posts to the client
        let response_json = serde_json::to_string(&posts).unwrap();
        if let Err(err) = sender.send(Message::Text(response_json)).await {
            eprintln!("Error sending initial posts to websocket client: {:?}", err);
            return;
        }
    }

    let mut change_stream = state.db.posts_collection.watch(None, None).await;
    while let Some(change_result) = change_stream.try_next().await.unwrap() {
        if let Some(change_event) = change_result {
            if let Some(post) = change_event.full_document {
                if post.channel_id == channel_id {
                    // Send the new post to the client
                    let response_json = serde_json::to_string(&vec![post]).unwrap();
                    if let Err(err) = sender.send(Message::Text(response_json)).await {
                        eprintln!("Error sending new post to websocket client: {:?}", err);
                        break;
                    }
                }
            }
        }
    }
}

async fn fetch_posts(
    state: State<AppState>,
    channel_id: ObjectId,
    pagination: Pagination,
) -> Option<Vec<Post>> {
    let filter = doc! {
        "channel_id": channel_id,
    };

    let options = Some(doc! {
        "limit": pagination.limit,
        "sort": { "_id": -1 },
        "$and": [
            { "_id": { "$lt": pagination.page * pagination.limit } }
        ],
    });

    let posts_result = state.db.posts_collection.find(filter, options).await;
    if let Ok(cursor) = posts_result {
        if let Ok(posts) = cursor.try_next().await {
            return Some(posts.unwrap());
        }
    }

    None
}
