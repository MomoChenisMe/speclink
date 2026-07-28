import { useState, type FormEvent } from "react";
import { Navigate, useSearchParams } from "react-router-dom";
import { Button, useI18n } from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { Field } from "../components/Field";
import { WebApiError } from "../api/client";

// 裝置核准頁（server-device-auth「核准頁 session 保護且明確確認」, D3）。需已登入；未
// 登入時導向登入頁並僅保留格式合格的裝置碼（成功登入後由 Server 依 device code 裁決導回
// 此頁）。載入不查授權狀態，只預填 URL 帶入的碼；使用者提交後才得到明確的核准／拒絕。

/** The confusable-free `XXXX-XXXX` device-code shape (matches the server). */
const CODE = /^[A-HJ-KM-NP-Z2-9]{4}-[A-HJ-KM-NP-Z2-9]{4}$/;

export function ActivatePage() {
  const { t } = useI18n();
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
    setMessage(error instanceof WebApiError ? error.message : t("activate.invalidCode"));
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
      setResult(status === "approved" ? t("activate.approved") : t("activate.denied"));
      setPhase("done");
    } catch (error) {
      fail(error);
    } finally {
      setPending(false);
    }
  }

  // 結果頁不止於單行結果：核准與拒絕都接一句返回 app 的收尾指引，讓使用者知道這裡
  // 已經結束、app 會自行取得結果（server-device-auth「核准頁 session 保護且明確確認」）。
  if (phase === "done") {
    return (
      <div>
        <h1 className="mb-2 text-2xl font-semibold">{t("activate.title")}</h1>
        <p role="status" aria-live="polite" className="text-muted-foreground">
          {result}
        </p>
        <p className="mt-2 text-muted-foreground">{t("activate.returnHint")}</p>
      </div>
    );
  }

  return (
    <div>
      <h1 className="mb-2 text-2xl font-semibold">{t("activate.title")}</h1>
      {phase === "enter" ? (
        <form onSubmit={onCheck} className="space-y-4" noValidate>
          <p className="text-muted-foreground">{t("activate.prompt")}</p>
          <Field id="user-code" label={t("activate.codeLabel")} value={code} onChange={setCode} />
          {message && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}
          <Button type="submit" disabled={pending}>
            {pending ? t("activate.checking") : t("common.next")}
          </Button>
        </form>
      ) : (
        <div className="space-y-4">
          <p className="text-muted-foreground">{t("activate.confirmBody").replace("{code}", code)}</p>
          {message && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}
          <div className="flex gap-3">
            <Button type="button" onClick={() => onDecide("approve")} disabled={pending}>
              {t("activate.approve")}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => onDecide("deny")}
              disabled={pending}
            >
              {t("activate.deny")}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
