import { useState } from "react";
import { Check, Copy } from "lucide-react";

import type { ArchivedItem } from "../adapter";
import { Input } from "./ui/input";

/** Spectra 式封存列：日期＋名稱＋複製完整封存名。 */
export function ArchivedRow({ item }: { item: ArchivedItem }) {
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

export interface ArchivedListProps {
  archived: ArchivedItem[];
  query: string;
  onQuery: (q: string) => void;
}

/** 已封存獨立頁：搜尋＋列表。 */
export function ArchivedList({ archived, query, onQuery }: ArchivedListProps) {
  const q = query.trim().toLowerCase();
  const filtered = archived.filter((a) => a.name.toLowerCase().includes(q));
  return (
    <div className="flex flex-col gap-3 max-w-3xl mx-auto w-full">
      <Input placeholder="搜尋已封存的變更…" value={query} onChange={(e) => onQuery(e.target.value)} />
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">已封存的變更</h2>
        <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
          {filtered.length}
        </span>
      </div>
      <div className="flex flex-col gap-2.5">
        {filtered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">沒有已封存的變更</div>
        ) : (
          filtered.map((a) => <ArchivedRow key={a.datedName} item={a} />)
        )}
      </div>
    </div>
  );
}
