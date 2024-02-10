use crate::AppState;
use crate::{models::author_model::Author, responses::OperationStatusResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::{doc, oid::ObjectId};

pub async fn delete_post_by_id(
    State(state): State<AppState>,
    Extension(author): Extension<Author>,
    Path((_channel_id, post_id)): Path<(ObjectId, ObjectId)>,
) -> impl IntoResponse {
    println!("{}", post_id);
    let post = state
        .db
        .posts_collection
        .find_one(doc! { "_id": post_id }, None)
        .await;

    match post {
        Ok(Some(post)) => {
            println!(
                "Post Author Id: {}, Author Id: {}",
                post.author.id, author.id
            );
            if post.author.id == author.id {
                let deletion_result = state
                    .db
                    .posts_collection
                    .delete_one(doc! { "_id": post_id }, None)
                    .await;

                match deletion_result {
                    Ok(result) => {
                        if result.deleted_count == 1 {
                            (
                                StatusCode::OK,
                                Json(OperationStatusResponse {
                                    success: true,
                                    error_message: None,
                                }),
                            )
                        } else {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(OperationStatusResponse {
                                    success: false,
                                    error_message: Some("Failed to delete post".to_string()),
                                }),
                            )
                        }
                    }
                    Err(err) => {
                        eprintln!("Error deleting post: {:?}", err);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(OperationStatusResponse {
                                success: false,
                                error_message: Some(
                                    "There was an error on the server side, try again later."
                                        .to_string(),
                                ),
                            }),
                        )
                    }
                }
            } else {
                (
                    StatusCode::FORBIDDEN,
                    Json(OperationStatusResponse {
                        success: false,
                        error_message: Some("You are not the author of this post".to_string()),
                    }),
                )
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(OperationStatusResponse {
                success: false,
                error_message: Some("Post not found".to_string()),
            }),
        ),
        Err(err) => {
            eprintln!("Error finding post: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationStatusResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            )
        }
    }
}
