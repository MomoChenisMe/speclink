/** 設定型卡內的一列灰字：沒有資料時的說明，或卡片開頭的一段說明（server-web-list-pages design D10）。 */
export function CardNote({ children }: { children: string }) {
  return <p className="m-0 px-4 py-3 text-[13px] text-muted-foreground">{children}</p>;
}
