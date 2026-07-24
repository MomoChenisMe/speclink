// 六個管理目的地的殼。各自的 view-model 讀取與 mutation 在 admin view-model 面
// （後續 knife）補上；此處先確立標題與路由，供導覽與深連結測試。
function Stub({ title, hint }: { title: string; hint: string }) {
  return (
    <div>
      <h1 className="text-2xl font-semibold">{title}</h1>
      <p className="mt-2 text-muted-foreground">{hint}</p>
    </div>
  );
}

export const UsersPage = () => <Stub title="使用者" hint="列表、邀請、停權／復權、membership 與 admin 旗標。" />;
export const RegistryPage = () => (
  <Stub title="專案與儲存庫" hint="建立與更名 project／repo；key 不可改。" />
);
export const CredentialsPage = () => (
  <Stub title="憑證" hint="全站 PAT 與裝置憑證 metadata 與強制撤銷。" />
);
export const DataPage = () => <Stub title="資料操作" hint="scope 匯出與 store 遷移。" />;
export const SystemPage = () => <Stub title="系統狀態" hint="引擎、API、store 與 outbox 健康狀態。" />;
export const AuditPage = () => <Stub title="稽核紀錄" hint="管理動作的唯讀稽核事件。" />;
