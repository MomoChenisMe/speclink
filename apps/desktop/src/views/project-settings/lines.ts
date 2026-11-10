// 行↔條目轉換（design D2）：一行一條——儲存時逐行修剪頭尾空白、空行滌除，行序即寫入順序。
// 產出規則、審查規則與子代理模型的自訂清單共用。

export const entriesToText = (entries: string[]) => entries.join("\n");

export const textToEntries = (text: string) =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");

/** 草稿文字→條目，沒編輯的節原樣沿用現值（review R3）：檔案裡手寫的多行條目顯示成多行，
 * 重新拆行會變成多條——只有使用者改過的節才照「一行一條」重切。 */
export const keepUnedited = (text: string, entries: string[]) =>
  text === entriesToText(entries) ? entries : textToEntries(text);
