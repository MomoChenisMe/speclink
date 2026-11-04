// 技能跳窗：選技能和搭配的變更或討論。Enter 把指令貼到目前分頁的輸入框，不送出。
// 勾選「開新分頁」時改成 herdr-dispatch 技能的做法：用 hc run 開新分頁並開工，變更可以複選。
// 由 speclink-new 的 mod.json 開啟，main.js 看到 props 的 view 就把畫面交給這裡。
import { div, svg } from "gpui-kit";
import { h_flex, v_flex, InputState, Input } from "gpui-base";
import { agents, closeOverlay, focused, hc, knownAgents, panes, props, run, shared, terminals, theme } from "herdr";
import {
  PICKER_SKILLS,
  batchPlan,
  buildBoard,
  isUninitialized,
  joinAgents,
  newTabPlan,
  placePlan,
  skillTargets,
} from "./model.js";
import { location, placeInInput, projectRoot, sendError, speclinkJson } from "./shared.js";

const STAGE_LABEL = { proposed: "提案中", "in-progress": "進行中", ready: "已就緒" };
// 顏色和 Claude Code 的技能列對應。
const GROUP_TONE = { 規劃: "ansi5", 實作: "ansi4", 品質: "ansi3", 收尾: "ansi2", 專案: "dim" };

export default class SkillPicker {
  init(cx) {
    // 面板的 ⋯ 會先寫好要選哪個技能和哪一列。太舊的不算：那次按的是關閉跳窗。
    const pick = shared("speclink.pick");
    const fresh = pick && Date.now() - pick.when < 3000;
    this.at = fresh ? Math.max(PICKER_SKILLS.findIndex((w) => w.skill === pick.skill), 0) : 0;
    this.target = fresh ? pick.target : null;
    this.checked = new Set();
    this.newTab = false;
    this.board = null;
    this.root = null;
    this.known = null;
    this.worktreePolicy = false;
    this.uninitialized = false;
    this.sending = false;
    this.error = null;
    this.handle = cx.focus_handle();
    this.note = InputState.new({ placeholder: "打完按 Enter" });
    this.note.on("change", (_e, cx) => cx.notify());
    this.note.on("submit", (_e, cx) => this.submit(cx));
    this.focusKeys();
    // 和 Herdr 內建的跳窗相同：剛開的前幾個畫面可能還拿不到焦點。
    cx.timer.after(600, () => this.focusKeys());
    this.load(cx);
  }

  skill() {
    return PICKER_SKILLS[this.at];
  }

  // 搭配變更的技能指令就是全部，要補字可以放到輸入框後再打。其他技能才在跳窗裡打字。
  hasNote() {
    return !this.uninitialized && this.skill().pair !== "變更";
  }

  // 只有變更能複選：討論一次接續一場，專案的技能不搭配東西。
  multi() {
    return this.newTab && this.skill().pair === "變更";
  }

  // 沒有輸入框時由跳窗本身接按鍵。
  focusKeys() {
    (this.hasNote() ? this.note : this.handle).focus();
  }

  load(cx) {
    const { cwd, on } = location();
    this.on = on;
    cx.spawn(async (cx) => {
      try {
        this.root = await projectRoot(cwd, on);
        this.known = await knownAgents({ on }).catch(() => []);
        const read = (...verb) => speclinkJson(verb, this.root, on);
        const [list, plan, discussions, config] = await Promise.all([
          read("list"),
          read("plan"),
          read("discuss", "list"),
          read("workflow-config", "show"),
        ]);
        this.board = buildBoard(list, plan, discussions);
        this.worktreePolicy = config.worktree === true;
      } catch (e) {
        const message = String(e.message ?? e);
        this.uninitialized = isUninitialized(message);
        if (!this.uninitialized) this.error = message;
        // 沒有 Speclink 時不畫輸入框，Esc 要由跳窗本身接。
        this.focusKeys();
      }
      cx.notify();
    });
  }

  targets(skill = this.skill().skill) {
    return this.board ? skillTargets(skill, this.board) : [];
  }

  // 開新分頁要用的資料。
  tabContext() {
    return {
      skill: this.skill().skill,
      note: "",
      root: this.root,
      agents: joinAgents(agents(), terminals(), panes()),
      known: this.known,
      worktreePolicy: this.worktreePolicy,
      space: focused()?.space,
      on: this.on,
    };
  }

  // 這個變更為什麼不能開新分頁。可以開時是 null。
  blockedReason(target) {
    const plan = newTabPlan({ ...this.tabContext(), target });
    return plan.kind === "none" ? plan.reason : null;
  }

  plan() {
    const targets = this.targets();
    if (this.multi()) {
      return batchPlan({ ...this.tabContext(), targets: targets.filter((t) => this.checked.has(t.name)) });
    }
    const note = this.hasNote() ? this.note.value().trim() : "";
    const target = targets.find((t) => t.name === this.target) ?? null;
    if (this.newTab) return newTabPlan({ ...this.tabContext(), target, note });
    const f = focused();
    return placePlan({
      skill: this.skill().skill,
      target,
      note,
      agents: joinAgents(agents(), terminals(), panes()),
      focus: { tab: f?.tab, terminal: f?.terminal },
    });
  }

  // 做完才關跳窗：關掉就收不到失敗的訊息了。
  submit(cx) {
    const plan = this.plan();
    if (this.sending || plan.kind === "none") return;
    this.sending = true;
    this.error = null;
    cx.notify();
    cx.spawn(async (cx) => {
      if (plan.kind === "place") await this.place(plan, cx);
      else await this.openTabs(plan.plans ?? [plan], cx);
    });
  }

  fail(result, cx) {
    this.sending = false;
    this.error = sendError(result);
    cx.notify();
  }

  async place(plan, cx) {
    const failed = await placeInInput(plan);
    if (failed) return this.fail(failed, cx);
    closeOverlay(props().overlay);
  }

  async openTabs(plans, cx) {
    const opened = [];
    for (const plan of plans) {
      const r = await hc(plan.hc);
      // 已經開好的分頁留著。失敗的那個寫出來，跳窗不關。
      if (r.code !== 0) {
        this.nameTabs(opened, cx, 20, false);
        return this.fail(r, cx);
      }
      opened.push({ address: JSON.parse(r.stdout).address, name: plan.tabName });
    }
    this.nameTabs(opened, cx, 20, true);
  }

  // hc run 回來時分頁可能還沒出現，等它出現才能改名。
  nameTabs(waiting, cx, tries, close) {
    const left = waiting.filter(({ address, name }) => {
      const pane = panes().find((p) => p.terminal === address);
      if (pane) run("tab.rename", { id: pane.tab, name });
      return !pane;
    });
    if (left.length > 0 && tries > 0) return cx.timer.after(150, (cx) => this.nameTabs(left, cx, tries - 1, close));
    if (close) closeOverlay(props().overlay);
  }

  toggleNewTab(cx) {
    this.newTab = !this.newTab;
    this.error = null;
    this.focusKeys();
    cx.notify();
  }

  moveSkill(step) {
    this.at = (this.at + step + PICKER_SKILLS.length) % PICKER_SKILLS.length;
    this.focusKeys();
  }

  // 複選時沒有「不指定」這一列。
  moveTarget(step) {
    const names = this.targets().map((t) => t.name);
    if (!this.multi()) names.unshift(null);
    if (names.length === 0) return;
    const at = Math.max(names.indexOf(this.target), 0);
    this.target = names[(at + step + names.length) % names.length];
  }

  toggleChecked(target) {
    if (this.blockedReason(target)) return;
    if (this.checked.has(target.name)) this.checked.delete(target.name);
    else this.checked.add(target.name);
  }

  checkAll() {
    const open = this.targets().filter((t) => !this.blockedReason(t));
    const all = open.every((t) => this.checked.has(t.name));
    for (const t of open) {
      if (all) this.checked.delete(t.name);
      else this.checked.add(t.name);
    }
  }

  skillList(t) {
    const rows = [];
    PICKER_SKILLS.forEach((w, i) => {
      if (PICKER_SKILLS[i - 1]?.group !== w.group) {
        rows.push(
          h_flex()
            .flex_none()
            .mx(8)
            .mt(i > 0 ? 5 : 0)
            .gap(5)
            .items_center()
            .child(div().w(7).h(7).flex_none().rounded_full().bg(t[GROUP_TONE[w.group]]))
            .child(div().text_size(10.5).text_color(t.dim).child(w.group)),
        );
      }
      const count = this.board && w.pair ? this.targets(w.skill).length : null;
      rows.push(
        h_flex()
          .id(`skill-${w.skill}`)
          .flex_none()
          .h(21)
          .px(8)
          .gap(6)
          .items_center()
          .rounded(5)
          .text_size(12)
          .cursor_pointer()
          // discuss、propose 沒有討論也能打字開工，只有搭配變更的技能才真的用不了。
          .when(count === 0 && w.pair === "變更", (el) => el.text_color(t.dim))
          .when(i === this.at, (el) => el.bg(t.selected).font_medium())
          .when(i !== this.at, (el) => el.hover((s) => s.bg(t.hover)))
          .on_click((_e, cx) => {
            this.at = i;
            this.focusKeys();
            cx.notify();
          })
          .child(div().flex_1().min_w_0().truncate().child(w.skill))
          .when(count !== null, (el) => el.child(div().flex_none().text_size(11).text_color(t.dim).child(String(count)))),
      );
    });
    return v_flex().id("skill-list").w(190).flex_none().h_full().p(6).gap(1).overflow_y_scrollbar().children(rows);
  }

  checkBox(on, t) {
    return div()
      .size(14)
      .flex()
      .flex_none()
      .items_center()
      .justify_center()
      .rounded(3)
      .border_1()
      .border_color(on ? t.accent : t.border)
      .bg(on ? t.accent : t.background)
      .when(on, (el) => el.child(svg("mods/speclink/icons/check.svg").size(11).text_color(t.background)));
  }

  targetRow({ id, name, side, topic, current, box, muted, click }, t) {
    return h_flex()
      .id(`target-${id}`)
      .flex_none()
      .px(8)
      .py(4)
      .gap(7)
      .rounded(5)
      .cursor_pointer()
      .when(current, (el) => el.bg(t.selected))
      .when(!current, (el) => el.hover((s) => s.bg(t.hover)))
      .on_click((_e, cx) => {
        click();
        this.focusKeys();
        cx.notify();
      })
      .when(box !== undefined, (el) => el.child(box))
      .child(
        v_flex()
          .flex_1()
          .min_w_0()
          .child(
            h_flex()
              .gap(8)
              .items_center()
              .child(div().flex_1().min_w_0().truncate().text_size(12).font_medium().text_color(muted ? t.dim : t.foreground).child(name))
              .when(Boolean(side), (el) => el.child(div().flex_none().text_size(11).text_color(t.dim).child(side))),
          )
          .when(Boolean(topic), (el) => el.child(div().truncate().text_size(11).text_color(t.muted).child(topic))),
      );
  }

  rowFor(x, selected, t) {
    const select = () => (this.target = x.name);
    if (x.kind === "discussion") {
      return this.targetRow({ id: x.name, name: x.name, topic: x.topic, current: selected === x.name, click: select }, t);
    }
    const progress = `${STAGE_LABEL[x.row.stage]} ${x.row.completed}/${x.row.total}`;
    if (!this.multi()) {
      return this.targetRow({ id: x.name, name: x.name, side: progress, current: selected === x.name, click: select }, t);
    }
    const reason = this.blockedReason(x);
    return this.targetRow(
      {
        id: x.name,
        name: x.name,
        side: reason ?? progress,
        current: selected === x.name,
        muted: Boolean(reason),
        // 不能勾的列留一個空位，名稱才對得齊。
        box: reason ? div().size(14).flex_none() : this.checkBox(this.checked.has(x.name), t),
        click: () => {
          select();
          this.toggleChecked(x);
        },
      },
      t,
    );
  }

  targetList(t) {
    const w = this.skill();
    const targets = this.targets();
    const multi = this.multi();
    // 複選時游標預設在第一列，單選時預設在「不指定」。
    const selected = targets.find((x) => x.name === this.target)?.name ?? (multi ? targets[0]?.name : null);
    const label = (text) => div().flex_none().mx(8).text_size(10.5).text_color(t.dim).child(text);
    return v_flex()
      .id("skill-targets")
      .flex_1()
      .min_w_0()
      .h_full()
      .p(6)
      .gap(2)
      .border_l_1()
      .border_color(t.border)
      .overflow_y_scrollbar()
      .child(div().flex_none().mx(8).mb(5).text_size(12).text_color(t.muted).child(w.about))
      .when(this.hasNote(), (el) =>
        el.child(label(w.skill === "discuss" ? "主題" : "補充說明（可留空）")).child(
          div()
            .flex_none()
            .mx(8)
            .mb(6)
            .child(
              Input.new(this.note)
                .w_full()
                .h(30)
                .px(8)
                .rounded(6)
                .border_1()
                .border_color(t.accent)
                .bg(t.background)
                .text_size(12)
                .text_color(t.foreground),
            ),
        ),
      )
      .when(!w.pair, (el) => el.child(label("這個技能用在整個專案，不用搭配")))
      .when(Boolean(w.pair) && targets.length === 0, (el) => el.child(label(`目前沒有可以搭配的${w.pair}`)))
      .when(targets.length > 0, (el) =>
        el
          .child(
            h_flex()
              .flex_none()
              .mr(8)
              .items_center()
              .child(div().flex_1().child(label(`搭配的${w.pair}`)))
              .when(multi, (row) =>
                row.child(
                  div()
                    .id("check-all")
                    .text_size(11)
                    .text_color(t.muted)
                    .cursor_pointer()
                    .on_click((_e, cx) => {
                      this.checkAll();
                      this.focusKeys();
                      cx.notify();
                    })
                    .child("全選 ⌘A"),
                ),
              ),
          )
          .when(!multi, (list) =>
            list.child(
              this.targetRow({ id: "none", name: "不指定", current: selected === null, click: () => (this.target = null) }, t),
            ),
          )
          .children(targets.map((x) => this.rowFor(x, selected, t))),
      );
  }

  newTabBox(t) {
    return h_flex()
      .id("new-tab")
      .flex_none()
      .gap(6)
      .items_center()
      .cursor_pointer()
      .on_click((_e, cx) => this.toggleNewTab(cx))
      .child(this.checkBox(this.newTab, t))
      .child("開新分頁")
      .child(div().text_size(11).text_color(t.dim).child("⌘O"));
  }

  goLabel(plan) {
    if (this.sending) return "處理中…";
    if (plan.plans?.length > 1) return `開 ${plan.plans.length} 個分頁並開工 ⏎`;
    return this.newTab ? "開新分頁並開工 ⏎" : "放到輸入框 ⏎";
  }

  footer(t) {
    const plan = this.plan();
    const ready = plan.kind !== "none" && !this.sending;
    const message = this.error ?? (plan.kind === "none" ? plan.reason : plan.text);
    return v_flex()
      .flex_none()
      .px(12)
      .py(8)
      .gap(7)
      .border_t_1()
      .border_color(t.border)
      .child(
        div()
          .truncate()
          .text_size(11.5)
          .text_color(this.error ? t.blocked : plan.kind === "none" ? t.muted : t.foreground)
          .child(message),
      )
      .child(
        h_flex()
          .gap(10)
          .items_center()
          .child(this.newTabBox(t))
          .child(div().flex_1())
          .child(
            div()
              .id("skill-go")
              .flex_none()
              .px(10)
              .py(4)
              .rounded(6)
              .border_1()
              .border_color(ready ? t.accent : t.border)
              .text_color(ready ? t.foreground : t.dim)
              .when(ready, (el) => el.cursor_pointer().hover((s) => s.bg(t.hover)))
              .on_click((_e, cx) => this.submit(cx))
              .child(this.goLabel(plan)),
          ),
      );
  }

  onKey(e, cx) {
    if (e.key === "escape") return closeOverlay(props().overlay);
    if (this.uninitialized) return;
    if (e.keystroke === "cmd-o") return this.toggleNewTab(cx);
    // 有輸入框時 Enter 由輸入框的 submit 處理。
    if (e.key === "enter" && !this.hasNote()) return this.submit(cx);
    // 輸入框有字時，左右鍵留給輸入框移動游標。
    const typing = this.hasNote() && this.note.value();
    if (e.key === "down") this.moveSkill(1);
    else if (e.key === "up") this.moveSkill(-1);
    else if (e.key === "right" && !typing) this.moveTarget(1);
    else if (e.key === "left" && !typing) this.moveTarget(-1);
    else if (e.key === "space" && this.multi()) {
      const targets = this.targets();
      const row = targets.find((x) => x.name === this.target) ?? targets[0];
      if (row) this.toggleChecked(row);
    } else if (e.keystroke === "cmd-a" && this.multi()) this.checkAll();
    else return;
    cx.stop_propagation();
    cx.notify();
  }

  render() {
    const t = theme();
    return v_flex()
      .id("skill-picker")
      .size_full()
      .track_focus(this.handle)
      .bg(t.popover)
      .text_size(12.5)
      .text_color(t.foreground)
      .on_key_down((e, cx) => this.onKey(e, cx))
      .child(
        h_flex()
          .flex_none()
          .h(36)
          .px(12)
          .gap(10)
          .items_center()
          .border_b_1()
          .border_color(t.border)
          .child(div().flex_1().font_semibold().child("技能"))
          .when(!this.uninitialized, (el) =>
            el.child(
              div()
                .text_size(11)
                .text_color(t.dim)
                .child(this.multi() ? "↑↓ 技能　←→ 移動　Space 勾選" : "↑↓ 技能　←→ 搭配"),
            ),
          )
          .child(div().text_size(11).text_color(t.dim).child("Esc")),
      )
      .when(this.uninitialized, (el) =>
        el.child(div().flex_1().p(12).text_color(t.muted).child("這個專案還沒有 Speclink。到面板按「初始化」後再用。")),
      )
      .when(!this.uninitialized, (el) =>
        el
          .child(h_flex().flex_1().min_h(0).items_start().child(this.skillList(t)).child(this.targetList(t)))
          .child(this.footer(t)),
      );
  }
}
