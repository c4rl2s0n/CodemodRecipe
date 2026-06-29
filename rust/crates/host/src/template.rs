use crate::naming::{to_camel_case, to_pascal_case, to_snake_case};
use std::collections::BTreeMap;

/// Replace `{{key}}`, `{{$camel key}}`, and `{{$map 'id' key}}` placeholders.
pub fn render_string(template: &str, args: &BTreeMap<String, String>) -> String {
    render_template(template, args, &BTreeMap::new())
}

pub fn render_template(
    template: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
) -> String {
    let mut out = render_map_helpers(template, args, maps);
    for (key, value) in args {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
    render_casing_helpers(&out, args)
}

fn render_map_helpers(
    template: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{$map") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 6..];
        let Some(end) = rest.find("}}") else {
            out.push_str("{{");
            out.push_str(rest);
            return out;
        };
        let inner = rest[..end].trim();
        rest = &rest[end + 2..];

        if let Some((map_id, key_token)) = parse_quoted_map_args(inner) {
            let lookup_key = args.get(&key_token).map(String::as_str).unwrap_or(&key_token);
            let replacement = maps
                .get(&map_id)
                .and_then(|entries| entries.get(lookup_key))
                .cloned()
                .unwrap_or_else(|| lookup_key.to_string());
            out.push_str(&replacement);
        } else {
            out.push_str("{{$map");
            out.push_str(inner);
            out.push_str("}}");
        }
    }
    out.push_str(rest);
    out
}

fn parse_quoted_map_args(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    let mut chars = text.chars();
    let quote = chars.next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let after_quote: String = chars.collect();
    let id_end = after_quote.find(quote)?;
    let map_id = after_quote[..id_end].to_string();
    let key_token = after_quote[id_end + 1..].trim().to_string();
    if map_id.is_empty() || key_token.is_empty() {
        return None;
    }
    Some((map_id, key_token))
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

        if inner.starts_with("map") {
            out.push_str("{{$");
            out.push_str(inner);
            out.push_str("}}");
            continue;
        }

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

    #[test]
    fn resolves_map_helper_with_arg_key() {
        let mut args = BTreeMap::new();
        args.insert("type".to_string(), "int".to_string());
        let mut maps = BTreeMap::new();
        let mut entries = BTreeMap::new();
        entries.insert("int".to_string(), "intColumn".to_string());
        maps.insert("columnType".to_string(), entries);

        assert_eq!(
            render_template("final {{$map 'columnType' type}} x;", &args, &maps),
            "final intColumn x;"
        );
    }

    #[test]
    fn map_helper_falls_back_to_lookup_key_when_map_missing() {
        let mut args = BTreeMap::new();
        args.insert("type".to_string(), "int".to_string());
        assert_eq!(
            render_template("{{$map 'missing' type}}", &args, &BTreeMap::new()),
            "int"
        );
    }
}
