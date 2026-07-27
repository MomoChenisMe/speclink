import { NavLink } from "react-router-dom";
import {
  Activity,
  FolderGit2,
  KeyRound,
  LayoutDashboard,
  ScrollText,
  Users,
  type LucideIcon,
} from "lucide-react";
import { cn, useI18n } from "@speclink/ui";

// 管理導覽的六個固定目的地，分為日常與維運兩組。帳號不是側欄目的地——入口在 header
// 的電子郵件連結（HeaderAccount）。圖示與選中態（實心主色）對齊 Desktop 側欄語彙。
// tour 是首次導覽的目標標記（components/Tour.tsx）；每個目的地一步。
type Destination = { to: string; labelKey: string; icon: LucideIcon; tour: string; end?: boolean };

const PRIMARY: Destination[] = [
  { to: "/admin", labelKey: "nav.overview", icon: LayoutDashboard, tour: "nav-overview", end: true },
  { to: "/admin/users", labelKey: "nav.users", icon: Users, tour: "nav-users" },
  { to: "/admin/registry", labelKey: "nav.registry", icon: FolderGit2, tour: "nav-registry" },
];

const OPERATIONS: Destination[] = [
  { to: "/admin/credentials", labelKey: "nav.credentials", icon: KeyRound, tour: "nav-credentials" },
  { to: "/admin/system", labelKey: "nav.system", icon: Activity, tour: "nav-system" },
  { to: "/admin/audit", labelKey: "nav.audit", icon: ScrollText, tour: "nav-audit" },
];

function itemClass(isActive: boolean): string {
  return cn(
    "flex min-h-11 items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
    isActive
      ? "bg-primary font-medium text-primary-foreground"
      : "text-muted-foreground hover:bg-muted hover:text-foreground",
  );
}

function Group({ items, onNavigate }: { items: Destination[]; onNavigate?: () => void }) {
  const { t } = useI18n();
  return (
    <ul className="space-y-1">
      {items.map(({ to, labelKey, icon: Icon, tour, end }) => (
        <li key={to}>
          <NavLink
            to={to}
            end={end}
            data-tour={tour}
            onClick={onNavigate}
            className={({ isActive }) => itemClass(isActive)}
          >
            <Icon aria-hidden="true" className="h-4 w-4 shrink-0" />
            {t(labelKey)}
          </NavLink>
        </li>
      ))}
    </ul>
  );
}

export function AdminNav({ onNavigate }: { onNavigate?: () => void }) {
  const { t } = useI18n();
  return (
    <>
      <Group items={PRIMARY} onNavigate={onNavigate} />
      {/* 分組標籤而非可點目的地：維運三項與日常三項視覺分離（proposal 版面藍圖）。 */}
      <p className="mt-3 border-t border-border px-3 pb-1 pt-3 text-xs font-medium tracking-wide text-muted-foreground">
        {t("shell.operations")}
      </p>
      <Group items={OPERATIONS} onNavigate={onNavigate} />
    </>
  );
}
