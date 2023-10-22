use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Author {
    pub author_id: ObjectId,
    pub author_nickname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub author: Author,
    pub channel_id: ObjectId,
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    pub written_challenge_day: usize,
    pub likes: usize,
    pub dislikes: usize,
    pub already_changed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdatePost {
    pub body: Option<String>,
    pub images: Option<Vec<String>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub already_changed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[validate(schema(function = "check_both_none", skip_on_field_errors = false))]
#[serde(rename_all = "snake_case")]
pub struct PostPayload {
    pub id: ObjectId,
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

fn check_both_none(post: &PostPayload) -> Result<(), ValidationError> {
    if post.body.is_none() && post.images.is_none() {
        return Err(ValidationError::new(
            "Post's body and images field cannot be both none",
        ));
    }

    Ok(())
}
