// 觸控板滑動返回（desktop-readonly-detail-pages design D8）：詳情頁開啟時，水平為主、朝返回
// 方向（手指向右）的滑動累計位移，主區左緣的「←」指示隨之推出、達門檻轉實心。DOM 的 wheel
// 事件不帶手指放開的相位，以最後一個事件後靜止 IDLE_MS 視為手勢結束（含慣性捲動尾段），
// 屆時已達門檻即返回、未達收回。
import { useEffect, useRef, useState } from "react";
import { ArrowLeft } from "lucide-react";
import { cn } from "@speclink/ui";

/** 觸發返回所需的累計水平位移（px）。 */
export const SWIPE_THRESHOLD = 120;
const IDLE_MS = 120;

/** 朝返回方向的位移（正值）。WebKit 以 webkitDirectionInvertedFromDevice 標出自然捲動：
 * 開啟（macOS 預設）時手指向右 deltaX 為負；關閉時相反。缺席（非 WebKit）視同開啟。 */
function backDelta(e: WheelEvent): number {
  const inverted = (e as WheelEvent & { webkitDirectionInvertedFromDevice?: boolean })
    .webkitDirectionInvertedFromDevice;
  return inverted === false ? e.deltaX : -e.deltaX;
}

/** 事件目標或其祖先是否為仍可沿 deltaX 方向捲動的水平捲動區塊（寬表格、程式碼區塊）——先讓它捲。
 * 方向看 deltaX 本身而非手指方向：自然捲動關閉時，手指向右是把區塊往右捲。只裁切不捲動的
 * 區塊（overflow hidden，如截斷的長名稱）不算。 */
function canScrollX(target: EventTarget | null, deltaX: number): boolean {
  for (let el = target instanceof Element ? target : null; el; el = el.parentElement) {
    if (el.scrollWidth <= el.clientWidth || !/auto|scroll/.test(getComputedStyle(el).overflowX)) continue;
    if (deltaX < 0 ? el.scrollLeft > 0 : el.scrollLeft + el.clientWidth < el.scrollWidth) return true;
  }
  return false;
}

export function SwipeBack({ enabled, onBack }: { enabled: boolean; onBack: () => void }) {
  const [progress, setProgress] = useState(0);
  const onBackRef = useRef(onBack);
  onBackRef.current = onBack;

  useEffect(() => {
    if (!enabled) return;
    let total = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const end = () => {
      const fire = total >= SWIPE_THRESHOLD;
      total = 0;
      timer = undefined;
      setProgress(0);
      if (fire) onBackRef.current();
    };
    const onWheel = (e: WheelEvent) => {
      // 縮放（ctrl＋滾輪）與垂直為主的捲動不是返回手勢。
      if (e.ctrlKey || Math.abs(e.deltaX) <= Math.abs(e.deltaY)) return;
      const delta = backDelta(e);
      // 手勢起點只認朝返回方向、且不在仍可沿此方向捲的區塊內；開始後反向滑回會扣減。
      if (total === 0 && (delta <= 0 || canScrollX(e.target, e.deltaX))) return;
      total = Math.max(0, total + delta);
      setProgress(Math.min(1, total / SWIPE_THRESHOLD));
      clearTimeout(timer);
      timer = setTimeout(end, IDLE_MS);
    };
    window.addEventListener("wheel", onWheel, { passive: true });
    return () => {
      window.removeEventListener("wheel", onWheel);
      clearTimeout(timer);
      setProgress(0);
    };
  }, [enabled]);

  const armed = progress >= 1;
  return (
    <div
      data-swipe-back
      data-progress={progress.toFixed(2)}
      data-armed={armed || undefined}
      aria-hidden="true"
      className={cn(
        "pointer-events-none absolute left-0 top-1/2 z-20 flex h-16 w-10 items-center justify-center rounded-r-2xl shadow-sm transition-colors duration-150",
        armed ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground",
      )}
      // 隨累計位移自左緣推出；位移為 0 時完全收在左緣外。
      style={{ transform: `translate(${(progress - 1) * 100}%, -50%)`, opacity: progress === 0 ? 0 : 1 }}
    >
      <ArrowLeft className="h-5 w-5" />
    </div>
  );
}
