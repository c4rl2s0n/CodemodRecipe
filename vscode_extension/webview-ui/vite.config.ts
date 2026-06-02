import path from 'node:path';
import { fileURLToPath } from 'node:url';
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite';

const webviewUiRoot = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root: webviewUiRoot,
  plugins: [vue()],
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
    __VUE_OPTIONS_API__: 'true',
    __VUE_PROD_DEVTOOLS__: 'false',
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: 'false',
  },
  build: {
    outDir: path.join(webviewUiRoot, '../media'),
    emptyOutDir: false,
    cssCodeSplit: false,
    lib: {
      entry: 'src/main.ts',
      name: 'codemodRecipeWebview',
      formats: ['iife'],
      fileName: () => 'recipeView.js',
    },
    rollupOptions: {
      output: {
        assetFileNames: 'recipeView.css',
        inlineDynamicImports: true,
      },
    },
  },
});
