import logoMark from "../assets/logo-mark.png";
import wordmark from "../assets/speclink-wordmark.png";

// Speclink 品牌 lockup：與 Desktop 頂欄同款的圖片標記＋字標（h-5）。PNG 經 Vite
// bundle 進 `/assets/`（內容雜湊、同源），符合 shell CSP 的 img-src 'self'。
export function Wordmark() {
  return (
    <span className="flex items-center gap-1.5">
      <img src={logoMark} alt="" aria-hidden="true" className="h-5 w-5" />
      <img src={wordmark} alt="Speclink" className="h-5 w-auto" />
    </span>
  );
}
