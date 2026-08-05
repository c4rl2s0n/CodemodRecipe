use crate::model::*;
use crate::QueryDefinition;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ComposeError {
    #[error("recipe reference not found: {0}")]
    RecipeNotFound(String),

    #[error("recipe composition cycle detected: {0}")]
    Cycle(String),
}

/// A step accepted by [`compose_recipe`] (mirrors Dart `CodemodStep`).
#[derive(Debug, Clone)]
pub enum ComposeStep {
    Recipe(Recipe),
    Edit(EditStep),
    PostExecution(PostExecution),
}

/// Extract the referenced recipe id from a `Step::RecipeRef`.
pub fn recipe_ref_id(recipe_ref: &RecipeRef) -> &str {
    &recipe_ref.id
}

/// Compose a recipe from explicit args and ordered steps (Dart `CodemodRecipe.compose`).
///
/// Explicit [args] take precedence over args contributed by nested recipes.
/// Post-execution actions from steps are appended in step order.
pub fn compose_recipe(
    id: String,
    name: Option<String>,
    description: Option<String>,
    args: Vec<Arg>,
    steps: Vec<ComposeStep>,
) -> Recipe {
    let mut merged_args: BTreeMap<String, Arg> = BTreeMap::new();
    for arg in args {
        merged_args.insert(arg.name.clone(), arg);
    }

    let mut out_steps: Vec<Step> = Vec::new();
    let mut post_execution: Vec<PostExecution> = Vec::new();
    let mut maps: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for step in steps {
        match step {
            ComposeStep::Recipe(recipe) => {
                for arg in &recipe.args {
                    merged_args
                        .entry(arg.name.clone())
                        .or_insert_with(|| arg.clone());
                }
                out_steps.extend(recipe.steps.clone());
                post_execution.extend(recipe.post_execution.clone());
                merge_maps_into(&mut maps, &recipe.maps);
            }
            ComposeStep::Edit(edit) => {
                out_steps.push(Step::Edit(edit));
            }
            ComposeStep::PostExecution(action) => {
                post_execution.push(action);
            }
        }
    }

    Recipe {
        id,
        name,
        description,
        args: merged_args.into_values().collect(),
        maps,
        queries: BTreeMap::new(),
        steps: out_steps,
        post_execution,
        explorer_menu: None,
    }
}

/// Expand `recipe:` reference steps using [registry] (YAML composition).
///
/// Referenced recipes contribute edit steps and merged args. Child args listed
/// in call-site `with` are not unioned into the parent. Child `postExecution`
/// is **not** inlined (matches Dart YAML compiler behaviour).
pub fn expand_recipe_references(
    recipe: &Recipe,
    registry: &BTreeMap<String, Recipe>,
) -> Result<Recipe, ComposeError> {
    let mut visiting = BTreeSet::new();
    expand_recipe_references_inner(recipe, registry, &mut visiting)
}

fn expand_recipe_references_inner(
    recipe: &Recipe,
    registry: &BTreeMap<String, Recipe>,
    visiting: &mut BTreeSet<String>,
) -> Result<Recipe, ComposeError> {
    if !visiting.insert(recipe.id.clone()) {
        return Err(ComposeError::Cycle(recipe.id.clone()));
    }

    let mut merged_args: BTreeMap<String, Arg> = recipe
        .args
        .iter()
        .map(|a| (a.name.clone(), a.clone()))
        .collect();
    let mut maps = recipe.maps.clone();
    let mut queries = recipe.queries.clone();
    let steps = expand_steps(
        &recipe.steps,
        registry,
        visiting,
        &mut merged_args,
        &mut maps,
        &mut queries,
    )?;

    visiting.remove(&recipe.id);

    Ok(Recipe {
        id: recipe.id.clone(),
        name: recipe.name.clone(),
        description: recipe.description.clone(),
        args: merged_args.into_values().collect(),
        maps,
        queries,
        steps,
        post_execution: recipe.post_execution.clone(),
        explorer_menu: recipe.explorer_menu.clone(),
    })
}

fn expand_steps(
    steps: &[Step],
    registry: &BTreeMap<String, Recipe>,
    visiting: &mut BTreeSet<String>,
    merged_args: &mut BTreeMap<String, Arg>,
    maps: &mut BTreeMap<String, BTreeMap<String, String>>,
    queries: &mut BTreeMap<String, QueryDefinition>,
) -> Result<Vec<Step>, ComposeError> {
    let mut out: Vec<Step> = Vec::new();
    for step in steps {
        match step {
            Step::Edit(edit) => out.push(Step::Edit(edit.clone())),
            Step::Create(create) => out.push(Step::Create(create.clone())),
            Step::Delete(delete) => out.push(Step::Delete(delete.clone())),
            Step::Unknown(_, _) => out.push(step.clone()),
            Step::Scoped(scoped) => {
                let child_steps = expand_steps(
                    &scoped.steps,
                    registry,
                    visiting,
                    merged_args,
                    maps,
                    queries,
                )?;
                out.push(Step::Scoped(ScopedStep {
                    with: scoped.with.clone(),
                    if_expr: scoped.if_expr.clone(),
                    if_not: scoped.if_not.clone(),
                    steps: child_steps,
                }));
            }
            Step::RecipeRef(recipe_ref) => {
                let ref_id = recipe_ref_id(recipe_ref);
                let child = registry
                    .get(ref_id)
                    .ok_or_else(|| ComposeError::RecipeNotFound(ref_id.to_string()))?;
                let expanded = expand_recipe_references_inner(child, registry, visiting)?;
                for arg in &expanded.args {
                    if recipe_ref.with.contains_key(&arg.name) {
                        continue;
                    }
                    merge_unbound_child_arg(merged_args, arg, ref_id);
                }
                let mut child_steps: Vec<Step> = Vec::new();
                for child_step in &expanded.steps {
                    match child_step {
                        Step::Edit(edit) => child_steps.push(Step::Edit(edit.clone())),
                        Step::Create(create) => child_steps.push(Step::Create(create.clone())),
                        Step::Delete(delete) => child_steps.push(Step::Delete(delete.clone())),
                        Step::Scoped(scoped) => child_steps.push(Step::Scoped(scoped.clone())),
                        Step::RecipeRef(_) | Step::Unknown(_, _) => {}
                    }
                }
                if recipe_ref.with.is_empty() && !recipe_ref.has_condition() {
                    out.extend(child_steps);
                } else {
                    out.push(Step::Scoped(ScopedStep {
                        with: recipe_ref.with.clone(),
                        if_expr: recipe_ref.if_expr.clone(),
                        if_not: recipe_ref.if_not.clone(),
                        steps: child_steps,
                    }));
                }
                merge_maps_into(maps, &expanded.maps);
                merge_queries_into(queries, &expanded.queries);
            }
        }
    }
    Ok(out)
}

fn merge_queries_into(
    target: &mut BTreeMap<String, QueryDefinition>,
    source: &BTreeMap<String, QueryDefinition>,
) {
    for (key, def) in source {
        target.entry(key.clone()).or_insert_with(|| def.clone());
    }
}

/// Union an unbound child arg into [merged_args], recording nested recipe provenance.
///
/// Parent-declared args (empty `from_recipes`) stay unmarked. Nested contributors
/// get leaf recipe ids: if the child already tracked provenance, those ids are
/// propagated; otherwise the immediate child id is the source.
fn merge_unbound_child_arg(merged_args: &mut BTreeMap<String, Arg>, arg: &Arg, child_id: &str) {
    let sources: Vec<String> = if arg.from_recipes.is_empty() {
        vec![child_id.to_string()]
    } else {
        arg.from_recipes.clone()
    };
    match merged_args.entry(arg.name.clone()) {
        Entry::Vacant(e) => {
            let mut inserted = arg.clone();
            inserted.from_recipes = sources;
            e.insert(inserted);
        }
        Entry::Occupied(mut e) => {
            // Parent-owned args were inserted first with empty provenance — leave unmarked.
            if e.get().from_recipes.is_empty() {
                return;
            }
            for source in sources {
                if !e.get().from_recipes.contains(&source) {
                    e.get_mut().from_recipes.push(source);
                }
            }
        }
    }
}

#[cfg(test)]
mod merge_queries_tests {
    use super::*;
    use crate::QueryDefinition;

    #[test]
    fn merge_queries_does_not_override_existing_keys() {
        let mut target = BTreeMap::from([(
            "a".to_string(),
            QueryDefinition {
                query: "first".to_string(),
            },
        )]);
        let source = BTreeMap::from([(
            "a".to_string(),
            QueryDefinition {
                query: "second".to_string(),
            },
        )]);
        merge_queries_into(&mut target, &source);
        assert_eq!(target["a"].query, "first");
    }
}

fn merge_maps_into(
    target: &mut BTreeMap<String, BTreeMap<String, String>>,
    source: &BTreeMap<String, BTreeMap<String, String>>,
) {
    for (id, entries) in source {
        target
            .entry(id.clone())
            .or_default()
            .extend(entries.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_arg(name: &str) -> Arg {
        Arg {
            name: name.to_string(),
            required: true,
            input_kind: None,
            abbr: None,
            help: None,
            defaults_to: None,
            options: vec![],
            allow_custom_value: None,
            context_key: None,
            from: None,
            from_recipes: vec![],
        }
    }

    fn edit_step(path: &str) -> EditStep {
        EditStep {
            path: path.to_string(),
            ops: vec![EditOp::Insert(InsertOp {
                query: QuerySpec::single("(identifier) @x"),
                capture: "x".to_string(),
                anchor: InsertAnchor::End,
                text: "x".to_string(),
            })],
            ..Default::default()
        }
    }

    fn recipe_named(id: &str, path: &str, args: Vec<Arg>) -> Recipe {
        Recipe {
            id: id.to_string(),
            name: None,
            description: None,
            args,
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![Step::Edit(edit_step(path))],
            post_execution: vec![],
            explorer_menu: None,
        }
    }

    fn recipe_ref(id: &str) -> Step {
        Step::RecipeRef(RecipeRef {
            id: id.to_string(),
            with: BTreeMap::new(),
            if_expr: None,
            if_not: None,
        })
    }

    fn recipe_ref_with(id: &str, with: BTreeMap<String, String>) -> Step {
        Step::RecipeRef(RecipeRef {
            id: id.to_string(),
            with,
            if_expr: None,
            if_not: None,
        })
    }

    #[test]
    fn compose_deduplicates_shared_args() {
        let shared = sample_arg("file");
        let first = recipe_named("first", "a.dart", vec![shared.clone()]);
        let second = recipe_named("second", "b.dart", vec![shared]);

        let composed = compose_recipe(
            "composed".to_string(),
            None,
            None,
            vec![],
            vec![ComposeStep::Recipe(first), ComposeStep::Recipe(second)],
        );

        assert_eq!(composed.args.len(), 1);
        assert_eq!(composed.steps.len(), 2);
    }

    #[test]
    fn compose_accepts_empty_steps() {
        let composed = compose_recipe("empty".to_string(), None, None, vec![], vec![]);
        assert!(composed.args.is_empty());
        assert!(composed.steps.is_empty());
        assert!(composed.post_execution.is_empty());
    }

    #[test]
    fn compose_explicit_args_override_recipe_args() {
        let nested = recipe_named("nested", "a.dart", vec![sample_arg("root")]);
        let composed = compose_recipe(
            "composed".to_string(),
            None,
            None,
            vec![Arg {
                name: "root".to_string(),
                required: false,
                input_kind: Some("text".to_string()),
                abbr: None,
                help: None,
                defaults_to: None,
                options: vec![],
                allow_custom_value: None,
                context_key: None,
                from: None,
                from_recipes: vec![],
            }],
            vec![ComposeStep::Recipe(nested)],
        );

        assert_eq!(composed.args.len(), 1);
        assert!(!composed.args[0].required);
    }

    #[test]
    fn compose_preserves_post_execution_order() {
        let with_format = Recipe {
            id: "r".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![],
            post_execution: vec![PostExecution::String("echo a".to_string())],
            explorer_menu: None,
        };

        let composed = compose_recipe(
            "composed".to_string(),
            None,
            None,
            vec![],
            vec![
                ComposeStep::Recipe(with_format),
                ComposeStep::PostExecution(PostExecution::String("build".to_string())),
            ],
        );

        assert_eq!(composed.post_execution.len(), 2);
        assert!(matches!(&composed.post_execution[0], PostExecution::String(s) if s == "echo a"));
        assert!(matches!(&composed.post_execution[1], PostExecution::String(s) if s == "build"));
    }

    #[test]
    fn expand_inlines_referenced_recipe_steps() {
        let child = recipe_named("child", "child.dart", vec![sample_arg("file")]);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("child")],
            post_execution: vec![],
            explorer_menu: None,
        };

        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        assert_eq!(expanded.steps.len(), 1);
        assert_eq!(expanded.args.len(), 1);
    }

    #[test]
    fn expand_detects_cycles() {
        let a = Recipe {
            id: "a".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("b")],
            post_execution: vec![],
            explorer_menu: None,
        };
        let b = Recipe {
            id: "b".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("a")],
            post_execution: vec![],
            explorer_menu: None,
        };
        let registry = BTreeMap::from([("a".to_string(), a.clone()), ("b".to_string(), b)]);

        let err = expand_recipe_references(&a, &registry).unwrap_err();
        assert!(matches!(err, ComposeError::Cycle(_)));
    }

    #[test]
    fn expand_with_excludes_bound_args_from_union() {
        let child = recipe_named(
            "child",
            "child.dart",
            vec![sample_arg("className"), sample_arg("fieldName")],
        );
        let mut with = BTreeMap::new();
        with.insert("className".to_string(), "{{ featureName }}".to_string());
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![sample_arg("featureName"), sample_arg("fieldName")],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref_with("child", with)],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let names: Vec<_> = expanded.args.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"featureName"));
        assert!(names.contains(&"fieldName"));
        assert!(!names.contains(&"className"));
        assert!(matches!(&expanded.steps[0], Step::Scoped(_)));
    }

    #[test]
    fn expand_partial_with_keeps_unbound_child_args() {
        let child = recipe_named(
            "child",
            "child.dart",
            vec![sample_arg("className"), sample_arg("fieldName")],
        );
        let mut with = BTreeMap::new();
        with.insert("className".to_string(), "{{ featureName }}".to_string());
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![sample_arg("featureName")],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref_with("child", with)],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        assert!(expanded.args.iter().any(|a| a.name == "fieldName"));
        assert!(!expanded.args.iter().any(|a| a.name == "className"));
    }

    #[test]
    fn expand_empty_with_inlines_without_scoped() {
        let child = recipe_named("child", "child.dart", vec![sample_arg("file")]);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref_with("child", BTreeMap::new())],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        assert!(matches!(&expanded.steps[0], Step::Edit(_)));
    }

    #[test]
    fn expand_conditioned_recipe_ref_keeps_scoped() {
        let child = recipe_named("child", "child.dart", vec![sample_arg("file")]);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![Step::RecipeRef(RecipeRef {
                id: "child".to_string(),
                with: BTreeMap::new(),
                if_expr: Some("includeTests".to_string()),
                if_not: None,
            })],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let Step::Scoped(scoped) = &expanded.steps[0] else {
            panic!("expected Scoped for conditioned recipe ref");
        };
        assert_eq!(scoped.if_expr.as_deref(), Some("includeTests"));
        assert_eq!(scoped.steps.len(), 1);
    }

    #[test]
    fn expand_if_step_expands_nested_recipe_refs() {
        let child = recipe_named(
            "child",
            "child.dart",
            vec![sample_arg("file"), sample_arg("extra")],
        );
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![sample_arg("includeTests")],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![Step::Scoped(ScopedStep {
                with: BTreeMap::new(),
                if_expr: Some("includeTests".to_string()),
                if_not: None,
                steps: vec![recipe_ref("child")],
            })],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let Step::Scoped(scoped) = &expanded.steps[0] else {
            panic!("expected outer Scoped if step");
        };
        assert_eq!(scoped.if_expr.as_deref(), Some("includeTests"));
        assert_eq!(scoped.steps.len(), 1);
        assert!(matches!(&scoped.steps[0], Step::Edit(_)));
        assert!(expanded.args.iter().any(|a| a.name == "file"));
        assert!(expanded.args.iter().any(|a| a.name == "extra"));
    }

    #[test]
    fn expand_nested_if_preserves_inner_gate() {
        let child = recipe_named("child", "child.dart", vec![]);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![Step::Scoped(ScopedStep {
                with: BTreeMap::new(),
                if_expr: Some("outer".to_string()),
                if_not: None,
                steps: vec![Step::Scoped(ScopedStep {
                    with: BTreeMap::new(),
                    if_expr: Some("inner".to_string()),
                    if_not: None,
                    steps: vec![recipe_ref("child")],
                })],
            })],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let Step::Scoped(outer) = &expanded.steps[0] else {
            panic!("expected outer Scoped");
        };
        assert_eq!(outer.if_expr.as_deref(), Some("outer"));
        let Step::Scoped(inner) = &outer.steps[0] else {
            panic!("expected inner Scoped");
        };
        assert_eq!(inner.if_expr.as_deref(), Some("inner"));
        assert!(matches!(&inner.steps[0], Step::Edit(_)));
    }

    #[test]
    fn expand_records_leaf_from_recipes_for_unbound_nested_args() {
        let leaf = recipe_named("leaf", "leaf.dart", vec![sample_arg("description")]);
        let mid = Recipe {
            id: "mid".to_string(),
            name: None,
            description: None,
            args: vec![sample_arg("barrelPath")],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("leaf")],
            post_execution: vec![],
            explorer_menu: None,
        };
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("mid")],
            post_execution: vec![],
            explorer_menu: None,
        };
        let registry = BTreeMap::from([
            ("leaf".to_string(), leaf),
            ("mid".to_string(), mid),
        ]);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let description = expanded
            .args
            .iter()
            .find(|a| a.name == "description")
            .expect("description bubbled");
        assert_eq!(description.from_recipes, vec!["leaf".to_string()]);
        let barrel = expanded
            .args
            .iter()
            .find(|a| a.name == "barrelPath")
            .expect("barrelPath bubbled");
        assert_eq!(barrel.from_recipes, vec!["mid".to_string()]);
    }

    #[test]
    fn expand_with_excludes_bound_args_from_provenance() {
        let child = recipe_named(
            "child",
            "child.dart",
            vec![sample_arg("className"), sample_arg("fieldName")],
        );
        let mut with = BTreeMap::new();
        with.insert("className".to_string(), "{{ featureName }}".to_string());
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![sample_arg("featureName")],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref_with("child", with)],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        assert!(!expanded.args.iter().any(|a| a.name == "className"));
        let field = expanded
            .args
            .iter()
            .find(|a| a.name == "fieldName")
            .expect("fieldName unbound");
        assert_eq!(field.from_recipes, vec!["child".to_string()]);
        let feature = expanded
            .args
            .iter()
            .find(|a| a.name == "featureName")
            .expect("featureName parent-owned");
        assert!(feature.from_recipes.is_empty());
    }

    #[test]
    fn expand_unions_from_recipes_from_sibling_refs() {
        let a = recipe_named("a", "a.dart", vec![sample_arg("shared")]);
        let b = recipe_named("b", "b.dart", vec![sample_arg("shared")]);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("a"), recipe_ref("b")],
            post_execution: vec![],
            explorer_menu: None,
        };
        let registry = BTreeMap::from([("a".to_string(), a), ("b".to_string(), b)]);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let shared = expanded
            .args
            .iter()
            .find(|a| a.name == "shared")
            .expect("shared");
        assert_eq!(shared.from_recipes.len(), 2);
        assert!(shared.from_recipes.contains(&"a".to_string()));
        assert!(shared.from_recipes.contains(&"b".to_string()));
    }

    #[test]
    fn expand_parent_declared_arg_has_no_from_recipes() {
        let child = recipe_named("child", "child.dart", vec![sample_arg("file")]);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![sample_arg("file")],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![recipe_ref("child")],
            post_execution: vec![],
            explorer_menu: None,
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);

        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let file = expanded
            .args
            .iter()
            .find(|a| a.name == "file")
            .expect("file");
        assert!(file.from_recipes.is_empty());
    }
}
