--- Luau recipe language-context helpers (completion / hover / args tables).

local GLOB = "vscode_extension/src/extension/language/context/**/*.ts"
local BARREL = "vscode_extension/src/extension/language/context/index.ts"

local function parentDir(path)
	return path:match("^(.*)/[^/]+$") or ""
end

local function resolveImport(fromPath, spec)
	if not spec or spec:sub(1, 1) ~= "." then
		return nil
	end
	local parts = {}
	for seg in (parentDir(fromPath) .. "/" .. spec):gmatch("[^/]+") do
		if seg == ".." then
			if #parts > 0 then
				table.remove(parts)
			end
		elseif seg ~= "." and seg ~= "" then
			parts[#parts + 1] = seg
		end
	end
	local base = table.concat(parts, "/")
	return {
		base .. ".ts",
		base .. "/index.ts",
	}
end

local function linkResolved(index, fromPath, spec, seen, edges, kind)
	local cands = resolveImport(fromPath, spec)
	if not cands then
		return
	end
	for _, cand in ipairs(cands) do
		local toCtx = "repo.vscode_language_context/" .. cand
		local toTs = "repo.ts_module/" .. cand
		local to = nil
		if index:byId(toCtx) then
			to = toCtx
		elseif index:byId(toTs) then
			to = toTs
		end
		if to and not seen[to] then
			seen[to] = true
			edges[#edges + 1] = { kind = kind, to = to }
			break
		end
	end
end

return {
	id = "repo.vscode_language_context",
	description = "Luau recipe language context modules (id/host-method/args/string helpers)",
	identify = function(c)
		local out = {}
		local seen = {}
		for _, path in ipairs(c:listFiles(GLOB)) do
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = { path = path }
			end
		end
		-- Ensure barrel is identified even if glob order changes.
		if not seen[BARREL] then
			for _, path in ipairs(c:listFiles(BARREL)) do
				out[#out + 1] = { path = path }
			end
		end
		return out
	end,
	parse = function(c, locator)
		local ast = c:component("repo.ts_module"):parse(locator)
		if not ast then
			return nil
		end
		ast.componentId = "repo.vscode_language_context"
		return ast
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path
			or instance.ast and instance.ast.id
		local edges = { { kind = "lives_in", to = "repo.zone/vscode.extension" } }
		local seen = {}
		for _, spec in ipairs(instance.ast.importSpecs or {}) do
			linkResolved(index, path, spec, seen, edges, "uses")
		end
		-- Barrel re-exports connect leaf helpers (moduleKeys, templatePath, …).
		if path == BARREL or (type(path) == "string" and path:match("/index%.ts$")) then
			for _, spec in ipairs(instance.ast.reexportSpecs or {}) do
				linkResolved(index, path, spec, seen, edges, "exports")
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local path = instance.ast.id or ""
		local stem = path:match("([^/]+)%.ts$") or path
		return {
			kind = "vscode-language-context",
			title = stem,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " exports" },
		}
	end,
}
