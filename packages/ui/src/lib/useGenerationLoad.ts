import { useEffect, useRef, type Dispatch, type SetStateAction } from "react";

/**
 * 詳情頁的文件載入：開啟時全量載入、世代遞增時就地重載（不清空）。`load` 收到「此回應仍是最新」
 * 的檢查——回應晚到且已被後一次載入取代即丟棄（latest-wins）。換目標由呼叫端以 key 重掛處理。
 */
export function useGenerationLoad(gen: number, load: (isLatest: () => boolean) => void) {
  const seq = useRef(0);
  const loadedGen = useRef(-1);
  useEffect(() => {
    if (gen <= loadedGen.current) return;
    loadedGen.current = gen;
    const mine = ++seq.current;
    load(() => seq.current === mine);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [gen]);
}

/** 失敗收斂：還沒有東西可顯示才落終態空值（undefined 停著＝永久骨架）；已有內容維持前值。 */
export function settle<T>(apply: Dispatch<SetStateAction<T | undefined>>, empty: NoInfer<T>, isLatest: () => boolean) {
  return () => {
    if (isLatest()) apply((prev) => (prev === undefined ? empty : prev));
  };
}
