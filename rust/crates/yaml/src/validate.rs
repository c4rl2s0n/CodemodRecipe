use crate::dsl;
use crate::model::*;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("unsupported top-level step kind: {0}")]
    UnsupportedStep(String),

    #[error("unsupported edit op kind: {0}")]
    UnsupportedOp(String),

    #[error("{op} op missing required field: {field}")]
    MissingRequiredField {
        op: &'static str,
        field: &'static str,
    },

    #[error("edit step has no ops")]
    EmptyEditOps,

    #[error("duplicate arg name: {0}")]
    DuplicateArgName(String),

    #[error("language not supported: {0}")]
    LanguageNotSupported(String),

    #[error("create step requires template or templateFile")]
    CreateMissingTemplate,

    #[error("create step cannot have both template and templateFile")]
    CreateConflictingTemplate,

    #[error("let binding name collides with recipe arg: {0}")]
    LetNameCollidesWithArg(String),

    #[error("let binding '{name}' requires query or as")]
    LetBindingMissingQuery { name: String },
}

impl ValidationError {
    /// Fragment to locate this error in recipe YAML source (first-match needle).
    pub fn needle(&self) -> String {
        match self {
            ValidationError::UnsupportedStep(kind) | ValidationError::UnsupportedOp(kind) => {
                kind.clone()
            }
            ValidationError::MissingRequiredField { field, .. } => format!("{field}:"),
            ValidationError::EmptyEditOps => format!("{}:", dsl::recipe::steps::edit::field::OPS),
            ValidationError::DuplicateArgName(name)
            | ValidationError::LetNameCollidesWithArg(name) => {
                format!("name: {name}")
            }
            ValidationError::LanguageNotSupported(lang) => lang.clone(),
            ValidationError::CreateMissingTemplate => {
                format!("{}:", dsl::recipe::steps::create::WIRE)
            }
            ValidationError::CreateConflictingTemplate => {
                format!(
                    "{}:",
                    dsl::recipe::steps::create::field::TEMPLATE_FILE
                )
            }
            ValidationError::LetBindingMissingQuery { name } => format!("name: {name}"),
        }
    }
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
            op: dsl::recipe::steps::recipe_ref::WIRE,
            field: dsl::recipe::field::STEPS,
        });
    }

    let arg_names: std::collections::BTreeSet<String> =
        recipe.args.iter().map(|a| a.name.clone()).collect();

    for step in &recipe.steps {
        validate_step(step, &is_known_language, &arg_names, &mut errors);
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
    arg_names: &std::collections::BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    match step {
        Step::Edit(edit) => {
            validate_edit(edit, arg_names, errors);
            if let Some(lang) = edit.language.as_deref() {
                let lang = lang.trim();
                if lang.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::WIRE,
                        field: dsl::recipe::steps::edit::field::LANGUAGE,
                    });
                } else if !is_known_language(lang) {
                    errors.push(ValidationError::LanguageNotSupported(lang.to_string()));
                }
            }
        }
        Step::Create(create) => validate_create(create, errors),
        Step::Delete(delete) => validate_delete(delete, errors),
        Step::RecipeRef(recipe_ref) => {
            if recipe_ref.id.trim().is_empty() {
                errors.push(ValidationError::MissingRequiredField {
                    op: dsl::recipe::steps::recipe_ref::WIRE,
                    field: dsl::recipe::steps::recipe_ref::object::field::ID,
                });
            }
        }
        Step::Scoped(scoped) => {
            for inner in &scoped.steps {
                validate_step(inner, is_known_language, arg_names, errors);
            }
        }
        Step::Unknown(kind, _) => {
            errors.push(ValidationError::UnsupportedStep(kind.to_string()));
        }
    }
}

fn validate_edit(
    edit: &EditStep,
    arg_names: &std::collections::BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if edit.path.trim().is_empty() {
        errors.push(ValidationError::MissingRequiredField {
            op: dsl::recipe::steps::edit::WIRE,
            field: dsl::recipe::steps::edit::field::PATH,
        });
    }
    if edit.ops.is_empty() {
        errors.push(ValidationError::EmptyEditOps);
    }
    for binding in &edit.let_bindings.0 {
        if binding.name.trim().is_empty() {
            errors.push(ValidationError::MissingRequiredField {
                op: dsl::recipe::steps::edit::field::LET,
                field: dsl::recipe::steps::edit::let_binding::field::NAME,
            });
        } else if arg_names.contains(&binding.name) {
            errors.push(ValidationError::LetNameCollidesWithArg(
                binding.name.clone(),
            ));
        }
        if binding.query.is_none() && binding.r#as.is_none() {
            errors.push(ValidationError::LetBindingMissingQuery {
                name: binding.name.clone(),
            });
        }
        if binding.query.as_ref().is_some_and(|q| q.is_empty()) {
            errors.push(ValidationError::MissingRequiredField {
                op: dsl::recipe::steps::edit::field::LET,
                field: dsl::recipe::steps::edit::let_binding::field::QUERY,
            });
        }
    }
    for op in &edit.ops {
        match op {
            EditOp::Insert(insert) => {
                if insert.query.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::insert::WIRE,
                        field: dsl::recipe::steps::edit::ops::insert::field::QUERY,
                    });
                }
                if insert.capture.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::insert::WIRE,
                        field: dsl::recipe::steps::edit::ops::insert::field::CAPTURE,
                    });
                }
                if insert.text.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::insert::WIRE,
                        field: dsl::recipe::steps::edit::ops::insert::field::TEXT,
                    });
                }
            }
            EditOp::Replace(replace) => {
                if replace.query.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::replace::WIRE,
                        field: dsl::recipe::steps::edit::ops::replace::field::QUERY,
                    });
                }
                if replace.capture.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::replace::WIRE,
                        field: dsl::recipe::steps::edit::ops::replace::field::CAPTURE,
                    });
                }
                if replace.text.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::replace::WIRE,
                        field: dsl::recipe::steps::edit::ops::replace::field::TEXT,
                    });
                }
            }
            EditOp::Remove(remove) => {
                if remove.query.is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::remove::WIRE,
                        field: dsl::recipe::steps::edit::ops::remove::field::QUERY,
                    });
                }
                if remove.capture.trim().is_empty() {
                    errors.push(ValidationError::MissingRequiredField {
                        op: dsl::recipe::steps::edit::ops::remove::WIRE,
                        field: dsl::recipe::steps::edit::ops::remove::field::CAPTURE,
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
            op: dsl::recipe::steps::create::WIRE,
            field: dsl::recipe::steps::create::field::PATH,
        });
    }
    let has_template = create.template.as_ref().is_some_and(|t| !t.is_empty());
    let has_file = create.template_file.as_ref().is_some_and(|t| !t.is_empty());
    match (has_template, has_file) {
        (false, false) => errors.push(ValidationError::CreateMissingTemplate),
        (true, true) => errors.push(ValidationError::CreateConflictingTemplate),
        _ => {}
    }
}

fn validate_delete(delete: &DeleteStep, errors: &mut Vec<ValidationError>) {
    if delete.path.trim().is_empty() {
        errors.push(ValidationError::MissingRequiredField {
            op: dsl::recipe::steps::delete::WIRE,
            field: dsl::recipe::steps::delete::field::PATH,
        });
    }
}
