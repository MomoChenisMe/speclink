// 自製輕量 i18n（design D7）：React context＋zh-TW／en 內建字典。
// 字串規模小（~150 key 以下），不引入 react-i18next 等執行期依賴。
import { createContext, useContext, useMemo, type ReactNode } from "react";

/** UI 介面語言（與 config.yaml 的 locale——AI artifacts 產出語言——無關）。 */
export type UiLocale = "zh-TW" | "en";

type Dict = Record<string, string>;

/**
 * 內建字典：packages/ui 元件的顯示字串，key 依元件命名空間。
 * 兩語言 key 集合必須相等（由 i18n.test 保證）。`{n}`／`{name}`／`{date}`
 * 為呼叫端以 replace 代入的佔位符。
 */
export const MESSAGES: Record<UiLocale, Dict> = {
  "zh-TW": {
    "common.loading": "載入中…",
    "common.archive": "封存",
    "common.copyName": "複製名稱",
    "common.analyze": "分析",
    "common.validate": "驗證",
    "common.tabProposal": "提案",
    "common.tabDesign": "設計",
    "common.tabTasks": "任務",
    "common.tabSpecs": "規格",
    "common.tasksCount": "{n} 任務",
    "common.rounds": "{n} 輪",
    "common.noContent": "（無內容）",
    "stage.proposed": "提案中",
    "stage.in-progress": "進行中",
    "stage.ready": "已就緒",
    "kanban.dropToArchive": "拖到此封存",
    "kanban.searchPlaceholder": "搜尋看板卡片…",
    "kanban.dragCard": "拖曳排序 {name}",
    "card.fromDiscussion": "來自討論",
    "card.fromDiscussionTitle": "來自討論：{name}",
    "board.empty": "沒有 active change",
    "drawer.subtitle": "Artifact 管線與任務",
    "viewer.empty": "選擇左側的 change 或 spec 以檢視內容",
    "tasks.empty": "（無任務）",
    "tasks.checkbox": "任務 {n}",
    "tasks.drag": "拖曳任務 {n}",
    "archived.searchPlaceholder": "搜尋已封存的變更與討論…",
    "archived.changesHeading": "已封存的變更",
    "archived.discussionsHeading": "已封存的討論",
    "archived.noChanges": "沒有已封存的變更",
    "archived.noDiscussions": "沒有已封存的討論",
    "archived.copyName": "複製封存名稱",
    "archived.noProposal": "（無提案文件）",
    "archived.noDesign": "（此變更無設計文件）",
    "archived.noSpecs": "（此變更無 delta 規格）",
    "archived.sectionContext": "背景",
    "archived.sectionRounds": "討論過程",
    "archived.sectionConclusion": "結論",
    "archived.noContext": "（無背景）",
    "archived.noRounds": "（無討論過程）",
    "archived.noConclusion": "（無結論）",
    "list.searchPlaceholder": "搜尋…",
    "list.viewActive": "進行中",
    "list.viewArchived": "已封存",
    "list.activeHeading": "進行中的變更",
    "list.noActive": "沒有進行中的變更",
    "list.noDesignDoc": "（此 change 無設計文件）",
    "list.noDeltaSpecs": "（此 change 無 delta 規格）",
    "discussion.heading": "討論",
    "discussion.none": "尚無討論",
    "discussion.statusOpen": "討論中",
    "discussion.statusConcluded": "已結論",
    "discussion.promote": "轉為變更",
    "discussion.promotedGroup": "已轉出變更的討論",
    "discussion.chipArchived": "已封存",
    "discussion.chipDeleted": "已刪除",
    "ddrawer.promoteAgain": "再轉出一個變更",
    "ddrawer.notPromoted": "尚未轉出任何變更。",
    "ddrawer.openCard": "開啟卡片",
    "ddrawer.stationPromoted": "轉出變更",
    "ddrawer.tabConclusion": "結論",
    "ddrawer.tabRounds": "討論過程",
    "ddrawer.tabContext": "背景",
    "ddrawer.tabPromote": "衍生變更",
    "ddrawer.tabRecord": "記錄",
    "ddrawer.noConclusion": "（尚無結論）",
    "ddrawer.noRounds": "（尚無討論過程）",
    "ddrawer.noContext": "（無背景）",
    "rdrawer.fullScreen": "全螢幕",
    "rdrawer.restore": "還原大小",
    "rdrawer.fromDiscussion": "來自討論：",
    "rdrawer.siblings": "同源：",
    "rdrawer.started": "{date} 開工",
    "rdrawer.delete": "刪除",
    "rdrawer.today": "今天",
    "rdrawer.yesterday": "昨天",
    "rdrawer.daysAgo": "{n} 天前",
  },
  en: {
    "common.loading": "Loading…",
    "common.archive": "Archive",
    "common.copyName": "Copy name",
    "common.analyze": "Analyze",
    "common.validate": "Validate",
    "common.tabProposal": "Proposal",
    "common.tabDesign": "Design",
    "common.tabTasks": "Tasks",
    "common.tabSpecs": "Specs",
    "common.tasksCount": "{n} tasks",
    "common.rounds": "{n} rounds",
    "common.noContent": "(no content)",
    "stage.proposed": "Proposed",
    "stage.in-progress": "In progress",
    "stage.ready": "Ready",
    "kanban.dropToArchive": "Drop here to archive",
    "kanban.searchPlaceholder": "Search board cards…",
    "kanban.dragCard": "Drag to reorder {name}",
    "card.fromDiscussion": "From discussion",
    "card.fromDiscussionTitle": "From discussion: {name}",
    "board.empty": "No active changes",
    "drawer.subtitle": "Artifact pipeline and tasks",
    "viewer.empty": "Select a change or spec on the left to view it",
    "tasks.empty": "(no tasks)",
    "tasks.checkbox": "Task {n}",
    "tasks.drag": "Drag task {n}",
    "archived.searchPlaceholder": "Search archived changes and discussions…",
    "archived.changesHeading": "Archived changes",
    "archived.discussionsHeading": "Archived discussions",
    "archived.noChanges": "No archived changes",
    "archived.noDiscussions": "No archived discussions",
    "archived.copyName": "Copy archived name",
    "archived.noProposal": "(no proposal document)",
    "archived.noDesign": "(this change has no design document)",
    "archived.noSpecs": "(this change has no delta specs)",
    "archived.sectionContext": "Context",
    "archived.sectionRounds": "Rounds",
    "archived.sectionConclusion": "Conclusion",
    "archived.noContext": "(no context)",
    "archived.noRounds": "(no rounds)",
    "archived.noConclusion": "(no conclusion)",
    "list.searchPlaceholder": "Search…",
    "list.viewActive": "Active",
    "list.viewArchived": "Archived",
    "list.activeHeading": "Active changes",
    "list.noActive": "No active changes",
    "list.noDesignDoc": "(this change has no design document)",
    "list.noDeltaSpecs": "(this change has no delta specs)",
    "discussion.heading": "Discussions",
    "discussion.none": "No discussions yet",
    "discussion.statusOpen": "Open",
    "discussion.statusConcluded": "Concluded",
    "discussion.promote": "Promote to change",
    "discussion.promotedGroup": "Promoted discussions",
    "discussion.chipArchived": "Archived",
    "discussion.chipDeleted": "Deleted",
    "ddrawer.promoteAgain": "Promote another change",
    "ddrawer.notPromoted": "No changes promoted yet.",
    "ddrawer.openCard": "Open card",
    "ddrawer.stationPromoted": "Promoted",
    "ddrawer.tabConclusion": "Conclusion",
    "ddrawer.tabRounds": "Rounds",
    "ddrawer.tabContext": "Context",
    "ddrawer.tabPromote": "Promoted changes",
    "ddrawer.tabRecord": "Record",
    "ddrawer.noConclusion": "(no conclusion yet)",
    "ddrawer.noRounds": "(no rounds yet)",
    "ddrawer.noContext": "(no context)",
    "rdrawer.fullScreen": "Full screen",
    "rdrawer.restore": "Restore size",
    "rdrawer.fromDiscussion": "From discussion:",
    "rdrawer.siblings": "Siblings:",
    "rdrawer.started": "started {date}",
    "rdrawer.delete": "Delete",
    "rdrawer.today": "today",
    "rdrawer.yesterday": "yesterday",
    "rdrawer.daysAgo": "{n} days ago",
  },
};

interface I18nContextValue {
  locale: UiLocale;
  /** 查內建＋app 合併字典；缺 key 回傳 key 本身（可見的失敗，非靜默錯語言）。 */
  t: (key: string) => string;
}

const I18nContext = createContext<I18nContextValue>({
  locale: "zh-TW",
  t: (key) => MESSAGES["zh-TW"][key] ?? key,
});

export interface I18nProviderProps {
  locale: UiLocale;
  /** app 層附加字典，與內建字典合併（同 key 時 app 覆蓋內建）。 */
  messages?: Partial<Record<UiLocale, Dict>>;
  children: ReactNode;
}

export function I18nProvider({ locale, messages, children }: I18nProviderProps) {
  const value = useMemo<I18nContextValue>(() => {
    const dict: Dict = { ...MESSAGES[locale], ...(messages?.[locale] ?? {}) };
    return { locale, t: (key) => dict[key] ?? key };
  }, [locale, messages]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

/** 取得 t(key) 與當前 locale；Provider 外預設 zh-TW（既有宿主不包也不炸）。 */
export function useI18n(): I18nContextValue {
  return useContext(I18nContext);
}
