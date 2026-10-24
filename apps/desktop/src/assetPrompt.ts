// 技能檔過期提示的裁決與略過記憶（desktop-instruction-staleness-prompt 決策 6/7）：
// 探測回報是引擎事實，是否顯示提示則是本機個人決定——「保留現狀」寫入 app 本機
// 持久化（專案路徑 → 已略過的產物層版號），不進 .speclink.yaml、不進任何 repo 檔案。
// 規格資料夾遷移由同一張提示承載（desktop-spec-dir-migration D2）。
import type { AssetProbeResult, MigrationProbe } from "./adapter/workspace";

/** 提示的呈現態：主動作依此分文案（過期→更新、缺失→安裝、較新→無改寫動作、
 * migrate＝技能檔是現版、只有規格資料夾要遷移）。 */
export interface AssetPromptState {
  kind: "stale" | "missing" | "newer" | "migrate";
  /** 將被新建或改寫且內容有異的受管檔數。 */
  fileCount: number;
  /** 此提示對應的產物層版號（「保留現狀」記的就是它）。 */
  version: string;
  /** 專案指令檔的標記版號（確認框「這個專案的技能檔版本」列）；從未安裝為 null。 */
  projectVersion: string | null;
  /** 按更新時規格資料夾會怎樣；較新態恆為 none（引擎拒絕這次更新），migrate 態恆為 pending。 */
  migration: MigrationProbe;
}

// 沿用改名前的 key：既有使用者的略過記憶存在這個名字下，換名等於全部失效。
const STORAGE_KEY = "speclink.instructionSkips";

/** 讀取略過記憶（專案路徑 → 已略過版號）。壞 JSON 或不識別形狀一律視為無記憶。 */
export function readAssetSkips(storage: Storage = localStorage): Record<string, string> {
  try {
    const raw = storage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) return {};
    const out: Record<string, string> = {};
    for (const [root, version] of Object.entries(parsed)) {
      if (typeof version === "string") out[root] = version;
    }
    return out;
  } catch {
    return {};
  }
}

/** 記下「此專案於此版號已保留現狀」。同專案再次略過時覆蓋舊版號。 */
export function writeAssetSkip(
  root: string,
  version: string,
  storage: Storage = localStorage,
): void {
  const skips = { ...readAssetSkips(storage), [root]: version };
  storage.setItem(STORAGE_KEY, JSON.stringify(skips));
}

/** 顯示裁決（規格「技能檔過期探測」的提示面）：過期、缺失或較新才提示，技能檔是現版
 * 但規格資料夾待遷移時也提示；且該專案未略過當前版號。技能檔是現版而遷移暫緩時不提示：
 * 略過以版號為鍵，暫緩時的略過會壓掉排除原因後的待遷移提示——暫緩原因只在過期或缺失的
 * 提示裡說明。無法判定一律不提示——無法判定不記入略過、也不視同現版。 */
export function assetPrompt(
  probe: AssetProbeResult,
  root: string,
  skips: Record<string, string>,
): AssetPromptState | null {
  const { migration, status } = probe;
  const kind =
    status === "stale" || status === "missing" || status === "newer"
      ? status
      : status === "current" && migration.state === "pending"
        ? "migrate"
        : null;
  if (!kind) return null;
  if (skips[root] === probe.currentVersion) return null;
  // 專案的技能檔版號取觸發這個狀態的工具：較新取領先的、過期取落後的；缺失（從未安裝）為 null；
  // 現版取第一個有版號的工具。
  const flagged =
    kind === "missing"
      ? undefined
      : probe.tools.find(
          (tool) =>
            (kind === "newer" ? tool.newer : kind === "stale" ? tool.stale : true) && tool.workspaceVersion !== null,
        );
  return {
    kind,
    fileCount: probe.differingFiles.length,
    version: probe.currentVersion,
    projectVersion: flagged?.workspaceVersion ?? null,
    migration: kind === "newer" ? { state: "none" } : migration,
  };
}
