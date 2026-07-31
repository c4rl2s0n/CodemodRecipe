use serde::{Deserialize, Deserializer, Serialize};

use crate::query_spec::QuerySpec;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LetBinding {
    pub name: String,
    #[serde(default)]
    pub query: Option<QuerySpec>,
    #[serde(default)]
    pub capture: Option<String>,
    #[serde(default)]
    pub extract: LetExtract,
    #[serde(default)]
    pub on_no_match: LetOnNoMatch,
    #[serde(default)]
    pub on_many_matches: LetOnManyMatches,
    #[serde(default)]
    pub join: Option<String>,
    /// Optional template to compute final value from prior locals (and recipe args).
    #[serde(default)]
    pub r#as: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LetExtract {
    #[default]
    Text,
    Kind,
    Exists,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LetOnNoMatch {
    #[default]
    Error,
    #[serde(rename = "use")]
    UseEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LetOnManyMatches {
    #[default]
    Error,
    First,
    Join,
}

/// Deserialize `let:` as a single binding or a list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LetBindings(pub Vec<LetBinding>);

impl<'de> Deserialize<'de> for LetBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum OneOrMany {
            One(LetBinding),
            Many(Vec<LetBinding>),
        }
        match OneOrMany::deserialize(deserializer)? {
            OneOrMany::One(b) => Ok(LetBindings(vec![b])),
            OneOrMany::Many(v) => Ok(LetBindings(v)),
        }
    }
}
