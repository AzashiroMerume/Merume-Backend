use std::usize;

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::author_model::Author;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub author: Author,
    pub name: String,
    pub goal: u32,
    pub channel_type: String,
    pub description: String,
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<ObjectId>>,
    pub followers: Followers,
    pub current_challenge_day: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_profile_picture_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Followers {
    pub current_following: usize,
    pub monthly_followers: Vec<usize>,
    pub yearly_followers: Vec<usize>,
    pub two_week_followers: Vec<usize>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[validate(custom = "validate_channel_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ChannelPayload {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(range(min = 1000, max = 2000))]
    pub goal: u32,
    #[validate(custom = "validate_channel_type")]
    pub channel_type: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub categories: Vec<String>,
    pub participants: Option<Vec<ObjectId>>,
    pub channel_profile_picture_url: Option<String>,
}

fn validate_channel_type(channel_type: &str) -> Result<(), ValidationError> {
    if channel_type == "Public" || channel_type == "Private" {
        return Ok(());
    }

    Err(ValidationError::new("Not correct channel type"))
}
