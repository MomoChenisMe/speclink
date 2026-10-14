import { useState, type FormEvent } from "react";
import { Navigate, useSearchParams } from "react-router-dom";
import { Button, useI18n } from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { Field } from "../components/Field";
import { FocusFooter, FocusHeader } from "../layouts/FocusLayout";
import { WebApiError } from "../api/client";
import { fillUser } from "../lib/userLabel";

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
      <>
        <FocusHeader title={t("activate.title")} />
        <div className="space-y-2 text-[13px] text-muted-foreground">
          <p role="status" aria-live="polite">
            {result}
          </p>
          <p>{t("activate.returnHint")}</p>
        </div>
      </>
    );
  }

  if (phase === "enter") {
    return (
      <form onSubmit={onCheck} className="flex flex-col gap-4" noValidate>
        <FocusHeader title={t("activate.title")} description={t("activate.prompt")} />
        <Field
          id="user-code"
          label={t("activate.codeLabel")}
          value={code}
          onChange={setCode}
          error={message ?? undefined}
        />
        <FocusFooter>
          <Button type="submit" disabled={pending}>
            {pending ? t("activate.checking") : t("common.next")}
          </Button>
        </FocusFooter>
      </form>
    );
  }

  // 確認：大字代碼讓使用者對照裝置上的畫面，身分句說清楚核准後誰會登入。
  const user = session.user;
  const shown = code.trim();
  return (
    <>
      <FocusHeader
        title={t("activate.confirmTitle")}
        description={t("activate.confirmBody").replace("{code}", shown)}
      />
      <div className="rounded-lg bg-muted/50 px-4 py-3 text-center">
        <p className="font-mono text-[22px] font-medium tracking-wider">{shown}</p>
        {user && (
          <p className="mt-1 text-[13px] text-muted-foreground">
            {fillUser(t("activate.identity"), user)}
          </p>
        )}
      </div>
      {message && (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      )}
      <FocusFooter>
        <Button type="button" variant="outline" onClick={() => onDecide("deny")} disabled={pending}>
          {t("activate.deny")}
        </Button>
        <Button type="button" onClick={() => onDecide("approve")} disabled={pending}>
          {t("activate.approve")}
        </Button>
      </FocusFooter>
    </>
  );
}
