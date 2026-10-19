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
	{ id = "relations_resolve", path = "rust/crates/host/src/component/relations/resolve.rs" },
	{ id = "relations_attach", path = "rust/crates/host/src/component/relations/attach.rs" },
	{ id = "relations_facade", path = "rust/crates/host/src/component/relations/facade.rs" },
	{ id = "relations_types", path = "rust/crates/host/src/component/relations/types.rs" },
	-- runtime/
	{ id = "runtime", path = "rust/crates/host/src/component/runtime/mod.rs" },
	{ id = "runtime_context", path = "rust/crates/host/src/component/runtime/context.rs" },
	{ id = "runtime_hook", path = "rust/crates/host/src/component/runtime/hook.rs" },
	{ id = "runtime_operations", path = "rust/crates/host/src/component/runtime/operations.rs" },
	{ id = "runtime_stack_ops", path = "rust/crates/host/src/component/runtime/stack_ops.rs" },
	{ id = "runtime_finalize", path = "rust/crates/host/src/component/runtime/finalize.rs" },
	-- other stages
	{ id = "relation_metrics", path = "rust/crates/host/src/component/relation_metrics.rs" },
	{ id = "coverage", path = "rust/crates/host/src/component/coverage.rs" },
	{ id = "snapshot", path = "rust/crates/host/src/component/snapshot.rs" },
	{ id = "membership", path = "rust/crates/host/src/component/membership.rs" },
	{ id = "delegate", path = "rust/crates/host/src/component/delegate.rs" },
	{ id = "graph_render", path = "rust/crates/host/src/component/graph_render.rs" },
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
		local ok = pcall(function()
			local fnNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_functions",
				capture = "fnName",
			})
			local fns = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_functions",
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
			local typeNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_types",
				capture = "typeName",
			})
			local types = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_types",
				capture = "type",
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
		if not ok then
			children = {}
		end
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
			relations = { "relation_metrics", "coverage" },
			relations_resolve = { "relations" },
			membership = { "relations" },
			relation_metrics = { "snapshot" },
			coverage = { "snapshot" },
			runtime = { "runtime_finalize" },
			runtime_hook = { "runtime" },
			runtime_operations = { "runtime" },
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
