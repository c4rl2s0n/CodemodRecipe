use std::collections::BTreeMap;

/// Replace `{{key}}` placeholders with values from `args`.
pub fn render_string(template: &str, args: &BTreeMap<String, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in args {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
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
}
