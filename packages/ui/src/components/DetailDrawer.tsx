import { CheckCircle2, Circle, CircleDot, Lock } from "lucide-react";
import type { ArtifactStatus } from "../adapter";
import { parseTasks } from "../tasks";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "./ui/sheet";

export interface DetailDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  changeName: string | null;
  artifacts: ArtifactStatus[];
  tasksMarkdown: string | null;
  doc: string | null;
}

function ArtifactIcon({ status }: { status: string }) {
  if (status === "done") return <CheckCircle2 className="h-4 w-4 text-primary" />;
  if (status === "ready") return <CircleDot className="h-4 w-4 text-muted-foreground" />;
  if (status === "blocked") return <Lock className="h-4 w-4 text-muted-foreground/60" />;
  return <Circle className="h-4 w-4 text-muted-foreground/60" />;
}

/** 選定 change 的側滑詳情：artifact DAG、tasks 清單、文件內容。 */
export function DetailDrawer({
  open,
  onOpenChange,
  changeName,
  artifacts,
  tasksMarkdown,
  doc,
}: DetailDrawerProps) {
  const tasks = parseTasks(tasksMarkdown);
  const doneCount = tasks.filter((t) => t.done).length;

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>{changeName ?? ""}</SheetTitle>
          <SheetDescription>Artifact 管線與任務</SheetDescription>
        </SheetHeader>

        <section className="flex flex-col gap-1.5">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Artifacts</h3>
          <ul className="flex flex-col gap-1 m-0 p-0 list-none">
            {artifacts.map((a) => (
              <li key={a.id} className="flex items-center gap-2 text-sm">
                <ArtifactIcon status={a.status} />
                <span className="font-medium">{a.id}</span>
                <span className="text-xs text-muted-foreground">{a.status}</span>
              </li>
            ))}
          </ul>
        </section>

        <section className="flex flex-col gap-1.5">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Tasks {tasks.length > 0 && <span className="tabular-nums">({doneCount}/{tasks.length})</span>}
          </h3>
          <ul className="flex flex-col gap-1 m-0 p-0 list-none max-h-64 overflow-y-auto">
            {tasks.map((t, i) => (
              <li key={i} className="flex items-start gap-2 text-[13px] leading-snug">
                {t.done ? (
                  <CheckCircle2 className="h-3.5 w-3.5 mt-0.5 shrink-0 text-primary" />
                ) : (
                  <Circle className="h-3.5 w-3.5 mt-0.5 shrink-0 text-muted-foreground/60" />
                )}
                <span className={t.done ? "text-muted-foreground line-through" : ""}>{t.text}</span>
              </li>
            ))}
          </ul>
        </section>

        {doc && (
          <section className="flex flex-col gap-1.5 min-h-0">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Document</h3>
            <pre className="rounded-md border border-border bg-muted/40 p-3 m-0 overflow-auto whitespace-pre-wrap break-words font-mono text-[11.5px] leading-relaxed max-h-72">
              {doc}
            </pre>
          </section>
        )}
      </SheetContent>
    </Sheet>
  );
}
