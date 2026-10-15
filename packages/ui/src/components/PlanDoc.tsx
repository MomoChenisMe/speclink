import type { ReactElement, ReactNode } from "react";
import { X } from "lucide-react";

import type { ChangeItem, DeltaOperation } from "../adapter";
import { useI18n, type Translator } from "../i18n";
import {
  planArchiveAfter,
  planBlockedBy,
  planBlockedLabel,
  planRequirementOverlap,
  planWave,
  planWaveLabel,
} from "../stage";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Callout } from "./ui/callout";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { DELTA_COLORS, DELTA_LABEL_KEYS } from "./DeltaBadges";
import type { DocSummary } from "./DetailShell";
import { NAME_CHIP_CLS, SECTION_HEADING_CLS } from "./DocParts";

/** 排程文件內的名稱籤（同波夥伴、前置、重疊、封存順序共用；與能力名籤同款）。 */
function PlanName({ name }: { name: string }) {
  return <span className={NAME_CHIP_CLS}>{name}</span>;
}

/** 重疊卡的 delta 操作小標籤：用詞與配色和規格文件的 delta 區段同源；title 標出屬於哪一方。 */
function OpTag({ op, owner }: { op: DeltaOperation; owner: string }) {
  const { t } = useI18n();
  const key = op.toLowerCase() as Lowercase<DeltaOperation>;
  return (
    <span title={owner} className={`rounded border border-border/60 px-1 py-0.5 text-[10px] ${DELTA_COLORS[key]}`}>
      {t(DELTA_LABEL_KEYS[key])}
    </span>
  );
}

/** 前置卡與閱讀卡籤槽的同一枚狀態徽章：blockedBy 空為「可以開工」、非空為「等 N 項」（title 列出前置名）。 */
function PlanStatusBadge({ blockedBy, t }: { blockedBy: string[]; t: (key: string) => string }) {
  return blockedBy.length === 0 ? (
    <Badge variant="secondary">{t("plan.canStart")}</Badge>
  ) : (
    <Badge variant="outline" title={planBlockedLabel(blockedBy, t)}>
      {t("plan.blockedCount").replace("{n}", String(blockedBy.length))}
    </Badge>
  );
}

/** 排程文件的一張卡片：外觀與工單文件的輪卡同一種邊框卡片，卡頭（16px 標題、右側可放籤）與內容區以分隔線分格。 */
function PlanCard({
  section,
  title,
  badge,
  children,
}: {
  section: "wave" | "depends" | "overlaps" | "archive";
  title: string;
  badge?: ReactElement;
  children: ReactNode;
}) {
  return (
    <section data-plan-section={section} className="rounded-md border border-border/60 bg-muted/20">
      <div data-plan-card-header className="flex items-center justify-between gap-2 px-3 py-2">
        <h3 data-doc-heading className={SECTION_HEADING_CLS}>
          {title}
        </h3>
        {badge}
      </div>
      <div data-plan-card-body className="border-t border-border/60 px-3 py-2 text-sm">
        {children}
      </div>
    </section>
  );
}

/** 卡頭右端的筆數籤（重疊卡、封存順序卡）。 */
const countBadge = (n: number) => (
  <Badge variant="secondary" className="tabular-nums">
    {n}
  </Badge>
);

/**
 * 排程文件（spec desktop-app「詳情頁的排程分頁」）：波次（同波夥伴）、前置（可增減，
 * 標題列的狀態徽章取代獨立的阻擋段）、重疊（唯讀）、封存順序四張卡片。清單項缺 wave
 * 時：remote 摘要（亦缺 dependsOn）只剩一句說明；plan 成環（payload 仍帶 dependsOn
 * 原文）另渲染前置卡（無徽章）、只能移除——解環的出口。判定沿 stage.ts 的 planWave／
 * planBlockedBy／planRequirementOverlap／planArchiveAfter 單一入口。
 */
export function PlanDoc({
  change,
  changes,
  roster,
  onSetDepends,
}: {
  change: ChangeItem;
  changes: ChangeItem[];
  /** 候選名冊：陣列＝查詢結果、null＝自清單派生、undefined＝查詢中（不給候選）。 */
  roster: string[] | null | undefined;
  onSetDepends?: (change: string, on: string[], remove: boolean) => void;
}) {
  const { t } = useI18n();
  const wave = planWave(change);
  const dependsOn = change.dependsOn ?? [];
  // 前置卡內文（成環與一般兩個分支共用）：宣告的前置每項一列，空時「尚無前置」。
  const dependsBody = (
    <>
      {dependsOn.length === 0 && <div className="text-muted-foreground">{t("plan.noDepends")}</div>}
      <ul className="flex flex-col gap-1">
        {dependsOn.map((n) => (
          <li key={n} className="flex items-center gap-2">
            <PlanName name={n} />
            {onSetDepends && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                aria-label={t("plan.remove").replace("{name}", n)}
                onClick={() => onSetDepends(change.name, [n], true)}
              >
                <X className="h-3.5 w-3.5" />
              </Button>
            )}
          </li>
        ))}
      </ul>
    </>
  );
  if (wave === null) {
    return (
      <div className="flex flex-col gap-3 text-sm">
        <div className="text-muted-foreground py-6">{t(change.dependsOn ? "plan.cycle" : "plan.unavailable")}</div>
        {change.dependsOn && (
          <PlanCard section="depends" title={t("plan.depends")}>
            {dependsBody}
          </PlanCard>
        )}
      </div>
    );
  }
  const requirementOverlaps = planRequirementOverlap(change);
  const archiveAfter = planArchiveAfter(change);
  const blockedBy = planBlockedBy(change);
  // 舊 server 的 blockedBy 另含先配置、共用 capability 的變更（本機與新 server 只含宣告
  // 前置）：列出 dependsOn 以外的名稱，徽章的數字才對得上卡片內容。
  const undeclaredWaits = blockedBy.filter((n) => !dependsOn.includes(n));
  const others = changes.filter((c) => c.name !== change.name);
  const mates = others.filter((c) => planWave(c) === wave).map((c) => c.name);
  // 新增候選：該變更所在名冊的其他作用中變更（add-change-plan-remote D9；未提供名冊時
  // 自清單派生），扣掉自己與已宣告的前置（成環留給引擎拒絕）。
  const pool = roster === undefined ? [] : (roster ?? others.map((c) => c.name));
  const candidates = pool.filter((n) => n !== change.name && !dependsOn.includes(n));
  // 封存卡的兩種警示：同名衝突（雙方都帶進或都拿走，先後救不了）與互相等待（雙方互列
  // archiveAfter，封存互卡）——兩者引擎都不會讓任一方「可直接封存」。
  const conflictWith = conflictNames(change);
  const mutualWaits = archiveAfter.filter((n) =>
    others.some((c) => c.name === n && planArchiveAfter(c).includes(change.name)),
  );
  const names = (list: string[]) => list.join(t("common.listSeparator"));
  return (
    <div className="flex flex-col gap-3 text-sm">
      <PlanCard section="wave" title={t("plan.waveLabel")}>
        <div className="font-medium">{planWaveLabel(wave, t)}</div>
        {mates.length === 0 ? (
          <div className="text-muted-foreground">{t("plan.onlyOne")}</div>
        ) : (
          <div className="mt-1 flex flex-wrap items-center gap-1.5">
            <span className="text-muted-foreground">{t("plan.waveMates")}</span>
            {mates.map((n) => (
              <PlanName key={n} name={n} />
            ))}
          </div>
        )}
      </PlanCard>
      <PlanCard section="depends" title={t("plan.depends")} badge={<PlanStatusBadge blockedBy={blockedBy} t={t} />}>
        {dependsBody}
        {undeclaredWaits.length > 0 && (
          <div data-plan-undeclared-waits className="mt-1 flex flex-wrap items-center gap-1.5">
            <span className="text-muted-foreground">{t("plan.undeclaredWaits")}</span>
            {undeclaredWaits.map((n) => (
              <PlanName key={n} name={n} />
            ))}
          </div>
        )}
        {onSetDepends && candidates.length > 0 && (
          // 只當挑選器用：value 固定為空，選擇即寫入、trigger 回到提示文字。
          <Select value="" onValueChange={(name) => onSetDepends(change.name, [name], false)}>
            <SelectTrigger aria-label={t("plan.add")} className="mt-2 w-full">
              <SelectValue placeholder={t("plan.add")} />
            </SelectTrigger>
            <SelectContent>
              {candidates.map((n) => (
                <SelectItem key={n} value={n}>
                  {n}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </PlanCard>
      <PlanCard section="overlaps" title={t("plan.overlaps")} badge={countBadge(requirementOverlaps.length)}>
        {requirementOverlaps.length === 0 && <div className="text-muted-foreground">{t("plan.noOverlaps")}</div>}
        <ul className="flex flex-col gap-1">
          {requirementOverlaps.map((o) => (
            <li key={`${o.change}/${o.capability}/${o.requirement}`} className="flex flex-wrap items-center gap-1.5">
              <PlanName name={o.change} />
              <span className="text-xs">{`${o.capability} › ${o.requirement}`}</span>
              {/* 雙方操作：本變更在前、對方在後。 */}
              <OpTag op={o.ownOperation} owner={change.name} />
              <OpTag op={o.otherOperation} owner={o.change} />
              {o.conflict && (
                <span className="rounded border border-destructive/60 px-1 py-0.5 text-[10px] text-destructive">
                  {t("plan.conflict")}
                </span>
              )}
            </li>
          ))}
        </ul>
      </PlanCard>
      <PlanCard section="archive" title={t("plan.archiveAfter")} badge={countBadge(archiveAfter.length)}>
        {archiveAfter.length === 0 && conflictWith.length === 0 && (
          <div className="text-muted-foreground">{t("plan.noArchiveAfter")}</div>
        )}
        {archiveAfter.length > 0 && (
          <>
            <div className="flex flex-wrap items-center gap-1.5">
              {archiveAfter.map((n) => (
                <PlanName key={n} name={n} />
              ))}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">{t("plan.archiveHint")}</p>
          </>
        )}
        {mutualWaits.length > 0 && (
          <p className="mt-1 text-xs text-destructive">{t("plan.archiveDeadlock").replace("{names}", names(mutualWaits))}</p>
        )}
        {conflictWith.length > 0 && (
          <p className="mt-1 text-xs text-destructive">{t("plan.archiveConflict").replace("{names}", names(conflictWith))}</p>
        )}
      </PlanCard>
    </div>
  );
}

/** 同名衝突的對方變更名（去重）。 */
function conflictNames(change: ChangeItem): string[] {
  return [...new Set(planRequirementOverlap(change).filter((o) => o.conflict).map((o) => o.change))];
}

/**
 * 排程文件的籤、數字磚與兩態提示框（spec「詳情頁的排程分頁」；design D7）：缺 wave 時全空。提示框標題
 * 由兩個子句組成——前置（可以開工／等 N 項前置）與封存（也可直接封存／封存前要等 M 個變更／有同名
 * 衝突要先改）；兩子句皆綠為 tip，否則 warning。
 */
export function planSummary(change: ChangeItem, { t, tn }: Translator): DocSummary {
  const wave = planWave(change);
  if (wave === null) return { badges: null, tiles: [], callout: null };
  const blockedBy = planBlockedBy(change);
  const archiveAfter = planArchiveAfter(change);
  const conflicts = conflictNames(change);
  const startClause =
    blockedBy.length === 0 ? t("plan.calloutStart") : tn("plan.calloutBlocked", blockedBy.length);
  const archiveClause =
    conflicts.length > 0
      ? t("plan.calloutConflict")
      : archiveAfter.length > 0
        ? tn("plan.calloutArchiveWait", archiveAfter.length)
        : t(blockedBy.length === 0 ? "plan.calloutArchiveFree" : "plan.noArchiveAfter");
  const green = blockedBy.length === 0 && archiveAfter.length === 0 && conflicts.length === 0;
  const lines = [
    { label: "plan.depends", names: blockedBy },
    { label: "plan.archiveAfter", names: archiveAfter },
    { label: "plan.conflict", names: conflicts },
  ].filter((l) => l.names.length > 0);
  return {
    badges: <PlanStatusBadge blockedBy={blockedBy} t={t} />,
    tiles: [
      { value: wave, label: t("plan.tileWave") },
      { value: (change.dependsOn ?? []).length, label: t("plan.tileDepends") },
      { value: planRequirementOverlap(change).length, label: t("plan.tileOverlaps") },
      { value: archiveAfter.length, label: t("plan.tileArchiveAfter") },
    ],
    callout: (
      <Callout
        tone={green ? "tip" : "warning"}
        title={`${startClause}${t("plan.calloutJoin")}${archiveClause}`}
        className="my-0"
      >
        {lines.map((l) => (
          <div key={l.label} className="my-1 flex flex-wrap items-center gap-1.5 text-sm">
            <span className="text-muted-foreground">{t(l.label)}</span>
            {l.names.map((n) => (
              <PlanName key={n} name={n} />
            ))}
          </div>
        ))}
        <p className="text-sm text-muted-foreground">{t("plan.calloutRecompute")}</p>
      </Callout>
    ),
  };
}
