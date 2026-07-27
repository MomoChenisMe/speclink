import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { Button, useI18n } from "@speclink/ui";
import { readTourSeen, writeTourSeen } from "../lib/tourSeen";

// 首次導覽（server-web-console「首次進入提供可略過的分步導覽」）：疊層分步指向畫面上
// 實際存在的元素，各附一句說明其用途，提供上一步／下一步／略過。
//
// 不引入 driver.js：它自行接管 DOM 與捲動，會與 Radix 的 focus trap 和既有的
// useFocusMain 互搶焦點。導覽要的只有「高亮一個目標、講一句話、上下步／略過」。
// 疊層自身不做 focus trap，也不吃 Escape 之外的鍵——鍵盤隨時可以離開。

/** 每一步指向的目標以 data-tour 屬性標記；找不到的步驟直接跳過，不呈現空的高亮框。 */
type Step = { key: string; messageKey: string };

const STEPS: Step[] = [
  { key: "nav-overview", messageKey: "tour.navOverview" },
  { key: "nav-users", messageKey: "tour.navUsers" },
  { key: "nav-registry", messageKey: "tour.navRegistry" },
  { key: "nav-credentials", messageKey: "tour.navCredentials" },
  { key: "nav-system", messageKey: "tour.navSystem" },
  { key: "nav-audit", messageKey: "tour.navAudit" },
  { key: "list-primary", messageKey: "tour.listPrimary" },
];

/** 重新啟動導覽的入口，供系統頁的「重看導覽」使用。 */
const TourContext = createContext<{ restart: () => void } | null>(null);

export function useTour(): { restart: () => void } {
  const ctx = useContext(TourContext);
  if (ctx === null) throw new Error("useTour 必須在 TourProvider 內使用");
  return ctx;
}

function targetOf(step: Step): HTMLElement | null {
  return document.querySelector<HTMLElement>(`[data-tour="${step.key}"]`);
}

const CARD_WIDTH = 384; // w-24rem
const CARD_HEIGHT = 160; // 兩行說明＋動作列的實測高度，只用來決定放上面還是下面
const GAP = 12;

/**
 * 卡片貼著目標放：優先在右側（側欄目標都靠左），放不下就改到下方，再放不下就上方。
 * 座標一律夾回視窗內——貼著目標但半張卡在畫面外，等於沒指到。
 */
function cardPosition(rect: DOMRect | undefined): { top: number; left: number } {
  if (!rect) return { top: GAP, left: GAP };
  const { innerWidth: vw, innerHeight: vh } = window;
  const clamp = (value: number, max: number) => Math.max(GAP, Math.min(value, max - GAP));

  const right = rect.right + GAP;
  if (right + CARD_WIDTH + GAP <= vw) {
    return { top: clamp(rect.top, vh - CARD_HEIGHT), left: right };
  }
  const below = rect.bottom + GAP;
  const top = below + CARD_HEIGHT + GAP <= vh ? below : rect.top - CARD_HEIGHT - GAP;
  return { top: clamp(top, vh - CARD_HEIGHT), left: clamp(rect.left, vw - CARD_WIDTH) };
}

/**
 * 導覽的宿主。`enabled` 為 false（非管理面、非管理員）時完全不啟動。
 * 另有 Sheet 或 AlertDialog 開啟時也不自動啟動——兩層疊在一起只會互相遮蔽。
 */
export function TourProvider({ enabled, children }: { enabled: boolean; children: ReactNode }) {
  const [running, setRunning] = useState(false);
  const [index, setIndex] = useState(0);

  const restart = useCallback(() => {
    setIndex(0);
    setRunning(true);
  }, []);

  useEffect(() => {
    if (!enabled || readTourSeen()) return;
    if (document.querySelector("[role='dialog'],[role='alertdialog']")) return;
    setIndex(0);
    setRunning(true);
  }, [enabled]);

  const value = useMemo(() => ({ restart }), [restart]);

  return (
    <TourContext.Provider value={value}>
      {children}
      {running && (
        <TourOverlay
          index={index}
          onIndex={setIndex}
          onLeave={() => {
            setRunning(false);
            writeTourSeen(true);
          }}
        />
      )}
    </TourContext.Provider>
  );
}

function TourOverlay({
  index,
  onIndex,
  onLeave,
}: {
  index: number;
  onIndex: (index: number) => void;
  onLeave: () => void;
}) {
  const { t } = useI18n();
  // 只走目標存在的步驟。當前版面下看不到的元素（例如總覽頁沒有列表 primary action）
  // 直接不列入，而不是走到那步才發現沒東西可指。
  const steps = useMemo(() => STEPS.filter((s) => targetOf(s) !== null), []);
  const step = steps[Math.min(index, steps.length - 1)];

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onLeave();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onLeave]);

  // 高亮框貼著目標畫，不覆蓋它——疊層擋住它正在指的元素就失去意義了。
  const rect = step ? targetOf(step)?.getBoundingClientRect() : undefined;

  if (!step) return null;
  const last = index >= steps.length - 1;
  const card = cardPosition(rect);

  return (
    <>
      {rect && (
        <div
          aria-hidden="true"
          className="pointer-events-none fixed z-40 rounded-md ring-2 ring-primary ring-offset-2 ring-offset-background"
          style={{ top: rect.top, left: rect.left, width: rect.width, height: rect.height }}
        />
      )}
      <section
        role="region"
        aria-label={t("tour.label")}
        data-tour-step={step.key}
        style={{ top: card.top, left: card.left }}
        className="fixed z-50 w-[min(24rem,calc(100vw-2rem))] rounded-md border border-border bg-card p-4 shadow-lg"
      >
        <p className="font-medium">{t(`${step.messageKey}.title`)}</p>
        <p className="mt-1 text-sm text-muted-foreground">{t(`${step.messageKey}.hint`)}</p>
        <div className="mt-3 flex items-center justify-between gap-3">
          <span className="text-xs tabular-nums text-muted-foreground">
            {index + 1} / {steps.length}
          </span>
          <div className="flex items-center gap-2">
            <Button type="button" variant="ghost" size="sm" onClick={onLeave}>
              {t("tour.skip")}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={index === 0}
              onClick={() => onIndex(index - 1)}
            >
              {t("tour.previous")}
            </Button>
            <Button
              type="button"
              size="sm"
              onClick={() => (last ? onLeave() : onIndex(index + 1))}
            >
              {last ? t("tour.done") : t("tour.next")}
            </Button>
          </div>
        </div>
      </section>
    </>
  );
}
