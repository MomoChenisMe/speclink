import { useState, type FormEvent } from "react";
import { Navigate, useSearchParams } from "react-router-dom";
import { Button } from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { Field } from "../components/Field";
import { WebApiError } from "../api/client";

// 裝置核准頁（server-device-auth「核准頁 session 保護且明確確認」, D3）。需已登入；未
// 登入時導向登入頁並僅保留格式合格的裝置碼（成功登入後由 Server 依 device code 裁決導回
// 此頁）。載入不查授權狀態，只預填 URL 帶入的碼；使用者提交後才得到明確的核准／拒絕。

/** The confusable-free `XXXX-XXXX` device-code shape (matches the server). */
const CODE = /^[A-HJ-KM-NP-Z2-9]{4}-[A-HJ-KM-NP-Z2-9]{4}$/;

export function ActivatePage() {
  const client = useClient();
  const { session } = useSession();
  const [params] = useSearchParams();
  const prefill = params.get("user_code") ?? "";
  const [code, setCode] = useState(prefill);
  const [phase, setPhase] = useState<"enter" | "confirm" | "done">("enter");
  const [result, setResult] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  // 未登入：導向登入頁，只保留格式合格的裝置碼。
  if (!session.authenticated) {
    const to = CODE.test(prefill) ? `/login?user_code=${encodeURIComponent(prefill)}` : "/login";
    return <Navigate to={to} replace />;
  }

  function fail(error: unknown) {
    setMessage(error instanceof WebApiError ? error.message : "這個裝置代碼無法使用。");
  }

  async function onCheck(event: FormEvent) {
    event.preventDefault();
    if (pending) return;
    setPending(true);
    setMessage(null);
    try {
      await client.checkActivation(code.trim());
      setPhase("confirm");
    } catch (error) {
      fail(error);
    } finally {
      setPending(false);
    }
  }

  async function onDecide(action: "approve" | "deny") {
    if (pending) return;
    setPending(true);
    setMessage(null);
    try {
      const { status } = await client.decideActivation(code.trim(), action);
      setResult(
        status === "approved"
          ? "已核准。你可以回到裝置繼續登入。"
          : "已拒絕這個裝置的登入請求。",
      );
      setPhase("done");
    } catch (error) {
      fail(error);
    } finally {
      setPending(false);
    }
  }

  if (phase === "done") {
    return (
      <div>
        <h1 className="mb-2 text-2xl font-semibold">裝置登入</h1>
        <p role="status" aria-live="polite" className="text-muted-foreground">
          {result}
        </p>
      </div>
    );
  }

  return (
    <div>
      <h1 className="mb-2 text-2xl font-semibold">裝置登入</h1>
      {phase === "enter" ? (
        <form onSubmit={onCheck} className="space-y-4" noValidate>
          <p className="text-muted-foreground">輸入裝置上顯示的代碼以核准登入。</p>
          <Field id="user-code" label="裝置代碼" value={code} onChange={setCode} />
          {message && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}
          <Button type="submit" disabled={pending}>
            {pending ? "檢查中…" : "下一步"}
          </Button>
        </form>
      ) : (
        <div className="space-y-4">
          <p className="text-muted-foreground">
            代碼 <code className="font-mono">{code}</code> 的裝置要求以你的身分登入。
          </p>
          {message && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}
          <div className="flex gap-3">
            <Button type="button" onClick={() => onDecide("approve")} disabled={pending}>
              核准
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => onDecide("deny")}
              disabled={pending}
            >
              拒絕
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
