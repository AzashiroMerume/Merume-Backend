use crate::models::author_model::Author;
use crate::responses::ErrorResponse;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
};
use bson::{doc, oid::ObjectId};
use std::sync::Arc;

pub async fn delete_post_by_id(
    State(state): State<Arc<AppState>>,
    Extension(author): Extension<Author>,
    Path((_channel_id, post_id)): Path<(ObjectId, ObjectId)>,
) -> Result<StatusCode, ErrorResponse> {
    let post = state
        .db
        .posts_collection
        .find_one(doc! { "_id": post_id }, None)
        .await;

    match post {
        Ok(Some(post)) => {
            if post.author.id == author.id {
                let deletion_result = state
                    .db
                    .posts_collection
                    .delete_one(doc! { "_id": post_id }, None)
                    .await;

                match deletion_result {
                    Ok(result) => {
                        if result.deleted_count == 1 {
                            Ok(StatusCode::OK)
                        } else {
                            Err(ErrorResponse::ServerError(None))
                        }
                    }
                    Err(err) => {
                        eprintln!("Error deleting post: {:?}", err);
                        Err(ErrorResponse::ServerError(None))
                    }
                }
            } else {
                Err(ErrorResponse::Forbidden(Some("Not an author of the post")))
            }
        }
        Ok(None) => Err(ErrorResponse::NotFound(None)),
        Err(err) => {
            eprintln!("Error finding post: {:?}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
