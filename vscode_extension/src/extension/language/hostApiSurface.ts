/**
 * Static Luau host API surface for recipe editor completions / hover.
 * Mirrors `rust/crates/host/src/lua_runtime.rs` `build_ctx` + skill reference.
 */

export type HostMethodSpec = {
  name: string;
  description: string;
  /** Spec-table keys when the method takes `{ ... }` (empty for string-only). */
  keys: string[];
  /** Keys that accept enum values. */
  enums?: Record<string, string[]>;
};

export const HOST_CONTEXT_FIELD = 'args';

export const HOST_METHODS: readonly HostMethodSpec[] = [
  {
    name: 'insert',
    description: 'Insert text at a tree-sitter capture (`path`, `query`, `capture`, `text`).',
    keys: ['path', 'query', 'capture', 'text', 'language', 'anchor'],
    enums: { anchor: ['start', 'end'] },
  },
  {
    name: 'replace',
    description: 'Replace a capture span with `text`.',
    keys: ['path', 'query', 'capture', 'text', 'language', 'includeLeadingTrivia'],
  },
  {
    name: 'remove',
    description: 'Remove a capture span.',
    keys: ['path', 'query', 'capture', 'language', 'includeLeadingTrivia'],
  },
  {
    name: 'create',
    description: 'Create a file (`path` + `content` or `template`).',
    keys: ['path', 'content', 'template', 'ifExists'],
    enums: { ifExists: ['fail', 'skip'] },
  },
  {
    name: 'delete',
    description: 'Delete a file.',
    keys: ['path', 'ifMissing'],
    enums: { ifMissing: ['fail', 'skip'] },
  },
  {
    name: 'read',
    description: 'Read file contents (string path or `{ path }`).',
    keys: ['path'],
  },
  {
    name: 'query',
    description: 'Run a tree-sitter query; returns `{ start, ["end"], text }` spans.',
    keys: ['path', 'query', 'capture', 'language'],
  },
  {
    name: 'queryExists',
    description: 'True if the query matches at least once.',
    keys: ['path', 'query', 'capture', 'language'],
  },
  {
    name: 'queryCount',
    description: 'Number of matches (optional `capture`).',
    keys: ['path', 'query', 'capture', 'language'],
  },
  {
    name: 'expectMatch',
    description: 'Error if the query has zero matches.',
    keys: ['path', 'query', 'capture', 'language'],
  },
  {
    name: 'expectNoMatch',
    description: 'Error if the query has any match.',
    keys: ['path', 'query', 'capture', 'language'],
  },
  {
    name: 'postExec',
    description: 'Shell command after apply (Jinja-rendered); preview records only.',
    keys: ['command', 'path'],
  },
  {
    name: 'applyTemplate',
    description: 'Render a MiniJinja string template.',
    keys: ['template', 'env'],
  },
  {
    name: 'renderFileTemplate',
    description: 'Render a MiniJinja template file.',
    keys: ['path', 'env'],
  },
  {
    name: 'map',
    description: 'Lookup a registered map by id → string→string table.',
    keys: [],
  },
  {
    name: 'var',
    description: 'Lookup registered variables by id → string→string table.',
    keys: [],
  },
  {
    name: 'queryLib',
    description: 'Lookup a registered query string by id (e.g. `lib.queryName`).',
    keys: [],
  },
  {
    name: 'assert',
    description: 'Runtime assertion (`cond`, optional message).',
    keys: [],
  },
  {
    name: 'run',
    description: 'Run a nested recipe by id with optional arg overrides.',
    keys: [],
  },
];

export const RECIPE_MODULE_KEYS: readonly string[] = [
  'id',
  'name',
  'description',
  'args',
  'apply',
  'maps',
  'types',
  'queries',
  'postExecution',
  'explorerMenu',
];

export const RECIPE_MODULE_DOCS: Record<string, string> = {
  id: 'Unique recipe id (dotted ids nest in the Recipes tab).',
  name: 'Display name in the recipe runner.',
  description: 'Short description shown in the runner and hover.',
  args: 'List of arg definition tables (`name`, `inputKind`, …).',
  apply: '`function(c)` — only mutation entrypoint.',
  maps: 'Recipe-local maps `{ mapId = { key = "value" } }`.',
  types: 'Recipe-local named types `{ TypeId = { fields = { … } } }`.',
  queries: 'Inline query strings `{ queryId = "(…) @cap" }`.',
  postExecution: 'Shell commands after successful apply (Jinja-rendered).',
  explorerMenu: 'VS Code Explorer submenu entries.',
};

export const ARG_DEFINITION_KEYS: readonly string[] = [
  'name',
  'required',
  'inputKind',
  'defaultsTo',
  'help',
  'abbr',
  'options',
  'optionsFromMap',
  'allowCustomValue',
  'from',
];

export const ARG_DEFINITION_DOCS: Record<string, string> = {
  name: 'Argument name (available as `c.args.<name>`).',
  required: 'Whether the runner requires a value.',
  inputKind:
    '`text` | `bool` | `choice` | `file` | `directory` | TypeId | `{ TypeId }` list.',
  defaultsTo: 'Default when the arg is omitted.',
  help: 'Help text in the recipe runner.',
  abbr: 'Short CLI abbreviation.',
  options: 'Choice list or text suggestions.',
  optionsFromMap: 'Map id whose keys become options.',
  allowCustomValue: 'Allow values outside `options` for choice args.',
  from: 'Derive from editor context (`activeFile`, query table, …).',
};

export const BUILTIN_INPUT_KINDS: readonly string[] = [
  'text',
  'bool',
  'choice',
  'file',
  'directory',
];

export const HOST_METHOD_DOCS: Record<string, string> = Object.fromEntries(
  HOST_METHODS.map((m) => [m.name, m.description])
);

export function hostMethodByName(name: string): HostMethodSpec | undefined {
  return HOST_METHODS.find((m) => m.name === name);
}
