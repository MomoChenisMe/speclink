// 更新頁（desktop-settings-mode design D6；spec「桌面自動更新」「更新日誌彈窗」）：頁標題右側
// 「檢查更新」；版本卡（目前版本、最新版本＋狀態籤）；下載與安裝卡（只在有事等使用者處置時）；
// 更新日誌卡（最新版本展開、較舊版本一列一個；條目與分組標題依介面語言）；自動檢查卡（背景檢查開關）。
import { useEffect, useRef, useState } from "react";
import {
  Button,
  SettingsCard,
  SettingsPageHeader,
  SettingsRow,
  StatusPill,
  Switch,
  useI18n,
  type StatusPillLabel,
} from "@speclink/ui";

import { UpdateInstallRow } from "../../components/UpdateInstallRow";
import { ReleaseNotesDialog } from "../../components/ReleaseNotesDialog";
import { updateNeedsAttention, type UpdaterState } from "../../core/updater";
import { entryText, RELEASE_NOTES } from "../../release-notes/release-notes";
import type { AppSettingsUpdaterProps } from "../AppSettingsView";

/** 「最新版本」列：等寬版號（無則「—」或不顯示）＋狀態籤；關於頁版本列的狀態籤共用。 */
export function latestOf(state: UpdaterState, currentVersion: string | null | undefined): {
  version: string | null;
  pill: StatusPillLabel;
} {
  switch (state.phase) {
    case "available":
    case "downloading":
    case "restartPending":
      return { version: state.version, pill: { key: "updater.page.pill.available", tone: "warning" } };
    case "upToDate":
      return { version: currentVersion ?? null, pill: { key: "updater.page.pill.upToDate", tone: "success" } };
    case "checking":
      return { version: null, pill: { key: "updater.page.pill.checking", tone: "neutral" } };
    case "checkFailed":
      return { version: null, pill: { key: "updater.page.pill.checkFailed", tone: "destructive" } };
    case "error":
      return { version: null, pill: { key: "updater.page.pill.error", tone: "destructive" } };
    case "idle":
      return { version: "—", pill: { key: "updater.page.pill.idle", tone: "neutral" } };
  }
}

/** 等寬版號。 */
const VersionText = ({ children }: { children: string }) => (
  <span className="font-mono text-[13px]">{children}</span>
);

export function UpdatesPage({ updater, currentVersion }: { updater?: AppSettingsUpdaterProps; currentVersion?: string | null }) {
  const { t, locale } = useI18n();
  const [notes, setNotes] = useState<{ version?: string } | null>(null);
  const ready = !!updater && updater.ready !== false;
  const state = updater?.state ?? { phase: "idle" };
  const version = currentVersion ?? updater?.currentVersion;
  const showNotes = (version?: string) => {
    if (!updater) setNotes({ version });
    else if (version === undefined) updater.onShowReleaseNotes();
    else updater.onShowReleaseNotes(version);
  };
  // 聚焦捲動（圖示列更新鈕、toast「查看」、技能檔較新態的入口舉旗）：捲到下載與安裝卡，
  // 沒有該卡時捲到版本卡；捲一次並回呼清旗標。
  const versionCard = useRef<HTMLDivElement>(null);
  const installCard = useRef<HTMLDivElement>(null);
  const focus = updater?.focus === true;
  const onFocusHandled = updater?.onFocusHandled;
  useEffect(() => {
    if (!focus) return;
    (installCard.current ?? versionCard.current)?.scrollIntoView({ block: "start", behavior: "smooth" });
    onFocusHandled?.();
  }, [focus, onFocusHandled]);

  const latest = latestOf(state, version);
  const [newest, ...older] = RELEASE_NOTES;
  const autoCheckLabel = t("updater.page.autoCheck");
  return (
    <>
      <SettingsPageHeader
        title={t("settings.section.updates")}
        description={t("updater.page.description")}
        actions={
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={!ready || state.phase === "checking"}
            onClick={updater?.onCheck}
          >
            {t("updater.check")}
          </Button>
        }
      />
      {!ready && <p role="status" className="text-sm text-muted-foreground">{t("updater.page.unavailable")}</p>}
      <div ref={versionCard}>
        <SettingsCard title={t("updater.page.versionCard")}>
          <SettingsRow label={t("updater.currentVersion")} control={<VersionText>{version ?? "—"}</VersionText>} />
          <SettingsRow
            label={t("updater.page.latest")}
            control={
              <>
                {latest.version && <VersionText>{latest.version}</VersionText>}
                <StatusPill tone={latest.pill.tone}>{t(latest.pill.key)}</StatusPill>
              </>
            }
          />
        </SettingsCard>
      </div>
      {updater?.lastCheck && <div role="status" aria-label={t("updater.page.lastCheck")} className="flex flex-wrap items-center gap-2 text-sm">
        <span>{new Date(updater.lastCheck.checkedAt).toLocaleString(locale)} · {t(`updater.page.result.${updater.lastCheck.status}`)}</span>
        {updater.lastCheck.errorCode && <span>{updater.lastCheck.errorCode}</span>}
        {updater.lastCheck.operationId && <span>{updater.lastCheck.operationId}</span>}
        {updater.lastCheck.status === "failed" && <>
          <Button size="sm" variant="outline" disabled={!ready || state.phase === "checking"} onClick={updater.onCheck}>{t("updater.row.retry")}</Button>
          {updater.onDiagnostics && <Button size="sm" variant="ghost" onClick={updater.onDiagnostics}>{t("network.title")}</Button>}
        </>}
      </div>}
      {updater && updateNeedsAttention(state) && (
        <div ref={installCard}>
          <SettingsCard title={t("updater.page.installCard")}>
            <UpdateInstallRow
              state={state}
              onAccept={updater.onAccept}
              onCancel={updater.onCancel}
              onRelaunch={updater.onRelaunch}
              onRetry={updater.onCheck}
            />
          </SettingsCard>
        </div>
      )}
      <SettingsCard title={t("releaseNotes.open")}>
        {newest && (
          <SettingsRow
            label={<VersionText>{newest.version}</VersionText>}
            description={newest.date}
            control={null}
          >
            <div className="flex flex-col gap-2">
              {newest.sections.map((section) => (
                <div key={section.kind} className="flex flex-col gap-1">
                  <span className="text-xs font-medium">{t(`releaseNotes.kind.${section.kind}`)}</span>
                  <ul className="m-0 flex list-disc flex-col gap-0.5 pl-5 text-[13px] text-muted-foreground">
                    {section.items.map((item, i) => (
                      <li key={i}>{entryText(item, locale)}</li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          </SettingsRow>
        )}
        {older.map((entry) => (
          <SettingsRow
            key={entry.version}
            label={<VersionText>{entry.version}</VersionText>}
            description={
              <span className="line-clamp-1">
                {entry.date} · {entry.sections[0] ? entryText(entry.sections[0].items[0], locale) : ""}
              </span>
            }
            control={
              <Button type="button" variant="ghost" size="sm" onClick={() => showNotes(entry.version)}>
                {t("updater.page.viewNotes")}
              </Button>
            }
          />
        ))}
        <div className="flex justify-end px-4 py-2">
          <Button type="button" variant="ghost" size="sm" onClick={() => showNotes()}>
            {t("updater.page.allNotes")}
          </Button>
        </div>
      </SettingsCard>
      <SettingsCard title={t("updater.page.autoCheckCard")}>
        <SettingsRow
          label={autoCheckLabel}
          description={t("updater.page.autoCheckHint")}
          control={
            <Switch
              aria-label={autoCheckLabel}
              checked={updater?.autoCheck ?? true}
              disabled={!ready}
              onCheckedChange={updater?.onAutoCheckChange}
            />
          }
        />
      </SettingsCard>
      <ReleaseNotesDialog open={notes !== null} mode="browse" entries={RELEASE_NOTES} focusVersion={notes?.version} onOpenChange={(open) => { if (!open) setNotes(null); }} />
    </>
  );
}
