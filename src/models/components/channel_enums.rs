use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeTypes {
    Fixed,
    Unfixed,
}

impl Default for ChallengeTypes {
    fn default() -> Self {
        ChallengeTypes::Fixed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisibilityTypes {
    Public,
    Private,
}

impl Default for VisibilityTypes {
    fn default() -> Self {
        VisibilityTypes::Public
    }
}
