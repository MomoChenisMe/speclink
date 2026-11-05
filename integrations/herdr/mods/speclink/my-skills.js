// 技能跳窗的「我的技能」頁：使用者自己裝的技能，依用途分組。這些技能不搭配變更，所以整頁是一份清單，
// 選中的列展開說明。放到輸入框和開新分頁由 skill-picker.js 的底部按鈕負責，規則在 model.js。
import { div, list } from "gpui-kit";
import { h_flex, v_flex, Input, InputState, Scrollbar, VirtualListScrollHandle } from "gpui-base";
import { agents, focused, panes, terminals } from "herdr";
import { customPlacePlan, customTabPlan, joinAgents, skillRows } from "./model.js";
import { tr } from "./i18n.js";
import { readSkills } from "./shared.js";

const SOURCE_TAG = { project: "專案", personal: "個人", plugin: "外掛" };
const NAME_WIDTH = 190;
const NESTED_INDENT = 14;

export default class MySkillsPage {
  constructor(host) {
    this.host = host;
    this.lists = new Map();
    this.cursor = null;
    this.groupOpen = {};
    this.familyOpen = {};
    this.inNote = false;
    this.scroll = VirtualListScrollHandle.new();
    this.search = InputState.new({ placeholder: tr("找技能：名稱或說明") });
    this.note = InputState.new();
    // 有輸入框時 Enter 由輸入框的 submit 處理。
    const submit = (e, cx) => this.submit(e, cx);
    this.search.on("change", (_e, cx) => {
      this.cursor = null;
      cx.notify();
    });
    this.search.on("submit", submit);
    this.search.on("focus", () => (this.inNote = false));
    this.note.on("change", (_e, cx) => cx.notify());
    this.note.on("submit", submit);
    this.note.on("focus", () => (this.inNote = true));
  }

  load(key, cx) {
    if (!key || this.lists.has(key)) return;
    this.lists.set(key, null);
    cx.spawn(async (cx) => {
      const found = await readSkills(key, this.host.root, this.host.on).catch((e) => ({
        skills: [],
        hasSpeclink: true,
        error: String(e.message ?? e),
      }));
      this.lists.set(key, found);
      cx.notify();
    });
  }

  found() {
    return this.lists.get(this.host.agentKey) ?? null;
  }

  rows() {
    const skills = this.found()?.skills ?? [];
    return skillRows({ skills, query: this.search.value(), groupOpen: this.groupOpen, familyOpen: this.familyOpen });
  }

  current(rows = this.rows()) {
    return rows.find((r) => r.key === this.cursor) ?? rows.find((r) => r.kind === "skill") ?? rows[0] ?? null;
  }

  focus() {
    (this.inNote ? this.note : this.search).focus();
  }

  toggle(row) {
    if (row.kind === "group") this.groupOpen[row.id] = !row.open;
    else this.familyOpen[row.id] = !row.open;
    this.cursor = row.key;
  }

  submit(e, cx) {
    const row = this.current();
    if (row && row.kind !== "skill") {
      this.toggle(row);
      cx.notify();
      return;
    }
    if (e.secondary) this.host.startTabs(cx);
    else this.host.putInInput(cx);
  }

  move(step) {
    const rows = this.rows();
    if (rows.length === 0) return;
    const next = (rows.indexOf(this.current(rows)) + step + rows.length) % rows.length;
    this.cursor = rows[next].key;
    this.scroll.scroll_to_item(next);
  }

  // 回傳有沒有處理這個按鍵。輸入框有字時，左右鍵留給輸入框移動游標。
  onKey(e) {
    if (e.key === "down") this.move(1);
    else if (e.key === "up") this.move(-1);
    else if (e.key === "right" && !this.inNote && !this.search.value()) this.note.focus();
    else if (e.key === "left" && this.inNote && !this.note.value()) this.search.focus();
    else return false;
    return true;
  }

  selection() {
    const row = this.current();
    return row?.kind === "skill" ? { name: row.skill.name, note: this.note.value().trim() } : null;
  }

  tabPlan() {
    const picked = this.selection();
    if (!picked) return { kind: "none", reason: tr("選一個技能") };
    const { root, known, agentKey, on } = this.host;
    return customTabPlan({ ...picked, root, known, agentKey, on, space: focused()?.space });
  }

  inputPlan() {
    const picked = this.selection();
    if (!picked) return { kind: "none", reason: tr("選一個技能") };
    const f = focused();
    return customPlacePlan({
      ...picked,
      agentKey: this.host.agentKey,
      agents: joinAgents(agents(), terminals(), panes()),
      focus: { tab: f?.tab, terminal: f?.terminal },
    });
  }

  groupRow(row, selected, t) {
    return h_flex()
      .h(24)
      .mx(6)
      .px(8)
      .gap(6)
      .items_center()
      .rounded(5)
      .when(selected, (el) => el.bg(t.selected))
      .when(!selected, (el) => el.hover((s) => s.bg(t.hover)))
      .child(div().w(7).h(7).flex_none().rounded_full().bg(t[row.group.tone]))
      .child(div().text_size(11).font_medium().text_color(t.muted).child(tr(row.group.label)))
      .child(div().text_size(11).text_color(t.dim).child(String(row.count)))
      .child(div().flex_1())
      .child(this.host.chevron(row.open, t));
  }

  familyRow(row, selected, t) {
    return h_flex()
      .mx(6)
      .px(8)
      .py(4)
      .gap(10)
      .items_center()
      .rounded(5)
      .when(selected, (el) => el.bg(t.selected))
      .when(!selected, (el) => el.hover((s) => s.bg(t.hover)))
      .child(
        h_flex()
          .flex_none()
          .w(NAME_WIDTH)
          .gap(5)
          .items_center()
          .child(this.host.chevron(row.open, t))
          .child(div().text_size(12).when(selected, (el) => el.font_medium()).child(row.name))
          .child(div().text_size(11).text_color(t.dim).child(tr("{0} 個", row.count))),
      )
      .child(div().flex_1().min_w_0().truncate().text_size(11.5).text_color(t.dim).child(row.hint));
  }

  skillRow({ skill, nested }, selected, t) {
    const indent = nested ? NESTED_INDENT : 0;
    const tag = div().flex_none().text_size(10.5).text_color(t.dim).child(tr(SOURCE_TAG[skill.source]));
    const name = div()
      .flex_none()
      .w(NAME_WIDTH - indent)
      .truncate()
      .text_size(12)
      .when(selected, (el) => el.font_medium())
      .child(skill.name);
    const line = h_flex().gap(10).items_center().pl(indent).child(name);
    if (!selected) {
      return v_flex()
        .mx(6)
        .px(8)
        .py(4)
        .rounded(5)
        .hover((s) => s.bg(t.hover))
        .child(line.child(div().flex_1().min_w_0().truncate().text_size(11.5).text_color(t.muted).child(skill.about)).child(tag));
    }
    const where = skill.source === "plugin" ? tr("外掛 {0}", skill.path) : skill.path;
    return v_flex()
      .mx(6)
      .px(8)
      .py(4)
      .gap(3)
      .rounded(5)
      .bg(t.selected)
      .child(line.child(div().flex_1()).child(tag))
      .when(Boolean(skill.about), (el) => el.child(div().pl(indent).text_size(11.5).text_color(t.muted).child(skill.about)))
      .child(div().pl(indent).truncate().text_size(11).text_color(t.dim).child(where));
  }

  emptyText(found) {
    if (!found) return tr("載入中…");
    if (found.error) return found.error;
    if (found.skills.length === 0) return tr("{0} 目前沒有 Speclink 以外的技能", this.host.agentName());
    return tr("找不到符合的技能");
  }

  render(t) {
    const rows = this.rows();
    const selected = this.current(rows);
    const rowEl = (row) => {
      const on = row === selected;
      if (row.kind === "group") return this.groupRow(row, on, t);
      if (row.kind === "family") return this.familyRow(row, on, t);
      return this.skillRow(row, on, t);
    };
    return v_flex()
      .flex_1()
      .min_h(0)
      .child(
        h_flex()
          .flex_none()
          .h(38)
          .px(14)
          .gap(8)
          .items_center()
          .border_b_1()
          .border_color(t.border)
          .child(this.host.searchIcon(t))
          .child(Input.new(this.search).flex_1().h_full().text_size(12.5).text_color(t.foreground)),
      )
      .child(
        div()
          .flex_1()
          .min_h(0)
          .relative()
          .py(6)
          .when(rows.length === 0, (el) => el.child(div().px(14).py(4).text_size(12).text_color(t.muted).child(this.emptyText(this.found()))))
          .when(rows.length > 0, (el) =>
            el
              .child(
                list("my-skills", rows.length, (i) => rows[i].key, (i) => rowEl(rows[i]))
                  .size_full()
                  .track_scroll(this.scroll)
                  .on_item_click((key, cx) => {
                    const row = rows.find((r) => r.key === key);
                    if (!row) return;
                    if (row.kind === "skill") this.cursor = key;
                    else this.toggle(row);
                    this.focus();
                    cx.notify();
                  }),
              )
              .child(Scrollbar.vertical("my-skills").absolute().inset_0()),
          ),
      )
      .child(
        h_flex()
          .flex_none()
          .h(42)
          .px(14)
          .gap(10)
          .items_center()
          .border_t_1()
          .border_color(t.border)
          .child(div().flex_none().text_size(11).text_color(t.dim).child(tr("補充說明（可留空）")))
          .child(
            Input.new(this.note)
              .flex_1()
              .h(28)
              .px(8)
              .rounded(6)
              .border_1()
              .border_color(this.inNote ? t.accent : t.border)
              .bg(t.background)
              .text_size(12)
              .text_color(t.foreground),
          ),
      );
  }
}
