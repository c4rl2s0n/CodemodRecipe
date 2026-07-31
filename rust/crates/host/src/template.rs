use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use codemod_recipe_core::resource_path::resolve_existing_resource;
use minijinja::value::Value;
use minijinja::{Environment, UndefinedBehavior};

use crate::naming::{
    to_camel_case, to_kebab_case, to_lower, to_pascal_case, to_screaming_snake, to_snake_case,
    to_upper,
};
use crate::path_filters::{path_basename, path_parent, path_stem};

const TEMPLATE_FUEL: usize = 50_000;

/// Render a template string (recipe paths, queries, inline create.template).
pub fn render_string(template: &str, args: &BTreeMap<String, String>) -> Result<String, String> {
    render_template(template, args, &BTreeMap::new(), &BTreeMap::new())
}

pub fn render_template(
    template: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<String, String> {
    let converted = convert_legacy_syntax(template);
    let env = build_environment(maps)?;
    let ctx = build_context(args, maps, vars);
    env.render_str(&converted, ctx).map_err(|e| e.to_string())
}

/// Render a file-backed template with `extends` / `include` support.
pub fn render_template_file(
    template_name: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    templates_root: &Path,
    recipe_file: Option<&Path>,
) -> Result<String, String> {
    let root = templates_root.to_path_buf();
    let recipe_file = recipe_file.map(|path| path.to_path_buf());
    let mut env = build_environment(maps)?;
    env.set_loader(move |name| -> Result<Option<String>, minijinja::Error> {
        let path = resolve_existing_resource(name, recipe_file.as_deref(), &root, None)
            .map_err(|e| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, e.message))?
            .ok_or_else(|| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    format!("failed to read template {name}: not found"),
                )
            })?;
        let content = std::fs::read_to_string(&path).map_err(|e| {
            minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("failed to read template {name}: {e}"),
            )
        })?;
        Ok(Some(convert_legacy_syntax(&content)))
    });
    let tmpl = env
        .get_template(template_name)
        .map_err(|e| format!("Template {template_name}: {e}"))?;
    let ctx = build_context(args, maps, vars);
    tmpl.render(ctx).map_err(|e| e.to_string())
}

fn build_environment(
    maps: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<Environment<'_>, String> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_fuel(Some(TEMPLATE_FUEL as u64));
    env.set_keep_trailing_newline(true);

    env.add_filter("snake_case", |value: String| -> String {
        to_snake_case(&value)
    });
    env.add_filter("camel_case", |value: String| -> String {
        to_camel_case(&value)
    });
    env.add_filter("pascal_case", |value: String| -> String {
        to_pascal_case(&value)
    });
    env.add_filter("lower", |value: String| -> String { to_lower(&value) });
    env.add_filter("upper", |value: String| -> String { to_upper(&value) });
    env.add_filter("screaming_snake", |value: String| -> String {
        to_screaming_snake(&value)
    });
    env.add_filter("kebab_case", |value: String| -> String {
        to_kebab_case(&value)
    });
    env.add_filter("trim", |value: String| -> String {
        value.trim().to_string()
    });

    env.add_filter("parent", |value: String| -> String { path_parent(&value) });
    env.add_filter("basename", |value: String| -> String {
        path_basename(&value)
    });
    env.add_filter("stem", |value: String| -> String { path_stem(&value) });

    env.add_filter("int", |value: Value| -> Result<i64, minijinja::Error> {
        value_to_i64(&value)
            .map_err(|e| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, e))
    });
    env.add_filter(
        "add",
        |value: i64, n: i64| -> Result<i64, minijinja::Error> {
            value.checked_add(n).ok_or_else(|| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "integer overflow in add",
                )
            })
        },
    );
    env.add_filter(
        "sub",
        |value: i64, n: i64| -> Result<i64, minijinja::Error> {
            value.checked_sub(n).ok_or_else(|| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "integer overflow in sub",
                )
            })
        },
    );
    env.add_filter("string", |value: i64| -> String { value.to_string() });
    env.add_filter("str", |value: i64| -> String { value.to_string() });

    let maps = Arc::new(maps.clone());
    env.add_filter(
        "map",
        move |value: String, map_id: String| -> Result<String, minijinja::Error> {
            let lookup = maps
                .get(&map_id)
                .and_then(|entries| entries.get(&value))
                .cloned()
                .unwrap_or(value);
            Ok(lookup)
        },
    );

    Ok(env)
}

fn build_context(
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> BTreeMap<String, Value> {
    let mut ctx: BTreeMap<String, Value> = BTreeMap::new();
    for (key, value) in args {
        ctx.insert(key.clone(), coerce_value(value));
    }
    ctx.insert("map".to_string(), Value::from_serialize(maps));
    ctx.insert("var".to_string(), Value::from_serialize(vars));
    ctx
}

fn coerce_value(raw: &str) -> Value {
    match raw {
        "true" => Value::from(true),
        "false" => Value::from(false),
        other => Value::from(other.to_string()),
    }
}

fn value_to_i64(value: &Value) -> Result<i64, String> {
    if let Some(n) = value.as_i64() {
        return Ok(n);
    }
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        return trimmed
            .parse::<i64>()
            .map_err(|_| format!("expected integer, got '{s}'"));
    }
    Err("expected integer value".to_string())
}

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

    fn render_ok(template: &str, args: &BTreeMap<String, String>) -> String {
        render_string(template, args).expect("render")
    }

    fn render_tpl(
        template: &str,
        args: &BTreeMap<String, String>,
        maps: &BTreeMap<String, BTreeMap<String, String>>,
    ) -> String {
        render_template(template, args, maps, &BTreeMap::new()).expect("render")
    }

    #[test]
    fn preserves_trailing_newline_in_literal_text() {
        let args = BTreeMap::new();
        assert_eq!(
            render_ok("    print('codemod');\n", &args),
            "    print('codemod');\n"
        );
    }

    #[test]
    fn replaces_placeholders() {
        let mut args = BTreeMap::new();
        args.insert("file".to_string(), "lib/foo.dart".to_string());
        assert_eq!(render_ok("path: {{file}}", &args), "path: lib/foo.dart");
    }

    #[test]
    fn strict_undefined_errors_on_missing() {
        let args = BTreeMap::new();
        assert!(render_string("{{missing}}", &args).is_err());
    }

    #[test]
    fn preserves_special_characters_in_values() {
        let mut args = BTreeMap::new();
        args.insert("x".to_string(), "a$b".to_string());
        assert_eq!(render_ok("{{x}}", &args), "a$b");
    }

    #[test]
    fn renders_unicode_values() {
        let mut args = BTreeMap::new();
        args.insert("emoji".to_string(), "🚀".to_string());
        assert_eq!(render_ok("// {{emoji}}", &args), "// 🚀");
    }

    #[test]
    fn renders_explicit_casing_helpers() {
        let mut args = BTreeMap::new();
        args.insert("feature".to_string(), "FeedList".to_string());
        assert_eq!(
            render_ok(
                "{{feature}} {{$snake feature}} {{$camel feature}} {{$pascal feature}}",
                &args
            ),
            "FeedList feed_list feedList FeedList"
        );
    }

    #[test]
    fn renders_jinja_casing_filters() {
        let mut args = BTreeMap::new();
        args.insert("feature".to_string(), "FeedList".to_string());
        assert_eq!(
            render_ok(
                "{{ feature | snake_case }} {{ feature | screaming_snake }}",
                &args
            ),
            "feed_list FEED_LIST"
        );
    }

    #[test]
    fn renders_jinja_path_filters() {
        let mut args = BTreeMap::new();
        args.insert(
            "featureDir".to_string(),
            "lib/features/feed/widgets".to_string(),
        );
        args.insert("file".to_string(), "lib/foo.dart".to_string());
        assert_eq!(
            render_ok(
                "{{ featureDir | parent }} {{ featureDir | basename }} {{ featureDir | parent | basename }} {{ file | stem }}",
                &args
            ),
            "lib/features/feed widgets feed foo"
        );
    }

    #[test]
    fn path_filters_strip_trailing_slash_and_normalize_separators() {
        let mut args = BTreeMap::new();
        args.insert("dir".to_string(), "lib\\features\\feed\\".to_string());
        assert_eq!(
            render_ok("{{ dir | parent }}/{{ dir | basename }}", &args),
            "lib/features/feed"
        );
    }

    #[test]
    fn conditional_if_block() {
        let mut args = BTreeMap::new();
        args.insert("include_tests".to_string(), "true".to_string());
        let tmpl = "{% if include_tests %}YES{% else %}NO{% endif %}";
        assert_eq!(render_ok(tmpl, &args), "YES");
        args.insert("include_tests".to_string(), "false".to_string());
        assert_eq!(render_ok(tmpl, &args), "NO");
    }

    #[test]
    fn renders_camel_field_in_recipe_snippet() {
        let mut args = BTreeMap::new();
        args.insert("field".to_string(), "counter".to_string());
        assert_eq!(
            render_ok("final int {{$camel field}};", &args),
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
            render_tpl("final {{$map 'columnType' type}} x;", &args, &maps),
            "final intColumn x;"
        );
    }

    #[test]
    fn map_helper_falls_back_to_lookup_key_when_map_missing() {
        let mut args = BTreeMap::new();
        args.insert("type".to_string(), "int".to_string());
        assert_eq!(
            render_tpl("{{$map 'missing' type}}", &args, &BTreeMap::new()),
            "int"
        );
    }

    #[test]
    fn jinja_map_filter() {
        let mut args = BTreeMap::new();
        args.insert("fieldName".to_string(), "tickCount".to_string());
        let mut maps = BTreeMap::new();
        let mut entries = BTreeMap::new();
        entries.insert("tickCount".to_string(), "int".to_string());
        maps.insert("field_kind".to_string(), entries);
        assert_eq!(
            render_tpl("{{ fieldName | map('field_kind') }}", &args, &maps),
            "int"
        );
    }

    #[test]
    fn numeric_filters_for_let_locals() {
        let mut args = BTreeMap::new();
        args.insert("schemaVersion".to_string(), "7".to_string());
        assert_eq!(
            render_ok("{{ schemaVersion | int | add(1) | string }}", &args),
            "8"
        );
    }

    #[test]
    fn template_extends_and_include() {
        use std::path::PathBuf;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../test/fixtures/template_inheritance/.codemod");
        let mut args = BTreeMap::new();
        args.insert("className".to_string(), "FeedList".to_string());
        let rendered = render_template_file(
            "templates/feature.template",
            &args,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &root,
            None,
        )
        .expect("render");
        assert!(rendered.contains("// Generated for FeedList"));
        assert!(rendered.contains("class FeedListWidget {}"));
    }

    #[test]
    fn prefers_recipe_local_template_over_codemod_fallback() {
        let workspace = std::env::temp_dir().join(format!(
            "codemod_template_local_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let codemod_root = workspace.join(".codemod");
        let recipe_dir = codemod_root.join("recipes");
        let recipe_file = recipe_dir.join("feature.yaml");
        std::fs::create_dir_all(recipe_dir.join("templates/partials")).unwrap();
        std::fs::create_dir_all(codemod_root.join("templates/partials")).unwrap();
        std::fs::write(
            recipe_dir.join("templates/widget.template"),
            "{% include \"templates/partials/body.template\" %}\n",
        )
        .unwrap();
        std::fs::write(
            recipe_dir.join("templates/partials/body.template"),
            "local {{ className }}",
        )
        .unwrap();
        std::fs::write(
            codemod_root.join("templates/widget.template"),
            "shared {{ className }}",
        )
        .unwrap();
        std::fs::write(
            codemod_root.join("templates/partials/body.template"),
            "shared-body {{ className }}",
        )
        .unwrap();

        let args = BTreeMap::from([("className".to_string(), "FeedList".to_string())]);
        let rendered = render_template_file(
            "templates/widget.template",
            &args,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &codemod_root,
            Some(&recipe_file),
        )
        .unwrap();

        assert_eq!(rendered.trim(), "local FeedList");
        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn renders_map_and_var_namespaces() {
        let args = BTreeMap::new();
        let mut maps = BTreeMap::new();
        let mut map_entries = BTreeMap::new();
        map_entries.insert("tickCount".to_string(), "int".to_string());
        maps.insert("field_kind".to_string(), map_entries);
        let mut vars = BTreeMap::new();
        let mut var_entries = BTreeMap::new();
        var_entries.insert("feature_root".to_string(), "lib/features".to_string());
        vars.insert("paths".to_string(), var_entries);
        assert_eq!(
            render_template(
                "{{ map.field_kind.tickCount }} {{ var.paths.feature_root }}",
                &args,
                &maps,
                &vars
            )
            .unwrap(),
            "int lib/features"
        );
    }
}
