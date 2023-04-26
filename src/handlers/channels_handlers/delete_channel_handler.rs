use crate::{models::channel_model::Channel, responses::bool_response::BoolResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn delete_channel_by_id(
    State(client): State<Client>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let channels_collection = client.database("Merume").collection::<Channel>("channels");

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

    let deletion_result = channels_collection
        .delete_one(doc! { "_id": channel_id }, None)
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
