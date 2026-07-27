import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { LogOut } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";
import { useClient, useSession } from "../app/context";

// 登出：撤銷 server session、reload 角色真相、導回登入頁。請求期間停用避免重複。
export function LogoutButton() {
  const { t } = useI18n();
  const client = useClient();
  const { refresh } = useSession();
  const navigate = useNavigate();
  const [pending, setPending] = useState(false);

  async function onLogout() {
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
    <Button variant="outline" size="sm" className="gap-1.5" disabled={pending} onClick={onLogout}>
      <LogOut aria-hidden="true" className="h-4 w-4" />
      {t("shell.logout")}
    </Button>
  );
}
