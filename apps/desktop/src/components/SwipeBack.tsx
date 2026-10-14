// 觸控板滑動返回（desktop-readonly-detail-pages design D8；desktop-swipe-back-finger-lift design D4）：
// DOM 的 wheel 事件不帶手指放開的相位，改由 macOS 原生層的 trackpad-gesture 事件界定一次手勢——
// 只在手指按下（began）到離開（ended）之間累計朝返回方向（手指向右）的位移，主區左緣的「←」指示
// 隨之推出、達門檻轉實心；手指離開的當下結算，已達門檻即返回、未達收回。慣性捲動與沒有手勢階段的
// 輸入（一般滑鼠滾輪）都落在按下之外，不推進。
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ArrowLeft } from "lucide-react";
import { cn } from "@speclink/ui";

/** 觸發返回所需的累計水平位移（px）。 */
export const SWIPE_THRESHOLD = 120;
/** 一次手勢累計垂直移動超過此值且多於累計水平移動時，整次手勢取消（px）。 */
const VERTICAL_CANCEL = 10;

/** 原生層的手指按下／離開（Rust speclink-desktop-core 的 TrackpadGestureEvent）；
 * swipeEnabled 為系統設定「在頁面之間滑動」是否允許兩指滑動，只在 began 有意義。 */
type TrackpadGesture = { phase: "began" | "ended"; swipeEnabled: boolean };

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
    // idle：手指未按下；tracking：累計中；cancelled：垂直為主，等手指離開。
    let state: "idle" | "tracking" | "cancelled" = "idle";
    let swipeEnabled = false;
    let total = 0; // 朝返回方向的累計位移
    let sumX = 0;
    let sumAbsY = 0;
    const reset = (next: typeof state) => {
      state = next;
      total = sumX = sumAbsY = 0;
      setProgress(0);
    };
    const onGesture = ({ payload }: { payload: TrackpadGesture }) => {
      if (payload.phase === "began") {
        // MayBegin 之後的 Began 也送 began：tracking 中只更新設定，不歸零。
        swipeEnabled = payload.swipeEnabled;
        if (state !== "tracking") reset(swipeEnabled ? "tracking" : "idle");
        return;
      }
      const fire = state === "tracking" && total >= SWIPE_THRESHOLD;
      reset("idle");
      if (fire) onBackRef.current();
    };
    const onWheel = (e: WheelEvent) => {
      // 縮放（ctrl＋滾輪）不是返回手勢。
      if (state !== "tracking" || !swipeEnabled || e.ctrlKey) return;
      sumX += e.deltaX;
      sumAbsY += Math.abs(e.deltaY);
      if (sumAbsY > VERTICAL_CANCEL && sumAbsY > Math.abs(sumX)) {
        reset("cancelled");
        return;
      }
      const delta = backDelta(e);
      // 手勢起點只認朝返回方向、且不在仍可沿此方向捲的區塊內；開始後反向滑回會扣減。
      if (total === 0 && (delta <= 0 || canScrollX(e.target, e.deltaX))) return;
      total = Math.max(0, total + delta);
      setProgress(Math.min(1, total / SWIPE_THRESHOLD));
    };
    // 訂閱失敗（非 Tauri 環境）靜默：沒有階段事件就不會觸發。
    const unlisten = listen<TrackpadGesture>("trackpad-gesture", onGesture).catch(() => undefined);
    window.addEventListener("wheel", onWheel, { passive: true });
    return () => {
      window.removeEventListener("wheel", onWheel);
      void unlisten.then((f) => f?.());
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
