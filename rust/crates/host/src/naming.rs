/// Converts a Dart-style identifier to snake_case.
pub fn to_snake_case(input: &str) -> String {
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

    buffer
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
}
