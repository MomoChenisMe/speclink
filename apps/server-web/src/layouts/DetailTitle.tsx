import { createContext, useContext, useEffect, useMemo, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { DetailNav } from "@speclink/ui";

// 詳情頁接主區頂列的唯一落點（server-web-detail-pages design D2）：ConsoleLayout 持有麵包屑第三段
// 的名稱與頂列右端的插槽節點，詳情頁以 DetailTitle 交出名稱、把 ‹ n / m › 以 portal 送進插槽。
// 只有字串進 state——ReactNode 進 state 會隨每次渲染換身分而造成迴圈，所以導覽元件走 portal。

type DetailTitleContextValue = {
  setLabel: (label: string | null) => void;
  /** 頂列右端的插槽；尚未掛上時為 null。 */
  slot: HTMLElement | null;
};

// 殼外（只渲染單頁的測試）沒有 provider：名稱無處可交、導覽不渲染。
const DetailTitleContext = createContext<DetailTitleContextValue>({ setLabel: () => {}, slot: null });

export function DetailTitleProvider({
  setLabel,
  slot,
  children,
}: DetailTitleContextValue & { children: ReactNode }) {
  const value = useMemo(() => ({ setLabel, slot }), [setLabel, slot]);
  return <DetailTitleContext.Provider value={value}>{children}</DetailTitleContext.Provider>;
}

/** 詳情頁的標題列：麵包屑第三段為 `label`，頂列右端為 ‹ n / m ›（`index` 為 -1 時整組缺席）。 */
export function DetailTitle({
  label,
  index,
  total,
  onStep,
}: {
  label: string;
  index: number;
  total: number;
  onStep: (delta: -1 | 1) => void;
}) {
  const { setLabel, slot } = useContext(DetailTitleContext);
  useEffect(() => {
    setLabel(label);
    return () => setLabel(null);
  }, [label, setLabel]);
  if (!slot) return null;
  return createPortal(<DetailNav index={index} total={total} onStep={onStep} />, slot);
}
