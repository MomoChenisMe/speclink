import { useRef, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { Card } from "@speclink/ui";
import { SkipLink } from "../components/SkipLink";
import { Wordmark } from "../components/Wordmark";
import { useFocusMain } from "../lib/useFocusMain";

// 專注流程殼（登入／setup／邀請／啟用）：只顯示 Speclink identity、步驟與主表單。
export function FocusLayout({ children }: { children: ReactNode }) {
  const mainRef = useRef<HTMLElement>(null);
  const location = useLocation();
  useFocusMain(mainRef, location.pathname);

  return (
    <div className="min-h-screen bg-background text-foreground">
      <SkipLink />
      <header className="px-4 py-4">
        <Wordmark />
      </header>
      <main id="main-content" ref={mainRef} className="mx-auto max-w-md px-4 pb-12 pt-6">
        {/* 專注流程卡片：表單置於卡面上（與 Desktop 對話框的卡片語彙一致）。 */}
        <Card className="p-6">{children}</Card>
      </main>
    </div>
  );
}
