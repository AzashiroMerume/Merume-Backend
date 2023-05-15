pub mod recommendations_handler;
pub mod trendings_handler;

use crate::models::{channel_model::Channel, post_model::Post};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ChannelWithLatestPost {
    channel: Channel,
    latest_post: Post,
}
