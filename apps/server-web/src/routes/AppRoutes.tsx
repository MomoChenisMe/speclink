import {
  lazy,
  Suspense,
  useCallback,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import { Button } from "@speclink/ui";
import type { SessionData } from "../api/client";
import { SessionProvider, useClient, useSession } from "../app/context";
import { FocusLayout } from "../layouts/FocusLayout";
import { AccountLayout } from "../layouts/AccountLayout";
import { AdminLayout } from "../layouts/AdminLayout";
import { LoginPage } from "../pages/LoginPage";
import { SetupPage } from "../pages/SetupPage";
import { InvitePage } from "../pages/InvitePage";
import { ActivatePage } from "../pages/ActivatePage";
import { AccountPage } from "../pages/AccountPage";
import { RouteErrorBoundary } from "../components/RouteErrorBoundary";
import { loginRedirect } from "../lib/returnTo";

// 管理面切為獨立 lazy chunk：登入／帳號初載 bundle 不含管理程式碼（D1）。
const AdminSection = lazy(() => import("../pages/admin/AdminSection"));

function BootLoading() {
  return (
    <div
      role="status"
      aria-live="polite"
      className="grid min-h-screen place-items-center bg-background text-muted-foreground"
    >
      載入中…
    </div>
  );
}

function BootError({ onRetry }: { onRetry: () => void }) {
  return (
    <div role="alert" className="grid min-h-screen place-items-center bg-background">
      <div className="text-center">
        <p className="text-destructive">無法載入，發生錯誤。</p>
        <Button type="button" variant="outline" size="sm" className="mt-3" onClick={onRetry}>
          重試
        </Button>
      </div>
    </div>
  );
}

// 開機：讀取一次 session 真相並 gate 全 app。refresh 供 login／logout 後更新。
function SessionGate({ children }: { children: ReactNode }) {
  const client = useClient();
  const [session, setSession] = useState<SessionData | null>(null);
  const [error, setError] = useState<unknown>(null);
  const refresh = useCallback(async () => {
    try {
      setError(null);
      setSession(await client.getSession());
    } catch (e) {
      setError(e);
    }
  }, [client]);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (error != null && session == null) return <BootError onRetry={() => void refresh()} />;
  if (session == null) return <BootLoading />;
  return <SessionProvider value={{ session, refresh }}>{children}</SessionProvider>;
}

function RequireAuth({ children }: { children: ReactNode }) {
  const { session } = useSession();
  const location = useLocation();
  if (!session.authenticated) {
    return <Navigate to={loginRedirect(location.pathname)} replace />;
  }
  return <>{children}</>;
}

// 非 admin 一律明確 403，且不顯示管理導覽（焦點流程殼呈現無權限狀態）。
function Forbidden() {
  return (
    <FocusLayout>
      <div role="alert">
        <h1 className="text-2xl font-semibold">沒有權限</h1>
        <p className="mt-2 text-muted-foreground">你沒有存取管理面的權限。</p>
      </div>
    </FocusLayout>
  );
}

function RequireAdmin({ children }: { children: ReactNode }) {
  const { session } = useSession();
  const location = useLocation();
  if (!session.authenticated) {
    return <Navigate to={loginRedirect(location.pathname)} replace />;
  }
  if (!session.user?.admin) return <Forbidden />;
  return <>{children}</>;
}

function RootRedirect() {
  const { session } = useSession();
  return <Navigate to={session.home} replace />;
}

export function AppRoutes() {
  return (
    <SessionGate>
      <Routes>
        <Route
          path="/login"
          element={
            <FocusLayout>
              <LoginPage />
            </FocusLayout>
          }
        />
        <Route
          path="/setup"
          element={
            <FocusLayout>
              <SetupPage />
            </FocusLayout>
          }
        />
        <Route
          path="/invite/:token"
          element={
            <FocusLayout>
              <InvitePage />
            </FocusLayout>
          }
        />
        <Route
          path="/activate"
          element={
            <FocusLayout>
              <ActivatePage />
            </FocusLayout>
          }
        />
        <Route
          path="/account"
          element={
            <RequireAuth>
              <AccountLayout>
                <AccountPage />
              </AccountLayout>
            </RequireAuth>
          }
        />
        <Route
          path="/admin/*"
          element={
            <RequireAdmin>
              <AdminLayout>
                <RouteErrorBoundary>
                  <Suspense
                    fallback={
                      <p role="status" aria-live="polite" className="text-muted-foreground">
                        載入中…
                      </p>
                    }
                  >
                    <AdminSection />
                  </Suspense>
                </RouteErrorBoundary>
              </AdminLayout>
            </RequireAdmin>
          }
        />
        <Route path="/" element={<RootRedirect />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </SessionGate>
  );
}
