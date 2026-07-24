use crate::model::*;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("unsupported top-level step kind: {0}")]
    UnsupportedStep(String),

    #[error("unsupported edit op kind: {0}")]
    UnsupportedOp(String),

    #[error("{op} op missing required field: {field}")]
    MissingRequiredField { op: &'static str, field: &'static str },

    #[error("edit step has no ops")]
    EmptyEditOps,

    #[error("duplicate arg name: {0}")]
    DuplicateArgName(String),

    #[error("unknown language: {0}")]
    UnknownLanguage(String),

    #[error("create step requires template or templateFile")]
    CreateMissingTemplate,

    #[error("create step cannot have both template and templateFile")]
    CreateConflictingTemplate,
}

pub fn validate_recipe(recipe: &Recipe) -> Result<(), Vec<ValidationError>> {
    validate_recipe_with(recipe, |_| true)
}

pub fn validate_recipe_with(
    recipe: &Recipe,
    is_known_language: impl Fn(&str) -> bool,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let mut arg_names = std::collections::BTreeSet::new();
    for arg in &recipe.args {
        if !arg_names.insert(arg.name.clone()) {
            errors.push(ValidationError::DuplicateArgName(arg.name.clone()));
        }
    }

    if recipe.steps.is_empty() {
        errors.push(ValidationError::MissingRequiredField {
            op: "recipe",
            field: "steps",
        });
    }

    for step in &recipe.steps {
        validate_step(step, &is_known_language, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_step(
    step: &Step,
    is_known_language: &impl Fn(&str) -> bool,
    errors: &mut Vec<ValidationError>,
) {
    match step {
        Step::Edit(edit) => {
            validate_edit(edit, errors);
            if let Some(lang) = edit.language.as_deref() {
                let lang = lang.trim();
                if lang.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "edit",
                        field: "language",
                    });
                } else if !is_known_language(lang) {
                    errors.push(ValidationError::UnknownLanguage(lang.to_string()));
                }
            }
        }
        Step::Create(create) => validate_create(create, errors),
        Step::Delete(delete) => validate_delete(delete, errors),
        Step::RecipeRef(recipe_ref) => {
            if recipe_ref.id.trim().is_empty() {
                errors.push(ValidationError::MissingRequiredField {
                    op: "recipe",
                    field: "id",
                });
            }
        }
        Step::Scoped(scoped) => {
            for inner in &scoped.steps {
                validate_step(inner, is_known_language, errors);
            }
        }
        Step::Unknown(kind, _) => {
            errors.push(ValidationError::UnsupportedStep(kind.to_string()));
        }
    }
}

fn validate_edit(edit: &EditStep, errors: &mut Vec<ValidationError>) {
    if edit.path.trim().is_empty() {
        errors.push(ValidationError::MissingRequiredField {
            op: "edit",
            field: "path",
        });
    }
    if edit.ops.is_empty() {
        errors.push(ValidationError::EmptyEditOps);
    }
    for op in &edit.ops {
        match op {
            EditOp::Insert(insert) => {
                if insert.query.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "insert",
                        field: "query",
                    });
                }
                if insert.capture.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "insert",
                        field: "capture",
                    });
                }
            }
            EditOp::Replace(replace) => {
                if replace.query.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "replace",
                        field: "query",
                    });
                }
                if replace.capture.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "replace",
                        field: "capture",
                    });
                }
                if replace.text.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "replace",
                        field: "text",
                    });
                }
            }
            EditOp::Remove(remove) => {
                if remove.query.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "remove",
                        field: "query",
                    });
                }
                if remove.capture.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: "remove",
                        field: "capture",
                    });
                }
            }
            EditOp::Unknown(kind, _) => {
                errors.push(ValidationError::UnsupportedOp(kind.to_string()));
            }
        }
    }
}

fn validate_create(create: &CreateStep, errors: &mut Vec<ValidationError>) {
    if create.path.trim().is_empty() {
        errors.push(ValidationError::MissingRequiredField {
            op: "create",
            field: "path",
        });
    }
    let has_template = create.template.as_ref().is_some_and(|t| !t.is_empty());
    let has_file = create
        .template_file
        .as_ref()
        .is_some_and(|t| !t.is_empty());
    match (has_template, has_file) {
        (false, false) => errors.push(ValidationError::CreateMissingTemplate),
        (true, true) => errors.push(ValidationError::CreateConflictingTemplate),
        _ => {}
    }
}

fn validate_delete(delete: &DeleteStep, errors: &mut Vec<ValidationError>) {
    if delete.path.trim().is_empty() {
        errors.push(ValidationError::MissingRequiredField {
            op: "delete",
            field: "path",
        });
    }
}
