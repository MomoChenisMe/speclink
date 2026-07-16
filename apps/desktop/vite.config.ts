import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri 讀取 build.outDir（tauri.conf.json 的 frontendDist=../dist）作為前端資產。
export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "./",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      // 多頁入口：main（主視窗）＋ panel（系統匣面板樣式，design D5）。
      input: {
        main: fileURLToPath(new URL("index.html", import.meta.url)),
        panel: fileURLToPath(new URL("panel.html", import.meta.url)),
      },
    },
  },
});
