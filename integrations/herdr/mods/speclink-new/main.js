// 這個資料夾只讓 mod.open:speclink-new 開得到技能跳窗，畫面在 speclink mod 的 skill-picker.js。
// 一個 mod 只能設定一種 overlay，Herdr 也不讓 mod 引用別的資料夾的程式，所以用 mod.json 指過去。
import { View, div } from "gpui-kit";

export default class SpeclinkNew extends View {
  render() {
    return div();
  }
}
