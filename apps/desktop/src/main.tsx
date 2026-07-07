import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import { createTauriDataSource } from "./adapter/tauriDataSource";
import { createWorkspaceAdapter } from "./adapter/workspace";
import "./index.css";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App dataSource={createTauriDataSource()} workspace={createWorkspaceAdapter()} />
    </StrictMode>,
  );
}
