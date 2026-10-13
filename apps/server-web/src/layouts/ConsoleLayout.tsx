import { useRef, useState, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { Menu } from "lucide-react";
import {
  Breadcrumbs,
  Button,
  Sheet,
  SheetContent,
  SheetTitle,
  SheetTrigger,
  useI18n,
  Wordmark,
  type Crumb,
} from "@speclink/ui";
import { SkipLink } from "../components/SkipLink";
import { ADMIN_DESTINATIONS, AdminNav, MemberNav } from "../components/AdminNav";
import { AccountMenu } from "../components/AccountMenu";
import { TourProvider } from "../components/Tour";
import { useSession } from "../app/context";
import { useFocusMain } from "../lib/useFocusMain";
import { useMediaQuery } from "../lib/useMediaQuery";

// 主控台殼（design D1）：管理員與一般成員同一種形狀——側欄（頂端字標、中段導覽、底部
// 帳號選單）加主欄（頂列麵包屑、下方主內容），沒有橫貫頂部的 header。導覽依 session 的
// admin 旗標裁切（伺服器回傳的角色真相，前端不自行推導）：管理員六個目的地，一般成員只有
// 帳號一項。管理員在 /account 時側欄整條保留但無項目高亮。
//
// ≥1024px 顯示固定側欄；更窄時改由頂列的「開啟導覽」鈕開出從左滑出、內容相同的 Sheet，
// 關閉後 focus 回到觸發鈕（Radix 預設處理）。頂列高度、側欄寬度與主內容內距採用 Desktop
// 應用程式殼的數值（h-12／w-[200px]／p-5）。
export function ConsoleLayout({ children }: { children: ReactNode }) {
  const { t } = useI18n();
  const mainRef = useRef<HTMLElement>(null);
  const location = useLocation();
  const { session } = useSession();
  useFocusMain(mainRef, location.pathname);
  const narrow = useMediaQuery("(max-width: 1023px)");
  const [sheetOpen, setSheetOpen] = useState(false);
  const admin = session.user?.admin === true;
  const crumbs = useCrumbs(location.pathname);

  return (
    // 滿高欄式版面：殼本身固定滿版不捲動，只有主內容區捲——與 Desktop 殼同結構。
    // 若讓整頁捲，側欄與頂列會跟著滾出畫面，導覽就不再是恆常可見的。
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <SkipLink />
      {!narrow && (
        <aside className="flex w-[200px] shrink-0 flex-col border-r border-border bg-sidebar">
          <SidebarBody admin={admin} />
        </aside>
      )}
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex h-12 shrink-0 items-center gap-2 border-b border-border px-5">
          {narrow && (
            <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
              <SheetTrigger asChild>
                <Button variant="ghost" size="icon" className="-ml-2 shrink-0" aria-label={t("shell.openNav")}>
                  <Menu aria-hidden="true" />
                </Button>
              </SheetTrigger>
              <SheetContent side="left" aria-describedby={undefined} className="w-[280px] gap-0 p-0">
                <SheetTitle className="sr-only">{t("shell.nav")}</SheetTitle>
                <SidebarBody admin={admin} onNavigate={() => setSheetOpen(false)} />
              </SheetContent>
            </Sheet>
          )}
          <Breadcrumbs crumbs={crumbs} />
        </div>
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

/** 側欄與窄螢幕 Sheet 共用的內容：字標、該角色的導覽、帳號選單。 */
function SidebarBody({ admin, onNavigate }: { admin: boolean; onNavigate?: () => void }) {
  const { t } = useI18n();
  return (
    <>
      <div className="flex h-12 shrink-0 items-center border-b border-border px-4">
        <Wordmark className="h-5" />
      </div>
      <nav aria-label={t(admin ? "shell.adminNav" : "shell.nav")} className="min-h-0 flex-1 overflow-y-auto p-2">
        {admin ? <AdminNav onNavigate={onNavigate} /> : <MemberNav onNavigate={onNavigate} />}
      </nav>
      <div className="shrink-0 border-t border-border p-2">
        <AccountMenu onNavigate={onNavigate} />
      </div>
    </>
  );
}

/** 麵包屑（design D4）：管理面為「後台 / <目的地>」且「後台」可點回總覽，帳號頁為「帳號」；
 *  未列在目的地表內的 /admin/* 路徑只顯示「後台」。 */
function useCrumbs(pathname: string): Crumb[] {
  const { t } = useI18n();
  const navigate = useNavigate();
  if (!pathname.startsWith("/admin")) return [{ label: t("account.title") }];
  const home: Crumb = { label: t("shell.console"), onClick: () => navigate("/admin") };
  const path = pathname.replace(/\/$/, "");
  const destination = ADMIN_DESTINATIONS.find((d) => d.to === path);
  return destination ? [home, { label: t(destination.labelKey) }] : [home];
}
