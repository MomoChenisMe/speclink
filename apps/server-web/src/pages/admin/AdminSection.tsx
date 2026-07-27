import { Navigate, Route, Routes } from "react-router-dom";
import { OverviewPage } from "./OverviewPage";
import { UsersPage } from "./UsersPage";
import { RegistryPage } from "./RegistryPage";
import { CredentialsPage } from "./CredentialsPage";
import { SystemPage } from "./SystemPage";
import { AuditPage } from "./AuditPage";

// 管理面的六個目的地。整個 section 由 router 以 lazy import 切出獨立 chunk，
// 使登入／帳號的初載 bundle 不含管理程式碼（D1）。
export default function AdminSection() {
  return (
    <Routes>
      <Route index element={<OverviewPage />} />
      <Route path="users" element={<UsersPage />} />
      <Route path="registry" element={<RegistryPage />} />
      <Route path="credentials" element={<CredentialsPage />} />
      <Route path="system" element={<SystemPage />} />
      <Route path="audit" element={<AuditPage />} />
      {/* 資料操作已併入系統：舊書籤送到內容搬去的地方，而不是空白畫面。 */}
      <Route path="data" element={<Navigate to="/admin/system" replace />} />
      <Route path="*" element={<Navigate to="/admin" replace />} />
    </Routes>
  );
}
