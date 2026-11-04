// 右邊那一欄每個 space 各自記住開或關，做法和內建的 Files／Changes 相同：切 space 就還原它的選擇，
// 手動開關就記下來。沒記過的 space 維持打開。
//
// 這件事不能放在那一欄的面板裡：收起來後面板不會重畫，Herdr 重開時停在收起的 space 面板還不會啟動。
// 所以 dock-panel.mjs 在版面放一個不畫東西的浮動區塊，它一直活著，由它來做。
import { div } from "gpui-kit";
import { layout, run, spaces } from "herdr";

const COLUMN = "speclink";

const findNode = (node, id) =>
  node?.id === id ? node : (node?.children ?? []).map((c) => findNode(c, id)).find(Boolean);

export default class ColumnSync {
  init() {
    this.open = JSON.parse(localStorage.getItem("columnOpen") ?? "{}");
  }

  render() {
    this.sync();
    return div();
  }

  sync() {
    const space = spaces().find((s) => s.active);
    const column = findNode(layout(), COLUMN);
    if (!space || !column) return;
    const open = !column.hidden;
    if (space.id !== this.space) {
      this.space = space.id;
      const want = this.open[space.id] ?? true;
      if (want !== open) {
        this.pending = { want, until: Date.now() + 1000 };
        run(want ? "layout.show" : "layout.hide", { id: COLUMN });
        return;
      }
    }
    // 還原還沒生效前，舊的開關狀態不算使用者的選擇。
    if (this.pending) {
      if (this.pending.want !== open && Date.now() < this.pending.until) return;
      this.pending = null;
    }
    if ((this.open[space.id] ?? true) === open) return;
    this.open[space.id] = open;
    localStorage.setItem("columnOpen", JSON.stringify(this.open));
  }
}
