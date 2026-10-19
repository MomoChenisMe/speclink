// 設定分類目錄（desktop-settings-mode design D1）：應用程式設定與專案設定兩種分類欄的單一來源。
// 應用程式分類帶各頁列標籤的 i18n 鍵——分類欄搜尋以分類名與這些列標籤做大小寫不敏感比對。
import {
  FileCog,
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

export interface AppSettingsCategory {
  id: "general" | "updates" | "cli" | "servers" | "network" | "security-data" | "about";
  labelKey: string;
  icon: LucideIcon;
  /** app＝「應用程式」群組；about＝其後的「關於」群組（desktop-about-page design D4）。 */
  group: "app" | "about";
  /** 該頁各列標籤的 i18n 鍵。 */
  rows: string[];
}

export interface ProjectSettingsCategory {
  id: "context" | "rules" | "review" | "workflow" | "speclink";
  labelKey: string;
  icon: LucideIcon;
  /** config＝openspec/config.yaml 組；project＝「專案」組（.speclink.yaml）。 */
  group: "config" | "project";
  /** 遠端專案是否呈現（.speclink.yaml 屬本機 checkout，遠端無此分類）。 */
  remote: boolean;
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
      "servers.page.deviceLogin",
      "servers.page.pat",
    ],
  },
  {id:"network",labelKey:"network.title",icon:Settings2,group:"app",rows:["network.proxyMode","network.proxyUrl","network.export","network.probe","network.diagnostics","network.debugHelp","network.debugMode"]},
  {
    id:"security-data",
    labelKey:"securityData.title",
    icon:ShieldCheck,
    group:"app",
    rows:["securityData.refresh","securityData.import","securityData.source","securityData.license"],
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
  { id: "review", labelKey: "settings.projectSection.review", icon: ShieldCheck, group: "config", remote: true },
  { id: "workflow", labelKey: "settings.projectSection.workflow", icon: Workflow, group: "config", remote: true },
  { id: "speclink", labelKey: "settings.projectSection.speclink", icon: FileCog, group: "project", remote: false },
];

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
