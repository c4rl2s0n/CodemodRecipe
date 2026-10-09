local hostCommand = require("../../lib/host_command")

local MOD_RS = "rust/crates/host/src/protocol/mod.rs"
local DISPATCH_RS = "rust/crates/host/src/protocol/dispatch/mod.rs"
local TYPES_TS = "vscode_extension/src/shared/types.ts"

local function collectDispatchArms(c)
	local arms = {}
	for _, qid in ipairs({ "repo_rust.host_command_dispatch_unit", "repo_rust.host_command_dispatch_struct" }) do
		for _, hit in ipairs(c:query({
			path = DISPATCH_RS,
			language = "rust",
			query = qid,
			capture = "dispatchVariant",
		})) do
			arms[hit.text] = true
		end
	end
	return arms
end

local function collectTsCommands(c)
	local ts = {}
	local decls = c:query({
		path = TYPES_TS,
		language = "typescript",
		query = "repo_typescript.host_command_alias_decl",
		capture = "aliasDecl",
	})
	local decl = decls[1]
	if not decl then
		return ts, false
	end
	for _, hit in ipairs(c:query({
		path = TYPES_TS,
		language = "typescript",
		query = "repo_typescript.host_command_union_members",
		capture = "tsCommand",
	})) do
		if hit.span.from >= decl.span.from and hit.span.to <= decl.span.to then
			ts[hit.text] = true
		end
	end
	return ts, true
end

return {
	id = "repo.host_protocol",
	description = "Host stdio protocol: HostCommand enum, dispatch arms, and the TS mirror union",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(MOD_RS)) do
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
			componentId = "repo.host_protocol",
			id = "host-protocol",
			path = path,
		}

		local children = {}
		for _, p in ipairs(hostCommand.parseEnum(c, path)) do
			local child = c:component("repo.host_command"):parse({
				source = { path = path, span = p.span },
				parent = identity,
				role = "command",
				hints = { variant = p.variant, rename = p.rename },
			})
			if child then
				child.componentId = "repo.host_command"
				child.role = "command"
				children[#children + 1] = child
			end
		end

		local dispatchArms = collectDispatchArms(c)
		local tsCommands, tsPresent = collectTsCommands(c)

		-- Spanned dispatch arms for coverage (dispatch.rs) — full match_arm body.
		local dispatchChildren = {}
		pcall(function()
			local function addArms(qid)
				local names = c:query({
					path = DISPATCH_RS,
					language = "rust",
					query = qid,
					capture = "dispatchVariant",
				})
				local arms = c:query({
					path = DISPATCH_RS,
					language = "rust",
					query = qid,
					capture = "dispatchArm",
				})
				table.sort(names, function(a, b)
					return a.span.from < b.span.from
				end)
				table.sort(arms, function(a, b)
					return a.span.from < b.span.from
				end)
				for i = 1, math.min(#names, #arms) do
					local child = c:component("repo.host_dispatch_arm"):parse({
						source = { path = DISPATCH_RS, span = arms[i].span },
						parent = identity,
						role = "dispatch",
						hints = { variant = names[i].text },
					})
					if child then
						child.componentId = "repo.host_dispatch_arm"
						child.role = "dispatch"
						dispatchChildren[#dispatchChildren + 1] = child
					end
				end
			end
			addArms("repo_rust.host_command_dispatch_unit")
			addArms("repo_rust.host_command_dispatch_struct")
		end)
		for _, child in ipairs(dispatchChildren) do
			children[#children + 1] = child
		end

		return {
			id = "host-protocol",
			componentId = "repo.host_protocol",
			source = { path = path },
			children = children,
			dispatchArms = dispatchArms,
			tsCommands = tsCommands,
			tsMirrorPresent = tsPresent,
		}
	end,
	verify = function(_c, instance)
		local findings = {}
		local ast = instance.ast
		local arms = ast.dispatchArms or {}
		local ts = ast.tsCommands or {}
		local rustVariants = {}
		local rustRenames = {}

		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.host_command" then
				rustVariants[child.variant] = true
				local childId = "repo.host_command/" .. (child.rename or child.variant)
				if child.rename then
					rustRenames[child.rename] = true
				else
					findings[#findings + 1] = {
						severity = "warning",
						code = "host_command.missing_rename",
						message = "HostCommand::" .. child.variant .. " has no serde rename attribute",
						instanceId = childId,
					}
				end
				if not arms[child.variant] then
					findings[#findings + 1] = {
						severity = "error",
						code = "host_command.missing_dispatch",
						message = "HostCommand::" .. child.variant .. " has no dispatch arm in dispatch.rs",
						instanceId = childId,
					}
				end
				if child.rename and ast.tsMirrorPresent and not ts[child.rename] then
					findings[#findings + 1] = {
						severity = "error",
						code = "host_command.missing_ts_mirror",
						message = "command '" .. child.rename .. "' is missing from the TS HostCommand union in shared/types.ts",
						instanceId = childId,
					}
				end
			end
		end

		for name in pairs(ts) do
			if not rustRenames[name] then
				findings[#findings + 1] = {
					severity = "warning",
					code = "host_protocol.ts_orphan",
					message = "TS HostCommand member '" .. name .. "' has no Rust variant",
					instanceId = instance.id,
				}
			end
		end
		for name in pairs(arms) do
			if not rustVariants[name] then
				findings[#findings + 1] = {
					severity = "error",
					code = "host_protocol.dispatch_orphan",
					message = "dispatch arm HostCommand::" .. name .. " has no enum variant",
					instanceId = instance.id,
				}
			end
		end

		local armCount = 0
		for _ in pairs(arms) do
			armCount += 1
		end
		-- Phase F: handle_command is a thin match delegating to dispatch/{catalog,query,recipe,components}.rs
		-- (dispatch_dual_model finding retired).

		return findings
	end,
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/command.adapters" } }
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		local commands = 0
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.host_command" then
				commands += 1
			end
		end
		local tsCount = 0
		for _ in pairs(ast.tsCommands or {}) do
			tsCount += 1
		end
		return {
			kind = "host-protocol",
			title = "HostCommand protocol",
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = {
				commands .. " commands",
				tsCount .. " TS mirrors",
			},
		}
	end,
}
