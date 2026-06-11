import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const webviewUiRoot = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root: webviewUiRoot,
  test: {
    include: ['src/**/*.test.ts'],
  },
});
