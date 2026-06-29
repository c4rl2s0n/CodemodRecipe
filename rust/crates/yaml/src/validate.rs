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
}

pub fn validate_recipe(recipe: &Recipe) -> Result<(), Vec<ValidationError>> {
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
        match step {
            Step::Edit(edit) => {
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
                            errors.push(ValidationError::UnsupportedOp(kind.to_string()))
                        }
                    }
                }
            }
            Step::Create(_) | Step::RecipeRef(_) => {
                // not implemented/validated in v1 slice; accepted for forward compat.
            }
            Step::Unknown(kind, _) => {
                errors.push(ValidationError::UnsupportedStep(kind.to_string()))
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
