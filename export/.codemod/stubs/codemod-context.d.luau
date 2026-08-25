--!strict
-- Codemod recipe host context stubs for luau-lsp / editor IntelliSense.
-- Mirrors rust/crates/host/src/lua_runtime.rs `build_ctx`.
-- Wire via luau-lsp `types.definitionFiles` or place under `.codemod/types/`.

export type Anchor = "start" | "end"
export type IfExists = "fail" | "skip"
export type IfMissing = "fail" | "skip"

export type QuerySpec = string | { [number]: string }

export type EditInsertSpec = {
	path: string,
	query: QuerySpec,
	capture: string,
	text: string,
	language: string?,
	anchor: Anchor?,
}

export type EditReplaceSpec = {
	path: string,
	query: QuerySpec,
	capture: string,
	text: string,
	language: string?,
	includeLeadingTrivia: boolean?,
}

export type EditRemoveSpec = {
	path: string,
	query: QuerySpec,
	capture: string,
	language: string?,
	includeLeadingTrivia: boolean?,
}

export type CreateSpec = {
	path: string,
	content: string?,
	template: string?,
	ifExists: IfExists?,
}

export type DeleteSpec = {
	path: string,
	ifMissing: IfMissing?,
}

export type QueryHelperSpec = {
	path: string,
	query: QuerySpec,
	capture: string?,
	language: string?,
}

export type QuerySpan = {
	start: number,
	["end"]: number,
	text: string,
}

export type PostExecSpec = {
	command: string?,
	path: string?,
}

export type ApplyTemplateSpec = {
	template: string,
	env: { [string]: any }?,
}

export type RenderFileTemplateSpec = {
	path: string,
	env: { [string]: any }?,
}

export type CodemodContext = {
	args: { [string]: any },

	insert: (self: CodemodContext, spec: EditInsertSpec) -> (),
	replace: (self: CodemodContext, spec: EditReplaceSpec) -> (),
	remove: (self: CodemodContext, spec: EditRemoveSpec) -> (),
	create: (self: CodemodContext, spec: CreateSpec) -> (),
	delete: (self: CodemodContext, spec: DeleteSpec) -> (),
	read: (self: CodemodContext, path: string | { path: string }) -> string,
	query: (self: CodemodContext, spec: QueryHelperSpec) -> { QuerySpan },
	queryExists: (self: CodemodContext, spec: QueryHelperSpec) -> boolean,
	queryCount: (self: CodemodContext, spec: QueryHelperSpec) -> number,
	expectMatch: (self: CodemodContext, spec: QueryHelperSpec) -> (),
	expectNoMatch: (self: CodemodContext, spec: QueryHelperSpec) -> (),
	postExec: (self: CodemodContext, spec: PostExecSpec) -> (),
	applyTemplate: (self: CodemodContext, spec: ApplyTemplateSpec) -> string,
	renderFileTemplate: (self: CodemodContext, spec: RenderFileTemplateSpec) -> string,
	map: (self: CodemodContext, id: string) -> { [string]: string },
	var: (self: CodemodContext, id: string) -> { [string]: string },
	queryLib: (self: CodemodContext, id: string) -> string,
	assert: (self: CodemodContext, cond: boolean, msg: string?) -> (),
	run: (self: CodemodContext, id: string, args: { [string]: any }?) -> (),
}

return {}
