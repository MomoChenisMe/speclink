import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
export interface RuntimePreferences {
  formatVersion: 1;
  debugEnabled: boolean;
  proxy: { mode: "system" | "manual" | "direct"; url: string | null };
}
export interface RuntimeSnapshot extends RuntimePreferences {
  readError: string | null;
  diagnosticsStatus: "off" | "recording" | "writeFailed";
}
export interface ProbeResult { operationId: string; results: Array<{destination:string;httpStatus:number|null;errorCode:string|null;durationMs:number}> }
export const readRuntimePreferences = async () => invoke<RuntimeSnapshot>("runtime_preferences_read");
export const writeRuntimePreferences = async (preferences:RuntimePreferences) => {
  const result=await invoke<RuntimeSnapshot>("runtime_preferences_write",{preferences});
  window.dispatchEvent(new CustomEvent("speclink:runtime-preferences",{detail:result}));
  return result;
};
export const probeUpdate = async () => invoke<ProbeResult>("network_probe_update");
export async function exportDiagnostics() {
  const path=await save({defaultPath:"speclink-diagnostics.zip",filters:[{name:"ZIP",extensions:["zip"]}]});
  return path ? invoke<{path:string;bytes:number;skippedRecords:number}>("diagnostics_export",{path}) : null;
}
export interface FrontendDiagnostic {
  formatVersion: 1; timestamp: string; operationId: string; area: "frontend";
  event: "render-error" | "unhandled-error" | "unhandled-rejection";
  durationMs: null; destination: null; proxyMode: null; route: null; proxyEndpoint: null;
  httpStatus: null; errorCode: string | null; errorChain: Array<{kind:string;code:string}>; droppedCount: 0;
}
export const appendDiagnostic = async (event:FrontendDiagnostic) => invoke<void>("diagnostics_append",{event});
