import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri 讀取 build.outDir（tauri.conf.json 的 frontendDist=../dist）作為 release 前端資產；
// dev 模式改由下方 server 供應（tauri.conf.json 的 devUrl 指向同一個埠）。
export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "./",
  // devUrl 是寫死的字串，埠一旦浮動就對不上——strictPort 讓埠被占用時直接報錯，
  // 而不是自動換埠後開出一個沒有錯誤訊息的空白視窗。
  server: {
    port: 1420,
    strictPort: true,
  },
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
