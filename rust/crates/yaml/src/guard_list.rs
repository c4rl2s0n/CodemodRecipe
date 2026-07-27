use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

use crate::query_spec::QuerySpec;

/// One or more guard queries (`when` / `whenNot`). Each entry is a full [`QuerySpec`] (inline, path, ref, or chain).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GuardList {
    pub guards: Vec<QuerySpec>,
}

impl GuardList {
    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }
}

impl<'de> Deserialize<'de> for GuardList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GuardListVisitor;

        impl<'de> Visitor<'de> for GuardListVisitor {
            type Value = GuardList;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a query string, query chain, or list of guard queries")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(GuardList {
                    guards: vec![QuerySpec::Single(value.to_string())],
                })
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(GuardList {
                    guards: vec![QuerySpec::Single(value)],
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut guards = Vec::new();
                while let Some(spec) = seq.next_element::<QuerySpec>()? {
                    guards.push(spec);
                }
                if guards.is_empty() {
                    return Err(de::Error::custom("guard list must not be empty"));
                }
                Ok(GuardList { guards })
            }
        }

        deserializer.deserialize_any(GuardListVisitor)
    }
}
