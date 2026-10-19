use mlua::{Table, Value};

use crate::model::{RelationKindDef, StackDef, ZoneDef, ZoneRole};

use super::error::LoadError;
use super::shared::parse_operations;
use super::table_util::{
    optional_bool, optional_string, optional_string_list, required_string, string_array,
};

pub fn load_stack_from_table(table: &Table, source: &str) -> Result<StackDef, LoadError> {
    let id = required_string(table, "id")?;
    let description = optional_string(table, "description")?;
    let components_tbl: Table = table
        .get("components")
        .map_err(|_| LoadError::Message("stack requires components = { ... }".to_string()))?;
    let components = string_array(&components_tbl)?;
    if components.is_empty() {
        return Err(LoadError::Message(
            "stack components must list at least one component id".to_string(),
        ));
    }
    let codebase = optional_string_list(table, "codebase")?;
    let operations = parse_operations(table)?;
    let relations = parse_relations(table)?;
    let zones = parse_zones(table)?;
    let zone_roles = parse_zone_roles(table)?;
    Ok(StackDef {
        id,
        description,
        components,
        codebase,
        operations,
        relations,
        zones,
        zone_roles,
        source: source.to_string(),
        path: None,
    })
}

pub(crate) fn parse_relations(
    table: &Table,
) -> Result<std::collections::BTreeMap<String, RelationKindDef>, LoadError> {
    let Ok(rels): Result<Table, _> = table.get("relations") else {
        return Ok(std::collections::BTreeMap::new());
    };
    let mut out = std::collections::BTreeMap::new();
    for pair in rels.pairs::<Value, Value>() {
        let (key, value) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        let kind = match key {
            Value::String(s) => s.to_str()?.to_string(),
            other => {
                return Err(LoadError::Message(format!(
                    "relations key must be a string kind name, got {}",
                    other.type_name()
                )));
            }
        };
        let layout = match value {
            Value::Table(tbl) => {
                let layout = optional_string(&tbl, "layout")?.unwrap_or_else(|| "edge".to_string());
                match layout.as_str() {
                    "edge" | "member" => layout,
                    other => {
                        return Err(LoadError::Message(format!(
                            "relations.{kind}.layout must be \"edge\" or \"member\", got {other:?}"
                        )));
                    }
                }
            }
            Value::String(s) => {
                let layout = s.to_str()?.to_string();
                match layout.as_str() {
                    "edge" | "member" => layout,
                    other => {
                        return Err(LoadError::Message(format!(
                            "relations.{kind} must be \"edge\" or \"member\", got {other:?}"
                        )));
                    }
                }
            }
            other => {
                return Err(LoadError::Message(format!(
                    "relations.{kind} must be a table or string, got {}",
                    other.type_name()
                )));
            }
        };
        out.insert(kind, RelationKindDef { layout });
    }
    Ok(out)
}

pub(crate) fn parse_zones(table: &Table) -> Result<Vec<ZoneDef>, LoadError> {
    let Ok(zones): Result<Table, _> = table.get("zones") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for value in zones.sequence_values::<Value>() {
        let value = value.map_err(|e| LoadError::Message(e.to_string()))?;
        let Value::Table(tbl) = value else {
            return Err(LoadError::Message(format!(
                "zones entries must be tables, got {}",
                value.type_name()
            )));
        };
        let id = required_string(&tbl, "id")?;
        if id.trim().is_empty() {
            return Err(LoadError::Message(
                "zones entry id must be non-empty".to_string(),
            ));
        }
        out.push(ZoneDef {
            id,
            role: optional_string(&tbl, "role")?,
            title: optional_string(&tbl, "title")?,
            parent_id: optional_string(&tbl, "parentId")?,
        });
    }
    Ok(out)
}

pub(crate) fn parse_zone_roles(
    table: &Table,
) -> Result<std::collections::BTreeMap<String, ZoneRole>, LoadError> {
    let Ok(roles): Result<Table, _> = table.get("zoneRoles") else {
        return Ok(std::collections::BTreeMap::new());
    };
    let mut out = std::collections::BTreeMap::new();
    for pair in roles.pairs::<Value, Value>() {
        let (key, value) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        let role_name = match key {
            Value::String(s) => s.to_str()?.to_string(),
            other => {
                return Err(LoadError::Message(format!(
                    "zoneRoles key must be a string role name, got {}",
                    other.type_name()
                )));
            }
        };
        let Value::Table(tbl) = value else {
            return Err(LoadError::Message(format!(
                "zoneRoles.{role_name} must be a table, got {}",
                value.type_name()
            )));
        };
        out.insert(
            role_name,
            ZoneRole {
                ignore_relations: optional_bool(&tbl, "ignoreRelations")?.unwrap_or(false),
                ignore_coverage: optional_bool(&tbl, "ignoreCoverage")?.unwrap_or(false),
            },
        );
    }
    Ok(out)
}

