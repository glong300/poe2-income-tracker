use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Realm {
    International,
    China,
}

impl Realm {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "international" => Some(Self::International),
            "china" => Some(Self::China),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::International => "international",
            Self::China => "china",
        }
    }
}
