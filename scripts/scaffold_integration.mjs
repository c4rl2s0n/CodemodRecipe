#!/usr/bin/env node
/**
 * End-to-end integration test for the Rust codemod_host against the scaffold
 * mini-project fixture. Copies fixtures into a temporary workspace (cleared on
 * each run) so source fixtures are never modified.
 *
 * Usage (from repo root):
 *   node scripts/scaffold_integration.mjs
 *   ./scripts/scaffold_integration.sh
 *   ./scripts/scaffold_integration.sh --keep   # leave workspace on disk for inspection
 *
 * Environment:
 *   CODEMOD_HOST_BIN — path to codemod_host binary (default: cargo run)
 *   CODEMOD_SCAFFOLD_WORKSPACE — fixed workspace path instead of $TMPDIR/...
 */

import { spawn } from 'child_process';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { fileURLToPath } from 'url';

const RESULT_BEGIN = '__CODEMOD_RESULT_BEGIN__';
const RESULT_END = '__CODEMOD_RESULT_END__';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const fixtureRoot = path.join(repoRoot, 'test/fixtures/scaffold_project');

const args = process.argv.slice(2);
const keepWorkspace = args.includes('--keep');
const workspaceArg = args.find((a) => a.startsWith('--workspace='));
const workspaceFromFlag = workspaceArg?.slice('--workspace='.length);

const defaultWorkspace = path.join(
  os.tmpdir(),
  `codemod_recipe_scaffold_${process.pid}`
);
const workspaceRoot = path.resolve(
  workspaceFromFlag ?? process.env.CODEMOD_SCAFFOLD_WORKSPACE ?? defaultWorkspace
);
const codemodRoot = path.join(workspaceRoot, '.codemod');

const scaffoldArgs = { className: 'Counter', fieldName: 'tickCount' };

function assert(cond, msg) {
  if (!cond) {
    console.error('FAIL:', msg);
    process.exit(1);
  }
  console.log('ok -', msg);
}

function copyDirRecursive(src, dst) {
  fs.mkdirSync(dst, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const from = path.join(src, entry.name);
    const to = path.join(dst, entry.name);
    if (entry.isDirectory()) {
      copyDirRecursive(from, to);
    } else {
      fs.copyFileSync(from, to);
    }
  }
}

function resetWorkspace() {
  if (fs.existsSync(workspaceRoot)) {
    fs.rmSync(workspaceRoot, { recursive: true, force: true });
  }
  copyDirRecursive(path.join(fixtureRoot, 'workspace'), workspaceRoot);
  copyDirRecursive(path.join(fixtureRoot, '.codemod'), codemodRoot);
}

function readFile(rel) {
  return fs.readFileSync(path.join(workspaceRoot, rel), 'utf8');
}

function fileExists(rel) {
  return fs.existsSync(path.join(workspaceRoot, rel));
}

function extractResult(output) {
  const begin = output.indexOf(RESULT_BEGIN);
  const end = output.indexOf(RESULT_END);
  if (begin === -1 || end === -1 || end < begin) return undefined;
  return output.slice(begin + RESULT_BEGIN.length, end).trim();
}

function hostCommand(command) {
  const hostBin = process.env.CODEMOD_HOST_BIN;
  const manifestPath = path.join(repoRoot, 'rust', 'Cargo.toml');

  const spawnArgs = hostBin
    ? [
        hostBin,
        '--stdio-server',
        '--workspace-root',
        workspaceRoot,
        '--codemod-root',
        codemodRoot,
      ]
    : [
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
      ];

  const executable = hostBin ? hostBin : 'cargo';
  const cwd = hostBin ? repoRoot : repoRoot;

  return new Promise((resolve, reject) => {
    const child = spawn(executable, spawnArgs, { cwd });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });
    child.on('error', reject);
    child.on('close', (code) => {
      const payload = extractResult(stdout);
      if (payload === undefined) {
        reject(
          new Error(
            `No result markers from codemod_host (exit ${code}).\n${stderr || stdout}`
          )
        );
        return;
      }
      resolve(JSON.parse(payload));
    });
    child.stdin.write(JSON.stringify(command));
    child.stdin.end();
  });
}

function countKind(files, kind) {
  return files.filter((f) => f.kind === kind).length;
}

console.log(`Workspace: ${workspaceRoot}`);
console.log(`Fixture:   ${fixtureRoot}\n`);

resetWorkspace();
assert(fileExists('lib/app.dart'), 'fixture workspace copied');
assert(fileExists('.codemod/recipes/scaffold_feature.yaml'), 'fixture recipes copied');

// --- validate / list / describe ---
{
  const validate = await hostCommand({ command: 'validate' });
  assert(validate.ok === true, 'validate returns ok');
  const diagnostics = validate.diagnostics ?? [];
  assert(
    diagnostics.every((d) => d.severity !== 'error'),
    'validate has no error-level diagnostics'
  );

  const list = await hostCommand({ command: 'list' });
  assert(list.ok === true, 'list returns ok');
  const ids = (list.recipes ?? []).map((r) => r.id);
  for (const id of [
    'scaffold_feature',
    'create_repository',
    'patch_counter',
    'patch_app',
  ]) {
    assert(ids.includes(id), `list includes recipe ${id}`);
  }
  assert((list.mapsLoaded ?? 0) >= 1, 'list reports loaded maps');

  const describe = await hostCommand({
    command: 'describe',
    recipe: 'scaffold_feature',
  });
  assert(describe.ok === true, 'describe returns ok');
  assert(describe.recipe?.id === 'scaffold_feature', 'describe returns scaffold_feature');
  assert((describe.recipe?.args ?? []).length >= 2, 'describe includes args');
}

// --- preview (create + 2 edits + delete) ---
{
  resetWorkspace();
  const preview = await hostCommand({
    command: 'preview',
    recipe: 'scaffold_feature',
    args: scaffoldArgs,
    snippetLines: 3,
  });
  assert(preview.ok === true, `preview returns ok (${preview.error ?? ''})`);
  const files = preview.files ?? [];
  assert(files.length === 4, 'preview returns four file changes');
  assert(countKind(files, 'create') === 1, 'preview includes one create');
  assert(countKind(files, 'edit') === 2, 'preview includes two edits');
  assert(countKind(files, 'delete') === 1, 'preview includes one delete');

  const create = files.find((f) => f.kind === 'create');
  assert(create?.path === 'lib/counter/counter_repository.dart', 'create targets repository');
  assert((create?.snippet ?? '').includes('CounterRepository'), 'create snippet looks correct');

  const counter = files.find((f) => f.path === 'lib/counter/counter.dart');
  assert((counter?.patches ?? []).length >= 1, 'counter edit has patches');
  assert((counter?.snippet ?? '').includes('tickCount'), 'counter snippet mentions tickCount');

  const app = files.find((f) => f.path === 'lib/app.dart');
  assert((app?.snippet ?? '').includes('scaffold'), 'app snippet mentions scaffold');

  const deleted = files.find((f) => f.path === 'lib/legacy/stale.dart');
  assert(deleted?.kind === 'delete', 'legacy file marked for delete');
  assert(preview.previewToken, 'preview returns previewToken');
}

// --- apply full scaffold ---
{
  resetWorkspace();
  const preview = await hostCommand({
    command: 'preview',
    recipe: 'scaffold_feature',
    args: scaffoldArgs,
  });
  assert(preview.ok === true, `preview before apply ok (${preview.error ?? ''})`);

  const apply = await hostCommand({
    command: 'apply',
    recipe: 'scaffold_feature',
    args: scaffoldArgs,
    previewToken: preview.previewToken,
    selection: {},
  });
  assert(apply.ok === true, `apply returns ok (${apply.error ?? ''})`);
  assert((apply.applied ?? []).length === 4, 'apply reports four files');

  assert(fileExists('lib/counter/counter_repository.dart'), 'repository file created');
  assert(readFile('lib/counter/counter_repository.dart').includes('class CounterRepository'), 'repository content');
  assert(readFile('lib/counter/counter.dart').includes('final int tickCount = 0;'), 'counter field inserted');
  assert(readFile('lib/app.dart').includes("print('scaffold')"), 'app log line inserted');
  assert(readFile('lib/app.dart').includes("print('starting')"), 'app original content preserved');
  assert(!fileExists('lib/legacy/stale.dart'), 'legacy file deleted');

  const replay = await hostCommand({
    command: 'preview',
    recipe: 'scaffold_feature',
    args: scaffoldArgs,
  });
  assert(replay.ok === false, 're-preview after apply fails');
  assert((replay.error ?? '').includes('already exists'), 're-preview reports existing file');
}

// --- delete ifMissing: skip (inline recipe) ---
{
  resetWorkspace();
  fs.rmSync(path.join(workspaceRoot, 'lib/legacy/stale.dart'));

  const inlineRecipe = {
    id: 'delete_only',
    steps: [{ delete: { path: 'lib/legacy/stale.dart', ifMissing: 'skip' } }],
  };

  const preview = await hostCommand({
    command: 'preview',
    inlineRecipe,
    args: {},
  });
  assert(preview.ok === true, 'delete skip preview ok');
  assert((preview.files ?? []).length === 0, 'skipped delete omitted from preview');

  const apply = await hostCommand({
    command: 'apply',
    inlineRecipe,
    args: {},
    previewToken: preview.previewToken,
    selection: {},
  });
  assert(apply.ok === true, 'delete skip apply ok');
  assert((apply.applied ?? []).length === 0, 'skipped delete applies nothing');
}

// --- per-file selection ---
{
  resetWorkspace();
  const preview = await hostCommand({
    command: 'preview',
    recipe: 'scaffold_feature',
    args: scaffoldArgs,
  });
  assert(preview.ok === true, 'selection preview ok');

  const apply = await hostCommand({
    command: 'apply',
    recipe: 'scaffold_feature',
    args: scaffoldArgs,
    previewToken: preview.previewToken,
    selection: { files: { 'lib/app.dart': { include: false } } },
  });
  assert(apply.ok === true, 'selection apply ok');
  assert(fileExists('lib/counter/counter_repository.dart'), 'selection: repository created');
  assert(readFile('lib/counter/counter.dart').includes('tickCount'), 'selection: counter patched');
  assert(!readFile('lib/app.dart').includes("print('scaffold')"), 'selection: app edit skipped');
  assert(!fileExists('lib/legacy/stale.dart'), 'selection: legacy deleted');
}

if (keepWorkspace) {
  console.log(`\nAll scaffold integration checks passed. Workspace kept at:\n  ${workspaceRoot}`);
} else {
  fs.rmSync(workspaceRoot, { recursive: true, force: true });
  console.log('\nAll scaffold integration checks passed.');
}
