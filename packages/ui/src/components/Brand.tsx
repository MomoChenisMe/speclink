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

/** 橫式鎖版字標：深色時取深色版；高度由呼叫端給（例如 h-5）。兩版各一張、以 dark: 切換——
 *  dark: 與深色 token 同一套觸發（theme.css），明示外觀也跟得上；<picture media> 只看系統偏好。
 *  隱藏的那張 display:none，不進無障礙樹。 */
export function Wordmark({ className }: { className?: string }) {
  return (
    <>
      <img src={horizontal} alt="Speclink" className={cn("w-auto dark:hidden", className)} />
      <img src={horizontalDark} alt="Speclink" className={cn("hidden w-auto dark:block", className)} />
    </>
  );
}
