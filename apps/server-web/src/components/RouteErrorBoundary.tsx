import { Component, type ReactNode } from "react";

// route chunk 或 render 失敗時顯示可重試訊息，而非白屏（D6）。route 資料層的
// 預期錯誤（fetch 失敗）由頁面 inline error 狀態處理；此邊界接住真正的 render crash。
export class RouteErrorBoundary extends Component<
  { children: ReactNode },
  { error: unknown }
> {
  state = { error: null as unknown };

  static getDerivedStateFromError(error: unknown) {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div role="alert" className="mx-auto max-w-md p-6 text-center">
          <p className="text-destructive">發生錯誤，無法顯示此頁面。</p>
          <button
            type="button"
            className="mt-3 rounded-md border px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onClick={() => this.setState({ error: null })}
          >
            重試
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
