use crate::{models::channel_model::Channel, responses::ErrorResponse, AppState};
use axum::{
    extract::{Path, State},
    response::Json,
};
use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct UserChannelsResponse {
    channels: Option<Vec<Channel>>,
}

pub async fn get_user_channels(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<ObjectId>,
) -> Result<Json<UserChannelsResponse>, ErrorResponse> {
    let filter = doc! {"author.id": user_id, "channel_visibility": "Public"};

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
                    ErrorResponse::ServerError(None)
                })
                .ok();

            let response = UserChannelsResponse { channels };

            Ok(Json(response))
        }
        Err(err) => {
            eprintln!("Error finding channels: {:?}", err);
            Err(ErrorResponse::ServerError(None))
        }
    }
}
