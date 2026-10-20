--- Component scan pipeline stages (named files under host/src/component/).

local STAGES = {
	-- scan/
	{ id = "scan", path = "rust/crates/host/src/component/scan/mod.rs" },
	{ id = "scan_full", path = "rust/crates/host/src/component/scan/full.rs" },
	{ id = "scan_graph_phases", path = "rust/crates/host/src/component/scan/graph_phases.rs" },
	{ id = "scan_identify", path = "rust/crates/host/src/component/scan/identify.rs" },
	{ id = "scan_incremental", path = "rust/crates/host/src/component/scan/incremental.rs" },
	{ id = "scan_types", path = "rust/crates/host/src/component/scan/types.rs" },
	-- relations/
	{ id = "relations", path = "rust/crates/host/src/component/relations/mod.rs" },
	{ id = "relations_resolve", path = "rust/crates/host/src/component/relations/resolve/mod.rs" },
	{ id = "relations_resolve_normalize", path = "rust/crates/host/src/component/relations/resolve/normalize.rs" },
	{ id = "relations_resolve_edges", path = "rust/crates/host/src/component/relations/resolve/edges.rs" },
	{ id = "relations_resolve_kind", path = "rust/crates/host/src/component/relations/resolve/kind.rs" },
	{ id = "relations_resolve_targets", path = "rust/crates/host/src/component/relations/resolve/targets.rs" },
	{ id = "relations_resolve_findings", path = "rust/crates/host/src/component/relations/resolve/findings.rs" },
	{ id = "relations_attach", path = "rust/crates/host/src/component/relations/attach.rs" },
	{ id = "relations_facade", path = "rust/crates/host/src/component/relations/facade/mod.rs" },
	{ id = "relations_facade_builders", path = "rust/crates/host/src/component/relations/facade/builders.rs" },
	{ id = "relations_types", path = "rust/crates/host/src/component/relations/types.rs" },
	-- runtime/
	{ id = "runtime", path = "rust/crates/host/src/component/runtime/mod.rs" },
	{ id = "runtime_context", path = "rust/crates/host/src/component/runtime/context.rs" },
	{ id = "runtime_hook", path = "rust/crates/host/src/component/runtime/hook.rs" },
	{ id = "runtime_operations", path = "rust/crates/host/src/component/runtime/operations.rs" },
	{ id = "runtime_scaffold", path = "rust/crates/host/src/component/runtime/scaffold.rs" },
	{ id = "runtime_stack_ops", path = "rust/crates/host/src/component/runtime/stack_ops.rs" },
	{ id = "runtime_finalize", path = "rust/crates/host/src/component/runtime/finalize.rs" },
	-- other stages
	{ id = "relation_metrics", path = "rust/crates/host/src/component/relation_metrics/mod.rs" },
	{ id = "relation_metrics_compute", path = "rust/crates/host/src/component/relation_metrics/compute.rs" },
	{ id = "relation_metrics_types", path = "rust/crates/host/src/component/relation_metrics/types.rs" },
	{ id = "coverage", path = "rust/crates/host/src/component/coverage/mod.rs" },
	{ id = "coverage_spans", path = "rust/crates/host/src/component/coverage/spans.rs" },
	{ id = "coverage_compute", path = "rust/crates/host/src/component/coverage/compute.rs" },
	{ id = "snapshot", path = "rust/crates/host/src/component/snapshot/mod.rs" },
	{ id = "snapshot_io", path = "rust/crates/host/src/component/snapshot/io.rs" },
	{ id = "cache", path = "rust/crates/host/src/component/cache/mod.rs" },
	{ id = "cache_body", path = "rust/crates/host/src/component/cache/body.rs" },
	{ id = "cache_types", path = "rust/crates/host/src/component/cache/types.rs" },
	{ id = "membership", path = "rust/crates/host/src/component/membership.rs" },
	{ id = "delegate", path = "rust/crates/host/src/component/delegate/mod.rs" },
	{ id = "delegate_stack", path = "rust/crates/host/src/component/delegate/stack.rs" },
	{ id = "delegate_invoke", path = "rust/crates/host/src/component/delegate/invoke.rs" },
	{ id = "types", path = "rust/crates/host/src/component/types/mod.rs" },
	{ id = "types_parse", path = "rust/crates/host/src/component/types/parse.rs" },
	{ id = "graph_render", path = "rust/crates/host/src/component/graph_render/mod.rs" },
	{ id = "graph_render_kind", path = "rust/crates/host/src/component/graph_render/kind.rs" },
	{ id = "graph_render_instance", path = "rust/crates/host/src/component/graph_render/instance.rs" },
	{ id = "graph_render_style", path = "rust/crates/host/src/component/graph_render/style.rs" },
}

return {
	id = "repo.component_pipeline",
	description = "Host component subsystem stages (scan/relate/runtime/coverage/…)",
	identify = function(c)
		local out = {}
		for _, stage in ipairs(STAGES) do
			for _, path in ipairs(c:listFiles(stage.path)) do
				out[#out + 1] = { path = path, hints = { stageId = stage.id } }
			end
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		local stageId = locator.hints and locator.hints.stageId
		if not path or not stageId then
			return nil
		end
		local children = {}
		local identity = {
			componentId = "repo.component_pipeline",
			id = stageId,
			path = path,
		}
		-- Per-family pcall so one bad query cannot erase successful leaves.
		pcall(function()
			local fnNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.all_functions",
				capture = "fnName",
			})
			local fns = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.all_functions",
				capture = "fn",
			})
			table.sort(fnNames, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(fns, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#fnNames, #fns) do
				local child = c:component("repo.host_module_item"):parse({
					source = { path = path, span = fns[i].span },
					parent = identity,
					role = "fn",
					hints = { name = fnNames[i].text, kind = "fn" },
				})
				if child then
					child.componentId = "repo.host_module_item"
					child.role = "fn"
					children[#children + 1] = child
				end
			end
		end)
		pcall(function()
			local typeNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.type_items",
				capture = "typeName",
			})
			local types = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.type_items",
				capture = "typeItem",
			})
			table.sort(typeNames, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(types, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#typeNames, #types) do
				local child = c:component("repo.host_module_item"):parse({
					source = { path = path, span = types[i].span },
					parent = identity,
					role = "type",
					hints = { name = typeNames[i].text, kind = "type" },
				})
				if child then
					child.componentId = "repo.host_module_item"
					child.role = "type"
					children[#children + 1] = child
				end
			end
		end)
		pcall(function()
			local constNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_str_consts",
				capture = "constName",
			})
			local consts = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_str_consts",
				capture = "constItem",
			})
			table.sort(constNames, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(consts, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#constNames, #consts) do
				local child = c:component("repo.host_module_item"):parse({
					source = { path = path, span = consts[i].span },
					parent = identity,
					role = "const",
					hints = { name = constNames[i].text, kind = "const" },
				})
				if child then
					child.componentId = "repo.host_module_item"
					child.role = "const"
					children[#children + 1] = child
				end
			end
		end)
		pcall(function()
			local attrs = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.attribute_items",
				capture = "attr",
			})
			for i, hit in ipairs(attrs) do
				local child = c:component("repo.host_module_item"):parse({
					source = { path = path, span = hit.span },
					parent = identity,
					role = "attr",
					hints = { name = "attr#" .. i, kind = "attr" },
				})
				if child then
					child.componentId = "repo.host_module_item"
					child.role = "attr"
					children[#children + 1] = child
				end
			end
		end)
		return {
			id = stageId,
			componentId = "repo.component_pipeline",
			source = { path = path },
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local edges = { { kind = "lives_in", to = "repo.zone/domain.components" } }
		local id = instance.ast and instance.ast.id
		-- Pipeline story: identify/scan → relate → metrics/coverage → snapshot; runtime consumes scan.
		local feeds = {
			scan = { "relations", "runtime" },
			scan_full = { "scan" },
			scan_identify = { "scan" },
			scan_incremental = { "scan" },
			scan_graph_phases = { "scan" },
			scan_types = { "scan" },
			relations = { "relation_metrics", "coverage" },
			relations_resolve = { "relations" },
			relations_resolve_normalize = { "relations_resolve" },
			relations_resolve_edges = { "relations_resolve" },
			relations_resolve_kind = { "relations_resolve" },
			relations_resolve_targets = { "relations_resolve" },
			relations_resolve_findings = { "relations_resolve" },
			relations_attach = { "relations" },
			relations_facade = { "relations" },
			relations_facade_builders = { "relations_facade" },
			relations_types = { "relations" },
			membership = { "relations" },
			delegate = { "runtime" },
			delegate_stack = { "delegate" },
			delegate_invoke = { "delegate" },
			types = { "scan", "relations" },
			types_parse = { "types" },
			graph_render = { "snapshot" },
			graph_render_kind = { "graph_render" },
			graph_render_instance = { "graph_render" },
			graph_render_style = { "graph_render" },
			relation_metrics = { "snapshot" },
			relation_metrics_compute = { "relation_metrics" },
			relation_metrics_types = { "relation_metrics" },
			coverage = { "snapshot" },
			coverage_spans = { "coverage" },
			coverage_compute = { "coverage" },
			snapshot = {},
			snapshot_io = { "snapshot" },
			cache = { "scan_incremental" },
			cache_body = { "cache" },
			cache_types = { "cache" },
			runtime = { "runtime_finalize" },
			runtime_context = { "runtime" },
			runtime_hook = { "runtime" },
			runtime_operations = { "runtime" },
			runtime_scaffold = { "runtime" },
			runtime_stack_ops = { "runtime" },
		}
		for _, toStage in ipairs(feeds[id] or {}) do
			local to = "repo.component_pipeline/" .. toStage
			if index:byId(to) then
				edges[#edges + 1] = { kind = "feeds", to = to }
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "component-pipeline-stage",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " leaves" },
		}
	end,
}
