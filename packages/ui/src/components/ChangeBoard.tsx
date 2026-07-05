import type { ChangeItem, Verb } from "../adapter";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { Card, CardContent, CardHeader } from "./ui/card";

const VERBS: Verb[] = ["validate", "analyze", "archive"];

export interface ChangeBoardProps {
  changes: ChangeItem[];
  onSelect?: (name: string) => void;
  onRunVerb?: (verb: Verb, change: string) => void;
}

/** 呈現 active change 清單：名稱、任務進度、動詞按鈕。純呈現，資料由 props 注入。 */
export function ChangeBoard({ changes, onSelect, onRunVerb }: ChangeBoardProps) {
  if (changes.length === 0) {
    return <div className="text-muted-foreground p-6 text-center">沒有 active change</div>;
  }
  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-3.5 list-none p-0 m-0">
      {changes.map((c) => {
        const pct = c.totalTasks > 0 ? Math.round((c.completedTasks / c.totalTasks) * 100) : 0;
        return (
          <li key={c.name}>
            <Card>
              <CardHeader className="flex-row items-center justify-between">
                <Button
                  variant="link"
                  className="h-auto p-0 font-semibold text-foreground hover:text-primary"
                  onClick={() => onSelect?.(c.name)}
                >
                  {c.name}
                </Button>
                <Badge variant={c.status === "in-progress" ? "default" : "secondary"}>
                  {c.status}
                </Badge>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <div className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden">
                    <div className="h-full rounded-full bg-primary transition-all" style={{ width: `${pct}%` }} />
                  </div>
                  <span className="text-xs text-muted-foreground tabular-nums">
                    {c.completedTasks} / {c.totalTasks}
                  </span>
                </div>
                {c.summary && (
                  <p className="text-xs text-muted-foreground line-clamp-2 m-0">{c.summary}</p>
                )}
                <div className="flex gap-1.5 flex-wrap">
                  {VERBS.map((v) => (
                    <Button key={v} variant="outline" size="sm" onClick={() => onRunVerb?.(v, c.name)}>
                      {v}
                    </Button>
                  ))}
                </div>
              </CardContent>
            </Card>
          </li>
        );
      })}
    </ul>
  );
}
