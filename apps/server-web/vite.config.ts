import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Speclink Server Web Console 的 production build（D5）：單一 index.html 入口，
// base "/"（由 speclink-server 從 root origin 服務），Vite 預設輸出內容雜湊資產，
// 供編譯期內嵌 binary。不做獨立部署、CDN 或第二 origin。
export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "/",
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
