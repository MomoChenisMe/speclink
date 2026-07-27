import { NavLink } from "react-router-dom";
import { cn } from "@speclink/ui";
import { LogoutButton } from "./LogoutButton";
import { LocaleSwitch } from "./LocaleSwitch";
import { useSession } from "../app/context";

// header 右上的帳號入口：當前使用者的電子郵件連結至 /account（在該頁高亮），與登出
// 並列。取代側欄的「帳號」目的地，同時解決「header 只有一顆登出、看不出自己是誰」。
// 兩個項目不值得引入 dropdown 原語（design：不新增 Radix dropdown 相依）。
export function HeaderAccount() {
  const { session } = useSession();
  const email = session.user?.email;

  return (
    <div className="flex min-w-0 items-center gap-2">
      {email && (
        <NavLink
          to="/account"
          className={({ isActive }) =>
            cn(
              "flex min-h-11 min-w-0 items-center truncate rounded-md px-2 py-1 text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              isActive
                ? "font-medium text-foreground"
                : "text-muted-foreground hover:text-foreground",
            )
          }
        >
          {email}
        </NavLink>
      )}
      <LocaleSwitch />
      <LogoutButton />
    </div>
  );
}
