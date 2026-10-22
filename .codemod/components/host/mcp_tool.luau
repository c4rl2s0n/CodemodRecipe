local MCP_RS = "rust/crates/host/src/bin/codemod_mcp.rs"
local INVENTORY_RS = "rust/crates/host/src/protocol/mcp_inventory.rs"
local createCommand = require("../../lib/create_command")

--- Arms routed through a helper fn instead of referencing HostCommand directly.
--- The declared variant is verified against the helper body (mcp_tool.helper_drift).
local INDIRECT = {
	preview_recipe = { helper = "mcp_preview_or_apply", variant = "Preview" },
	apply_recipe = { helper = "mcp_preview_or_apply", variant = "Apply" },
	component_dry_run_hook = { helper = "mcp_component_invoke_hook", variant = "ComponentInvokeHook" },
	component_apply_hook = { helper = "mcp_component_invoke_hook", variant = "ComponentInvokeHook" },
	stack_dry_run_operation = { helper = "mcp_stack_invoke_operation", variant = "StackInvokeOperation" },
	stack_apply_operation = { helper = "mcp_stack_invoke_operation", variant = "StackInvokeOperation" },
}

--- Arms that legitimately bypass the protocol (no HostCommand involved).
local DIRECT = {
	bootstrap_project = true,
}

local function contains(outer, inner)
	return inner.from >= outer.from and inner.to <= outer.to
end

--- Pair the two captures of repo_rust.string_match_arms by document order.
local function collectArms(c, path)
	local names = c:query({
		path = path,
		language = "rust",
		query = "repo_rust.string_match_arms",
		capture = "stringArmName",
	})
	local values = c:query({
		path = path,
		language = "rust",
		query = "repo_rust.string_match_arms",
		capture = "stringArmValue",
	})
	table.sort(names, function(a, b)
		return a.span.from < b.span.from
	end)
	table.sort(values, function(a, b)
		return a.span.from < b.span.from
	end)
	local arms = {}
	for i = 1, math.min(#names, #values) do
		arms[#arms + 1] = { name = names[i].text, nameSpan = names[i].span, valueSpan = values[i].span }
	end
	return arms
end

--- Tool names declared in mcp_inventory.rs (SSOT for tools/list).
local function collectListedTools(c, inventoryPath)
	local listed, order = {}, {}
	local src = c:read(inventoryPath)
	for name in src:gmatch('name:%s*"([%w_]+)"') do
		if not listed[name] then
			listed[name] = true
			order[#order + 1] = name
		end
	end
	return listed, order
end

--- All HostCommand variant references in expression position, with spans.
local function collectVariantRefs(c, path)
	local refs = {}
	for _, qid in ipairs({ "repo_rust.host_command_expr_unit", "repo_rust.host_command_expr_struct" }) do
		for _, hit in ipairs(c:query({
			path = path,
			language = "rust",
			query = qid,
			capture = "hcVariant",
		})) do
			refs[#refs + 1] = { variant = hit.text, span = hit.span }
		end
	end
	return refs
end

local function variantsIn(refs, span)
	local out, seen = {}, {}
	for _, ref in ipairs(refs) do
		if contains(span, ref.span) and not seen[ref.variant] then
			seen[ref.variant] = true
			out[#out + 1] = ref.variant
		end
	end
	table.sort(out)
	return out
end

local function collectFnSpans(c, path)
	local names = c:query({
		path = path,
		language = "rust",
		query = "repo_rust.function_items",
		capture = "fnName",
	})
	local fns = c:query({
		path = path,
		language = "rust",
		query = "repo_rust.function_items",
		capture = "fn",
	})
	table.sort(names, function(a, b)
		return a.span.from < b.span.from
	end)
	table.sort(fns, function(a, b)
		return a.span.from < b.span.from
	end)
	local out = {}
	for i = 1, math.min(#names, #fns) do
		out[names[i].text] = fns[i].span
	end
	return out
end

--- Callees invoked inside a span (helper-call detection).
local function calleesIn(c, path, span)
	local out = {}
	for _, hit in ipairs(c:query({
		path = path,
		language = "rust",
		query = "repo_rust.call_exprs",
		capture = "callee",
	})) do
		if contains(span, hit.span) then
			out[hit.text] = true
		end
	end
	return out
end

return {
	id = "repo.mcp_tool",
	description = "MCP adapter surface: inventory SSOT ↔ tools/call routing ↔ HostCommand mapping",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(MCP_RS)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.mcp_tool",
			id = "mcp-surface",
			path = path,
		}

		local arms = collectArms(c, path)
		local roots, toolsCallArm, toolsListArm = {}, nil, nil
		for _, arm in ipairs(arms) do
			local nested = false
			for _, other in ipairs(arms) do
				if other ~= arm and contains(other.valueSpan, arm.nameSpan) then
					nested = true
					break
				end
			end
			if not nested then
				roots[arm.name] = true
				if arm.name == "tools/call" then
					toolsCallArm = arm
				elseif arm.name == "tools/list" then
					toolsListArm = arm
				end
			end
		end

		local listed, listedOrder = collectListedTools(c, INVENTORY_RS)
		local refs = collectVariantRefs(c, path)
		local fnSpans = collectFnSpans(c, path)

		local helperRefs = {}
		for _, spec in pairs(INDIRECT) do
			local span = fnSpans[spec.helper]
			if span then
				helperRefs[spec.helper] = variantsIn(refs, span)
			end
		end

		-- tools/list must derive from inventory (architecture policy).
		local listUsesInventory = false
		if toolsListArm then
			local callees = calleesIn(c, path, toolsListArm.valueSpan)
			for callee in pairs(callees) do
				if callee == "tools_array" or callee:match("tools_array$") then
					listUsesInventory = true
				end
			end
			-- Scoped paths like mcp_inventory::tools_array are not always captured as
			-- identifier callees; fall back to source substring in the arm span.
			if not listUsesInventory then
				local src = c:read(path)
				local slice = src:sub(toolsListArm.valueSpan.from + 1, toolsListArm.valueSpan.to)
				if slice:find("tools_array", 1, true) then
					listUsesInventory = true
				end
			end
		end

		local children = {}
		if toolsCallArm then
			for _, arm in ipairs(arms) do
				if arm.nameSpan.from > toolsCallArm.valueSpan.from and arm.nameSpan.to <= toolsCallArm.valueSpan.to then
					local variants = variantsIn(refs, arm.valueSpan)
					local indirect = INDIRECT[arm.name]
					local helper = nil
					if #variants == 0 and indirect then
						local callees = calleesIn(c, path, arm.valueSpan)
						if callees[indirect.helper] then
							helper = indirect.helper
							variants = { indirect.variant }
						end
					end
					local child = c:component("repo.mcp_tool_entry"):parse({
						source = { path = path, span = arm.nameSpan },
						parent = identity,
						role = "tool",
						hints = {
							name = arm.name,
							variants = variants,
							direct = DIRECT[arm.name] == true and #variants == 0,
							helper = helper,
							listed = listed[arm.name] == true,
						},
					})
					if child then
						child.componentId = "repo.mcp_tool_entry"
						child.role = "tool"
						children[#children + 1] = child
					end
				end
			end
		end

		-- Coverage leaves: MCP binary + inventory SSOT (sole path owners after dual-root opt-out).
		local function paintFns(fpath, idPrefix)
			pcall(function()
				local fnNames = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.all_functions",
					capture = "fnName",
				})
				local fns = c:query({
					path = fpath,
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
					children[#children + 1] = {
						id = idPrefix .. fnNames[i].text .. "@" .. tostring(fns[i].span.from),
						componentId = "repo.host_module_item",
						source = { path = fpath, span = fns[i].span },
						kind = "fn",
						name = fnNames[i].text,
						role = "fn",
					}
				end
			end)
		end
		paintFns(path, "fn.")
		paintFns(INVENTORY_RS, "inv.fn.")

		return {
			id = "mcp-surface",
			componentId = "repo.mcp_tool",
			source = { path = path },
			children = children,
			listedOrder = listedOrder,
			helperRefs = helperRefs,
			outerArms = roots,
			listUsesInventory = listUsesInventory,
			extraPaths = { INVENTORY_RS },
		}
	end,
	verify = function(_c, instance, graph)
		local findings = {}
		local ast = instance.ast
		local byName = {}
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.mcp_tool_entry" then
				byName[child.id] = child
			end
		end

		if not ast.listUsesInventory then
			findings[#findings + 1] = {
				severity = "error",
				code = "mcp_tool.handwritten_list",
				message = "tools/list must derive from protocol::mcp_inventory::tools_array (HostCommand-derived adapter policy)",
				instanceId = instance.id,
			}
		end

		-- inventory ↔ tools/call parity, both directions
		for _, name in ipairs(ast.listedOrder or {}) do
			if not byName[name] then
				findings[#findings + 1] = {
					severity = "error",
					code = "mcp_tool.missing_arm",
					message = "tool '" .. name .. "' is in mcp_inventory but has no tools/call arm",
					instanceId = instance.id,
				}
			end
		end
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.mcp_tool_entry" then
				local name = child.id
				if not child.listed then
					findings[#findings + 1] = {
						severity = "error",
						code = "mcp_tool.unlisted",
						message = "tool '" .. name .. "' has a tools/call arm but no mcp_inventory entry",
						instanceId = "repo.mcp_tool_entry/" .. child.id,
					}
				end
				if not child.direct and #(child.variants or {}) == 0 then
					findings[#findings + 1] = {
						severity = "error",
						code = "mcp_tool.no_command",
						message = "tool '" .. name .. "' routes to no HostCommand variant (undeclared indirection?)",
						instanceId = "repo.mcp_tool_entry/" .. child.id,
					}
				end
				if graph then
					for _, variant in ipairs(child.variants or {}) do
						local found = false
						for _, hc in ipairs(graph:byComponent("repo.host_command")) do
							if hc.ast.variant == variant then
								found = true
								break
							end
						end
						if not found then
							findings[#findings + 1] = {
								severity = "error",
								code = "mcp_tool.unknown_variant",
								message = "tool '"
									.. name
									.. "' routes to HostCommand::"
									.. variant
									.. " which has no enum variant",
								instanceId = "repo.mcp_tool_entry/" .. child.id,
							}
						end
					end
				end
				local indirect = INDIRECT[name]
				if indirect and child.helper then
					local bodyVariants = (ast.helperRefs or {})[indirect.helper] or {}
					local ok = false
					for _, v in ipairs(bodyVariants) do
						if v == indirect.variant then
							ok = true
							break
						end
					end
					if not ok then
						findings[#findings + 1] = {
							severity = "error",
							code = "mcp_tool.helper_drift",
							message = "tool '"
								.. name
								.. "' is declared to route via "
								.. indirect.helper
								.. " to HostCommand::"
								.. indirect.variant
								.. ", but the helper body does not reference it",
							instanceId = "repo.mcp_tool_entry/" .. child.id,
						}
					end
				end
				if DIRECT[name] and not child.direct then
					findings[#findings + 1] = {
						severity = "error",
						code = "mcp_tool.direct_drift",
						message = "tool '" .. name .. "' is declared protocol-free but its arm references HostCommand",
						instanceId = "repo.mcp_tool_entry/" .. child.id,
					}
				end
			end
		end

		return findings
	end,
	operations = {
		createCommand = {
			description = "Add an MCP inventory entry + tools/call arm for a HostCommand",
			args = {
				{ name = "name", required = true },
				{ name = "ifExists", required = false },
			},
			apply = function(c, args, _target, graph)
				createCommand.applyMcp(c, args, graph)
			end,
		},
	},
	relate = function(_c, instance, index)
		local edges = { { kind = "lives_in", to = "repo.zone/command.adapters" } }
		for _, child in ipairs(instance.ast.children or {}) do
			if child.componentId == "repo.mcp_tool_entry" and child.id then
				local to = "repo.mcp_tool_entry/" .. child.id
				if index:byId(to) then
					edges[#edges + 1] = { kind = "defines", to = to }
				end
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		local tools = 0
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.mcp_tool_entry" then
				tools += 1
			end
		end
		return {
			kind = "mcp-tool-surface",
			title = "MCP tool surface",
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = {
				tools .. " tools",
				#(ast.listedOrder or {}) .. " inventoried",
			},
		}
	end,
}
