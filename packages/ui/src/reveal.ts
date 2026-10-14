/** 列滑過才出現的控制項（複製鈕、server 後台的撤銷鈕）：平時透明，所在 `group` 滑過或鍵盤聚焦時出現；
 *  沒有滑鼠移入能力的裝置恆常顯示。只改不透明度，隱藏期間仍在 Tab 順序內、螢幕閱讀器讀得到。 */
export const REVEAL_ON_GROUP_HOVER =
  "opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100 [@media(hover:none)]:opacity-100";
