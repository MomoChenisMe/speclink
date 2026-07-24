import { useRef, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { SkipLink } from "../components/SkipLink";
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
        <span className="text-lg font-semibold text-primary">Speclink</span>
      </header>
      <main id="main-content" ref={mainRef} className="mx-auto max-w-md px-4 pb-12">
        {children}
      </main>
    </div>
  );
}
