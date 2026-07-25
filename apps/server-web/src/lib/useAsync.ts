import { useCallback, useEffect, useState } from "react";

import { WebApiError } from "../api/client";
import { useSession } from "../app/context";

// 每個 route 以此表示 loading／success／error 狀態（D6：一致且可恢復）。
// server 資料留在呼叫 route 的 component state，不進全域 store（D1）。

// 401 有兩種語意：session 失效（code=unauthenticated）與登入密碼錯誤
// （code=invalid_credentials）。只有前者代表已載入的 route 失去授權，須重讀
// session 讓 route guard 導回登入頁並保留 returnTo；後者屬登入表單的欄位錯誤，
// 由 LoginPage 自行呈現，不走這裡（LoginPage 不使用 useAsync）。
function isSessionExpired(error: unknown): boolean {
  return error instanceof WebApiError && error.status === 401 && error.code === "unauthenticated";
}

export type AsyncState<T> = {
  loading: boolean;
  data: T | null;
  error: unknown;
  reload: () => void;
};

/** Run `fn` on mount and whenever `deps` change; `reload()` re-runs it. */
export function useAsync<T>(fn: () => Promise<T>, deps: unknown[]): AsyncState<T> {
  const [state, setState] = useState<{ loading: boolean; data: T | null; error: unknown }>({
    loading: true,
    data: null,
    error: null,
  });
  const [nonce, setNonce] = useState(0);
  const reload = useCallback(() => setNonce((n) => n + 1), []);
  const { refresh } = useSession();

  useEffect(() => {
    let alive = true;
    setState({ loading: true, data: null, error: null });
    fn().then(
      (data) => {
        if (alive) setState({ loading: false, data, error: null });
      },
      (error) => {
        if (alive) setState({ loading: false, data: null, error });
        // session 失效：重讀 session 真相，由 route guard 導向登入頁並帶安全 returnTo。
        if (isSessionExpired(error)) void refresh();
      },
    );
    return () => {
      alive = false;
    };
    // fn 每次 render 皆為新閉包，故以呼叫端的 deps 控制重跑；nonce 觸發 reload。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, nonce]);

  return { ...state, reload };
}
