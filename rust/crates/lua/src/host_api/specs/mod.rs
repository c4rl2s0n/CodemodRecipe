//! Luau-facing host API input/return shapes, split by capability.

mod common;
mod edits;
mod files;
mod queries;
mod query;
mod template;
mod traits;

pub use common::{Anchor, Env, IfExists, IfMissing};
pub use edits::{InsertSpec, RemoveSpec, ReplaceSpec};
pub use files::{CreateSpec, DeleteSpec, ListFilesSpec, ReadSpec};
pub use queries::{
    ByteSpan, MatchFileRow, QueryCaptureSpec, QueryHit, QueryMatchRow, QueryTableSpec, SpanTextSpec,
};
pub use query::QuerySpec;
pub use template::{PostExecSpec, TemplateSpec};
pub use traits::{HostSpec, HostType};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_sandbox;
    use mlua::{FromLua, Lua, Value};

    fn lua() -> Lua {
        new_sandbox().expect("sandbox")
    }

    #[test]
    fn query_spec_single_string() {
        let lua = lua();
        let value = lua.load("\"foo\"").eval::<Value>().unwrap();
        let spec = QuerySpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec, QuerySpec::Single("foo".to_string()));
    }

    #[test]
    fn query_spec_chain() {
        let lua = lua();
        let value = lua.load("{\"a\", \"b\"}").eval::<Value>().unwrap();
        let spec = QuerySpec::from_lua(value, &lua).unwrap();
        assert_eq!(
            spec,
            QuerySpec::Chain(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn insert_spec_defaults() {
        let lua = lua();
        let value = lua
            .load("{path = \"p.lua\", query = \"q\", capture = \"c\", text = \"t\"}")
            .eval::<Value>()
            .unwrap();
        let spec = InsertSpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec.path, "p.lua");
        assert_eq!(spec.anchor, None);
        assert_eq!(spec.language, None);
    }

    #[test]
    fn read_spec_string() {
        let lua = lua();
        let value = lua.load("\"foo.lua\"").eval::<Value>().unwrap();
        let spec = ReadSpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec.path, "foo.lua");
    }

    #[test]
    fn read_spec_table() {
        let lua = lua();
        let value = lua.load("{path = \"foo.lua\"}").eval::<Value>().unwrap();
        let spec = ReadSpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec.path, "foo.lua");
    }

    #[test]
    fn env_table() {
        let lua = lua();
        let value = lua.load("{a = 1, b = \"two\"}").eval::<Value>().unwrap();
        let env = Env::from_lua(value, &lua).unwrap();
        assert_eq!(env.0.get("a").unwrap(), &serde_json::json!(1));
        assert_eq!(env.0.get("b").unwrap(), &serde_json::json!("two"));
    }

    #[test]
    fn lua_spec_macro_exposes_host_spec() {
        assert_eq!(InsertSpec::NAME, "InsertSpec");
        let schema = InsertSpec::schema();
        let props = schema.schema.object.as_ref().unwrap().properties.clone();
        assert!(props.contains_key("path"));
        assert!(props.contains_key("query"));
        assert!(props.contains_key("capture"));
        assert!(props.contains_key("text"));
        assert!(props.contains_key("language"));
        assert!(props.contains_key("anchor"));
    }
}
