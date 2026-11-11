// 設定分類目錄（desktop-settings-mode design D1）：應用程式設定與專案設定兩種分類欄的單一來源。
// 應用程式分類帶各頁列標籤的 i18n 鍵——分類欄搜尋以分類名與這些列標籤做大小寫不敏感比對。
import {
  Bot,
  FileText,
  Info,
  ListChecks,
  RefreshCw,
  Server,
  Settings2,
  ShieldCheck,
  SquareTerminal,
  Workflow,
  type LucideIcon,
} from "lucide-react";
import { DEFAULT_SPEC_DIR } from "@speclink/ui";

export interface AppSettingsCategory {
  id: "general" | "updates" | "cli" | "servers" | "about";
  labelKey: string;
  icon: LucideIcon;
  /** app＝「應用程式」群組；about＝其後的「關於」群組（desktop-about-page design D4）。 */
  group: "app" | "about";
  /** 該頁各列標籤的 i18n 鍵。 */
  rows: string[];
}

export interface ProjectSettingsCategory {
  id: "context" | "rules" | "review" | "subagent" | "workflow" | "speclink";
  labelKey: string;
  /** 子項（帶 parent）不渲染圖示，不給。 */
  icon?: LucideIcon;
  /** config＝「工作流程」組（規格資料夾的 config.yaml）；project＝「專案」組（.speclink.yaml）。 */
  group: "config" | "project";
  /** 遠端專案是否呈現（.speclink.yaml 屬本機 checkout，遠端無此分類）。 */
  remote: boolean;
  /** 屬於上層分類「品質關卡」（QUALITY_PARENT）的子項；子項在目錄裡相鄰。 */
  parent?: "quality";
}

/** 上層分類「品質關卡」（目前唯一的上層分類）：沒有自己的頁，點了開 `opens` 那頁。 */
export const QUALITY_PARENT: { labelKey: string; icon: LucideIcon; opens: ProjectSettingsCategory["id"] } = {
  labelKey: "settings.projectSection.quality",
  icon: ShieldCheck,
  opens: "review",
};

/** 工作流設定檔的專案根相對路徑（desktop-spec-dir-migration D4）：專案設定各頁的檔案註記由它
 * 組成；快照沒帶規格資料夾（遠端、尚未載入）時用預設 speclink。 */
export function workflowConfigFile(specDir: string | undefined): string {
  return `${specDir ?? DEFAULT_SPEC_DIR}/config.yaml`;
}

export type SettingsSection = AppSettingsCategory["id"];
export type ProjectSettingsSection = ProjectSettingsCategory["id"];

export const APP_SETTINGS_CATEGORIES: AppSettingsCategory[] = [
  {
    id: "general",
    labelKey: "settings.section.general",
    icon: Settings2,
    group: "app",
    rows: ["settings.displayLanguage", "settings.appearance", "settings.shortcutsTitle"],
  },
  {
    id: "updates",
    labelKey: "settings.section.updates",
    icon: RefreshCw,
    group: "app",
    rows: [
      "updater.check",
      "updater.currentVersion",
      "updater.page.latest",
      "releaseNotes.open",
      "updater.page.autoCheck",
    ],
  },
  {
    id: "cli",
    labelKey: "settings.section.cli",
    icon: SquareTerminal,
    group: "app",
    rows: ["cliInstall.rowLabel", "cli.page.deployDir"],
  },
  {
    id: "servers",
    labelKey: "settings.section.servers",
    icon: Server,
    group: "app",
    rows: [
      "servers.page.addConnection",
      "servers.page.connectionsCard",
      "servers.page.defaultForNew",
      "servers.page.browserLogin",
      "servers.page.deviceLogin",
      "servers.page.pat",
    ],
  },
  {
    id: "about",
    labelKey: "settings.section.about",
    icon: Info,
    group: "about",
    rows: [
      "about.copyInfo",
      "about.source",
      "about.docs",
      "about.report",
      "about.license",
      "about.thirdParty",
    ],
  },
];

export const PROJECT_SETTINGS_CATEGORIES: ProjectSettingsCategory[] = [
  { id: "context", labelKey: "settings.projectSection.context", icon: FileText, group: "config", remote: true },
  { id: "rules", labelKey: "settings.projectSection.rules", icon: ListChecks, group: "config", remote: true },
  { id: "review", labelKey: "settings.projectSection.review", group: "config", remote: true, parent: "quality" },
  { id: "subagent", labelKey: "settings.projectSection.subagent", group: "config", remote: true, parent: "quality" },
  { id: "workflow", labelKey: "settings.projectSection.workflow", icon: Workflow, group: "config", remote: true },
  { id: "speclink", labelKey: "settings.projectSection.speclink", icon: Bot, group: "project", remote: false },
];

/** 麵包屑在「專案設定」之後的段：子項的頁為 [上層分類（點了開 `opens`）, 頁名]，其他頁為 [頁名]。 */
export function projectSettingsTrail(
  section: ProjectSettingsSection,
): Array<{ labelKey: string; opens?: ProjectSettingsSection }> {
  const category = PROJECT_SETTINGS_CATEGORIES.find((c) => c.id === section)!;
  const page = { labelKey: category.labelKey };
  return category.parent ? [QUALITY_PARENT, page] : [page];
}

/** 分類欄搜尋：分類名或任一列標籤含查詢字串（大小寫不敏感）；空查詢全數保留。 */
export function matchAppSettings(
  query: string,
  t: (key: string) => string,
): AppSettingsCategory[] {
  const q = query.trim().toLowerCase();
  if (!q) return APP_SETTINGS_CATEGORIES;
  return APP_SETTINGS_CATEGORIES.filter((category) =>
    [category.labelKey, ...category.rows].some((key) => t(key).toLowerCase().includes(q)),
  );
}
