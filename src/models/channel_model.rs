use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::usize;
use validator::Validate;

use super::{
    author_model::Author,
    components::channel_enums::{ChallengeType, Visibility},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub author: Author,
    pub name: String,
    pub visibility: Visibility,
    pub description: String,
    pub categories: Vec<String>,
    pub challenge: Challenge,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributors: Option<Vec<ObjectId>>,
    pub followers: Followers,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_pfp_link: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct Challenge {
    pub challenge_type: ChallengeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1000, max = 2000))]
    pub goal: Option<u32>,
    pub points: usize,
    pub current_day: usize,
    pub streak: usize,
    pub missed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missed_days: Option<Vec<DateTime<Utc>>>,
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
    #[validate(length(min = 1))]
    pub name: String,
    pub challenge_type: String,
    #[validate(range(min = 1000, max = 2000))]
    pub goal: Option<u32>,
    pub visibility: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributors: Option<Vec<ObjectId>>,
    pub channel_pfp_link: Option<String>,
}

impl ChannelPayload {
    pub fn challenge_type_enum(&self) -> ChallengeType {
        match &self.challenge_type.to_lowercase()[..] {
            "fixed" => ChallengeType::Fixed,
            "unfixed" => ChallengeType::Unfixed,
            _ => ChallengeType::Fixed,
        }
    }

    pub fn visibility_enum(&self) -> Visibility {
        match &self.visibility.to_lowercase()[..] {
            "public" => Visibility::Public,
            "private" => Visibility::Private,
            _ => Visibility::Public,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<u32>,
    pub visibility: Visibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_pfp_link: Option<String>,
}
