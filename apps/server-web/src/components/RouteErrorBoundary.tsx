import { Component, type ReactNode } from "react";
import { Button, useI18n } from "@speclink/ui";

// route chunk 或 render 失敗時顯示可重試訊息，而非白屏（D6）。route 資料層的
// 預期錯誤（fetch 失敗）由頁面 inline error 狀態處理；此邊界接住真正的 render crash。
//
// 邊界本身必須是 class（只有 class 有 getDerivedStateFromError），class 不能用 hook，
// 所以文案交給下面這個 function component 讀。
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
      return <RenderFailed onRetry={() => this.setState({ error: null })} />;
    }
    return this.props.children;
  }
}

function RenderFailed({ onRetry }: { onRetry: () => void }) {
  const { t } = useI18n();
  return (
    <div role="alert" className="mx-auto max-w-md p-6 text-center">
      <p className="text-destructive">{t("common.renderFailed")}</p>
      <Button type="button" variant="outline" size="sm" className="mt-3" onClick={onRetry}>
        {t("common.retry")}
      </Button>
    </div>
  );
}
