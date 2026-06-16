// Standalone integration smoke test for the Dart host protocol.
//
// Mirrors the spawn + marker-extraction logic in src/host/dartBridge.ts
// (without the vscode dependency) so the TypeScript <-> Dart contract can be verified from
// the command line. Run with: node scripts/smoke.mjs
//
// Uses bin/codemod_host.dart from the codemod_recipe package with --codemod-root flag.
import { spawn } from 'child_process';
import * as path from 'path';

const RESULT_BEGIN = '__CODEMOD_RESULT_BEGIN__';
const RESULT_END = '__CODEMOD_RESULT_END__';

function extractResult(output) {
  const begin = output.indexOf(RESULT_BEGIN);
  const end = output.indexOf(RESULT_END);
  if (begin === -1 || end === -1 || end < begin) return undefined;
  return output.slice(begin + RESULT_BEGIN.length, end).trim();
}

function send(codemodRoot, command) {
  // Workspace root is the current directory
  const cwd = process.cwd();
  return new Promise((resolve, reject) => {
    const child = spawn('dart', [
      'run', 
      'bin/codemod_host.dart',
      '--workspace-root', cwd,
      '--codemod-root', codemodRoot
    ], { cwd });
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

// Default codemod root - can be overridden via command line argument
const codemodRoot = process.argv[2] ?? '.codemod';

const assert = (cond, msg) => {
  if (!cond) {
    console.error('FAIL:', msg);
    process.exit(1);
  }
  console.log('ok -', msg);
};

const list = await send(codemodRoot, { command: 'list' });
assert(list.ok === true, 'list returns ok');
assert(Array.isArray(list.recipes) && list.recipes.length >= 1, 'list has recipes');

const preview = await send(codemodRoot, {
  command: 'preview',
  recipe: 'add_method',
  args: { file: 'lib/counter.dart', class: 'Counter', method: 'reset' },
});
assert(preview.ok === true, 'preview returns ok');
assert(preview.files.length === 1, 'preview returns one file');
assert(preview.files[0].modified.includes('void reset()'), 'preview shows new method');
assert(preview.files[0].patches.length === 1, 'preview has one patch');

const missing = await send(codemodRoot, {
  command: 'preview',
  recipe: 'add_method',
  args: { file: 'lib/counter.dart' },
});
assert(missing.ok === false, 'preview reports validation error');

console.log('\nAll smoke checks passed.');
