// 使用者的顯示文字：帳號選單、裝置核准的身分句與帳號頁說明共用同一條規則。
type Person = { display: string; email: string };

/** 顯示名稱；空字串時退回 email，頭像字與名稱列都不留白。 */
export function userLabel(user: Person): string {
  return user.display || user.email;
}

/** 一次代入 `{display}`／`{email}`：值以字面文字放入（`$&` 之類不展開），也不會被下一個佔位符再掃到。 */
export function fillUser(template: string, user: Person): string {
  return template.replace(/\{(display|email)\}/g, (_, key: string) => (key === "email" ? user.email : userLabel(user)));
}
