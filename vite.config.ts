import { URL, fileURLToPath } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';

import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST || false;

const hmr = host ? { protocol: 'ws', host, port: 1421 } : undefined;

export default defineConfig({
  plugins: [tailwindcss(), vue()],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },

  // prevent vite from obscuring rust errors
  clearScreen: false,
  server: {
    port: 1420,
    // tauri expects a fixed port, fail if that port is not available
    strictPort: true,
    host,
    hmr,
    watch: { ignored: ['**/src-tauri/**'] },
  },
});
