use std::usize;

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub owner_id: ObjectId,
    pub name: String,
    pub channel_type: String,
    pub description: String,
    pub categories: Vec<String>,
    pub subscriptions: Subscriptions,
    pub current_challenge_day: usize,
    pub base_image: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Subscriptions {
    pub current_subscriptions: usize,
    pub monthly_subscribers: Vec<usize>,
    pub yearly_subscribers: Vec<usize>,
    pub two_week_subscribers: Vec<usize>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ChannelPayload {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(custom = "validate_channel_type")]
    pub channel_type: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub categories: Vec<String>,
    pub base_image: Option<String>,
}

fn validate_channel_type(channel_type: &str) -> Result<(), ValidationError> {
    if channel_type == "Public" || channel_type == "Private" {
        return Ok(());
    }

    Err(ValidationError::new("Not correct channel type"))
}
