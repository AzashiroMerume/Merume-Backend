use crate::{
    models::{
        author_model::Author,
        channel_read_tracker_model::{ChannelReadTracker, ChannelReadTrackerPayload},
    },
    responses::ErrorResponse,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};
use std::sync::Arc;

pub async fn update_read_tracker_handler(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<ObjectId>,
    Extension(author): Extension<Author>,
    Json(payload): Json<ChannelReadTrackerPayload>,
) -> Result<StatusCode, ErrorResponse> {
    let serialized_channel_read_tracker = match bson::to_bson(&payload) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to serialize payload: {}", err);
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let document = match serialized_channel_read_tracker.as_document() {
        Some(document) => document,
        None => {
            eprintln!("Failed to convert serialized data to document");
            return Err(ErrorResponse::ServerError(None));
        }
    };

    let filter = doc! { "channel_id": channel_id, "user_id": author.id };
    let update = doc! {"$set": document};

    match state
        .db
        .channel_read_trackers_collection
        .find_one(filter.clone(), None)
        .await
    {
        Ok(Some(_)) => {
            match state
                .db
                .channel_read_trackers_collection
                .find_one_and_update(filter, update, None)
                .await
            {
                Ok(_) => Ok(StatusCode::OK),
                Err(err) => {
                    eprintln!("Failed to update channel read tracker: {}", err);
                    Err(ErrorResponse::ServerError(None))
                }
            }
        }
        Ok(None) => {
            let channel_read_tracker = ChannelReadTracker {
                id: ObjectId::new(),
                user_id: author.id,
                channel_id,
                last_read_post_id: payload.last_read_post_id,
            };

            match state
                .db
                .channel_read_trackers_collection
                .insert_one(channel_read_tracker.to_owned(), None)
                .await
            {
                Ok(_) => Ok(StatusCode::OK),
                Err(err) => {
                    eprintln!("Failed to insert channel read tracker: {}", err);
                    Err(ErrorResponse::ServerError(None))
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to find channel read tracker: {}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
