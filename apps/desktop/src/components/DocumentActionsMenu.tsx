// 詳情頁內容卡標頭右端的「開啟 ▾」（design D4）：對目前分頁的檔案提供以預設 App
// 開啟、在檔案管理員顯示、複製路徑；remote 無本機 checkout 時只剩「複製內容」。
import { useRef, useState } from "react";
import { ChevronDown, Copy, ExternalLink, FileText, FolderOpen } from "lucide-react";
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
  useI18n,
  type DocumentRef,
  type DocumentTarget,
} from "@speclink/ui";

import type { DocumentLocation } from "../adapter/fsActions";
import type { Platform } from "../platform";

export function DocumentActionsMenu({
  target,
  local,
  platform,
  locate,
  onOpenDefault,
  onReveal,
  onCopyPath,
  onCopyContent,
}: {
  /** 分頁對應的文件與已載入原文；null＝此分頁沒有檔案（排程），整個選單缺席。 */
  target: DocumentTarget | null;
  /** 專案有本機目錄（local 或已綁 checkout 的 remote）。 */
  local: boolean;
  platform: Platform;
  /** 文件的磁碟位置（宿主向 core 查詢）；找不到為 null。 */
  locate: (doc: DocumentRef) => Promise<DocumentLocation | null>;
  onOpenDefault: (location: DocumentLocation) => void;
  onReveal: (location: DocumentLocation) => void;
  onCopyPath: (location: DocumentLocation) => void;
  onCopyContent: (text: string) => void;
}) {
  const { t } = useI18n();
  // 展開時才查位置（文件可能剛被封存搬走）；序號擋掉較早一次展開的過期回應。
  const [location, setLocation] = useState<DocumentLocation | null>(null);
  const seq = useRef(0);
  if (target === null) return null;
  // remote 無本機目錄時唯一的動作是複製原文；分頁不載原文（工單）就沒有可做的事。
  if (!local && target.content === undefined) return null;
  const { doc, content } = target;

  const onOpenChange = (open: boolean) => {
    if (!open || !local) return;
    const mine = ++seq.current;
    setLocation(null);
    void locate(doc).then((found) => {
      if (seq.current === mine) setLocation(found);
    });
  };

  return (
    <DropdownMenu onOpenChange={onOpenChange}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="gap-1 text-muted-foreground">
          <FileText className="h-3.5 w-3.5" />
          {t("detail.open")}
          <ChevronDown className="h-3.5 w-3.5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {local ? (
          <>
            {location && (
              <DropdownMenuLabel className="max-w-80 truncate font-mono text-[11px] text-muted-foreground">
                {location.relative}
              </DropdownMenuLabel>
            )}
            <DropdownMenuItem disabled={!location} onSelect={() => location && onOpenDefault(location)}>
              <ExternalLink />
              <span>{t("detail.openDefault")}</span>
            </DropdownMenuItem>
            <DropdownMenuItem disabled={!location} onSelect={() => location && onReveal(location)}>
              <FolderOpen />
              <span>{t(`fs.revealIn.${platform}`)}</span>
            </DropdownMenuItem>
            <DropdownMenuItem disabled={!location} onSelect={() => location && onCopyPath(location)}>
              <Copy />
              <span>{t("fs.copyPath")}</span>
            </DropdownMenuItem>
          </>
        ) : (
          <DropdownMenuItem
            disabled={content === null}
            onSelect={() => {
              if (typeof content === "string") onCopyContent(content);
            }}
          >
            <Copy />
            <span>{t("detail.copyContent")}</span>
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
