import { Fragment, useRef, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { TriangleAlert } from "lucide-react";
import { Card, cn, SEMANTIC_SURFACE, SEMANTIC_TONE, StepProgress, useI18n, Wordmark } from "@speclink/ui";
import { SkipLink } from "../components/SkipLink";
import { UI_LOCALES, useLocalePreference } from "../i18n/LocaleContext";
import { useFocusMain } from "../lib/useFocusMain";

// 專注流程殼（登入／setup／邀請／啟用；design D7）：畫面正中一個大字標，下方一張與 Desktop
// 對話框同款的卡片（12px 圓角、15px 標題加灰字說明、頁尾細線上方右側為主要動作），卡片下方
// 是語言切換。卡片內的標題區、頁尾與提示卡由下方三個小元件提供，四頁共用。
export function FocusLayout({ children }: { children: ReactNode }) {
  const mainRef = useRef<HTMLElement>(null);
  const location = useLocation();
  useFocusMain(mainRef, location.pathname);

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6 text-foreground">
      <SkipLink />
      <main id="main-content" ref={mainRef} className="flex w-[400px] max-w-full flex-col items-center gap-6">
        <Wordmark className="h-10" />
        <Card size="nested" className="flex w-full flex-col gap-4 p-5">
          {children}
        </Card>
        <LocaleToggle />
      </main>
    </div>
  );
}

// 登入前的語言切換：每種語言一個鈕（跟隨系統在登入後的帳號選單裡），與帳號選單寫入同一個
// 偏好。點目前已標示的語言不寫入——跟隨系統的人點了「中文」，偏好不該被悄悄固定下來。
function LocaleToggle() {
  const { t, locale } = useI18n();
  const { setPref } = useLocalePreference();
  return (
    <div role="group" aria-label={t("locale.label")} className="flex items-center gap-1.5 text-xs text-muted-foreground">
      {UI_LOCALES.map(({ value, short }, i) => (
        <Fragment key={value}>
          {i > 0 && <span aria-hidden="true">·</span>}
          <button
            type="button"
            aria-pressed={locale === value}
            onClick={() => {
              if (locale !== value) setPref(value);
            }}
            className={cn(
              "rounded-sm transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              locale === value && "text-foreground",
            )}
          >
            {short}
          </button>
        </Fragment>
      ))}
    </div>
  );
}

/** 卡片標題區：15px 標題＋灰字說明；多步驟流程在上方加共用的步驟列（與 Desktop WorkspaceChooser 同一個元件）。 */
export function FocusHeader({
  title,
  description,
  step,
}: {
  title: string;
  description?: ReactNode;
  step?: { current: number; total: number; label: string };
}) {
  return (
    <div className="flex flex-col gap-1">
      {step && <StepProgress prefix={step.label} current={step.current} total={step.total} />}
      <h1 className="text-[15px] font-medium">{title}</h1>
      {description && <p className="text-[13px] text-muted-foreground">{description}</p>}
    </div>
  );
}

/** 卡片頁尾：上緣細線、右對齊的按鈕列。 */
export function FocusFooter({ children }: { children: ReactNode }) {
  return (
    <div data-focus-footer className="flex justify-end gap-2 border-t border-border pt-3.5">
      {children}
    </div>
  );
}

/** 連結無效與無權限的提示卡：警示色圓形圖示、標題與說明，整塊以 role=alert 宣告。 */
export function FocusNotice({ title, body }: { title: string; body: string }) {
  return (
    <div role="alert" className="flex gap-3">
      <span
        className={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-full border", SEMANTIC_SURFACE.warning)}
      >
        <TriangleAlert aria-hidden="true" className={cn("h-4 w-4", SEMANTIC_TONE.warning)} />
      </span>
      <div className="min-w-0">
        <FocusHeader title={title} description={body} />
      </div>
    </div>
  );
}
