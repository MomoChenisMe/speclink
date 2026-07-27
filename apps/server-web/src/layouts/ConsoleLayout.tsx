import { useRef, useState, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import {
  Button,
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
  useI18n,
} from "@speclink/ui";
import { SkipLink } from "../components/SkipLink";
import { AdminNav } from "../components/AdminNav";
import { TourProvider } from "../components/Tour";
import { HeaderAccount } from "../components/HeaderAccount";
import { Wordmark } from "../components/Wordmark";
import { useSession } from "../app/context";
import { useFocusMain } from "../lib/useFocusMain";
import { useMediaQuery } from "../lib/useMediaQuery";

// 主控台殼：管理員與一般成員共用，依角色裁切側欄。header 恆常呈現（品牌、電子郵件
// 連結與登出）；側欄只在 session 帶 admin 旗標時渲染——可見性依伺服器回傳的角色真相，
// 前端不自行推導。管理員在 /account 時側欄整條保留但無項目高亮（帳號已不是側欄目的地）。
//
// ≥1024px 顯示 icon＋label 的固定側欄；更窄以有可見 trigger 的 Sheet 提供相同六個目的地，
// 關閉後 focus 回到 trigger（Radix 預設處理）。header 高度、側欄寬度與主內容內距採用
// Desktop 應用程式殼的數值（h-12／w-[200px]／p-5）。
export function ConsoleLayout({ children }: { children: ReactNode }) {
  const { t } = useI18n();
  const mainRef = useRef<HTMLElement>(null);
  const location = useLocation();
  const { session } = useSession();
  useFocusMain(mainRef, location.pathname);
  const narrow = useMediaQuery("(max-width: 1023px)");
  const [sheetOpen, setSheetOpen] = useState(false);
  const admin = session.user?.admin === true;

  return (
    // 滿高欄式版面：殼本身固定滿版不捲動，只有主內容區捲——與 Desktop 殼同結構。
    // 若讓整頁捲，header 與側欄會跟著滾出畫面，導覽就不再是恆常可見的。
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <SkipLink />
      <header className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border px-4">
        <div className="flex min-w-0 items-center gap-3">
          {admin && narrow && (
            <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
              <SheetTrigger asChild>
                <Button variant="outline" size="sm" aria-label={t("shell.openNav")}>
                  {t("shell.menu")}
                </Button>
              </SheetTrigger>
              <SheetContent className="w-72">
                <SheetHeader>
                  <SheetTitle>{t("shell.adminNav")}</SheetTitle>
                </SheetHeader>
                <nav aria-label={t("shell.adminNav")} className="mt-4">
                  <AdminNav onNavigate={() => setSheetOpen(false)} />
                </nav>
              </SheetContent>
            </Sheet>
          )}
          <Wordmark />
        </div>
        <HeaderAccount />
      </header>
      <div className="flex flex-1 overflow-hidden">
        {admin && !narrow && (
          <nav
            aria-label={t("shell.adminNav")}
            className="w-[200px] shrink-0 overflow-y-auto border-r border-border bg-card p-2"
          >
            <AdminNav />
          </nav>
        )}
        <main id="main-content" ref={mainRef} className="min-w-0 flex-1 overflow-y-auto p-5">
          {/* 導覽只在管理面啟動：帳號頁沒有側欄目的地可指，一般成員更看不到管理面。 */}
          <TourProvider enabled={admin && location.pathname.startsWith("/admin")}>
            {children}
          </TourProvider>
        </main>
      </div>
    </div>
  );
}
