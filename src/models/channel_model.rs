use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::usize;
use validator::{Validate, ValidationError};

use super::author_model::Author;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub author: Author,
    pub channel_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<u32>,
    pub channel_visibility: String,
    pub description: String,
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributors: Option<Vec<ObjectId>>,
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

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ChannelPayload {
    pub channel_type: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(range(min = 1000, max = 2000))]
    pub goal: Option<u32>,
    #[validate(custom(function = "validate_channel_visibility"))]
    pub channel_visibility: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub categories: Vec<String>,
    pub contributors: Option<Vec<ObjectId>>,
    pub channel_profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<u32>,
    #[validate(custom(function = "validate_channel_visibility_option"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_profile_picture_url: Option<String>,
}

fn validate_channel_visibility(channel_visibility: &str) -> Result<(), ValidationError> {
    if channel_visibility == "Public" || channel_visibility == "Private" {
        return Ok(());
    }

    Err(ValidationError::new("Not correct channel type"))
}

fn validate_channel_visibility_option(
    channel_visibility: &Option<String>,
) -> Result<(), ValidationError> {
    match channel_visibility {
        Some(visibility) => validate_channel_visibility(visibility),
        None => Ok(()),
    }
}
