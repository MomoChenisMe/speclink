// @speclink/ui — 跨桌面/web 共用的呈現元件庫。
//
// 元件為純呈現、資料由 props 注入；資料源經 SpeclinkDataSource 介面解耦（見 adapter.ts）。

export const UI_PACKAGE = "@speclink/ui";

// shadcn/ui 設計系統原語（跨桌面/web 共用）
export { cn } from "./lib/utils";
export { Button, buttonVariants, type ButtonProps } from "./components/ui/button";
export {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter,
} from "./components/ui/card";
export { Badge, badgeVariants, type BadgeProps } from "./components/ui/badge";
export {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
} from "./components/ui/alert-dialog";

export type {
  SpeclinkDataSource,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  Verb,
} from "./adapter";
export { ChangeBoard } from "./components/ChangeBoard";
export type { ChangeBoardProps } from "./components/ChangeBoard";
export { DocumentTree } from "./components/DocumentTree";
export type { DocumentTreeProps, TreeSelection } from "./components/DocumentTree";
export { DocumentViewer } from "./components/DocumentViewer";
export type { DocumentViewerProps } from "./components/DocumentViewer";

// 看板
export { changeStage, STAGES, STAGE_LABEL, type Stage } from "./stage";
export { parseTasks, type TaskLine } from "./tasks";
export type { ArtifactStatus, StatusReport } from "./adapter";
export { ChangeCard, type ChangeCardProps } from "./components/ChangeCard";
export { KanbanBoard, type KanbanBoardProps } from "./components/KanbanBoard";
export { DetailDrawer, type DetailDrawerProps } from "./components/DetailDrawer";

// Spectra 風清單 + 分頁 + 富文本
export { Markdown, type MarkdownProps } from "./components/Markdown";
export { ChangeListItem, type ChangeListItemProps } from "./components/ChangeListItem";
export { ChangeList, type ChangeListProps, type ListView } from "./components/ChangeList";
export { Tabs, TabsList, TabsTrigger, TabsContent } from "./components/ui/tabs";
export { Input } from "./components/ui/input";

// Spectra 級詳情
export type { ChangeMetaInfo } from "./adapter";
export { specDeltaCounts, sumDeltaCounts, formatDeltaCounts, type DeltaCounts } from "./delta";
export { RichDetailDrawer, type RichDetailDrawerProps } from "./components/RichDetailDrawer";

// 互動任務、彩色 delta、封存頁
export { parseTaskDoc, type TaskDocItem } from "./tasks";
export { TaskList, type TaskListProps } from "./components/TaskList";
export { DeltaBadges } from "./components/DeltaBadges";
export { ArchivedList, ArchivedRow, type ArchivedListProps } from "./components/ArchivedList";
