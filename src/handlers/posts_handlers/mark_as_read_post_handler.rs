use crate::models::author_model::Author;
use crate::AppState;
use crate::{models::post_actioned_model::ReadPost, responses::OperationStatusResponse};

use axum::Extension;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::oid::ObjectId;

pub async fn mark_as_read(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Json(payload): Json<Vec<ReadPost>>,
) -> impl IntoResponse {
    let read_posts: Vec<ReadPost> = payload
        .into_iter()
        .map(|mut post| {
            post.id = ObjectId::new(); // Generate a new ObjectId for the read post
            post.user_id_who_read = author.id;
            post
        })
        .collect();

    match state
        .db
        .read_posts_collection
        .insert_many(read_posts, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(OperationStatusResponse {
                success: true,
                error_message: None,
            }),
        ),
        Err(err) => {
            eprintln!("Error inserting read posts: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some("Failed to insert read posts".to_string()),
                }),
            )
        }
    }
}
