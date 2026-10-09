// 專案欄（design D4）：每專案的四頁＋底部專案設定；點擊即切頁（無 toggle），
// 設定模式下全部非作用中（作用中的是圖示列齒輪）。底部的 notice 槽放技能檔提示卡
// （desktop-notice-relocation design D1）：提示是這個專案的事，住在專案欄、不佔主區。
import type { ReactNode } from "react";
import { Archive, BookOpen, FileText, GitBranch, SlidersHorizontal } from "lucide-react";
import { NavItem, useI18n } from "@speclink/ui";

import type { BoardView } from "../store";

export function ProjectColumn({
  boardView,
  archivedCount,
  onNavigate,
  notice,
}: {
  boardView: BoardView;
  archivedCount: number;
  onNavigate: (view: BoardView) => void;
  /** 彈性空白與專案設定項之間的提示卡（null＝無）。 */
  notice?: ReactNode;
}) {
  const { t } = useI18n();
  return (
    <nav aria-label={t("app.projectColumn")} className="flex w-[200px] shrink-0 flex-col gap-0.5 p-2 pb-3">
      <NavItem
        icon={<GitBranch />}
        label={t("app.navChanges")}
        active={boardView === "board"}
        onClick={() => onNavigate("board")}
      />
      <NavItem
        icon={<Archive />}
        label={t("app.archived")}
        ariaLabel={t("app.archived")}
        count={archivedCount}
        active={boardView === "archived"}
        onClick={() => onNavigate("archived")}
      />
      <NavItem
        icon={<FileText />}
        label={t("app.navSpecs")}
        active={boardView === "specs"}
        onClick={() => onNavigate("specs")}
      />
      <NavItem
        icon={<BookOpen />}
        label={t("app.navManual")}
        ariaLabel={t("app.navManual")}
        active={boardView === "manual"}
        onClick={() => onNavigate("manual")}
      />
      <div className="mt-auto flex flex-col gap-2">
        {notice}
        <NavItem
          icon={<SlidersHorizontal />}
          label={t("app.navProjectSettings")}
          active={boardView === "project-settings"}
          onClick={() => onNavigate("project-settings")}
        />
      </div>
    </nav>
  );
}
