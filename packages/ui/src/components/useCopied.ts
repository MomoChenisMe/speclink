import { useEffect, useRef, useState } from "react";

/** 複製鈕的 copied 回饋：觸發後亮 1.2 秒自動復原，重複觸發重新計時。
 * 計時器隨 unmount 取消——晚於卸載觸發的 setState 在測試環境拆除後會炸
 * （window 已不存在），真實頁面則是對已卸載元件白做工。 */
export function useCopied(): [boolean, () => void] {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
  const markCopied = () => {
    setCopied(true);
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopied(false), 1200);
  };
  return [copied, markCopied];
}
