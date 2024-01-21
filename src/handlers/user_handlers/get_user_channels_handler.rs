use crate::{models::channel_model::Channel, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserChannelsResponse {
    success: bool,
    channels: Option<Vec<Channel>>,
    error: Option<String>,
}

pub async fn get_user_channels(
    State(state): State<AppState>,
    Path(user_id): Path<ObjectId>,
) -> impl IntoResponse {
    let filter = doc! {"author.id": user_id};

    let channels_result = state
        .db
        .channels_collection
        .find(filter.to_owned(), None)
        .await;

    match channels_result {
        Ok(cursor) => {
            let channels = cursor
                .try_collect()
                .await
                .map_err(|err| {
                    eprintln!("Error collecting channels: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UserChannelsResponse {
                            success: false,
                            channels: None,
                            error: Some("Failed to fetch channels".to_string()),
                        }),
                    )
                })
                .ok();

            let response = UserChannelsResponse {
                success: channels.is_some(),
                channels,
                error: None,
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            eprintln!("Error finding channels: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserChannelsResponse {
                    success: false,
                    channels: None,
                    error: Some("Failed to fetch channels".to_string()),
                }),
            )
        }
    }
}
