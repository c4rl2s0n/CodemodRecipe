use crate::naming::{to_camel_case, to_pascal_case, to_snake_case};
use std::collections::BTreeMap;

/// Replace `{{key}}` and `{{$camel key}}` style placeholders with values from `args`.
pub fn render_string(template: &str, args: &BTreeMap<String, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in args {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
    out = render_casing_helpers(&out, args);
    out
}

fn render_casing_helpers(template: &str, args: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{$") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 3..];
        let Some(end) = rest.find("}}") else {
            out.push_str("{{$");
            out.push_str(rest);
            return out;
        };
        let inner = rest[..end].trim();
        rest = &rest[end + 2..];

        let Some((helper, key)) = inner.split_once(char::is_whitespace) else {
            out.push_str("{{$");
            out.push_str(inner);
            out.push_str("}}");
            continue;
        };

        let replacement = match helper {
            "snake" => args.get(key).map(|v| to_snake_case(v)),
            "camel" => args.get(key).map(|v| to_camel_case(v)),
            "pascal" => args.get(key).map(|v| to_pascal_case(v)),
            _ => None,
        };

        match replacement {
            Some(value) => out.push_str(&value),
            None => {
                out.push_str("{{$");
                out.push_str(inner);
                out.push_str("}}");
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_placeholders() {
        let mut args = BTreeMap::new();
        args.insert("file".to_string(), "lib/foo.dart".to_string());
        assert_eq!(render_string("path: {{file}}", &args), "path: lib/foo.dart");
    }

    #[test]
    fn leaves_unknown_placeholders() {
        let args = BTreeMap::new();
        assert_eq!(render_string("{{missing}}", &args), "{{missing}}");
    }

    #[test]
    fn preserves_special_characters_in_raw_placeholders() {
        let mut args = BTreeMap::new();
        args.insert("x".to_string(), "a$b".to_string());
        assert_eq!(render_string("{{x}}", &args), "a$b");
    }

    #[test]
    fn renders_unicode_values() {
        let mut args = BTreeMap::new();
        args.insert("emoji".to_string(), "🚀".to_string());
        assert_eq!(render_string("// {{emoji}}", &args), "// 🚀");
    }

    #[test]
    fn renders_explicit_casing_helpers() {
        let mut args = BTreeMap::new();
        args.insert("feature".to_string(), "FeedList".to_string());
        assert_eq!(
            render_string(
                "{{feature}} {{$snake feature}} {{$camel feature}} {{$pascal feature}}",
                &args
            ),
            "FeedList feed_list feedList FeedList"
        );
    }

    #[test]
    fn leaves_missing_casing_helper_placeholders() {
        let args = BTreeMap::new();
        assert_eq!(render_string("{{$camel field}}", &args), "{{$camel field}}");
    }

    #[test]
    fn renders_camel_field_in_recipe_snippet() {
        let mut args = BTreeMap::new();
        args.insert("field".to_string(), "counter".to_string());
        assert_eq!(
            render_string("final int {{$camel field}};", &args),
            "final int counter;"
        );
    }
}
