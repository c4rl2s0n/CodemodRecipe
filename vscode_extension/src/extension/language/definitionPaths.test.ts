import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { afterEach, describe, expect, it } from 'vitest';
import { resolveTemplatePath } from './navigationLogic';

describe('resolveTemplatePath', () => {
  const dirs: string[] = [];

  afterEach(() => {
    for (const d of dirs.splice(0)) {
      fs.rmSync(d, { recursive: true, force: true });
    }
  });

  it('prefers recipe-local template over codemod root', () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'codemod-tpl-'));
    dirs.push(root);
    const recipeDir = path.join(root, '.codemod', 'recipes', 'feat');
    const codemodTemplates = path.join(root, '.codemod', 'templates');
    fs.mkdirSync(recipeDir, { recursive: true });
    fs.mkdirSync(codemodTemplates, { recursive: true });
    fs.writeFileSync(path.join(recipeDir, 'widget.template'), 'local');
    fs.writeFileSync(path.join(codemodTemplates, 'widget.template'), 'global');

    const resolved = resolveTemplatePath(
      root,
      '.codemod',
      'widget.template',
      '.codemod/recipes/feat/recipe.luau'
    );
    expect(resolved).toBe(path.join(recipeDir, 'widget.template'));
  });

  it('falls back to codemod root', () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'codemod-tpl-'));
    dirs.push(root);
    const codemodTemplates = path.join(root, '.codemod');
    fs.mkdirSync(codemodTemplates, { recursive: true });
    fs.writeFileSync(path.join(codemodTemplates, 'x.template'), 'global');

    const resolved = resolveTemplatePath(root, '.codemod', 'x.template');
    expect(resolved).toBe(path.join(codemodTemplates, 'x.template'));
  });
});
