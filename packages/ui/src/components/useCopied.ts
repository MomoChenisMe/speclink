import { useEffect, useRef, useState } from "react";

/** 勾號維持時間（spec「共用元件唯一來源」：主色勾號 1.5 秒）。 */
const COPIED_MS = 1500;

/** 複製鈕的 copied 回饋：觸發後亮 1.5 秒自動復原，重複觸發重新計時。
 * 計時器隨 unmount 取消；卸載後才到的觸發（剪貼簿非同步寫入晚於卸載完成）直接忽略——
 * 晚於卸載的 setState 在測試環境拆除後會炸（window 已不存在），真實頁面則是對已卸載
 * 元件白做工、留下沒人清的計時器。 */
export function useCopied(): [boolean, () => void] {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (timer.current) clearTimeout(timer.current);
    };
  }, []);
  const markCopied = () => {
    if (!mounted.current) return;
    setCopied(true);
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopied(false), COPIED_MS);
  };
  return [copied, markCopied];
}
