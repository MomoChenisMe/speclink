import type { ChangeItem, SpecItem } from "../adapter";
import { Button } from "./ui/button";

/** 樹上被選取的節點：一個 change 或一個 spec。 */
export interface TreeSelection {
  kind: "change" | "spec";
  id: string;
}

export interface DocumentTreeProps {
  changes: ChangeItem[];
  specs: SpecItem[];
  onSelect?: (sel: TreeSelection) => void;
}

function SectionHeading({ children }: { children: React.ReactNode }) {
  return (
    <h2 className="text-[11px] uppercase tracking-wider text-muted-foreground px-2 mb-1.5 mt-0">
      {children}
    </h2>
  );
}

/** 導覽樹：change 與 spec 兩區，點選觸發 onSelect。純呈現，資料由 props 注入。 */
export function DocumentTree({ changes, specs, onSelect }: DocumentTreeProps) {
  return (
    <nav className="flex flex-col gap-4">
      <section>
        <SectionHeading>Changes</SectionHeading>
        <ul className="list-none m-0 p-0 flex flex-col gap-0.5">
          {changes.map((c) => (
            <li key={c.name}>
              <Button
                variant="ghost"
                className="w-full justify-start h-8 px-2.5 font-normal"
                onClick={() => onSelect?.({ kind: "change", id: c.name })}
              >
                {c.name}
              </Button>
            </li>
          ))}
        </ul>
      </section>
      <section>
        <SectionHeading>Specs</SectionHeading>
        <ul className="list-none m-0 p-0 flex flex-col gap-0.5">
          {specs.map((s) => (
            <li key={s.id}>
              <Button
                variant="ghost"
                className="w-full justify-start h-8 px-2.5 font-normal"
                onClick={() => onSelect?.({ kind: "spec", id: s.id })}
              >
                {s.id}
              </Button>
            </li>
          ))}
        </ul>
      </section>
    </nav>
  );
}
