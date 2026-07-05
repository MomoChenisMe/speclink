import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** 合併 Tailwind class：clsx 條件組合 + tailwind-merge 消解衝突。shadcn 標準工具。 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
