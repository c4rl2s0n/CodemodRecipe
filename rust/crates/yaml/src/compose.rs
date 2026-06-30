use crate::model::*;
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

/// Extract the referenced recipe id from a `Step::RecipeRef` value.
pub fn recipe_ref_id(value: &serde_yaml::Value) -> Option<&str> {
    value.as_str()
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
        steps: out_steps,
        post_execution,
    }
}

/// Expand `recipe:` reference steps using [registry] (YAML composition).
///
/// Referenced recipes contribute edit steps and merged args. Child
/// `postExecution` is **not** inlined (matches Dart YAML compiler behaviour).
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
    let mut steps: Vec<Step> = Vec::new();
    let mut maps = recipe.maps.clone();

    for step in &recipe.steps {
        match step {
            Step::Edit(edit) => steps.push(Step::Edit(edit.clone())),
            Step::RecipeRef(value) => {
                let Some(ref_id) = recipe_ref_id(value) else {
                    continue;
                };
                let child = registry
                    .get(ref_id)
                    .ok_or_else(|| ComposeError::RecipeNotFound(ref_id.to_string()))?;
                let expanded = expand_recipe_references_inner(child, registry, visiting)?;
                for arg in &expanded.args {
                    merged_args
                        .entry(arg.name.clone())
                        .or_insert_with(|| arg.clone());
                }
                for child_step in &expanded.steps {
                    if let Step::Edit(edit) = child_step {
                        steps.push(Step::Edit(edit.clone()));
                    }
                }
                merge_maps_into(&mut maps, &expanded.maps);
            }
            Step::Create(_) | Step::Unknown(_, _) => steps.push(step.clone()),
        }
    }

    visiting.remove(&recipe.id);

    Ok(Recipe {
        id: recipe.id.clone(),
        name: recipe.name.clone(),
        description: recipe.description.clone(),
        args: merged_args.into_values().collect(),
        maps,
        steps,
        post_execution: recipe.post_execution.clone(),
    })
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
        }
    }

    fn edit_step(path: &str) -> EditStep {
        EditStep {
            path: path.to_string(),
            language: None,
            ops: vec![EditOp::Insert(InsertOp {
                query: "(identifier) @x".to_string(),
                capture: "x".to_string(),
                anchor: InsertAnchor::End,
                text: "x".to_string(),
            })],
        }
    }

    fn recipe_named(id: &str, path: &str, args: Vec<Arg>) -> Recipe {
        Recipe {
            id: id.to_string(),
            name: None,
            description: None,
            args,
            maps: BTreeMap::new(),
            steps: vec![Step::Edit(edit_step(path))],
            post_execution: vec![],
        }
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
            vec![
                ComposeStep::Recipe(first),
                ComposeStep::Recipe(second),
            ],
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
            steps: vec![],
            post_execution: vec![PostExecution::String("dartFormat".to_string())],
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
        assert!(matches!(&composed.post_execution[0], PostExecution::String(s) if s == "dartFormat"));
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
            steps: vec![Step::RecipeRef(serde_yaml::Value::String(
                "child".to_string(),
            ))],
            post_execution: vec![],
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
            steps: vec![Step::RecipeRef(serde_yaml::Value::String("b".to_string()))],
            post_execution: vec![],
        };
        let b = Recipe {
            id: "b".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            steps: vec![Step::RecipeRef(serde_yaml::Value::String("a".to_string()))],
            post_execution: vec![],
        };
        let registry = BTreeMap::from([("a".to_string(), a.clone()), ("b".to_string(), b)]);

        let err = expand_recipe_references(&a, &registry).unwrap_err();
        assert!(matches!(err, ComposeError::Cycle(_)));
    }
}
