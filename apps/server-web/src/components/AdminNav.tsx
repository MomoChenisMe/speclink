import { NavLink } from "react-router-dom";
import {
  Activity,
  CircleUser,
  Database,
  FolderGit2,
  KeyRound,
  LayoutDashboard,
  ScrollText,
  Users,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@speclink/ui";

// 管理導覽的七個固定目的地，加上帳號入口（D6）。側欄與窄螢幕 Sheet 共用。
// 圖示與選中態（實心主色）對齊 Desktop 側欄語彙。
const DESTINATIONS: { to: string; label: string; icon: LucideIcon; end?: boolean }[] = [
  { to: "/admin", label: "總覽", icon: LayoutDashboard, end: true },
  { to: "/admin/users", label: "使用者", icon: Users },
  { to: "/admin/registry", label: "專案與儲存庫", icon: FolderGit2 },
  { to: "/admin/credentials", label: "憑證", icon: KeyRound },
  { to: "/admin/data", label: "資料操作", icon: Database },
  { to: "/admin/system", label: "系統狀態", icon: Activity },
  { to: "/admin/audit", label: "稽核紀錄", icon: ScrollText },
];

function itemClass(isActive: boolean): string {
  return cn(
    "flex min-h-11 items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
    isActive
      ? "bg-primary font-medium text-primary-foreground"
      : "text-muted-foreground hover:bg-muted hover:text-foreground",
  );
}

export function AdminNav({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <ul className="space-y-1">
      {DESTINATIONS.map(({ to, label, icon: Icon, end }) => (
        <li key={to}>
          <NavLink
            to={to}
            end={end}
            onClick={onNavigate}
            className={({ isActive }) => itemClass(isActive)}
          >
            <Icon aria-hidden="true" className="h-4 w-4 shrink-0" />
            {label}
          </NavLink>
        </li>
      ))}
      <li className="mt-2 border-t border-border pt-2">
        <NavLink
          to="/account"
          onClick={onNavigate}
          className={({ isActive }) => itemClass(isActive)}
        >
          <CircleUser aria-hidden="true" className="h-4 w-4 shrink-0" />
          帳號
        </NavLink>
      </li>
    </ul>
  );
}
