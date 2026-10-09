local HANDLERS = "rust/crates/host/src/host_api/handlers.rs"
local STUB = ".codemod/stubs/codemod-context.d.luau"
local SURFACE = "vscode_extension/src/extension/language/hostApiSurface.ts"

local function baseFiles(overrides)
	local files = {
		[HANDLERS] = [[
#[host_method(
    name = "insert",
    description = "Insert text",
    spec = InsertSpec,
    return_type = Nil,
)]
pub(crate) fn do_insert() {}

#[host_method(
    name = "log",
    description = "Log",
    return_type = Nil,
)]
pub(crate) fn do_log() {}
]],
		["rust/crates/host/src/lua_runtime.rs"] = [[
// no host methods
]],
		["rust/crates/host/src/component/delegate.rs"] = [[
static SURFACE: HostMethodSurface = HostMethodSurface {
    name: "component",
    description: "Delegate",
};
]],
		[STUB] = [[
export type CodemodContext = {
	component: (self: CodemodContext, id: string) -> any,
	insert: (self: CodemodContext, spec: any) -> (),
	log: (self: CodemodContext, ...any) -> (),
}
]],
		[SURFACE] = [[
export const HOST_API_SURFACE = [
  { name: 'component', description: 'd' },
  { name: 'insert', description: 'd' },
  { name: 'log', description: 'd' },
];
]],
	}
	for k, v in pairs(overrides or {}) do
		files[k] = v
	end
	return files
end

local function codes(findings)
	local out = {}
	for _, f in ipairs(findings) do
		out[#out + 1] = f.code
	end
	table.sort(out)
	return out
end

return {
	id = "repo.host_api_method.tests",
	target = "repo.host_api_method",
	tests = {
		consistent_surface_has_no_findings = function(t)
			local ws = t:workspace({ files = baseFiles() })
			local scan = ws:scan()
			t:assertEq(#scan.findings, 0)
			local root
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.host_api_method" then
					root = inst
				end
			end
			t:assert(root, "root missing")
			t:assertEq(#root.ast.methodOrder, 3)
		end,

		missing_stub_is_flagged = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[STUB] = [[
export type CodemodContext = {
	insert: (self: CodemodContext, spec: any) -> (),
	log: (self: CodemodContext, ...any) -> (),
}
]],
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(scan.findings), { "host_api.missing_stub" })
		end,

		missing_ts_surface_is_flagged = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[SURFACE] = [[
export const HOST_API_SURFACE = [
  { name: 'insert', description: 'd' },
  { name: 'log', description: 'd' },
];
]],
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(scan.findings), { "host_api.missing_ts_surface" })
		end,
	},
}
