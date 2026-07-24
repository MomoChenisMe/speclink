import { WebApiError } from "../api/client";

// 表單提交失敗的一致讀取：WebApiError 帶出可公開的 message 與 fieldErrors；其他錯誤
// 退回 fallback 訊息。呼叫端保留輸入並以 role=alert 宣告（D6）。
export function readFormError(
  error: unknown,
  fallback: string,
): { message: string; fieldErrors: Record<string, string> } {
  if (error instanceof WebApiError) {
    return { message: error.message, fieldErrors: error.fieldErrors ?? {} };
  }
  return { message: fallback, fieldErrors: {} };
}
