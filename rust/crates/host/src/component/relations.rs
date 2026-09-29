use std::collections::{BTreeMap, BTreeSet};

use mlua::{Function, Lua, Table};
use serde::Serialize;

use codemod_recipe_lua::eval_return_table;
use codemod_recipe_lua::json_lua::json_to_lua;

use crate::component::types::{context_path, ComponentFinding, ComponentInstance};

/// Luau chunk building the `index` / `graph` facades from plain JSON data.
/// Kept in Luau (not mlua closures) so the query logic stays readable and hackable.
///
/// Vocabulary: instances are the graph **nodes**, relations are **edges**. The
/// parse-hook payload on a node is `inst.ast` ("node" only ever means a vertex).
const GRAPH_FACADE_LUA: &str = r#"
local M = {}

local function index_maps(flat)
	local byId, byComponent = {}, {}
	for _, inst in ipairs(flat) do
		byId[inst.id] = inst
		local list = byComponent[inst.componentId]
		if list == nil then
			list = {}
			byComponent[inst.componentId] = list
		end
		list[#list + 1] = inst
	end
	return byId, byComponent
end

local function filter_kind(list, kind)
	if kind == nil then
		return list
	end
	local out = {}
	for _, edge in ipairs(list) do
		if edge.kind == kind then
			out[#out + 1] = edge
		end
	end
	return out
end

function M.build_index(flat)
	local byId, byComponent = index_maps(flat)
	return {
		byId = function(_self, id)
			return byId[id]
		end,
		byComponent = function(_self, componentId)
			local list = byComponent[componentId]
			if list == nil then
				return {}
			end
			return list
		end,
		nodes = function(_self)
			return flat
		end,
	}
end

function M.build_graph(flat, edges)
	local byId, byComponent = index_maps(flat)
	local out, inc = {}, {}
	for _, edge in ipairs(edges) do
		local outList = out[edge.from]
		if outList == nil then
			outList = {}
			out[edge.from] = outList
		end
		outList[#outList + 1] = edge
		if edge.to ~= nil then
			local incList = inc[edge.to]
			if incList == nil then
				incList = {}
				inc[edge.to] = incList
			end
			incList[#incList + 1] = edge
		end
	end

	-- Node-centric traversal: every node table carries bound methods via metatable.
	-- Methods live on __index, so pairs()/JSON serialization stay clean.
	local nodeMT = {
		__index = function(inst, key)
			if key == "outgoing" then
				return function(_self, kind)
					return filter_kind(out[inst.id] or {}, kind)
				end
			elseif key == "incoming" then
				return function(_self, kind)
					return filter_kind(inc[inst.id] or {}, kind)
				end
			elseif key == "follow" then
				-- Other endpoint of the edge (nil for external symbol targets).
				return function(_self, edge)
					if edge.from == inst.id then
						if edge.to == nil then
							return nil
						end
						return byId[edge.to]
					end
					return byId[edge.from]
				end
			elseif key == "neighbors" then
				-- Deduped adjacent nodes across both directions (kind-filtered).
				return function(_self, kind)
					local seen, result = {}, {}
					local function add(id)
						if id ~= nil and id ~= inst.id and seen[id] == nil and byId[id] ~= nil then
							seen[id] = true
							result[#result + 1] = byId[id]
						end
					end
					for _, edge in ipairs(out[inst.id] or {}) do
						if kind == nil or edge.kind == kind then
							add(edge.to)
						end
					end
					for _, edge in ipairs(inc[inst.id] or {}) do
						if kind == nil or edge.kind == kind then
							add(edge.from)
						end
					end
					return result
				end
			end
			return nil
		end,
	}
	for _, inst in ipairs(flat) do
		setmetatable(inst, nodeMT)
	end

	return {
		byId = function(_self, id)
			return byId[id]
		end,
		byComponent = function(_self, componentId)
			local list = byComponent[componentId]
			if list == nil then
				return {}
			end
			return list
		end,
		nodes = function(_self)
			return flat
		end,
		edges = function(_self)
			return edges
		end,
		outgoing = function(_self, id, kind)
			return filter_kind(out[id] or {}, kind)
		end,
		incoming = function(_self, id, kind)
			return filter_kind(inc[id] or {}, kind)
		end,
		resolve = function(_self, edge)
			if edge.to == nil then
				return nil
			end
			return byId[edge.to]
		end,
		resolveFrom = function(_self, edge)
			return byId[edge.from]
		end,
		reachable = function(_self, fromId, kind)
			local seen = {}
			local queue = { fromId }
			while #queue > 0 do
				local current = table.remove(queue, 1)
				for _, edge in ipairs(out[current] or {}) do
					if (kind == nil or edge.kind == kind) and edge.to ~= nil and seen[edge.to] == nil then
						seen[edge.to] = true
						queue[#queue + 1] = edge.to
					end
				end
			end
			return seen
		end,
	}
end

return M
"#;

/// A resolved relation between two flat instances (or an external symbol).
/// Duplicates by `(from, kind, target)` merge into one canonical edge that always
/// carries `sources` (multiplicity is `sources.len()` — no stored count).
/// Extra fields from the component's edge table pass through onto the JSON.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentEdge {
    pub from: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub sources: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Kind-level aggregation entry: `(fromComponentId, kind, toComponentId)` + count.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindEdge {
    pub from: String,
    pub kind: String,
    pub to: String,
    pub count: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub external: bool,
}

/// A node in the instance forest with a `componentId`, flattened for graph phases.
#[derive(Debug, Clone)]
pub struct FlatInstance {
    pub id: String,
    pub component_id: String,
    /// Parse-hook payload (the AST projection); serialized as `ast`.
    pub ast: serde_json::Value,
    pub locator: serde_json::Value,
    pub parent_id: Option<String>,
    pub root_id: String,
}

impl FlatInstance {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "componentId": self.component_id,
            "locator": self.locator,
            "ast": self.ast,
            "parentId": self.parent_id,
            "rootId": self.root_id,
        })
    }
}

/// Flat identity for child nodes: `id` → `name` → filename stem → `unknown`.
/// (Roots keep the original `instance_id` rule: `id` → stem → `unknown`.)
fn child_instance_id(
    component_id: &str,
    node: &serde_json::Value,
    locator: &serde_json::Value,
) -> String {
    if let Some(id) = node.get("id").and_then(|v| v.as_str()) {
        return format!("{component_id}/{id}");
    }
    if let Some(name) = node.get("name").and_then(|v| v.as_str()) {
        return format!("{component_id}/{name}");
    }
    if let Some(path) = context_path(locator) {
        if let Some(stem) = crate::sandbox::path_template::stem_from_path(path) {
            return format!("{component_id}/{stem}");
        }
    }
    format!("{component_id}/unknown")
}

fn child_locator(
    child: &serde_json::Value,
    parent_locator: &serde_json::Value,
) -> serde_json::Value {
    if let Some(source) = child.get("source") {
        return serde_json::json!({ "source": source });
    }
    parent_locator.clone()
}

/// Flatten the instance forest: every node with a `componentId` under `children`
/// (recursively) becomes a flat instance alongside the roots.
pub fn flatten_instances(instances: &[ComponentInstance]) -> Vec<FlatInstance> {
    let mut out = Vec::new();
    for instance in instances {
        flatten_node(
            &instance.ast,
            &instance.id,
            &instance.component_id,
            &instance.locator,
            None,
            &instance.id,
            &mut out,
        );
    }
    out
}

fn flatten_node(
    node: &serde_json::Value,
    id: &str,
    component_id: &str,
    locator: &serde_json::Value,
    parent_id: Option<String>,
    root_id: &str,
    out: &mut Vec<FlatInstance>,
) {
    out.push(FlatInstance {
        id: id.to_string(),
        component_id: component_id.to_string(),
        ast: node.clone(),
        locator: locator.clone(),
        parent_id: parent_id.clone(),
        root_id: root_id.to_string(),
    });
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            let Some(child_component) = child.get("componentId").and_then(|v| v.as_str()) else {
                continue;
            };
            let locator = child_locator(child, locator);
            let child_id = child_instance_id(child_component, child, &locator);
            flatten_node(
                child,
                &child_id,
                child_component,
                &locator,
                Some(id.to_string()),
                root_id,
                out,
            );
        }
    }
}

/// An edge table as returned from a component's `relate` hook (pre-resolution).
#[derive(Debug, Clone)]
pub struct RawEdge {
    pub from: String,
    pub kind: String,
    pub to: Option<serde_json::Value>,
    pub symbol: Option<String>,
    pub source: Option<serde_json::Value>,
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Normalize a `relate` return value: nil → none, object → one edge, array → many.
/// `default_from` is the relating instance's flat id.
pub fn normalize_raw_edges(
    raw: serde_json::Value,
    default_from: &str,
) -> (Vec<RawEdge>, Vec<ComponentFinding>) {
    let items = match raw {
        serde_json::Value::Null => return (Vec::new(), Vec::new()),
        serde_json::Value::Array(arr) => arr,
        obj @ serde_json::Value::Object(_) => vec![obj],
        _ => {
            return (
                Vec::new(),
                vec![relation_finding(
                    "relation.invalid",
                    "relate must return an edge table, an array of edges, or nil",
                    default_from,
                    None,
                )],
            );
        }
    };
    let mut edges = Vec::new();
    let mut findings = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            findings.push(relation_finding(
                "relation.invalid",
                "edge must be a table",
                default_from,
                None,
            ));
            continue;
        };
        let from = obj
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or(default_from)
            .to_string();
        let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) else {
            findings.push(relation_finding(
                "relation.invalid",
                "edge is missing kind",
                &from,
                obj.get("source").cloned(),
            ));
            continue;
        };
        let source = obj.get("source").cloned();
        // Target forms: `to` as instanceId string / `{ componentId, key }` ref /
        // `{ symbol = "..." }` table, or top-level `symbol = "..."`.
        let mut to = obj.get("to").cloned();
        let mut symbol = obj
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        if let Some(serde_json::Value::Object(map)) = &to {
            if let Some(sym) = map.get("symbol").and_then(|v| v.as_str()) {
                symbol = Some(sym.to_string());
                to = None;
            }
        }
        if to.is_none() && symbol.is_none() {
            findings.push(relation_finding(
                "relation.invalid",
                &format!("edge '{kind}' is missing a target (to or symbol)"),
                &from,
                source,
            ));
            continue;
        }
        let mut extra = BTreeMap::new();
        for (key, value) in obj {
            if !matches!(key.as_str(), "from" | "kind" | "to" | "symbol" | "source") {
                extra.insert(key.clone(), value.clone());
            }
        }
        edges.push(RawEdge {
            from,
            kind: kind.to_string(),
            to,
            symbol,
            source,
            extra,
        });
    }
    (edges, findings)
}

/// Resolve raw edges against the flat forest, merging duplicates by
/// `(from, kind, target)` into one canonical edge (provenance accumulates in
/// `sources`; extras stay first-wins), and emit `relation.unresolved` findings for
/// dangling refs. Identical-provenance duplicates yield one `relation.duplicate`
/// info finding per canonical edge; distinct provenance merges silently.
pub fn resolve_edges(
    raw_edges: Vec<RawEdge>,
    flat: &[FlatInstance],
) -> (Vec<ComponentEdge>, Vec<ComponentFinding>) {
    let by_id: BTreeMap<&str, &FlatInstance> = flat.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut by_component: BTreeMap<&str, Vec<&FlatInstance>> = BTreeMap::new();
    for f in flat {
        by_component
            .entry(f.component_id.as_str())
            .or_default()
            .push(f);
    }

    let mut edges: Vec<ComponentEdge> = Vec::new();
    let mut findings = Vec::new();
    // Canonical edge per `(from, kind, target)` → index into `edges`.
    let mut canonical: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    // All provenance values seen per canonical edge (including absent sources), so
    // identical-provenance duplicates can be told apart from distinct derivations.
    let mut provenance_seen: BTreeMap<(String, String, String), Vec<Option<serde_json::Value>>> =
        BTreeMap::new();
    let mut reported_duplicates: BTreeSet<(String, String, String)> = BTreeSet::new();
    // Findings dedupe by raw target descriptor: an unresolved duplicate carries no
    // information either (edges are facts, not events — and neither are findings).
    let mut seen_findings: BTreeSet<(String, String, String)> = BTreeSet::new();

    for raw in raw_edges {
        let raw_target_key = match &raw.to {
            Some(serde_json::Value::String(s)) => format!("id:{s}"),
            Some(serde_json::Value::Object(map)) => format!(
                "ref:{}/{}",
                map.get("componentId")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                map.get("key").and_then(|v| v.as_str()).unwrap_or("")
            ),
            _ => raw
                .symbol
                .clone()
                .map(|s| format!("symbol:{s}"))
                .unwrap_or_default(),
        };
        if !by_id.contains_key(raw.from.as_str()) {
            if seen_findings.insert((raw.from.clone(), raw.kind.clone(), raw_target_key)) {
                findings.push(relation_finding(
                    "relation.unresolved",
                    &format!("edge '{}' has unknown from instance {}", raw.kind, raw.from),
                    &raw.from,
                    raw.source.clone(),
                ));
            }
            continue;
        }
        let (to, symbol) = match &raw.to {
            None => (None, raw.symbol.clone()),
            Some(serde_json::Value::String(s)) => {
                if by_id.contains_key(s.as_str()) {
                    (Some(s.clone()), None)
                } else {
                    if seen_findings.insert((raw.from.clone(), raw.kind.clone(), raw_target_key)) {
                        findings.push(relation_finding(
                            "relation.unresolved",
                            &format!("{} --{}--> unresolved instance id {s}", raw.from, raw.kind),
                            &raw.from,
                            raw.source.clone(),
                        ));
                    }
                    continue;
                }
            }
            Some(serde_json::Value::Object(map)) => {
                let component_id = map.get("componentId").and_then(|v| v.as_str());
                let key = map.get("key").and_then(|v| v.as_str());
                match (component_id, key) {
                    (Some(component_id), Some(key)) => {
                        let direct = format!("{component_id}/{key}");
                        let resolved = if by_id.contains_key(direct.as_str()) {
                            Some(direct)
                        } else {
                            by_component
                                .get(component_id)
                                .and_then(|candidates| {
                                    candidates.iter().find(|f| {
                                        f.ast.get("id").and_then(|v| v.as_str()) == Some(key)
                                    })
                                })
                                .map(|f| f.id.clone())
                        };
                        match resolved {
                            Some(id) => (Some(id), None),
                            None => {
                                if seen_findings.insert((
                                    raw.from.clone(),
                                    raw.kind.clone(),
                                    raw_target_key,
                                )) {
                                    findings.push(relation_finding(
                                        "relation.unresolved",
                                        &format!(
                                            "{} --{}--> unresolved ref {component_id}/{key}",
                                            raw.from, raw.kind
                                        ),
                                        &raw.from,
                                        raw.source.clone(),
                                    ));
                                }
                                continue;
                            }
                        }
                    }
                    _ => {
                        if seen_findings.insert((
                            raw.from.clone(),
                            raw.kind.clone(),
                            raw_target_key,
                        )) {
                            findings.push(relation_finding(
                                "relation.invalid",
                                &format!(
                                    "edge '{}' target table requires componentId + key",
                                    raw.kind
                                ),
                                &raw.from,
                                raw.source.clone(),
                            ));
                        }
                        continue;
                    }
                }
            }
            Some(_) => {
                if seen_findings.insert((raw.from.clone(), raw.kind.clone(), raw_target_key)) {
                    findings.push(relation_finding(
                        "relation.invalid",
                        &format!("edge '{}' target must be a string or table", raw.kind),
                        &raw.from,
                        raw.source.clone(),
                    ));
                }
                continue;
            }
        };

        let target_key = to
            .clone()
            .map(|t| format!("id:{t}"))
            .or_else(|| symbol.clone().map(|s| format!("symbol:{s}")))
            .unwrap_or_default();
        let key = (raw.from.clone(), raw.kind.clone(), target_key);
        if let Some(&idx) = canonical.get(&key) {
            // Merge into the canonical edge: edges are facts, not events.
            let provenance = provenance_seen.entry(key.clone()).or_default();
            if provenance.contains(&raw.source) {
                // Same derivation seen again — redundant relate output.
                if reported_duplicates.insert(key) {
                    findings.push(relation_info_finding(
                        "relation.duplicate",
                        &format!(
                            "duplicate edge '{}' from {} (identical provenance) — adjust relate",
                            raw.kind, raw.from
                        ),
                        &raw.from,
                        raw.source.clone(),
                    ));
                }
            } else {
                if let Some(source) = &raw.source {
                    edges[idx].sources.push(source.clone());
                }
                provenance.push(raw.source.clone());
            }
            continue;
        }
        canonical.insert(key.clone(), edges.len());
        provenance_seen.insert(key, vec![raw.source.clone()]);
        edges.push(ComponentEdge {
            from: raw.from,
            kind: raw.kind,
            to,
            symbol,
            sources: raw.source.clone().into_iter().collect(),
            extra: raw.extra,
        });
    }
    (edges, findings)
}

/// Aggregate resolved edges into the kind-level architecture graph.
pub fn kind_graph(edges: &[ComponentEdge], flat: &[FlatInstance]) -> Vec<KindEdge> {
    let by_id: BTreeMap<&str, &FlatInstance> = flat.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut counts: BTreeMap<(String, String, String, bool), usize> = BTreeMap::new();
    for edge in edges {
        let Some(from) = by_id.get(edge.from.as_str()) else {
            continue;
        };
        let (to, external) = match (&edge.to, &edge.symbol) {
            (Some(to_id), _) => match by_id.get(to_id.as_str()) {
                Some(target) => (target.component_id.clone(), false),
                None => continue,
            },
            (None, Some(symbol)) => (symbol.clone(), true),
            (None, None) => continue,
        };
        *counts
            .entry((from.component_id.clone(), edge.kind.clone(), to, external))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .map(|((from, kind, to, external), count)| KindEdge {
            from,
            kind,
            to,
            count,
            external,
        })
        .collect()
}

/// Attach outgoing edges onto instance nodes as `node.relations` (grouped by kind),
/// replacing any previously attached relations.
pub fn attach_relations(instances: &mut [ComponentInstance], edges: &[ComponentEdge]) {
    let mut by_from: BTreeMap<&str, BTreeMap<String, Vec<serde_json::Value>>> = BTreeMap::new();
    for edge in edges {
        let mut entry = serde_json::Map::new();
        if let Some(to) = &edge.to {
            entry.insert("to".to_string(), serde_json::Value::String(to.clone()));
        }
        if let Some(symbol) = &edge.symbol {
            entry.insert(
                "symbol".to_string(),
                serde_json::Value::String(symbol.clone()),
            );
        }
        entry.insert(
            "sources".to_string(),
            serde_json::Value::Array(edge.sources.clone()),
        );
        for (key, value) in &edge.extra {
            entry.insert(key.clone(), value.clone());
        }
        by_from
            .entry(edge.from.as_str())
            .or_default()
            .entry(edge.kind.clone())
            .or_default()
            .push(serde_json::Value::Object(entry));
    }

    for instance in instances {
        let root_id = instance.id.clone();
        let locator = instance.locator.clone();
        attach_node_relations(&mut instance.ast, &root_id, &locator, &by_from);
    }
}

fn attach_node_relations(
    node: &mut serde_json::Value,
    id: &str,
    locator: &serde_json::Value,
    by_from: &BTreeMap<&str, BTreeMap<String, Vec<serde_json::Value>>>,
) {
    if let Some(obj) = node.as_object_mut() {
        match by_from.get(id) {
            Some(kinds) => {
                let relations: serde_json::Map<String, serde_json::Value> = kinds
                    .iter()
                    .map(|(kind, entries)| {
                        (kind.clone(), serde_json::Value::Array(entries.clone()))
                    })
                    .collect();
                obj.insert(
                    "relations".to_string(),
                    serde_json::Value::Object(relations),
                );
            }
            None => {
                obj.remove("relations");
            }
        }
        if let Some(children) = obj.get_mut("children").and_then(|v| v.as_array_mut()) {
            for child in children {
                let Some(child_component) = child.get("componentId").and_then(|v| v.as_str())
                else {
                    continue;
                };
                let child_locator = child_locator(child, locator);
                let child_id = child_instance_id(child_component, child, &child_locator);
                attach_node_relations(child, &child_id, &child_locator, by_from);
            }
        }
    }
}

fn relation_finding(
    code: &str,
    message: &str,
    instance_id: &str,
    locator: Option<serde_json::Value>,
) -> ComponentFinding {
    ComponentFinding {
        severity: "warning".to_string(),
        code: code.to_string(),
        message: message.to_string(),
        instance_id: Some(instance_id.to_string()),
        locator,
    }
}

fn relation_info_finding(
    code: &str,
    message: &str,
    instance_id: &str,
    locator: Option<serde_json::Value>,
) -> ComponentFinding {
    ComponentFinding {
        severity: "info".to_string(),
        ..relation_finding(code, message, instance_id, locator)
    }
}

/// Build the `index` facade (relate phase) inside the given sandbox.
pub fn build_index_facade(lua: &Lua, flat: &[FlatInstance]) -> Result<Table, String> {
    let module =
        eval_return_table(lua, GRAPH_FACADE_LUA, "graph_facade").map_err(|e| e.to_string())?;
    let build: Function = module
        .get("build_index")
        .map_err(|e| format!("graph facade build_index missing: {e}"))?;
    let flat_json: Vec<serde_json::Value> = flat.iter().map(|f| f.to_json()).collect();
    let data = json_to_lua(lua, &serde_json::Value::Array(flat_json))?;
    build
        .call::<Table>(data)
        .map_err(|e| format!("graph facade build_index failed: {e}"))
}

/// Build the `graph` facade (verify phase, operations) inside the given sandbox.
pub fn build_graph_facade(
    lua: &Lua,
    flat: &[FlatInstance],
    edges: &[ComponentEdge],
) -> Result<Table, String> {
    let module =
        eval_return_table(lua, GRAPH_FACADE_LUA, "graph_facade").map_err(|e| e.to_string())?;
    let build: Function = module
        .get("build_graph")
        .map_err(|e| format!("graph facade build_graph missing: {e}"))?;
    let flat_json: Vec<serde_json::Value> = flat.iter().map(|f| f.to_json()).collect();
    let data = json_to_lua(lua, &serde_json::Value::Array(flat_json))?;
    let edges_json = json_to_lua(
        lua,
        &serde_json::to_value(edges).map_err(|e| e.to_string())?,
    )?;
    build
        .call::<Table>((data, edges_json))
        .map_err(|e| format!("graph facade build_graph failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instance(id: &str, component_id: &str, ast: serde_json::Value) -> ComponentInstance {
        ComponentInstance {
            id: id.to_string(),
            component_id: component_id.to_string(),
            locator: serde_json::json!({ "path": "lib/x.dart" }),
            ast,
            summary: None,
        }
    }

    #[test]
    fn flatten_includes_children_with_component_id() {
        let root = instance(
            "app.model/recipe",
            "app.model",
            serde_json::json!({
                "id": "recipe",
                "children": [
                    { "componentId": "app.property", "name": "title" },
                    { "componentId": "app.property", "name": "subtitle" },
                    { "name": "plain data node — skipped" }
                ]
            }),
        );
        let flat = flatten_instances(&[root]);
        let ids: Vec<&str> = flat.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "app.model/recipe",
                "app.property/title",
                "app.property/subtitle"
            ]
        );
        assert_eq!(flat[1].parent_id.as_deref(), Some("app.model/recipe"));
        assert_eq!(flat[1].root_id, "app.model/recipe");
    }

    #[test]
    fn resolve_edges_resolves_refs_and_merges_duplicates() {
        let flat = flatten_instances(&[
            instance(
                "app.model/recipe",
                "app.model",
                serde_json::json!({ "id": "recipe" }),
            ),
            instance(
                "app.store/recipe",
                "app.store",
                serde_json::json!({ "id": "recipe" }),
            ),
        ]);
        let raw = vec![
            RawEdge {
                from: "app.store/recipe".to_string(),
                kind: "stores".to_string(),
                to: Some(serde_json::json!({ "componentId": "app.model", "key": "recipe" })),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
            RawEdge {
                from: "app.store/recipe".to_string(),
                kind: "stores".to_string(),
                to: Some(serde_json::json!("app.model/recipe")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
        ];
        let (edges, findings) = resolve_edges(raw, &flat);
        assert_eq!(edges.len(), 1, "duplicate edge must merge into one");
        assert_eq!(edges[0].to.as_deref(), Some("app.model/recipe"));
        assert!(edges[0].sources.is_empty(), "no provenance given");
        // Both duplicates lack provenance → identical → one info finding.
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "relation.duplicate");
        assert_eq!(findings[0].severity, "info");
    }

    #[test]
    fn resolve_edges_merges_distinct_provenance_silently() {
        let flat = flatten_instances(&[
            instance("app.model/a", "app.model", serde_json::json!({ "id": "a" })),
            instance("app.store/a", "app.store", serde_json::json!({ "id": "a" })),
        ]);
        let mk = |kind: &str, path: &str| RawEdge {
            from: "app.store/a".to_string(),
            kind: kind.to_string(),
            to: Some(serde_json::json!("app.model/a")),
            symbol: None,
            source: Some(serde_json::json!({ "path": path })),
            extra: BTreeMap::new(),
        };
        let (edges, findings) = resolve_edges(
            vec![
                mk("stores", "lib/one.dart"),
                mk("stores", "lib/two.dart"),
                // Different kind: never merges, even with identical provenance.
                mk("watches", "lib/one.dart"),
            ],
            &flat,
        );
        assert_eq!(edges.len(), 2, "kind is part of the merge key");
        let stores = edges.iter().find(|e| e.kind == "stores").expect("stores");
        assert_eq!(stores.sources.len(), 2, "multiplicity is #sources");
        assert!(
            findings.is_empty(),
            "distinct provenance merges without findings"
        );
    }

    #[test]
    fn resolve_edges_flags_unresolved_refs() {
        let flat = flatten_instances([].as_slice());
        let raw = vec![RawEdge {
            from: "app.store/recipe".to_string(),
            kind: "stores".to_string(),
            to: Some(serde_json::json!({ "componentId": "app.model", "key": "ghost" })),
            symbol: None,
            source: None,
            extra: BTreeMap::new(),
        }];
        // from is unknown too, so we get the from-finding only
        let (edges, findings) = resolve_edges(raw, &flat);
        assert!(edges.is_empty());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "relation.unresolved");
    }

    #[test]
    fn kind_graph_aggregates_with_counts() {
        let flat = flatten_instances(&[
            instance("app.model/a", "app.model", serde_json::json!({ "id": "a" })),
            instance("app.model/b", "app.model", serde_json::json!({ "id": "b" })),
            instance("app.store/a", "app.store", serde_json::json!({ "id": "a" })),
            instance("app.store/b", "app.store", serde_json::json!({ "id": "b" })),
        ]);
        let mk = |from: &str, to: &str| RawEdge {
            from: from.to_string(),
            kind: "stores".to_string(),
            to: Some(serde_json::json!(to)),
            symbol: None,
            source: None,
            extra: BTreeMap::new(),
        };
        let (edges, _) = resolve_edges(
            vec![
                mk("app.store/a", "app.model/a"),
                mk("app.store/b", "app.model/b"),
            ],
            &flat,
        );
        let graph = kind_graph(&edges, &flat);
        assert_eq!(graph.len(), 1);
        assert_eq!(graph[0].from, "app.store");
        assert_eq!(graph[0].to, "app.model");
        assert_eq!(graph[0].count, 2);
        assert!(!graph[0].external);
    }

    #[test]
    fn attach_relations_groups_by_kind_on_nodes() {
        let mut root = instance(
            "app.model/recipe",
            "app.model",
            serde_json::json!({
                "id": "recipe",
                "children": [ { "componentId": "app.property", "name": "title" } ]
            }),
        );
        let flat = flatten_instances(&[root.clone()]);
        let raw = vec![
            RawEdge {
                from: "app.model/recipe".to_string(),
                kind: "has".to_string(),
                to: Some(serde_json::json!("app.property/title")),
                symbol: None,
                source: Some(serde_json::json!({ "path": "lib/x.dart" })),
                extra: BTreeMap::from([("cardinality".to_string(), serde_json::json!("one"))]),
            },
            RawEdge {
                from: "app.model/recipe".to_string(),
                kind: "extends".to_string(),
                to: None,
                symbol: Some("dart:core/Object".to_string()),
                source: None,
                extra: BTreeMap::new(),
            },
        ];
        let (edges, findings) = resolve_edges(raw, &flat);
        assert!(findings.is_empty());
        attach_relations(std::slice::from_mut(&mut root), &edges);
        let relations = root.ast["relations"].as_object().expect("relations");
        assert_eq!(relations["has"][0]["to"], "app.property/title");
        assert_eq!(relations["has"][0]["cardinality"], "one");
        assert_eq!(relations["has"][0]["sources"][0]["path"], "lib/x.dart");
        assert_eq!(relations["extends"][0]["symbol"], "dart:core/Object");
        assert_eq!(
            relations["extends"][0]["sources"],
            serde_json::json!([]),
            "sources is always a list"
        );
    }
}
