import react from '@vitejs/plugin-react';
import path from 'path';
import { defineConfig } from 'vite';

/**
 * Custom-elements build — consumed by the Jekyll docs site.
 *
 * Nothing is external: the docs site loads a single `<script type="module">`
 * with no import map, so React and the graph renderer are bundled in. Output
 * is copied to `docs/assets/js/` by `make docs` before Jekyll runs.
 *
 * `emptyOutDir` is off because the React build (vite.react.config.ts) writes to
 * the same directory and runs first.
 */
export default defineConfig({
  plugins: [react()],
  define: {
    // Bundled React must not fall through to the development build.
    'process.env.NODE_ENV': JSON.stringify('production'),
  },
  build: {
    outDir: 'dist',
    emptyOutDir: false,
    sourcemap: false,
    cssCodeSplit: false,
    lib: {
      entry: path.resolve(__dirname, 'src/elements.ts'),
      formats: ['es'],
      fileName: () => 'elements.js',
      cssFileName: 'elements',
    },
  },
});
