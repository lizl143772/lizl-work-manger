import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],
  server: {
    port: 5173,
    strictPort: true,
    host: true, // or '0.0.0.0'
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
