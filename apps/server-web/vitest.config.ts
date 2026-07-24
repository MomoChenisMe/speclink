import { defineConfig } from "vitest/config";

// 與 apps/desktop 一致：jsdom + globals + 自動 JSX；不做任何 console 過濾或 act
// 警告壓制（delivery-baseline「禁止以壓制方式清零」——警告一律由測試側明確等待消除）。
export default defineConfig({
  esbuild: { jsx: "automatic", jsxImportSource: "react" },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.ts"],
  },
});
