import { Component, useEffect, useRef, type ReactNode } from "react";
import { Button, useI18n } from "@speclink/ui";
import { NetworkDiagnosticsPage } from "./views/settings/NetworkDiagnosticsPage";
import { appendDiagnostic, readRuntimePreferences, type FrontendDiagnostic, type RuntimeSnapshot } from "./adapter/runtime";

function scriptLocation(raw?: string, line?: number, column?: number): string | undefined {
  if (!raw) return;
  try {
    const url = new URL(raw, window.location.href);
    if (url.origin !== window.location.origin || url.search || url.hash || !/^\/assets\/[A-Za-z0-9_-]{1,100}\.js$/.test(url.pathname)) return;
    if (!Array.from(document.scripts).some(script => script.src === url.href)) return;
    return `app:${url.pathname.slice(1)}:${Math.min(Math.max(line ?? 0, 0), 9999999)}:${Math.min(Math.max(column ?? 0, 0), 9999999)}`;
  } catch { return; }
}

function eventFor(event: FrontendDiagnostic["event"], error: unknown, location?: string): FrontendDiagnostic {
  if (!location && error instanceof Error) {
    for (const script of Array.from(document.scripts)) {
      const index = error.stack?.indexOf(`${script.src}:`) ?? -1;
      if (index < 0) continue;
      const match = error.stack?.slice(index + script.src.length).match(/^:(\d{1,7}):(\d{1,7})/);
      if (match) location = scriptLocation(script.src, Number(match[1]), Number(match[2]));
      if (location) break;
    }
  }
  const name = error instanceof Error && ["Error", "TypeError", "SyntaxError", "RangeError", "ReferenceError"].includes(error.name) ? error.name : "Error";
  return {
    formatVersion: 1, timestamp: new Date().toISOString(), operationId: "", area: "frontend", event,
    durationMs: null, destination: null, proxyMode: null, route: null, proxyEndpoint: null, httpStatus: null,
    errorCode: name, errorChain: [{ kind: "frontend", code: name }, ...(location ? [{ kind: "frontend", code: location }] : [])], droppedCount: 0,
  };
}

export function DiagnosticsBoundary({ children }: { children: ReactNode }) {
  const { t } = useI18n();
  const enabled = useRef<boolean | null>(null);
  const buffered = useRef<FrontendDiagnostic[]>([]);
  const report = (event: FrontendDiagnostic["event"], error: unknown, location?: string) => {
    const entry = eventFor(event, error, location);
    if (enabled.current === null) {
      if (buffered.current.length < 16) buffered.current.push(entry);
    } else if (enabled.current) void appendDiagnostic(entry).catch(() => {});
  };
  const reportRef = useRef(report);
  reportRef.current = report;
  useEffect(() => {
    let disposed = false;
    let preferencesChanged = false;
    const apply = (snapshot: RuntimeSnapshot | undefined) => {
      if (disposed) return;
      enabled.current = snapshot?.debugEnabled === true && !snapshot?.readError;
      if (enabled.current) for (const entry of buffered.current) void appendDiagnostic(entry).catch(() => {});
      buffered.current = [];
    };
    void readRuntimePreferences().then(value => { if (!preferencesChanged) apply(value); }).catch(() => { if (!preferencesChanged) apply(undefined); });
    const preferences = (event: Event) => {
      preferencesChanged = true;
      apply((event as CustomEvent<RuntimeSnapshot>).detail);
    };
    const error = (event: ErrorEvent) => {
      reportRef.current("unhandled-error", event.error, scriptLocation(event.filename, event.lineno, event.colno));
    };
    const rejection = (event: PromiseRejectionEvent) => {
      reportRef.current("unhandled-rejection", event.reason);
    };
    window.addEventListener("speclink:runtime-preferences", preferences);
    window.addEventListener("error", error);
    window.addEventListener("unhandledrejection", rejection);
    return () => {
      disposed = true;
      window.removeEventListener("speclink:runtime-preferences", preferences);
      window.removeEventListener("error", error);
      window.removeEventListener("unhandledrejection", rejection);
    };
  }, []);
  return <Boundary onError={error => report("render-error", error)} labels={{ error: t("network.renderError"), retry: t("network.retry"), diagnostics: t("network.openDiagnostics") }}>{children}</Boundary>;
}

class Boundary extends Component<{
  children: ReactNode;
  onError: (error: unknown) => void;
  labels: { error: string; retry: string; diagnostics: string };
}, { view: "content" | "error" | "diagnostics" }> {
  state: { view: "content" | "error" | "diagnostics" } = { view: "content" };
  static getDerivedStateFromError() { return { view: "error" }; }
  componentDidCatch(error: Error) { this.props.onError(error); }
  render() {
    // This exit remains usable while the main App subtree cannot mount.
    if (this.state.view === "diagnostics") return <div className="space-y-4 p-6">
      <Button variant="secondary" onClick={() => this.setState({ view: "content" })}>{this.props.labels.retry}</Button>
      <NetworkDiagnosticsPage />
    </div>;
    if (this.state.view === "content") return this.props.children;
    return <div role="alert" className="flex flex-col items-start gap-4 p-6">
      <p>{this.props.labels.error}</p>
      <Button variant="secondary" onClick={() => this.setState({ view: "content" })}>{this.props.labels.retry}</Button>
      <Button variant="secondary" onClick={() => this.setState({ view: "diagnostics" })}>{this.props.labels.diagnostics}</Button>
    </div>;
  }
}
