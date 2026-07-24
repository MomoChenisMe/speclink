import { useCallback, useEffect, useState } from "react";

// 每個 route 以此表示 loading／success／error 狀態（D6：一致且可恢復）。
// server 資料留在呼叫 route 的 component state，不進全域 store（D1）。

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

  useEffect(() => {
    let alive = true;
    setState({ loading: true, data: null, error: null });
    fn().then(
      (data) => {
        if (alive) setState({ loading: false, data, error: null });
      },
      (error) => {
        if (alive) setState({ loading: false, data: null, error });
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
