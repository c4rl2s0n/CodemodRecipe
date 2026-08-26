use schemars::JsonSchema;
use serde_json::json;

pub const BUILTIN_TEXT: &str = "text";
pub const BUILTIN_BOOL: &str = "bool";
pub const BUILTIN_CHOICE: &str = "choice";
pub const BUILTIN_FILE: &str = "file";
pub const BUILTIN_DIRECTORY: &str = "directory";

const RESERVED: &[&str] = &[
    BUILTIN_TEXT,
    BUILTIN_BOOL,
    BUILTIN_CHOICE,
    BUILTIN_FILE,
    BUILTIN_DIRECTORY,
    "symbol",
    "list",
    "object",
    "dartType",
    "enum",
];

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub enum InputKind {
    Text,
    Bool,
    Choice,
    File,
    Directory,
    Type(String),
    List(Box<InputKind>),
}

impl InputKind {
    pub fn from_name(name: &str) -> Self {
        match name {
            BUILTIN_TEXT | "symbol" => Self::Text,
            BUILTIN_BOOL => Self::Bool,
            BUILTIN_CHOICE | "enum" => Self::Choice,
            BUILTIN_FILE => Self::File,
            BUILTIN_DIRECTORY => Self::Directory,
            other => Self::Type(other.to_string()),
        }
    }

    pub fn is_reserved_id(id: &str) -> bool {
        RESERVED.contains(&id)
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn item(&self) -> Option<&InputKind> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn type_id(&self) -> Option<&str> {
        match self {
            Self::Type(id) => Some(id.as_str()),
            Self::List(inner) => inner.type_id(),
            _ => None,
        }
    }

    pub fn as_name(&self) -> String {
        match self {
            Self::Text => BUILTIN_TEXT.to_string(),
            Self::Bool => BUILTIN_BOOL.to_string(),
            Self::Choice => BUILTIN_CHOICE.to_string(),
            Self::File => BUILTIN_FILE.to_string(),
            Self::Directory => BUILTIN_DIRECTORY.to_string(),
            Self::Type(id) => id.clone(),
            Self::List(inner) => format!("[{}]", inner.as_name()),
        }
    }

    /// Wire form: `"text"` or `["ClassProperty"]`.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::List(inner) => json!([inner.as_name()]),
            other => serde_json::Value::String(other.as_name()),
        }
    }
}
