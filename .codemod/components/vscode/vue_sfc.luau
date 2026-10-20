--- Coarse Vue SFC: file + script/template/style section leaves (+ scaffold).

local zonesLib = require("../../lib/zones")

--- Find all blocks of `tag` with depth-aware matching (handles nested same-tag).
local function findTagBlocks(src, tag)
	local openPat = "<" .. tag
	local closePat = "</" .. tag .. ">"
	local blocks = {}
	local i = 1
	while true do
		local from1 = src:find(openPat, i, true)
		if not from1 then
			break
		end
		-- Ensure tag boundary (not e.g. <templateX)
		local after = src:sub(from1 + #openPat, from1 + #openPat)
		if after ~= "" and after ~= ">" and after ~= " " and after ~= "\t" and after ~= "\n" and after ~= "/" then
			i = from1 + 1
		else
			local depth = 1
			local pos = from1 + #openPat
			local endPos = nil
			while depth > 0 do
				local nextOpen = src:find(openPat, pos, true)
				local nextClose = src:find(closePat, pos, true)
				if not nextClose then
					break
				end
				-- Skip false opens (same boundary check)
				while nextOpen do
					local a = src:sub(nextOpen + #openPat, nextOpen + #openPat)
					if a ~= "" and a ~= ">" and a ~= " " and a ~= "\t" and a ~= "\n" and a ~= "/" then
						nextOpen = src:find(openPat, nextOpen + 1, true)
					else
						break
					end
				end
				if nextOpen and nextOpen < nextClose then
					depth = depth + 1
					pos = nextOpen + #openPat
				else
					depth = depth - 1
					if depth == 0 then
						endPos = nextClose + #closePat - 1
					end
					pos = nextClose + #closePat
				end
			end
			if endPos then
				blocks[#blocks + 1] = { from = from1 - 1, to = endPos } -- 0-based exclusive end = endPos (1-based inclusive) → to = endPos
				-- endPos is 1-based inclusive last char index; exclusive to = endPos
				blocks[#blocks].to = endPos
				i = endPos + 1
			else
				i = from1 + 1
			end
		end
	end
	return blocks
end

local function findSections(src)
	local sections = {}
	for _, kind in ipairs({ "script", "template", "style" }) do
		for _, span in ipairs(findTagBlocks(src, kind)) do
			sections[#sections + 1] = { kind = kind, span = span }
		end
	end
	table.sort(sections, function(a, b)
		return a.span.from < b.span.from
	end)
	return sections
end

local function parseChildren(c, path, parent, src)
	local children = {}
	for _, sec in ipairs(findSections(src)) do
		local child = c:component("repo.vue_section"):parse({
			source = { path = path, span = sec.span },
			parent = parent,
			role = sec.kind,
			hints = { kind = sec.kind },
		})
		if child then
			child.componentId = "repo.vue_section"
			child.role = sec.kind
			children[#children + 1] = child
		end
	end
	return children
end

local function parentDir(path)
	return path:match("^(.*)/[^/]+$") or ""
end

local function resolveImport(fromPath, spec)
	if not spec or spec:sub(1, 1) ~= "." then
		return {}
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
	if #parts == 0 then
		return {}
	end
	-- Specs often include extensions (./Foo.vue, ./bar.js); strip before probing.
	local last = parts[#parts]
	last = last:gsub("%.vue$", ""):gsub("%.tsx?$", ""):gsub("%.jsx?$", "")
	parts[#parts] = last
	local base = table.concat(parts, "/")
	return {
		base .. ".vue",
		base .. ".ts",
		base .. "/index.vue",
		base .. "/index.ts",
	}
end


return {
	id = "repo.vue_sfc",
	description = "Vue single-file component: script/template/style sections + scaffoldSfc",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("vscode_extension/src/**/*.vue")) do
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
			componentId = "repo.vue_sfc",
			id = path,
			path = path,
		}
		local ok, src = pcall(function()
			return c:read(path)
		end)
		if not ok or type(src) ~= "string" then
			return {
				id = path,
				componentId = "repo.vue_sfc",
				source = { path = path },
				children = {},
				importSpecs = {},
			}
		end
		local importSpecs = {}
		for spec in src:gmatch("from%s*[\"'](%.[^\"']+)[\"']") do
			importSpecs[#importSpecs + 1] = spec
		end
		local children = parseChildren(c, path, identity, src)
		return {
			id = path,
			componentId = "repo.vue_sfc",
			source = { path = path, span = { from = 0, to = #src } },
			children = children,
			importSpecs = importSpecs,
		}
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path
		local zone = zonesLib.vscodeZoneForPath(path)
		local edges = { { kind = "lives_in", to = "repo.zone/" .. zone } }
		local seenTo = {}
		for _, spec in ipairs(instance.ast.importSpecs or {}) do
			for _, cand in ipairs(resolveImport(path, spec)) do
				local toVue = "repo.vue_sfc/" .. cand
				local toTs = "repo.ts_module/" .. cand
				local to = nil
				if index:byId(toVue) then
					to = toVue
				elseif index:byId(toTs) then
					to = toTs
				end
				if to and not seenTo[to] then
					seenTo[to] = true
					edges[#edges + 1] = { kind = "imports", to = to }
					break
				end
			end
		end
		return edges
	end,
	operations = {
		scaffoldSfc = {
			description = "Create a stub .vue SFC with script/template/style sections",
			args = {
				{ name = "path", required = true, help = "Target path ending in .vue (workspace-relative)" },
				{ name = "name", required = false, help = "Component display name (default from filename)" },
				{ name = "ifExists", required = false, help = "skip (default) | error" },
			},
			apply = function(c, args, _target, _graph)
				local path = args.path
				if not path or not path:match("%.vue$") then
					error("scaffoldSfc requires args.path ending in .vue")
				end
				local exists = false
				pcall(function()
					local _ = c:read(path)
					exists = true
				end)
				if exists then
					if args.ifExists == "error" then
						error("SFC already exists: " .. path)
					end
					return
				end
				local stem = args.name or path:match("([^/]+)%.vue$") or "Component"
				local content = table.concat({
					'<script setup lang="ts">',
					"// " .. stem,
					"</script>",
					"",
					"<template>",
					'  <div class="' .. stem .. '"></div>',
					"</template>",
					"",
					"<style scoped>",
					"</style>",
					"",
				}, "\n")
				c:writeFile({ path = path, text = content })
			end,
		},
	},
	toSummary = function(_c, instance)
		local kinds = {}
		for _, ch in ipairs(instance.ast.children or {}) do
			kinds[#kinds + 1] = ch.kind or ch.role or "?"
		end
		return {
			kind = "vue-sfc",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = {
				#(instance.ast.children or {}) .. " sections",
				table.concat(kinds, ","),
			},
		}
	end,
}
