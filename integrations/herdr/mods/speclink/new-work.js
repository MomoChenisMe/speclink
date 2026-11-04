// 新工作跳窗：像技能面板，選技能和搭配的變更或討論。Enter 把指令打到目前分頁的輸入框，不送出。
// 勾選「開新分頁」時改成 herdr-dispatch 技能的做法：用 hc run 開新分頁並開工。
// 由 speclink-new 的 mod.json 開啟，main.js 看到 props 的 view 就把畫面交給這裡。
import { div, svg } from "gpui-kit";
import { h_flex, v_flex, InputState, Input } from "gpui-base";
import { agents, closeOverlay, focused, hc, knownAgents, panes, props, run, terminals, theme } from "herdr";
import { NEW_WORK, buildBoard, isUninitialized, joinAgents, newTabPlan, newWorkTargets, placePlan } from "./model.js";
import { location, projectRoot, sendError, speclinkJson } from "./shared.js";

const STAGE_LABEL = { proposed: "提案中", "in-progress": "進行中", ready: "已就緒" };

export default class NewWork {
  init(cx) {
    this.at = 0;
    this.target = null;
    this.newTab = false;
    this.board = null;
    this.root = null;
    this.known = null;
    this.uninitialized = false;
    this.sending = false;
    this.error = null;
    this.handle = cx.focus_handle();
    this.note = InputState.new({ placeholder: "主題或補充說明，開工時一起送出，可留空" });
    this.note.on("change", (_e, cx) => cx.notify());
    this.note.on("submit", (_e, cx) => this.submit(cx));
    this.focusKeys();
    // 和 Herdr 內建的跳窗相同：剛開的前幾個畫面可能還拿不到焦點。
    cx.timer.after(600, () => this.focusKeys());
    this.load(cx);
  }

  // 沒有輸入框時由跳窗本身接按鍵。
  focusKeys() {
    (this.newTab ? this.note : this.handle).focus();
  }

  load(cx) {
    const { cwd, on } = location();
    this.on = on;
    cx.spawn(async (cx) => {
      try {
        this.root = await projectRoot(cwd, on);
        this.known = await knownAgents({ on }).catch(() => []);
        const read = (...verb) => speclinkJson(verb, this.root, on);
        const [list, plan, discussions] = await Promise.all([read("list"), read("plan"), read("discuss", "list")]);
        this.board = buildBoard(list, plan, discussions);
      } catch (e) {
        const message = String(e.message ?? e);
        this.uninitialized = isUninitialized(message);
        if (!this.uninitialized) this.error = message;
      }
      cx.notify();
    });
  }

  targets() {
    return this.board ? newWorkTargets(NEW_WORK[this.at].skill, this.board) : [];
  }

  plan() {
    const f = focused();
    const ask = {
      skill: NEW_WORK[this.at].skill,
      target: this.targets().find((t) => t.name === this.target) ?? null,
      agents: joinAgents(agents(), terminals(), panes()),
    };
    if (!this.newTab) return placePlan({ ...ask, focus: { tab: f?.tab, terminal: f?.terminal } });
    return newTabPlan({
      ...ask,
      note: this.note.value().trim(),
      root: this.root,
      known: this.known,
      space: f?.space,
      on: this.on,
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
      const r = await hc(plan.hc);
      if (r.code !== 0) {
        this.sending = false;
        this.error = sendError(r);
        cx.notify();
      } else if (plan.kind === "place") {
        run("pane.focus", { terminal: plan.agent.address });
        closeOverlay(props().overlay);
      } else {
        this.nameTab(JSON.parse(r.stdout).address, plan.tabName, cx, 20);
      }
    });
  }

  // hc run 回來時分頁可能還沒出現，等它出現才能改名。
  nameTab(address, name, cx, tries) {
    const pane = panes().find((p) => p.terminal === address);
    if (pane) run("tab.rename", { id: pane.tab, name });
    if (pane || tries === 0) return closeOverlay(props().overlay);
    cx.timer.after(150, (cx) => this.nameTab(address, name, cx, tries - 1));
  }

  toggleNewTab(cx) {
    this.newTab = !this.newTab;
    this.error = null;
    this.focusKeys();
    cx.notify();
  }

  moveSkill(step) {
    this.at = (this.at + step + NEW_WORK.length) % NEW_WORK.length;
  }

  moveTarget(step) {
    const names = [null, ...this.targets().map((t) => t.name)];
    const at = Math.max(names.indexOf(this.target), 0);
    this.target = names[(at + step + names.length) % names.length];
  }

  skillList(t) {
    const rows = [];
    NEW_WORK.forEach((w, i) => {
      if (NEW_WORK[i - 1]?.group !== w.group) {
        rows.push(div().flex_none().mx(8).mt(i > 0 ? 5 : 0).text_size(10.5).text_color(t.dim).child(w.group));
      }
      rows.push(
        div()
          .id(`skill-${w.skill}`)
          .flex_none()
          .h(21)
          .px(8)
          .flex()
          .items_center()
          .truncate()
          .rounded(5)
          .text_size(12)
          .cursor_pointer()
          .when(i === this.at, (el) => el.bg(t.selected).font_medium())
          .when(i !== this.at, (el) => el.hover((s) => s.bg(t.hover)))
          .on_click((_e, cx) => {
            this.at = i;
            this.focusKeys();
            cx.notify();
          })
          .child(w.skill),
      );
    });
    return v_flex().id("new-work-skills").w(190).flex_none().h_full().p(6).gap(1).overflow_y_scrollbar().children(rows);
  }

  targetRow(id, name, side, topic, selected, t) {
    return v_flex()
      .id(`target-${id}`)
      .flex_none()
      .px(8)
      .py(4)
      .rounded(5)
      .cursor_pointer()
      .when(selected, (el) => el.bg(t.selected))
      .when(!selected, (el) => el.hover((s) => s.bg(t.hover)))
      .on_click((_e, cx) => {
        this.target = id === "none" ? null : name;
        this.focusKeys();
        cx.notify();
      })
      .child(
        h_flex()
          .gap(8)
          .items_center()
          .child(div().flex_1().min_w_0().truncate().text_size(12).font_medium().child(name))
          .when(Boolean(side), (el) => el.child(div().flex_none().text_size(11).text_color(t.dim).child(side))),
      )
      .when(Boolean(topic), (el) => el.child(div().truncate().text_size(11).text_color(t.muted).child(topic)));
  }

  targetList(t) {
    const targets = this.targets();
    const selected = targets.find((x) => x.name === this.target)?.name ?? null;
    const label = (text) => div().flex_none().mx(8).text_size(10.5).text_color(t.dim).child(text);
    return v_flex()
      .id("new-work-targets")
      .flex_1()
      .min_w_0()
      .h_full()
      .p(6)
      .gap(2)
      .border_l_1()
      .border_color(t.border)
      .overflow_y_scrollbar()
      .child(div().flex_none().mx(8).mb(5).text_size(12).text_color(t.muted).child(NEW_WORK[this.at].about))
      .when(this.newTab, (el) =>
        el.child(
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
      .when(targets.length === 0, (el) => el.child(label("沒有可以搭配的變更或討論")))
      .when(targets.length > 0, (el) =>
        el
          .child(label("搭配"))
          .child(this.targetRow("none", "不指定", null, null, selected === null, t))
          .children(
            targets.map((x) =>
              x.kind === "change"
                ? this.targetRow(x.name, x.name, `${STAGE_LABEL[x.row.stage]} ${x.row.completed}/${x.row.total}`, null, selected === x.name, t)
                : this.targetRow(x.name, x.name, "討論", x.topic, selected === x.name, t),
            ),
          ),
      );
  }

  checkbox(t) {
    return h_flex()
      .id("new-tab")
      .flex_none()
      .gap(6)
      .items_center()
      .cursor_pointer()
      .on_click((_e, cx) => this.toggleNewTab(cx))
      .child(
        div()
          .size(14)
          .flex()
          .items_center()
          .justify_center()
          .rounded(3)
          .border_1()
          .border_color(this.newTab ? t.accent : t.border)
          .bg(this.newTab ? t.accent : t.background)
          .when(this.newTab, (el) => el.child(svg("mods/speclink/icons/check.svg").size(11).text_color(t.background))),
      )
      .child("開新分頁")
      .child(div().text_size(11).text_color(t.dim).child("⌘O"));
  }

  footer(t) {
    const plan = this.plan();
    const ready = plan.kind !== "none" && !this.sending;
    const message = this.error ?? (plan.kind === "none" ? plan.reason : plan.text);
    const verb = this.newTab ? "開新分頁並開工" : "放到輸入框";
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
          .child(this.checkbox(t))
          .child(div().flex_1())
          .child(
            div()
              .id("new-work-go")
              .flex_none()
              .px(10)
              .py(4)
              .rounded(6)
              .border_1()
              .border_color(ready ? t.accent : t.border)
              .text_color(ready ? t.foreground : t.dim)
              .when(ready, (el) => el.cursor_pointer().hover((s) => s.bg(t.hover)))
              .on_click((_e, cx) => this.submit(cx))
              .child(this.sending ? "處理中…" : `${verb} ⏎`),
          ),
      );
  }

  render() {
    const t = theme();
    return v_flex()
      .id("new-work")
      .size_full()
      .track_focus(this.handle)
      .bg(t.popover)
      .text_size(12.5)
      .text_color(t.foreground)
      .on_key_down((e, cx) => {
        if (e.key === "escape") return closeOverlay(props().overlay);
        if (e.keystroke === "cmd-o") return this.toggleNewTab(cx);
        // 勾選時 Enter 由輸入框的 submit 處理。
        if (e.key === "enter" && !this.newTab) return this.submit(cx);
        // 輸入框有字時，左右鍵留給輸入框移動游標。
        const typing = this.newTab && this.note.value();
        if (e.key === "down") this.moveSkill(1);
        else if (e.key === "up") this.moveSkill(-1);
        else if (e.key === "right" && !typing) this.moveTarget(1);
        else if (e.key === "left" && !typing) this.moveTarget(-1);
        else return;
        cx.stop_propagation();
        cx.notify();
      })
      .child(
        h_flex()
          .flex_none()
          .h(36)
          .px(12)
          .gap(10)
          .items_center()
          .border_b_1()
          .border_color(t.border)
          .child(div().flex_1().font_semibold().child("＋ 新工作"))
          .when(!this.uninitialized, (el) =>
            el.child(div().text_size(11).text_color(t.dim).child("↑↓ 技能　←→ 搭配")),
          )
          .child(div().text_size(11).text_color(t.dim).child("Esc")),
      )
      .when(this.uninitialized, (el) =>
        el.child(
          div().flex_1().p(12).text_color(t.muted).child("這個專案還沒有 Speclink。到面板按「初始化」後再開新工作。"),
        ),
      )
      .when(!this.uninitialized, (el) =>
        el
          .child(h_flex().flex_1().min_h(0).items_start().child(this.skillList(t)).child(this.targetList(t)))
          .child(this.footer(t)),
      );
  }
}
