use std::collections::BTreeMap;

pub fn stem_from_path(path: &str) -> Option<String> {
    let path = path.trim().replace('\\', "/");
    let file = path.rsplit('/').next()?;
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_string())
    }
}

pub fn resolve_template(template: &str, vars: &BTreeMap<String, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_strips_extension() {
        assert_eq!(
            stem_from_path("src/entities/foo.dart").as_deref(),
            Some("foo")
        );
    }

    #[test]
    fn resolve_substitutes_vars() {
        let mut vars = BTreeMap::new();
        vars.insert("stem".to_string(), "recipe".to_string());
        assert_eq!(
            resolve_template("src/stores/{stem}_store.dart", &vars),
            "src/stores/recipe_store.dart"
        );
    }
}
