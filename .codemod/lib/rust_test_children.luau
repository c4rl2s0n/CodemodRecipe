--- Shared parse of #[test] cases + helper fns inside a span (mod tests or whole file).

local M = {}

local function contained(span, outer)
	return span and outer and span.from >= outer.from and span.to <= outer.to
end

--- Collect test-case and helper children for `parent` within optional `outerSpan`.
--- When `outerSpan` is nil, all functions in `path` are considered.
function M.parseChildren(c, path, parent, outerSpan)
	local children = {}
	local testSpans = {}
	local ok = pcall(function()
		local testNames = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.test_functions",
			capture = "fnName",
		})
		local testFns = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.test_functions",
			capture = "fn",
		})
		table.sort(testNames, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(testFns, function(a, b)
			return a.span.from < b.span.from
		end)
		for i = 1, math.min(#testNames, #testFns) do
			local span = testFns[i].span
			if not outerSpan or contained(span, outerSpan) then
				testSpans[span.from] = true
				local child = c:component("repo.rust_test_case"):parse({
					source = { path = path, span = span },
					parent = parent,
					role = "test",
					hints = { name = testNames[i].text, kind = "test" },
				})
				if child then
					child.componentId = "repo.rust_test_case"
					child.role = "test"
					children[#children + 1] = child
				end
			end
		end
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
			local span = fns[i].span
			if testSpans[span.from] then
				-- already a test case
			elseif not outerSpan or contained(span, outerSpan) then
				local child = c:component("repo.rust_test_case"):parse({
					source = { path = path, span = span },
					parent = parent,
					role = "helper",
					hints = { name = fnNames[i].text, kind = "helper" },
				})
				if child then
					child.componentId = "repo.rust_test_case"
					child.role = "helper"
					children[#children + 1] = child
				end
			end
		end
	end)
	if not ok then
		return {}
	end
	return children
end

return M
