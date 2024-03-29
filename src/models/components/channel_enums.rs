use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    Fixed,
    Unfixed,
}

impl Default for ChallengeType {
    fn default() -> Self {
        ChallengeType::Fixed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}
