// 應用程式設定（desktop-settings-mode design D5）：與任何專案無關；依設定分類欄的所在分類渲染
// 一頁——一般、語言與外觀、更新、CLI、伺服器。
import type { Platform } from "../platform";
import type { UpdaterState } from "../core/updater";
import type { SettingsSection } from "../settingsCatalog";
import { AppearancePage, type InterfacePrefsProps } from "./settings/AppearancePage";
import { CliPage, type CliInstallProps } from "./settings/CliPage";
import { GeneralPage } from "./settings/GeneralPage";
import { ServersPage, type ServersPageProps } from "./settings/ServersPage";
import { UpdatesPage } from "./settings/UpdatesPage";

/** 軟體更新面：狀態機現值＋手動檢查入口＋更新日誌瀏覽入口＋常駐現版號（null＝尚未取得）
 * ＋「下載與安裝」的動作＋聚焦捲動＋背景檢查開關。 */
export interface AppSettingsUpdaterProps {
  state: UpdaterState;
  onCheck: () => void;
  /** 開啟更新日誌對話框的瀏覽模式（desktop-app「更新日誌彈窗」）。 */
  onShowReleaseNotes: () => void;
  currentVersion?: string | null;
  /** 使用者同意：下載並套用。 */
  onAccept: () => void;
  /** 取消下載＝放棄本次同意、回閒置。 */
  onCancel: () => void;
  /** 套用完成後重啟為新版。 */
  onRelaunch: () => void;
  /** true＝進更新頁後捲到下載與安裝卡一次（圖示列更新鈕、toast「查看」、技能檔較新態的入口）。 */
  focus?: boolean;
  /** 捲到卡後回呼，呼叫端清旗標。 */
  onFocusHandled?: () => void;
  /** 啟動時與回到前景的背景檢查開關現值（預設開）。 */
  autoCheck: boolean;
  onAutoCheckChange: (on: boolean) => void;
}

export interface AppSettingsViewProps extends InterfacePrefsProps {
  /** 設定分類欄的所在分類。 */
  section: SettingsSection;
  /** 鍵盤快捷鍵卡依平台列出實際生效的快捷鍵（desktop-native-menu design D5）。 */
  platform: Platform;
  /** 面板建立失敗時於一般頁底浮出的單行錯誤。 */
  trayPanelError?: string | null;
  /** app 全域伺服器管理面；測試或不支援連線的殼層可不注入。 */
  servers?: ServersPageProps;
  /** 軟體更新面（desktop-app「桌面自動更新」）；測試或不支援更新的殼層可不注入。 */
  updater?: AppSettingsUpdaterProps;
  /** 安裝 CLI 指令面（desktop-app「安裝 CLI 指令到 PATH」）；探測前或無殼層支援時不注入。 */
  cliInstall?: CliInstallProps;
}

export function AppSettingsView({
  section,
  platform,
  localePref,
  onLocalePrefChange,
  themePref,
  onThemePrefChange,
  trayPanelError = null,
  servers,
  updater,
  cliInstall,
}: AppSettingsViewProps) {
  const prefs = { localePref, onLocalePrefChange, themePref, onThemePrefChange };
  return (
    <div className="mx-auto flex w-full max-w-[768px] flex-col gap-6 px-6 pb-10 pt-7">
      {section === "general" && (
        <GeneralPage
          {...prefs}
          platform={platform}
          updater={updater}
          cliInstall={cliInstall}
          trayPanelError={trayPanelError}
        />
      )}
      {section === "appearance" && <AppearancePage {...prefs} />}
      {section === "updates" && <UpdatesPage updater={updater} />}
      {section === "cli" && <CliPage cliInstall={cliInstall} />}
      {section === "servers" && servers && <ServersPage {...servers} />}
    </div>
  );
}
