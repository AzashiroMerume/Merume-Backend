use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use bson::oid::ObjectId;
use chrono::Utc;
use std::sync::Arc;
use validator::Validate;

use crate::{models::author_model::Author, responses::ErrorResponse};
use crate::{
    models::post_model::{Post, PostPayload},
    AppState,
};

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Extension(current_challenge_day): Extension<usize>,
    Path(channel_id): Path<ObjectId>,
    Json(payload): Json<PostPayload>,
) -> Result<StatusCode, ErrorResponse> {
    // Validate the payload
    match payload.validate() {
        Ok(()) => {} // Validation successful, do nothing
        Err(err) => {
            eprintln!("Error validating payload: {:?}", err);
            return Err(ErrorResponse::UnprocessableEntity(None));
        }
    }

    let now = Utc::now();
    let author = Author {
        id: author.id,
        nickname: author.nickname,
        username: author.username,
        pfp_link: author.pfp_link,
        is_online: author.is_online,
        last_time_online: author.last_time_online,
    };

    let post = Post {
        id: payload.id,
        author,
        channel_id,
        body: payload.body,
        images: payload.images,
        written_challenge_day: current_challenge_day,
        likes: 0,
        dislikes: 0,
        already_changed: false,
        created_at: now,
        updated_at: now,
    };

    let result = state.db.posts_collection.insert_one(post, None).await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            eprintln!("Error inserting user: {:?}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
