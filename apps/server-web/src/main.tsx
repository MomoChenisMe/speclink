import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
// Noto Sans TC 隨資產打包（離線可用、不從 Google Fonts 載入——D5/D6）。
import "@fontsource-variable/noto-sans-tc";
import "./index.css";
import { App } from "./App";
import { createHttpClient } from "./api/client";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App client={createHttpClient()} />
    </StrictMode>,
  );
}
