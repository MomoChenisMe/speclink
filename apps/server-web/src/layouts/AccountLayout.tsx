import { useRef, type ReactNode } from "react";
import { NavLink, useLocation } from "react-router-dom";
import { SkipLink } from "../components/SkipLink";
import { LogoutButton } from "../components/LogoutButton";
import { useFocusMain } from "../lib/useFocusMain";

// 一般成員的帳號殼：帳號入口與登出空間上分離，不顯示任何管理導覽。
export function AccountLayout({ children }: { children: ReactNode }) {
  const mainRef = useRef<HTMLElement>(null);
  const location = useLocation();
  useFocusMain(mainRef, location.pathname);

  return (
    <div className="min-h-screen bg-background text-foreground">
      <SkipLink />
      <header className="flex items-center justify-between border-b px-4 py-3">
        <span className="text-lg font-semibold text-primary">Speclink</span>
        <div className="flex items-center gap-3">
          <NavLink
            to="/account"
            className="rounded-md px-2 py-1 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            帳號
          </NavLink>
          <LogoutButton />
        </div>
      </header>
      <main id="main-content" ref={mainRef} className="mx-auto max-w-3xl px-4 py-6">
        {children}
      </main>
    </div>
  );
}
