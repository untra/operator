import react from '@vitejs/plugin-react';
import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    alias: {
      '@operator/bindings': path.resolve(__dirname, '../bindings'),
      // Built by `make webcomponents` before any ui build.
      '@operator/webcomponents': path.resolve(__dirname, '../webcomponents/dist/index.js'),
    },
  },
  server: {
    host: '127.0.0.1',
    port: 5173,
    proxy: {
      '/api': 'http://127.0.0.1:7008',
      '/swagger-ui': 'http://127.0.0.1:7008',
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
});
