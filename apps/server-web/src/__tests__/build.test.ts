// @vitest-environment node
// 以 node 環境執行：Vite build 內部使用 esbuild，在 jsdom 環境下其
// TextEncoder/Uint8Array 不變式會被破壞。此測試僅檢視 build 產物，不需 DOM。
import { describe, it, expect } from "vitest";
import { build, type RollupOutput } from "vite";
import { fileURLToPath } from "node:url";

// Production build 契約（server-web-console D5 / delivery-baseline）：Vite 產出的
// JS chunk 與 CSS 資產必須帶內容雜湊——immutable 快取與 binary 內嵌都靠這個保證。
// 以 Vite build API 就地建置（write:false，不落地），檢視 emitted chunk 檔名。
const SERVER_WEB_ROOT = fileURLToPath(new URL("../../", import.meta.url));

const HASHED = /-[A-Za-z0-9_-]{8,}\.(js|css)$/;

describe("Vite production build", () => {
  it(
    "emits content-hashed JS chunks and CSS assets",
    async () => {
      const result = (await build({
        root: SERVER_WEB_ROOT,
        logLevel: "silent",
        build: { write: false },
      })) as RollupOutput | RollupOutput[];

      const outputs = Array.isArray(result) ? result : [result];
      const items = outputs.flatMap((o) => o.output);

      const jsChunks = items.filter(
        (i) => i.type === "chunk" && i.fileName.endsWith(".js"),
      );
      const cssAssets = items.filter(
        (i) => i.type === "asset" && i.fileName.endsWith(".css"),
      );

      expect(jsChunks.length).toBeGreaterThan(0);
      expect(cssAssets.length).toBeGreaterThan(0);

      for (const item of [...jsChunks, ...cssAssets]) {
        expect(item.fileName, `${item.fileName} should be content-hashed`).toMatch(
          HASHED,
        );
      }
    },
    60_000,
  );
});
