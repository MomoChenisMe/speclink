// 新工作跳窗（討論 herdr-speclink-panel 定案）：左邊列不帶變更名稱的技能，右邊打主題或接續討論。
// 由 speclink-new 的 mod.json 開啟，main.js 看到 props 的 view 就把畫面交給這裡。
import { div } from "gpui-kit";
import { h_flex, v_flex, InputState, Input } from "gpui-base";
import { agents, closeOverlay, exec, focused, hc, knownAgents, panes, props, run, terminals, theme } from "herdr";
import { enterHint, isUninitialized, joinAgents, newWorkActions, planDelivery } from "./model.js";
import { firstLine, location, projectRoot, sendError, spaceFor } from "./shared.js";

export default class NewWork {
  init(cx) {
    this.at = 0;
    this.root = null;
    this.known = null;
    this.discussions = [];
    this.uninitialized = false;
    this.sending = false;
    this.error = null;
    this.input = InputState.new({ placeholder: "打完按 Enter" });
    this.input.on("change", (_e, cx) => cx.notify());
    this.input.on("submit", (_e, cx) => this.submit(cx));
    this.input.focus();
    // 和 Herdr 內建的跳窗相同：剛開的前幾個畫面可能還拿不到焦點。
    cx.timer.after(600, () => this.input.focus());
    this.load(cx);
  }

  load(cx) {
    const { cwd, on } = location();
    cx.spawn(async (cx) => {
      try {
        this.root = await projectRoot(cwd, on);
        this.known = await knownAgents({ on }).catch(() => []);
        const r = await exec("speclink", ["discuss", "list", "--json"], { cwd: this.root, on });
        if (r.code === 0) this.discussions = JSON.parse(r.stdout).discussions.filter((d) => d.status === "open");
        else if (isUninitialized(r.stderr)) this.uninitialized = true;
        else this.error = firstLine(r.stderr);
      } catch (e) {
        this.error = String(e.message ?? e);
      }
      cx.notify();
    });
  }

  context() {
    const f = focused();
    return {
      root: this.root,
      agents: joinAgents(agents(), terminals(), panes()),
      focus: { tab: f?.tab ?? null, terminal: f?.terminal ?? null },
      known: this.known,
    };
  }

  // 開好或送出才關跳窗：hc send 要等 agent 開工才回來，關掉就收不到失敗的訊息了。
  send(skill, args, cx) {
    if (this.sending) return;
    const d = planDelivery({ ...this.context(), skill, args });
    if (d.kind === "new") {
      run("tab.new", { folder: d.folder, argv: d.argv, task: d.text, name: d.tabName, space: spaceFor(d.folder)?.id });
      closeOverlay(props().overlay);
    } else if (d.kind === "send") {
      this.sending = true;
      cx.notify();
      cx.spawn(async (cx) => {
        const r = await hc(d.hc);
        if (r.code === 0) {
          run("pane.focus", { terminal: d.agent.address });
          closeOverlay(props().overlay);
          return;
        }
        this.sending = false;
        this.error = sendError(r);
        cx.notify();
      });
    }
  }

  submit(cx) {
    const { skill } = newWorkActions(this.context())[this.at];
    const text = this.input.value().trim();
    // 空的 discuss 沒有東西可以談：不小心按到 Enter 不該送出去。
    if (skill === "discuss" && !text) return;
    this.send(skill, text || undefined, cx);
  }

  skillRow(a, i, t) {
    return v_flex()
      .id(`skill-${a.skill}`)
      .flex_none()
      .px(8)
      .py(3)
      .rounded(6)
      .cursor_pointer()
      .when(i === this.at, (el) => el.bg(t.selected))
      .when(i !== this.at, (el) => el.hover((s) => s.bg(t.hover)))
      .on_click((_e, cx) => {
        this.at = i;
        this.input.focus();
        cx.notify();
      })
      .child(div().font_medium().text_size(12).child(a.skill))
      .child(div().truncate().text_size(11).text_color(t.muted).child(a.about));
  }

  skillList(actions, t) {
    const rows = [];
    actions.forEach((a, i) => {
      if (actions[i - 1]?.group !== a.group) {
        rows.push(div().flex_none().mx(8).mt(4).text_size(10.5).text_color(t.dim).child(a.group));
      }
      rows.push(this.skillRow(a, i, t));
    });
    return v_flex().id("new-work-skills").flex_1().min_w_0().p(6).gap(2).overflow_y_scrollbar().children(rows);
  }

  discussionRow(d, t) {
    return v_flex()
      .id(`resume-${d.slug}`)
      .flex_none()
      .px(8)
      .py(3)
      .rounded(6)
      .cursor_pointer()
      .hover((s) => s.bg(t.hover))
      .on_click((_e, cx) => this.send("discuss", d.slug, cx))
      .child(div().truncate().font_medium().text_size(12).child(d.slug))
      .child(div().truncate().text_size(11).text_color(t.muted).child(`第 ${d.rounds} 輪`));
  }

  detail(skill, t) {
    const label = (text) => div().flex_none().mx(8).text_size(11).text_color(t.dim).child(text);
    return v_flex()
      .id("new-work-detail")
      .flex_1()
      .min_w_0()
      .p(6)
      .pt(8)
      .gap(5)
      .border_l_1()
      .border_color(t.border)
      .overflow_y_scrollbar()
      .child(label(skill === "discuss" ? "主題" : "補充說明（可留空）"))
      .child(
        div()
          .flex_none()
          .mx(8)
          .child(
            Input.new(this.input)
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
      )
      .when(skill === "discuss" && this.discussions.length > 0, (el) =>
        el
          .child(div().flex_none().h(4))
          .child(label("或接續一場討論"))
          .children(this.discussions.map((d) => this.discussionRow(d, t))),
      );
  }

  footer(action) {
    if (this.sending) return "送出中…";
    if (action.skill === "discuss" && !this.input.value().trim()) return "打一句主題後按 Enter";
    return enterHint(action.delivery);
  }

  render() {
    const t = theme();
    const actions = newWorkActions(this.context());
    const current = actions[this.at];
    return v_flex()
      .id("new-work")
      .size_full()
      .bg(t.popover)
      .text_size(12.5)
      .text_color(t.foreground)
      .on_key_down((e, cx) => {
        if (e.key === "escape") return closeOverlay(props().overlay);
        if (e.key === "down") this.at = (this.at + 1) % actions.length;
        else if (e.key === "up") this.at = (this.at + actions.length - 1) % actions.length;
        else return;
        cx.stop_propagation();
        cx.notify();
      })
      .child(
        h_flex()
          .flex_none()
          .h(36)
          .px(12)
          .items_center()
          .border_b_1()
          .border_color(t.border)
          .child(div().flex_1().font_semibold().child("＋ 新工作"))
          .child(div().text_size(11).text_color(t.dim).child("Esc")),
      )
      .when(this.uninitialized, (el) =>
        el.child(
          div().flex_1().p(12).text_color(t.muted).child("這個專案還沒有 Speclink。到面板按「初始化」後再開新工作。"),
        ),
      )
      .when(!this.uninitialized, (el) =>
        el.child(
          h_flex().flex_1().min_h(0).items_start().child(this.skillList(actions, t)).child(this.detail(current.skill, t)),
        ),
      )
      .when(!this.uninitialized, (el) =>
        el.child(
          div()
            .flex_none()
            .px(12)
            .py(8)
            .truncate()
            .border_t_1()
            .border_color(t.border)
            .text_size(11.5)
            .text_color(this.error ? t.blocked : t.muted)
            .child(this.error ?? this.footer(current)),
        ),
      );
  }
}
