use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::doc;
use futures::stream::TryStreamExt;
use mongodb::options::FindOptions;

use crate::{
    models::{channel_model::Channel, user_model::User},
    responses::main_response::MainResponse,
    AppState,
};

pub async fn recommendations(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let user_preferences = user.preferences.unwrap();

    let options = FindOptions::builder().limit(20).build();
    let cursor = state
        .db
        .channels_collection
        .find(doc! {"categories": {"$in": user_preferences}}, options)
        .await;

    let mut channels = vec![];

    match cursor {
        Ok(mut cursor) => {
            while let Some(channel) = cursor.try_next().await.unwrap() {
                channels.push(channel);
            }
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MainResponse {
                    success: false,
                    data: None,
                    error_message: Some(format!("Failed to find recomendations")),
                }),
            );
        }
    }

    // Step 2: Calculate percentage increase in two-week subscribers for each channel
    let mut channels_with_percentage_increase: Vec<(Channel, usize)> = Vec::new();

    for channel in channels {
        let two_week_subscribers = channel.subscriptions.two_week_subscribers.len() as usize;
        let previous_week_subscribers = channel
            .subscriptions
            .two_week_subscribers
            .last()
            .cloned()
            .unwrap_or(0) as usize;

        let percentage_increase = if previous_week_subscribers == 0 {
            // If previous week subscribers is zero, percentage increase is undefined
            0
        } else {
            ((two_week_subscribers - previous_week_subscribers) / previous_week_subscribers) * 100
        };

        channels_with_percentage_increase.push((channel, percentage_increase));
    }

    // Step 3: Sort channels by percentage increase in two-week subscribers in descending order
    channels_with_percentage_increase.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Step 4: Return sorted list of channels
    let sorted_channels = channels_with_percentage_increase
        .into_iter()
        .map(|(channel, _)| channel)
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        Json(MainResponse {
            success: true,
            data: Some(sorted_channels),
            error_message: None,
        }),
    )
}
