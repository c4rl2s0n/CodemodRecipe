--- HostCommand enum parse helpers shared by protocol/mcp/bridge components.
--- Library module: no top-level id/apply/parse.

local M = {}

--- Pair enum variants with their preceding serde rename attribute by span order.
--- A rename belongs to the first variant that starts after it; variants without
--- their own rename get nil (reported by verify).
function M.pairVariantsBySpan(variantHits, renameHits)
	table.sort(variantHits, function(a, b)
		return a.span.from < b.span.from
	end)
	table.sort(renameHits, function(a, b)
		return a.span.from < b.span.from
	end)
	local out = {}
	local ri = 1
	for _, v in ipairs(variantHits) do
		local rename = nil
		while ri <= #renameHits and renameHits[ri].span.to <= v.span.from do
			rename = renameHits[ri]
			ri += 1
		end
		out[#out + 1] = { variant = v.text, rename = rename and rename.text or nil, span = v.span }
	end
	return out
end

--- Query HostCommand variants + renames from command.rs and return paired rows.
--- `c` is the component context; `path` defaults to the protocol command.rs.
function M.parseEnum(c, path)
	path = path or "rust/crates/host/src/protocol/command.rs"
	local variants = c:query({
		path = path,
		language = "rust",
		query = "repo_rust.host_command_variants",
		capture = "variant",
	})
	local renames = c:query({
		path = path,
		language = "rust",
		query = "repo_rust.host_command_renames",
		capture = "rename",
	})
	return M.pairVariantsBySpan(variants, renames)
end

return M
