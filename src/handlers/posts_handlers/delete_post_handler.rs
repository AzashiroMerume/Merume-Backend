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

pub async fn delete_post_by_id(
    State(client): State<Client>,
    Path((channel_id, post_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let channel_collection = client.database("Merume").collection::<Channel>("channels");
    let post_collection = client.database("Merume").collection::<Post>("posts");

    let channel_id = match ObjectId::parse_str(&channel_id) {
        Ok(channel_id) => channel_id,
        Err(err) => {
            eprintln!("Error parsing channel_id: {:?}", err);
            return (
                StatusCode::BAD_REQUEST,
                Json(BoolResponse {
                    success: false,
                    error_message: Some("Invalid channel ID".to_string()),
                }),
            );
        }
    };

    let post_id = match ObjectId::parse_str(&post_id) {
        Ok(post_id) => post_id,
        Err(err) => {
            eprintln!("Error parsing post_id: {:?}", err);
            return (
                StatusCode::BAD_REQUEST,
                Json(BoolResponse {
                    success: false,
                    error_message: Some("Invalid post ID".to_string()),
                }),
            );
        }
    };

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
        Err(err) => {
            eprintln!("Error checking channel: {:?}", err);
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

    let deletion_result = post_collection
        .delete_one(doc! { "_id": post_id }, None)
        .await;

    match deletion_result {
        Ok(result) => {
            if result.deleted_count == 1 {
                (
                    StatusCode::OK,
                    Json(BoolResponse {
                        success: true,
                        error_message: None,
                    }),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(BoolResponse {
                        success: false,
                        error_message: Some("Channel not found".to_string()),
                    }),
                )
            }
        }
        Err(err) => {
            eprintln!("Error deleting post: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoolResponse {
                    success: false,
                    error_message: Some(format!("Failed to delete channel: {}", err.to_string())),
                }),
            )
        }
    }
}
