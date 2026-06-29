// Standalone integration smoke test for the host protocol.
//
// Mirrors the spawn + marker-extraction logic in src/host/dartBridge.ts
// (without the vscode dependency) so the TypeScript <-> Rust contract can be
// verified from the command line.
//
// Run from repo root: node vscode_extension/scripts/smoke.mjs
// Or from vscode_extension: node scripts/smoke.mjs
import { spawn } from 'child_process';
import * as path from 'path';
import { fileURLToPath } from 'url';

const RESULT_BEGIN = '__CODEMOD_RESULT_BEGIN__';
const RESULT_END = '__CODEMOD_RESULT_END__';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../..');

function extractResult(output) {
  const begin = output.indexOf(RESULT_BEGIN);
  const end = output.indexOf(RESULT_END);
  if (begin === -1 || end === -1 || end < begin) return undefined;
  return output.slice(begin + RESULT_BEGIN.length, end).trim();
}

function send(workspaceRoot, codemodRoot, command) {
  const manifestPath = path.join(repoRoot, 'rust', 'Cargo.toml');
  return new Promise((resolve, reject) => {
    const child = spawn(
      'cargo',
      [
        'run',
        '-q',
        '--manifest-path',
        manifestPath,
        '-p',
        'codemod_recipe_host',
        '--bin',
        'codemod_host',
        '--',
        '--stdio-server',
        '--workspace-root',
        workspaceRoot,
        '--codemod-root',
        codemodRoot,
      ],
      { cwd: repoRoot }
    );
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (c) => (stdout += c.toString()));
    child.stderr.on('data', (c) => (stderr += c.toString()));
    child.on('error', reject);
    child.on('close', () => {
      const payload = extractResult(stdout);
      if (payload === undefined) {
        reject(new Error(`No result markers found.\n${stderr || stdout}`));
        return;
      }
      resolve(JSON.parse(payload));
    });
    child.stdin.write(JSON.stringify(command));
    child.stdin.end();
  });
}

const codemodRootRelative = process.argv[2] ?? '.codemod';
const workspaceRoot = repoRoot;
const codemodRoot = path.isAbsolute(codemodRootRelative)
  ? codemodRootRelative
  : path.join(workspaceRoot, codemodRootRelative);

const assert = (cond, msg) => {
  if (!cond) {
    console.error('FAIL:', msg);
    process.exit(1);
  }
  console.log('ok -', msg);
};

const list = await send(workspaceRoot, codemodRoot, { command: 'list' });
assert(list.ok === true, 'list returns ok');
assert(Array.isArray(list.recipes) && list.recipes.length >= 1, 'list has recipes');
assert(typeof list.mapsLoaded === 'number' && list.mapsLoaded >= 1, 'list reports loaded maps');

const preview = await send(workspaceRoot, codemodRoot, {
  command: 'preview',
  recipe: 'insert_log_line',
  args: { file: 'test/fixtures/ast_paths/settings.dart' },
});
assert(preview.ok === true, 'preview returns ok');
assert(preview.files.length === 1, 'preview returns one file');
const patches = preview.files[0].patches ?? [];
assert(patches.length >= 1, 'preview returns patches');

assert(preview.previewToken, 'preview returns previewToken');

const missing = await send(workspaceRoot, codemodRoot, {
  command: 'preview',
  recipe: 'insert_log_line',
  args: {},
});
assert(missing.ok === false, 'preview reports validation error');

console.log('\nAll smoke checks passed.');
