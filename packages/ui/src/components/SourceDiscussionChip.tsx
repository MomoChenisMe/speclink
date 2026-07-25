import { Button } from "./ui/button";

/**
 * 來源討論標記：變更詳情抽屜與已封存抽屜共用的單一實作。
 *
 * 討論 topic 是自由文字、可長達整句話。Button 為 inline-flex 且帶 whitespace-nowrap，
 * 截斷必須落在內層區塊子項（text-overflow 不作用於 flex 容器本身），外層則需
 * max-w-full＋min-w-0 才會被抽屜寬度約束——否則單一標記寬於容器，父層的 flex-wrap
 * 也救不了（換行只能在項目之間斷開）。
 */
export function SourceDiscussionChip({
  topic,
  onClick,
}: {
  topic: string;
  onClick?: () => void;
}) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      title={topic}
      className="h-auto min-w-0 max-w-full rounded-full bg-primary/10 px-2 py-0.5 font-medium text-primary hover:bg-primary/20 hover:text-primary"
      onClick={onClick}
    >
      <span data-source-discussion-label className="truncate">
        {topic}
      </span>
    </Button>
  );
}
