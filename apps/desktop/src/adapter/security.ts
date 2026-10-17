import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";

export interface StandardSelection { enabled: boolean; version: string | null }
export interface SecurityPolicy {
  enabled: boolean;
  scope: "off" | "web-api" | "api" | "web";
  standards: {
    owaspTop10: StandardSelection;
    owaspApiTop10: StandardSelection;
    owaspAsvs: StandardSelection & { level: "L1" | "L2" | "L3" };
    cwe: StandardSelection;
  };
  reviews: { spec: boolean; code: boolean; apiBoundary: boolean };
  disabledRules: string[];
  excludeDirs: string[];
}

export const DEFAULT_SECURITY_POLICY: SecurityPolicy = {
  enabled: false, scope: "off",
  standards: {
    owaspTop10: { enabled: false, version: null },
    owaspApiTop10: { enabled: false, version: null },
    owaspAsvs: { enabled: false, version: null, level: "L1" },
    cwe: { enabled: false, version: null },
  },
  reviews: { spec: true, code: true, apiBoundary: true },
  disabledRules: [], excludeDirs: [],
};

type ObjectValue = Record<string, unknown>;
const isObject = (value: unknown): value is ObjectValue =>
  value !== null && typeof value === "object" && !Array.isArray(value);

/** Only edited leaves cross the write boundary; unknown stored fields stay untouched. */
export function securityPolicyPatch(before: SecurityPolicy, after: SecurityPolicy): string | undefined {
  function diff(a: ObjectValue, b: ObjectValue): ObjectValue {
    return Object.fromEntries(Object.entries(b).flatMap(([key, value]) => {
      const next = isObject(value) && isObject(a[key]) ? diff(a[key], value) : value;
      if (isObject(next) ? Object.keys(next).length === 0 : JSON.stringify(a[key]) === JSON.stringify(next)) return [];
      return [[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`), next]];
    }));
  }
  const patch = diff(before as unknown as ObjectValue, after as unknown as ObjectValue);
  return Object.keys(patch).length ? JSON.stringify(patch) : undefined;
}

/** Adopt the same leaf patch over the latest snapshot after a resolved CAS conflict. */
export function applySecurityPolicyPatch(base: SecurityPolicy, patch?: string): SecurityPolicy {
  if (!patch) return base;
  function merge(a: ObjectValue, b: ObjectValue): ObjectValue {
    const result = { ...a };
    for (const [name, value] of Object.entries(b)) {
      const key = name.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase());
      result[key] = isObject(value) ? merge(isObject(a[key]) ? a[key] : {}, value) : value;
    }
    return result;
  }
  return merge(base as unknown as ObjectValue, JSON.parse(patch)) as unknown as SecurityPolicy;
}

export interface StandardDataList {
  standards: Array<{ standardId: string; versions: Array<{ version: string; status: string; sourceUrl: string; licenseName: string; licenseUrl: string; attribution: string; sourceFormat: string; error?: string | null }> }>;
}
export const listSecurityData = () => invoke<StandardDataList>("security_data_list");

export interface SecurityJobTicket { jobId: string; projectKey: string }
export interface SecurityProgress extends SecurityJobTicket { phase: string; completedBytes: number; totalBytes: number | null; completedFiles: number; message: string }
export type SecurityDataRequest = { action: "refresh" } | {action:"install";standardId:string;version:string;reinstall:boolean} | {action:"import";path:string} | {action:"remove";standardId:string;version:string};
export const startSecurityData = (request:SecurityDataRequest) => invoke<SecurityJobTicket>("security_data_start",{request});
export const waitSecurityData = (jobId:string) => invoke<unknown>("security_data_wait",{jobId,projectKey:"application-data"});
export const cancelSecurityJob = (jobId:string,projectKey="application-data") => invoke<boolean>("security_cancel",{jobId,projectKey});
export const watchSecurityProgress = (onProgress:(progress:SecurityProgress)=>void) => listen<SecurityProgress>("security-progress",({payload})=>onProgress(payload));
export async function chooseSecurityPackage():Promise<string|null> {
  const selected = await open({multiple:false,directory:false,filters:[{name:"ZIP",extensions:["zip"]}]});
  return typeof selected === "string" ? selected : null;
}
export const rememberSecurityPolicy = (key:string,policy:SecurityPolicy) => invoke<void>("security_remember_policy",{key,policy});


export type SecurityReviewTarget = {kind:"local";root:string} | {kind:"remote";connectionId:string;project:string;repo:string;checkoutRoot?:string};
/** All commands carry their original target; switching tabs cannot redirect a job. */
export function createSecurityReviewAdapter(target:SecurityReviewTarget, ipc:import("../session").InvokeFn=invoke):import("@speclink/ui").SecurityReviewAdapter {
 return {
  inspect:change=>ipc("security_inspect",{target,change}),
  start:change=>ipc("security_start",{target,change}),
  ...(target.kind==="remote"?{prepare:(change:string)=>ipc<SecurityJobTicket>("security_prepare",{target,change})}:{}),
  wait:ticket=>ipc("security_wait",{...ticket}),
  cancel:ticket=>ipc("security_cancel",{...ticket}),
  report:id=>ipc("security_report",{id}),
  async exportReport(id,format) {
   const path=await save({defaultPath:`security-${id}.${format==="json"?"json":"md"}`,filters:[{name:format,extensions:[format==="json"?"json":"md"]}]});
   if(path) await ipc("security_report_export",{id,format,path});
  },
  async copySummary(id) {
   const summary=await ipc<string>("security_report_summary",{id});
   const {writeText}=await import("@tauri-apps/plugin-clipboard-manager");await writeText(summary);
  },
  subscribe:watchSecurityProgress,
  async reveal(id,path,line) { await ipc("security_reveal",{target,id,path,line}); },
 };
}
