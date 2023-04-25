use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

use crate::models::{channel_model::Channel, post_model::Post};
use crate::responses::bool_response::BoolResponse;

pub async fn delete_post(
    State(client): State<Client>,
    Path((channel_id, post_id)): Path<(ObjectId, ObjectId)>,
) -> impl IntoResponse {
    let channel_collection = client.database("Merume").collection::<Channel>("channels");
    let post_collection = client.database("Merume").collection::<Post>("posts");

    //check the channel for existence
    match channel_collection
        .find_one(doc! {"_id": channel_id}, None)
        .await
    {
        Ok(None) => {
            let main_response = BoolResponse {
                success: false,
                error_message: Some("Channel does not exist.".to_string()),
            };
            return (StatusCode::BAD_REQUEST, Json(main_response));
        }
        Err(e) => {
            eprintln!("Error checking email: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(
                        "There was an error on the server side, try again later.".to_string(),
                    ),
                }),
            );
        }
        _ => {} // continue checking for nickname
    }

    let result = post_collection
        .delete_one(doc! {"_id": post_id}, None)
        .await;

    match result {
        Ok(_) => {
            return (
                StatusCode::OK,
                Json(BoolResponse {
                    success: true,
                    error_message: None,
                }),
            )
        }
        Err(err) => {
            eprintln!("Error deleting post: {}", err.to_string());
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(format!("Error deleting post")),
                }),
            );
        }
    }
}
