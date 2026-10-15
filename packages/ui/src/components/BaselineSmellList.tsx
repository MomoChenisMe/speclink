import type { HTMLAttributes } from "react";

/** 審查站預設 smell 名稱清單（名稱由引擎提供）：desktop 設定頁的審查規則頁與工單分頁的
 * 套用規則列共用同一個元件。 */
export function BaselineSmellList({ names, ...rest }: { names: readonly string[] } & HTMLAttributes<HTMLUListElement>) {
  return (
    <ul data-baseline-smells {...rest} className="m-0 flex list-none flex-wrap gap-1.5 p-0">
      {names.map((name) => (
        <li key={name} className="rounded-md border border-border bg-muted/40 px-2 py-0.5 font-mono text-xs">
          {name}
        </li>
      ))}
    </ul>
  );
}
