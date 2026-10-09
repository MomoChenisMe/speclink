import { NavLink, useMatch } from "react-router-dom";
import {
  Activity,
  FolderGit2,
  KeyRound,
  LayoutDashboard,
  ScrollText,
  Users,
  type LucideIcon,
} from "lucide-react";
import { NavItem, useI18n } from "@speclink/ui";

// 管理導覽的六個固定目的地，分為日常與維運兩組。帳號不是側欄目的地——入口在 header
// 的電子郵件連結（HeaderAccount）。導覽項是共用 NavItem（design D4），與 Desktop 專案欄同一份。
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

/** 作用中與否以 NavLink 同一套比對（含 end）決定，交給 NavItem 套樣式。 */
function Item({
  to,
  labelKey,
  icon: Icon,
  tour,
  end,
  onNavigate,
}: Destination & { onNavigate?: () => void }) {
  const { t } = useI18n();
  const active = useMatch({ path: to, end: end ?? false }) !== null;
  return (
    <NavItem asChild active={active} icon={<Icon aria-hidden="true" />} label={t(labelKey)}>
      <NavLink to={to} end={end} data-tour={tour} onClick={onNavigate} />
    </NavItem>
  );
}

function Group({ items, onNavigate }: { items: Destination[]; onNavigate?: () => void }) {
  return (
    <ul className="space-y-1">
      {items.map((item) => (
        <li key={item.to}>
          <Item {...item} onNavigate={onNavigate} />
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
