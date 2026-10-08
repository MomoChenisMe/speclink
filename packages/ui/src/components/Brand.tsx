// 品牌標記與字標（spec desktop-app「共用元件唯一來源」；design D8）：資產只在
// packages/ui/src/assets/ 一處，換 Logo 只換那三個 SVG。
import mark from "../assets/logo-mark.svg?raw";
import horizontal from "../assets/logo-horizontal.svg";
import horizontalDark from "../assets/logo-horizontal-dark.svg";
import { cn } from "../lib/utils";

/**
 * 裸標記：SVG 以 currentColor 內嵌，顏色跟 text-primary 走（淺色主色 teal、深色淺青綠）。
 * `mark` 是 `?raw` 匯入的建置期常數，不經任何執行期輸入，可安全內嵌。
 */
export function BrandMark({ size = 26, className }: { size?: number; className?: string }) {
  return (
    <span
      role="img"
      aria-label="Speclink"
      className={cn("inline-block text-primary [&>svg]:h-full [&>svg]:w-auto", className)}
      style={{ height: size }}
      dangerouslySetInnerHTML={{ __html: mark }}
    />
  );
}

/** 橫式鎖版字標：系統偏好深色時取深色版；高度由呼叫端給（例如 h-5）。 */
export function Wordmark({ className }: { className?: string }) {
  return (
    <picture>
      <source media="(prefers-color-scheme: dark)" srcSet={horizontalDark} />
      <img src={horizontal} alt="Speclink" className={cn("w-auto", className)} />
    </picture>
  );
}
