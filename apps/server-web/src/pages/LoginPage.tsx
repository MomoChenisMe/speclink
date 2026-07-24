import { useState, type FormEvent } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Button, Input, Label } from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { WebApiError } from "../api/client";

// 本機密碼登入。提交期間停用避免重複；失敗保留輸入、把錯誤放欄位旁並以 role=alert
// 宣告；成功依 Server 回傳的 destination 導向（D3）。
export function LoginPage() {
  const client = useClient();
  const { refresh } = useSession();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const returnTo = params.get("returnTo") ?? undefined;
  const userCode = params.get("user_code") ?? undefined;

  const [email, setEmail] = useState("");
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
      const { destination } = await client.login({ email, password, userCode, returnTo });
      await refresh();
      navigate(destination);
    } catch (error) {
      if (error instanceof WebApiError) {
        setMessage(error.message);
        setFieldErrors(error.fieldErrors ?? {});
      } else {
        setMessage("登入時發生錯誤");
      }
      setPending(false);
    }
  }

  const hasFieldErrors = Object.keys(fieldErrors).length > 0;

  return (
    <div>
      <h1 className="mb-6 text-2xl font-semibold">登入</h1>
      <form onSubmit={onSubmit} className="space-y-4" noValidate>
        <div className="space-y-1.5">
          <Label htmlFor="email">電子郵件</Label>
          <Input
            id="email"
            type="email"
            autoComplete="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            aria-invalid={Boolean(fieldErrors.email)}
            aria-describedby={fieldErrors.email ? "email-error" : undefined}
          />
          {fieldErrors.email && (
            <p id="email-error" role="alert" className="text-sm text-destructive">
              {fieldErrors.email}
            </p>
          )}
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="password">密碼</Label>
          <Input
            id="password"
            type="password"
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            aria-invalid={Boolean(fieldErrors.password)}
            aria-describedby={fieldErrors.password ? "password-error" : undefined}
          />
          {fieldErrors.password && (
            <p id="password-error" role="alert" className="text-sm text-destructive">
              {fieldErrors.password}
            </p>
          )}
        </div>
        {message && !hasFieldErrors && (
          <p role="alert" className="text-sm text-destructive">
            {message}
          </p>
        )}
        <Button type="submit" disabled={pending}>
          {pending ? "登入中…" : "登入"}
        </Button>
      </form>
    </div>
  );
}
