//! Legacy `{{$…}}` helper → MiniJinja filter conversion.

/// True when the template still uses legacy `{{$…}}` helpers.
pub fn contains_legacy_syntax(template: &str) -> bool {
    template.contains("{{$")
}

/// Convert legacy `{{$snake x}}` / `{{$map 'id' key}}` to Jinja filter syntax.
pub fn convert_legacy_syntax(template: &str) -> String {
    let after_maps = convert_legacy_map_helpers(template);
    convert_legacy_casing_helpers(&after_maps)
}

fn convert_legacy_map_helpers(template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{$map") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 6..];
        let Some(end) = rest.find("}}") else {
            out.push_str("{{$map");
            out.push_str(rest);
            return out;
        };
        let inner = rest[..end].trim();
        rest = &rest[end + 2..];

        if let Some((map_id, key_token)) = parse_quoted_map_args(inner) {
            out.push_str("{{ ");
            out.push_str(&key_token);
            out.push_str(" | map('");
            out.push_str(&map_id);
            out.push_str("') }}");
        } else {
            out.push_str("{{$map");
            out.push_str(inner);
            out.push_str("}}");
        }
    }
    out.push_str(rest);
    out
}

fn convert_legacy_casing_helpers(template: &str) -> String {
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

        let filter = match helper {
            "snake" => Some("snake_case"),
            "camel" => Some("camel_case"),
            "pascal" => Some("pascal_case"),
            _ => None,
        };

        if let Some(filter_name) = filter {
            out.push_str("{{ ");
            out.push_str(key);
            out.push_str(" | ");
            out.push_str(filter_name);
            out.push_str(" }}");
        } else {
            out.push_str("{{$");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_legacy_prefix() {
        assert!(contains_legacy_syntax("{{$snake x}}"));
        assert!(!contains_legacy_syntax("{{ x | snake_case }}"));
    }

    #[test]
    fn converts_casing_helpers() {
        assert_eq!(
            convert_legacy_syntax("{{$snake feature}} {{$camel feature}}"),
            "{{ feature | snake_case }} {{ feature | camel_case }}"
        );
    }

    #[test]
    fn converts_map_helper() {
        assert_eq!(
            convert_legacy_syntax("{{$map 'columnType' type}}"),
            "{{ type | map('columnType') }}"
        );
    }
}
