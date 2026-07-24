// @speclink/ui — 跨桌面/web 共用的呈現元件庫。
//
// 元件為純呈現、資料由 props 注入；資料源經 SpeclinkDataSource 介面解耦（見 adapter.ts）。

export const UI_PACKAGE = "@speclink/ui";

// UI 介面 i18n（zh-TW／en；與 config locale 無關）
export { I18nProvider, useI18n, MESSAGES, type UiLocale, type I18nProviderProps } from "./i18n";

// shadcn/ui 設計系統原語（跨桌面/web 共用）
export { cn } from "./lib/utils";
export { Toaster } from "./components/ui/sonner";
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
export { Label } from "./components/ui/label";
export {
  Sheet,
  SheetContent,
  SheetTrigger,
  SheetClose,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "./components/ui/sheet";

export type {
  SpeclinkDataSource,
  CardKind,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionItem,
  DiscussionLists,
  SearchHit,
  Verb,
  AnalyzeFinding,
  AnalyzeDimension,
  AnalyzeReport,
  VerbDrawerResult,
} from "./adapter";
export { cardDndId, parseCardDndId, resolveCardDrop, type ColumnCards } from "./boardDnd";
export { ChangeBoard } from "./components/ChangeBoard";
export type { ChangeBoardProps } from "./components/ChangeBoard";
export { DocumentTree } from "./components/DocumentTree";
export type { DocumentTreeProps, TreeSelection } from "./components/DocumentTree";
export { DocumentViewer } from "./components/DocumentViewer";
export type { DocumentViewerProps } from "./components/DocumentViewer";

// 看板
export { changeStage, STAGE_BADGE, STAGE_BAR, STAGE_ICON, STAGES, type Stage } from "./stage";
export { siblingChangesOf } from "./siblings";
export { parseTasks, type TaskLine } from "./tasks";
export type { ArtifactStatus, StatusReport } from "./adapter";
export { ChangeCard, type ChangeCardProps } from "./components/ChangeCard";
export { KanbanBoard, type KanbanBoardProps } from "./components/KanbanBoard";
export {
  DiscussionColumn,
  discussionChipStage,
  type DiscussionColumnProps,
} from "./components/DiscussionColumn";
export {
  DiscussionDrawer,
  splitDiscussionSections,
  type DiscussionDrawerProps,
  type DiscussionSections,
} from "./components/DiscussionDrawer";
export { DetailDrawer, type DetailDrawerProps } from "./components/DetailDrawer";

// Spectra 風清單 + 分頁 + 富文本
export { Markdown, type MarkdownProps } from "./components/Markdown";
export { ChangeListItem, type ChangeListItemProps } from "./components/ChangeListItem";
export { ChangeList, type ChangeListProps, type ListView } from "./components/ChangeList";
export { Tabs, TabsList, TabsTrigger, TabsContent } from "./components/ui/tabs";
export { Input } from "./components/ui/input";
export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from "./components/ui/tooltip";
export { NativeSelect, type NativeSelectProps } from "./components/ui/select";
export { Checkbox, type CheckboxProps } from "./components/ui/checkbox";
export { Textarea, type TextareaProps } from "./components/ui/textarea";

// Spectra 級詳情
export type { ChangeMetaInfo } from "./adapter";
export { specDeltaCounts, sumDeltaCounts, formatDeltaCounts, type DeltaCounts } from "./delta";
export { RichDetailDrawer, type RichDetailDrawerProps } from "./components/RichDetailDrawer";

// 互動任務、彩色 delta、封存頁
export { parseTaskDoc, type TaskDocItem } from "./tasks";
export { TaskList, type TaskListProps } from "./components/TaskList";
export { DeltaBadges } from "./components/DeltaBadges";
export { ArchivedList, type ArchivedListProps } from "./components/ArchivedList";
export { ArchivedDrawer, type ArchivedDrawerProps, type ArchivedTarget } from "./components/ArchivedDrawer";

// 規格頁
export { SpecList, type SpecListProps } from "./components/SpecList";
export { SpecDrawer, type SpecDrawerProps } from "./components/SpecDrawer";
