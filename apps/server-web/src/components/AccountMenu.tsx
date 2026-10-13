import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ChevronsUpDown, LogOut, UserRound } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  useI18n,
  type LocalePreference,
} from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { useLocalePreference } from "../i18n/LocaleContext";

// 側欄底部的帳號選單（design D2）：觸發鈕顯示自己是誰，選單收帳號、介面語言與登出。
// 語言名稱維持母語寫法——把「English」翻成「英文」只會讓看不懂當前語言的人找不到自己的語言。

/** 跟隨系統在 LocalePreference 裡是 null，但 Radix 單選項的 value 不接受空字串。 */
const SYSTEM = "__system__";

export function AccountMenu({ onNavigate }: { onNavigate?: () => void }) {
  const { t } = useI18n();
  const client = useClient();
  const { session, refresh } = useSession();
  const { pref, setPref } = useLocalePreference();
  const navigate = useNavigate();
  const [pending, setPending] = useState(false);
  const user = session.user;
  if (!user) return null;
  // 顯示名稱為空時退回 email：頭像字與名稱列都不留白。
  const name = user.display || user.email;

  // 登出：撤銷 server session、reload 角色真相、導向 server 回傳的去處；失敗也回登入頁，
  // 不留在仍顯示帳號資訊的殼。
  async function logout() {
    setPending(true);
    try {
      const { destination } = await client.logout();
      await refresh();
      navigate(destination);
    } catch {
      navigate("/login");
    } finally {
      setPending(false);
    }
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        aria-label={t("shell.accountMenu").replace("{name}", name)}
        className="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left transition-colors hover:bg-foreground/5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <span
          aria-hidden="true"
          className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/12 text-xs font-medium text-primary"
        >
          {Array.from(name)[0]}
        </span>
        <span className="min-w-0 flex-1">
          <span className="block truncate text-[13px]">{name}</span>
          <span className="block truncate text-[11px] text-muted-foreground">{user.email}</span>
        </span>
        <ChevronsUpDown aria-hidden="true" className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      </DropdownMenuTrigger>
      <DropdownMenuContent side="top" align="start">
        <DropdownMenuLabel>
          <span className="block truncate text-[13px] text-foreground">{name}</span>
          <span className="block truncate">
            {`${user.email} · ${t(user.admin ? "users.roleAdmin" : "users.roleMember")}`}
          </span>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onSelect={() => {
            onNavigate?.();
            navigate("/account");
          }}
        >
          <UserRound aria-hidden="true" />
          {t("account.title")}
        </DropdownMenuItem>
        <DropdownMenuLabel>{t("locale.label")}</DropdownMenuLabel>
        <DropdownMenuRadioGroup
          value={pref ?? SYSTEM}
          onValueChange={(v) => setPref(v === SYSTEM ? null : (v as LocalePreference))}
        >
          <DropdownMenuRadioItem value="zh-TW">繁體中文</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="en">English</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value={SYSTEM}>{t("locale.followSystem")}</DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem disabled={pending} onSelect={() => void logout()}>
          <LogOut aria-hidden="true" />
          {t("shell.logout")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
