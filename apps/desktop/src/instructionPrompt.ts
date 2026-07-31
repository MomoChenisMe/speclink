// 指令檔過期提示的裁決與略過記憶（desktop-instruction-staleness-prompt 決策 6/7）：
// 探測回報是引擎事實，是否顯示提示則是本機個人決定——「保留現狀」寫入 app 本機
// 持久化（專案路徑 → 已略過的產物層版號），不進 .speclink.yaml、不進任何 repo 檔案。
import type { InstructionProbeResult } from "./adapter/workspace";

/** 提示的呈現態：主動作依此分文案（過期→更新、缺失→安裝）。 */
export interface InstructionPromptState {
  kind: "stale" | "missing";
  /** 將被新建或改寫且內容有異的受管檔數。 */
  fileCount: number;
  /** 此提示對應的產物層版號（「保留現狀」記的就是它）。 */
  version: string;
}

const STORAGE_KEY = "speclink.instructionSkips";

/** 讀取略過記憶（專案路徑 → 已略過版號）。壞 JSON 或不識別形狀一律視為無記憶。 */
export function readInstructionSkips(storage: Storage = localStorage): Record<string, string> {
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
export function writeInstructionSkip(
  root: string,
  version: string,
  storage: Storage = localStorage,
): void {
  const skips = { ...readInstructionSkips(storage), [root]: version };
  storage.setItem(STORAGE_KEY, JSON.stringify(skips));
}

/** 顯示裁決（規格「指令檔過期提示」）：僅過期或缺失才提示，且該專案未略過當前
 * 版號；現版與無法判定一律不提示——無法判定不記入略過、也不視同現版。 */
export function instructionPrompt(
  probe: InstructionProbeResult,
  root: string,
  skips: Record<string, string>,
): InstructionPromptState | null {
  if (probe.status !== "stale" && probe.status !== "missing") return null;
  if (skips[root] === probe.currentVersion) return null;
  return {
    kind: probe.status,
    fileCount: probe.differingFiles.length,
    version: probe.currentVersion,
  };
}
