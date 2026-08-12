// @speclink/ui — 跨桌面/web 共用的呈現元件庫。
//
// 元件為純呈現、資料由 props 注入；資料源經 SpeclinkDataSource 介面解耦（見 adapter.ts）。

export const UI_PACKAGE = "@speclink/ui";

// UI 介面 i18n（zh-TW／en；與 config locale 無關）
export { I18nProvider, useI18n, MESSAGES, type UiLocale, type I18nProviderProps } from "./i18n";
export {
  detectSystemLocale,
  readLocalePreference,
  writeLocalePreference,
  resolveUiLocale,
  type LocalePreference,
} from "./locale";

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
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
  TableCaption,
} from "./components/ui/table";
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
  ListView,
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
  RevertBlockedEvidence,
} from "./adapter";
export { RevertBlockedError, toRevertError } from "./adapter";
export {
  RevertBlockedDialog,
  type RevertBlockedInfo,
  type RevertBlockedDialogProps,
} from "./components/RevertBlockedDialog";
export { cardDndId, parseCardDndId, resolveCardDrop, type ColumnCards } from "./boardDnd";
export { DocumentTree } from "./components/DocumentTree";
export type { DocumentTreeProps, TreeSelection } from "./components/DocumentTree";
export { DocumentViewer } from "./components/DocumentViewer";
export type { DocumentViewerProps } from "./components/DocumentViewer";

// 狀態語意色（單一來源；三紅分工見 tone.ts 表頭）
export { SEMANTIC_TONE, SEMANTIC_SURFACE, type SemanticTone } from "./tone";

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

// 清單 + 分頁 + 富文本
export { Markdown, type MarkdownProps } from "./components/Markdown";
export { Tabs, TabsList, TabsTrigger, TabsContent } from "./components/ui/tabs";
export { Input } from "./components/ui/input";
export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from "./components/ui/tooltip";
export {
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectLabel,
  SelectItem,
} from "./components/ui/select";
export { Checkbox, type CheckboxProps } from "./components/ui/checkbox";
export { Textarea, type TextareaProps } from "./components/ui/textarea";
export { Skeleton } from "./components/ui/skeleton";
export {
  CardSkeleton,
  ColumnSkeleton,
  ColumnLoadFailed,
  RowSkeleton,
  DocSkeleton,
} from "./components/skeletons";

// 富詳情
export type { ChangeMetaInfo } from "./adapter";
export { specDeltaCounts, sumDeltaCounts, formatDeltaCounts, type DeltaCounts } from "./delta";
export { RichDetailDrawer, type RichDetailDrawerProps } from "./components/RichDetailDrawer";

// 互動任務、彩色 delta、封存頁
export { parseTaskDoc, type TaskDocItem } from "./tasks";
export { TaskList, type TaskListProps } from "./components/TaskList";
export { DeltaBadges } from "./components/DeltaBadges";
export { ArchivedList, type ArchivedListProps } from "./components/ArchivedList";
export { ArchivedDrawer, type ArchivedDrawerProps, type ArchivedTarget } from "./components/ArchivedDrawer";
export { ReviewArchiveDialog, type ReviewArchiveDialogProps } from "./components/ReviewArchiveDialog";
export { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_TONE, type ReviewBadgeStatus } from "./components/reviewStyle";
export { VERIFY_ICON, VERIFY_LABEL_KEY, VERIFY_TONE, type VerifyBadgeStatus } from "./components/verifyStyle";

// 規格頁
export { SpecList, type SpecListProps } from "./components/SpecList";
export { SpecDrawer, type SpecDrawerProps } from "./components/SpecDrawer";
