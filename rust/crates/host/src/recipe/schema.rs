//! Catalog / describe projection helpers for [`super::registry::RecipeRegistry`].

use std::collections::BTreeMap;

use codemod_recipe_lua::model::{Arg, ArgFrom, ExplorerMenuEntry, TypeDef};
use codemod_recipe_lua::InputKind;

use crate::protocol::{
    ExplorerMenuEntrySchema, RecipeArg, TypeSchema,
};

pub(crate) fn arg_schema(
    arg: &Arg,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    types: &BTreeMap<String, TypeDef>,
) -> RecipeArg {
    let options = crate::recipe::args::choice_options(arg, maps);
    let allow_custom = !matches!(arg.input_kind, InputKind::Choice);
    let fields = arg.input_kind.type_id().and_then(|id| {
        types.get(id).map(|def| {
            def.fields
                .iter()
                .map(|f| arg_schema(f, maps, types))
                .collect()
        })
    });
    RecipeArg {
        name: arg.name.clone(),
        abbr: arg.abbr.clone(),
        help: arg.help.clone(),
        required: arg.required,
        defaults_to: arg.defaults_to.clone(),
        input_kind: arg.input_kind.to_json(),
        options,
        options_from_map: arg.options_from_map.clone(),
        allow_custom_value: arg.allow_custom_value.unwrap_or(allow_custom),
        context_key: None,
        from: arg.from.as_ref().map(|f| match f {
            ArgFrom::Builtin(s) => serde_json::Value::String(s.clone()),
            ArgFrom::Spec(spec) => serde_json::json!({
                "template": spec.template,
                "query": spec.query,
                "capture": spec.capture,
            }),
        }),
        from_recipes: vec![],
        fields,
    }
}

pub(crate) fn type_schema(
    def: &TypeDef,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    types: &BTreeMap<String, TypeDef>,
) -> TypeSchema {
    TypeSchema {
        id: def.id.clone(),
        description: def.description.clone(),
        fields: def
            .fields
            .iter()
            .map(|f| arg_schema(f, maps, types))
            .collect(),
    }
}

pub(crate) fn explorer_schema(entry: &ExplorerMenuEntry) -> ExplorerMenuEntrySchema {
    ExplorerMenuEntrySchema {
        kind: entry.kind.as_str().to_string(),
        if_expr: entry.if_expr.clone(),
        args: entry.args.clone(),
    }
}
