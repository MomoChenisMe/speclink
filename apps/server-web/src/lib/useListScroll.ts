import { useRef } from "react";

/** 列表卡的捲動區：`scrollRef` 交給 ListCard，換頁、改每頁筆數或切分頁後呼叫 `scrollToTop` 捲回頂部。 */
export function useListScroll() {
  const scrollRef = useRef<HTMLDivElement>(null);
  const scrollToTop = () => {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  };
  return { scrollRef, scrollToTop };
}
