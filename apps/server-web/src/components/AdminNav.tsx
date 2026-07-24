import { NavLink } from "react-router-dom";
import { cn } from "@speclink/ui";

// 管理導覽的七個固定目的地，加上帳號入口（D6）。側欄與窄螢幕 Sheet 共用。
const DESTINATIONS = [
  { to: "/admin", label: "總覽", end: true },
  { to: "/admin/users", label: "使用者" },
  { to: "/admin/registry", label: "專案與儲存庫" },
  { to: "/admin/credentials", label: "憑證" },
  { to: "/admin/data", label: "資料操作" },
  { to: "/admin/system", label: "系統狀態" },
  { to: "/admin/audit", label: "稽核紀錄" },
];

function itemClass(isActive: boolean): string {
  return cn(
    "flex min-h-11 items-center rounded-md px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
    isActive ? "bg-accent font-medium text-accent-foreground" : "hover:bg-muted",
  );
}

export function AdminNav({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <ul className="space-y-1">
      {DESTINATIONS.map((item) => (
        <li key={item.to}>
          <NavLink
            to={item.to}
            end={item.end}
            onClick={onNavigate}
            className={({ isActive }) => itemClass(isActive)}
          >
            {item.label}
          </NavLink>
        </li>
      ))}
      <li className="pt-2">
        <NavLink
          to="/account"
          onClick={onNavigate}
          className={({ isActive }) => itemClass(isActive)}
        >
          帳號
        </NavLink>
      </li>
    </ul>
  );
}
