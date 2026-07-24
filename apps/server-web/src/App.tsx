// Task 1 的最小應用殼層：證明 workspace 可用共用 semantic theme 渲染。
// 角色導覽、深連結與專注／帳號／管理三殼層在後續（server-web-console 導覽 knife）
// 建立，屆時會取代此 placeholder。
export function App() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-background text-foreground">
      <div className="space-y-2 text-center">
        <h1 className="text-2xl font-semibold text-primary">Speclink</h1>
        <p className="text-muted-foreground">Server Web Console</p>
      </div>
    </main>
  );
}
