use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Tree-sitter query on an edit op: one pattern or a chained list of steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuerySpec {
    Single(String),
    Chain(Vec<String>),
}

impl QuerySpec {
    pub fn single(text: impl Into<String>) -> Self {
        QuerySpec::Single(text.into())
    }

    /// Ordered query steps after resolution (length >= 1 when valid).
    pub fn steps(&self) -> Vec<&str> {
        match self {
            QuerySpec::Single(s) => vec![s.as_str()],
            QuerySpec::Chain(v) => v.iter().map(String::as_str).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            QuerySpec::Single(s) => s.trim().is_empty(),
            QuerySpec::Chain(v) => v.is_empty() || v.iter().all(|s| s.trim().is_empty()),
        }
    }

    pub fn step_count(&self) -> usize {
        match self {
            QuerySpec::Single(_) => 1,
            QuerySpec::Chain(v) => v.len(),
        }
    }

    /// All query step strings (for template validation / scanning).
    pub fn step_strings(&self) -> Vec<&str> {
        self.steps()
    }
}

impl Serialize for QuerySpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            QuerySpec::Single(s) => serializer.serialize_str(s),
            QuerySpec::Chain(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for QuerySpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QuerySpecVisitor;

        impl<'de> Visitor<'de> for QuerySpecVisitor {
            type Value = QuerySpec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tree-sitter query string or a list of query strings")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(QuerySpec::Single(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(QuerySpec::Single(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element::<String>()? {
                    items.push(item);
                }
                if items.is_empty() {
                    return Err(de::Error::custom("query list must not be empty"));
                }
                if items.len() == 1 {
                    Ok(QuerySpec::Single(items.into_iter().next().unwrap()))
                } else {
                    Ok(QuerySpec::Chain(items))
                }
            }
        }

        deserializer.deserialize_any(QuerySpecVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct QueryDefinition {
    pub query: String,
}
