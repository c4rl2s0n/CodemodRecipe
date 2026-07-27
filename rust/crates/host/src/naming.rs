/// Converts a Dart-style identifier to snake_case.
pub fn to_snake_case(input: &str) -> String {
    let input = normalize_phrase_separators(input);
    if input.is_empty() {
        return String::new();
    }

    let mut buffer = String::new();
    let mut previous_was_upper = false;
    let chars: Vec<char> = input.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        let is_upper = ch.is_ascii_uppercase();

        if is_upper {
            if i > 0
                && (!previous_was_upper
                    || (i + 1 < chars.len() && chars[i + 1].is_ascii_lowercase()))
            {
                buffer.push('_');
            }
            buffer.push(ch.to_ascii_lowercase());
        } else {
            buffer.push(ch);
        }

        previous_was_upper = is_upper;
    }

    buffer = collapse_underscores(&buffer);
    buffer
}

fn collapse_underscores(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_underscore = false;
    for ch in input.chars() {
        if ch == '_' {
            if !prev_underscore && !out.is_empty() {
                out.push('_');
            }
            prev_underscore = true;
        } else {
            out.push(ch);
            prev_underscore = false;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

/// Trim and map whitespace/hyphen runs to underscores for phrase-style inputs.
fn normalize_phrase_separators(input: &str) -> String {
    let trimmed = input.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut last_was_sep = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() || ch == '-' {
            if !last_was_sep && !out.is_empty() {
                out.push('_');
            }
            last_was_sep = true;
        } else {
            out.push(ch);
            last_was_sep = false;
        }
    }
    out
}

pub fn to_pascal_case(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    to_snake_case(input)
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

pub fn to_camel_case(input: &str) -> String {
    let pascal = to_pascal_case(input);
    if pascal.is_empty() {
        return pascal;
    }
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
    }
}

pub fn to_lower(input: &str) -> String {
    input.to_ascii_lowercase()
}

pub fn to_upper(input: &str) -> String {
    input.to_ascii_uppercase()
}

pub fn to_screaming_snake(input: &str) -> String {
    to_snake_case(input).to_ascii_uppercase()
}

pub fn to_kebab_case(input: &str) -> String {
    to_snake_case(input).replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_explicit_casing_helpers() {
        assert_eq!(to_snake_case("FeedList"), "feed_list");
        assert_eq!(to_camel_case("FeedList"), "feedList");
        assert_eq!(to_pascal_case("FeedList"), "FeedList");
    }

    #[test]
    fn handles_acronym_boundaries() {
        assert_eq!(to_snake_case("URLValue"), "url_value");
    }

    #[test]
    fn screaming_snake_and_kebab() {
        assert_eq!(to_screaming_snake("FeedList"), "FEED_LIST");
        assert_eq!(to_kebab_case("FeedList"), "feed-list");
    }

    #[test]
    fn phrase_with_spaces_and_hyphens() {
        assert_eq!(to_snake_case("test name"), "test_name");
        assert_eq!(to_snake_case("Test Name"), "test_name");
        assert_eq!(to_snake_case("test-name"), "test_name");
        assert_eq!(to_camel_case("test name"), "testName");
        assert_eq!(to_pascal_case("test name"), "TestName");
    }
}
