import type { ReactNode } from "react";
import { Navigate, Route, Routes, useNavigate } from "react-router-dom";
import { Button, Card, EmptyState, Tabs, TabsList, type EmptyStateProps } from "@speclink/ui";
import { AdminError, AdminLoading } from "./states";

// 使用者與專案兩個區段共用的詳情頁骨架（server-web-detail-pages design D1、D5、D6）：區段的內層路由、
// 對不到資料時的說明，以及卡片標頭分頁接內容卡。

/** 區段的內層路由：index 是列表、`:<param>` 是詳情頁（資料未到時為載入中或可重試的錯誤），其他路徑導回列表。 */
export function SectionRoutes({
  base,
  param,
  list,
  detail,
  error,
  onRetry,
}: {
  base: string;
  param: string;
  list: ReactNode;
  /** 資料已載入時的詳情頁；尚未載入時為 null。 */
  detail: ReactNode | null;
  error: unknown;
  onRetry: () => void;
}) {
  return (
    <Routes>
      <Route index element={list} />
      <Route
        path={`:${param}`}
        element={detail ?? (error != null ? <AdminError onRetry={onRetry} /> : <AdminLoading />)}
      />
      <Route path="*" element={<Navigate to={base} replace />} />
    </Routes>
  );
}

/** 網址對不到資料：說明找不到，附回到列表的按鈕。 */
export function DetailNotFound({
  icon,
  title,
  body,
  backLabel,
  backTo,
}: {
  icon: EmptyStateProps["icon"];
  title: string;
  body: string;
  backLabel: string;
  backTo: string;
}) {
  const navigate = useNavigate();
  return (
    <EmptyState
      icon={icon}
      title={title}
      description={body}
      action={
        <Button type="button" variant="outline" onClick={() => navigate(backTo)}>
          {backLabel}
        </Button>
      }
    />
  );
}

/** 卡片標頭分頁：分頁列為內容卡的頂部，窄螢幕時在卡內水平捲動；`children` 為各分頁的 TabsContent。 */
export function DetailTabs({
  value,
  onValueChange,
  triggers,
  children,
}: {
  value: string;
  onValueChange: (value: string) => void;
  triggers: ReactNode;
  children: ReactNode;
}) {
  return (
    <Tabs value={value} onValueChange={onValueChange} className="flex flex-col">
      <TabsList variant="card" className="overflow-x-auto">
        {triggers}
      </TabsList>
      <Card attached="top">{children}</Card>
    </Tabs>
  );
}
