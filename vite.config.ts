import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  // 防止 Vite 清除 Rust 显示的错误信息
  clearScreen: false,
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Tauri 使用固定端口
  server: {
    port: 1420,
    strictPort: true,
  },
  // 构建优化
  build: {
    minify: 'esbuild',
    cssMinify: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('@xyflow/react')) return 'xyflow';
          if (id.includes('react-dom') || id.includes('react/')) return 'react';
          if (id.includes('framer-motion')) return 'framer';
        },
      },
    },
  },
  // 让 Tauri 使用环境变量设置的 host
  envPrefix: ['VITE_', 'TAURI_'],
});