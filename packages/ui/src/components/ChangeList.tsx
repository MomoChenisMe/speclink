import { useState } from "react";
import { Check, Copy } from "lucide-react";

import type { ChangeItem, ArchivedItem, Verb } from "../adapter";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { ChangeListItem } from "./ChangeListItem";

/** Spectra 式封存列：日期＋名稱＋複製完整封存名。 */
function ArchivedRow({ item }: { item: ArchivedItem }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    void navigator.clipboard?.writeText(item.datedName);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  return (
    <div className="group flex items-center gap-2.5 rounded-lg border border-border bg-card p-3">
      <span className="text-xs text-muted-foreground tabular-nums shrink-0">{item.date}</span>
      <span className="font-medium text-sm truncate flex-1">{item.name}</span>
      <button
        type="button"
        aria-label="複製封存名稱"
        className={`shrink-0 text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
        onClick={copy}
      >
        {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
      </button>
    </div>
  );
}

export type ListView = "active" | "archived";

export interface ChangeListProps {
  changes: ChangeItem[];
  archived: ArchivedItem[];
  view: ListView;
  onViewChange: (v: ListView) => void;
  query: string;
  onQuery: (q: string) => void;
  expandedName: string | null;
  onToggle: (name: string) => void;
  loadDocument: (change: string, artifact: string) => Promise<string | null>;
  loadCapabilities: (change: string) => Promise<string[]>;
  onRunVerb?: (verb: Verb, change: string) => void;
}

export function ChangeList({
  changes,
  archived,
  view,
  onViewChange,
  query,
  onQuery,
  expandedName,
  onToggle,
  loadDocument,
  loadCapabilities,
  onRunVerb,
}: ChangeListProps) {
  const q = query.trim().toLowerCase();
  const activeFiltered = changes.filter((c) => c.name.toLowerCase().includes(q));
  const archivedFiltered = archived.filter((a) => a.name.toLowerCase().includes(q));
  const count = view === "active" ? activeFiltered.length : archivedFiltered.length;

  return (
    <div className="flex flex-col gap-3 max-w-4xl mx-auto w-full">
      {/* 工具列 */}
      <div className="flex items-center gap-3">
        <Input
          placeholder="搜尋…"
          value={query}
          onChange={(e) => onQuery(e.target.value)}
          className="flex-1"
        />
        <div className="flex rounded-md border border-border overflow-hidden shrink-0">
          <button
            className={`px-3 py-1.5 text-sm ${view === "active" ? "bg-primary text-primary-foreground" : "bg-background text-muted-foreground hover:text-foreground"}`}
            onClick={() => onViewChange("active")}
          >
            進行中
          </button>
          <button
            className={`px-3 py-1.5 text-sm ${view === "archived" ? "bg-primary text-primary-foreground" : "bg-background text-muted-foreground hover:text-foreground"}`}
            onClick={() => onViewChange("archived")}
          >
            已封存
          </button>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">{view === "active" ? "進行中的變更" : "已封存的變更"}</h2>
        <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-primary text-primary-foreground text-xs font-medium">
          {count}
        </span>
      </div>

      {/* 清單 */}
      <div className="flex flex-col gap-2.5">
        {view === "active" ? (
          activeFiltered.length === 0 ? (
            <div className="text-muted-foreground text-sm py-8 text-center">沒有進行中的變更</div>
          ) : (
            activeFiltered.map((c) => (
              <ChangeListItem
                key={c.name}
                change={c}
                expanded={expandedName === c.name}
                onToggle={onToggle}
                loadDocument={(a) => loadDocument(c.name, a)}
                loadCapabilities={() => loadCapabilities(c.name)}
                onRunVerb={onRunVerb}
              />
            ))
          )
        ) : archivedFiltered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">沒有已封存的變更</div>
        ) : (
          archivedFiltered.map((a) => <ArchivedRow key={a.datedName} item={a} />)
        )}
      </div>
    </div>
  );
}
