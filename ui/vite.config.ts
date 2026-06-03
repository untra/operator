import react from '@vitejs/plugin-react';
import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    alias: {
      '@operator/bindings': path.resolve(__dirname, '../bindings'),
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
