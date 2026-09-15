import react from '@vitejs/plugin-react';
import path from 'node:path';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    alias: [
      { find: '@operator/bindings', replacement: path.resolve(__dirname, '../bindings') },
      {
        find: '@operator/webcomponents/styles.css',
        replacement: path.resolve(__dirname, '../webcomponents/dist/index.css'),
      },
      {
        find: /^@operator\/webcomponents$/,
        replacement: path.resolve(__dirname, '../webcomponents/dist/index.js'),
      },
    ],
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
