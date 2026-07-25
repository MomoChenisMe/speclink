import { useRef, useState, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import {
  Button,
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@speclink/ui";
import { SkipLink } from "../components/SkipLink";
import { AdminNav } from "../components/AdminNav";
import { LogoutButton } from "../components/LogoutButton";
import { Wordmark } from "../components/Wordmark";
import { useFocusMain } from "../lib/useFocusMain";
import { useMediaQuery } from "../lib/useMediaQuery";

// 管理殼：≥1024px 顯示 icon＋label 的固定側欄；更窄以有可見 trigger 的 Sheet 提供
// 相同七個目的地，關閉後 focus 回到 trigger（Radix 預設處理）。
export function AdminLayout({ children }: { children: ReactNode }) {
  const mainRef = useRef<HTMLElement>(null);
  const location = useLocation();
  useFocusMain(mainRef, location.pathname);
  const narrow = useMediaQuery("(max-width: 1023px)");
  const [sheetOpen, setSheetOpen] = useState(false);

  return (
    // 滿高欄式版面：側欄（bg-card＋border-r）隨視窗延伸到底，內容短時不留斷裂空白。
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      <SkipLink />
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-border px-4">
        <div className="flex items-center gap-3">
          {narrow && (
            <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
              <SheetTrigger asChild>
                <Button variant="outline" size="sm" aria-label="開啟導覽">
                  選單
                </Button>
              </SheetTrigger>
              <SheetContent side="left" className="w-72">
                <SheetHeader>
                  <SheetTitle>管理導覽</SheetTitle>
                </SheetHeader>
                <nav aria-label="管理導覽" className="mt-4">
                  <AdminNav onNavigate={() => setSheetOpen(false)} />
                </nav>
              </SheetContent>
            </Sheet>
          )}
          <Wordmark />
        </div>
        <LogoutButton />
      </header>
      <div className="flex flex-1">
        {!narrow && (
          <nav
            aria-label="管理導覽"
            className="w-56 shrink-0 border-r border-border bg-card p-3"
          >
            <AdminNav />
          </nav>
        )}
        <main id="main-content" ref={mainRef} className="min-w-0 flex-1 px-4 py-6">
          {children}
        </main>
      </div>
    </div>
  );
}
