import { useState, type FormEvent } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Button } from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { useAsync } from "../lib/useAsync";
import { readFormError } from "../lib/formError";
import { Field } from "../components/Field";

// 邀請接受（server-identity「邀請一次性且到期失效」, D2／D3）。token 由 URL path 帶入；
// 有效邀請顯示設定密碼表單，提交後 Server 原子建帳號並建立 session，SPA 依回傳的
// destination 導向（admin→/admin，一般→/account）。已用／過期／未知 token 一律顯示
// 不可區分的「邀請無效」且無表單。
export function InvitePage() {
  const client = useClient();
  const { refresh } = useSession();
  const navigate = useNavigate();
  const { token = "" } = useParams();
  const { loading, data } = useAsync(() => client.getInvitation(token), [token]);
  const [password, setPassword] = useState("");
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (pending) return;
    setPending(true);
    setMessage(null);
    setFieldErrors({});
    try {
      const { destination } = await client.acceptInvitation(token, { password });
      await refresh();
      navigate(destination);
    } catch (error) {
      const read = readFormError(error, "接受邀請時發生錯誤");
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
      setPending(false);
    }
  }

  if (loading) {
    return (
      <p role="status" aria-live="polite" className="text-muted-foreground">
        載入中…
      </p>
    );
  }
  if (!data) {
    // 已用／過期／未知 token：不可區分的固定訊息，不回顯內部原因。
    return (
      <div role="alert">
        <h1 className="mb-2 text-2xl font-semibold">邀請無效</h1>
        <p className="text-muted-foreground">
          這個邀請連結無法使用。請向邀請你的人索取新的連結。
        </p>
      </div>
    );
  }

  return (
    <div>
      <h1 className="mb-2 text-2xl font-semibold">接受邀請</h1>
      <p className="mb-6 text-muted-foreground">為 {data.email} 設定登入密碼。</p>
      <form onSubmit={onSubmit} className="space-y-4" noValidate>
        <Field
          id="password"
          label="密碼"
          type="password"
          autoComplete="new-password"
          value={password}
          onChange={setPassword}
          error={fieldErrors.password}
        />
        {message && Object.keys(fieldErrors).length === 0 && (
          <p role="alert" className="text-sm text-destructive">
            {message}
          </p>
        )}
        <Button type="submit" disabled={pending}>
          {pending ? "建立中…" : "建立帳號"}
        </Button>
      </form>
    </div>
  );
}
