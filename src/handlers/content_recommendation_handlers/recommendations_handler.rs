use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use futures::stream::TryStreamExt;
use mongodb::{options::UpdateOptions, Client};

use crate::models::{channel_model::Channel, post_model::Post, user_model::User};

pub async fn recommendations(
    State(client): State<Client>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let user_preferences = user.preferences.unwrap();
    let channels_collection = client.database("Merume").collection::<Channel>("channels");

    let mut cursor = channels_collection
        .find(doc! {"categories": {"$in": user_preferences}}, None)
        .await
        .unwrap();

    let mut channels = vec![];

    while let Some(channel) = cursor.try_next().await.unwrap() {
        channels.push(channel);
    }

    // Step 2: Calculate percentage increase in two-week subscribers for each channel
    let mut channels_with_percentage_increase: Vec<(Channel, usize)> = Vec::new();

    for channel in channels {
        let two_week_subscribers = channel.subscriptions.two_week_subscribers.len() as usize;
        let previous_week_subscribers = channel
            .subscriptions
            .monthly_subscribers
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

    (StatusCode::OK, Json(sorted_channels))
}
