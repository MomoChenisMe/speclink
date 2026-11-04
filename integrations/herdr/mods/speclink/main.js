// Speclink 面板：目前 space 的專案有哪些變更與討論、各自做到哪裡，以及按一下把技能交給對的
// agent。資料只來自 speclink CLI 的 --json；判斷邏輯都在 model.js。
import { View, div, image, svg } from "gpui-kit";
import { h_flex, v_flex, InputState, Input } from "gpui-base";
import {
  agents,
  closeOverlay,
  exec,
  focused,
  hc,
  knownAgents,
  notify,
  panes,
  props,
  run,
  spaces,
  terminals,
  theme,
  watchFolder,
} from "herdr";
import {
  buildBoard,
  changeActions,
  changeAgents,
  deliveryHint,
  discussionActions,
  inside,
  joinAgents,
  newWorkActions,
  planDelivery,
} from "./model.js";

const ICONS = "mods/speclink/icons";
// 溢出摺疊的門檻與桌面 app 的系統匣面板相同。
const OVERFLOW_LIMIT = 5;
const STAGES = [
  { id: "proposed", label: "提案中", tone: "ansi3" },
  { id: "in-progress", label: "進行中", tone: "ansi4" },
  { id: "ready", label: "已就緒", tone: "ansi2" },
];
const AGENT_STATE = { working: "做事中", blocked: "在等你回答", done: "做完了", idle: "閒置" };

const icon = (name, size, color) => svg(`${ICONS}/${name}.svg`).size(size).flex_none().text_color(color);
const baseName = (path) => path.split("/").filter(Boolean).pop() ?? path;
const firstLine = (text) => text.trim().split("\n")[0];

// program.icon 是相對於 Herdr 設定資料夾的路徑：tint 表示單色圖示，其餘是圖片。
function programIcon(program, size, color) {
  if (!program?.icon) return div();
  return program.tint
    ? svg(program.icon).size(size).flex_none().text_color(color)
    : image(program.icon).w(size).h(size).flex_none();
}

// 目前 space 所在的資料夾：焦點所在的終端機，否則該 space 第一個終端機。spaces() 的 folder
// 只是開新終端機的起點，不一定是它現在的位置。
function location() {
  const space = spaces().find((s) => s.active);
  const f = focused();
  const ps = panes();
  const pane = f?.pane ? ps.find((p) => p.pane === f.pane) : null;
  const tabs = new Set(space?.tabs.map((t) => t.id));
  const first = ps.find((p) => tabs.has(p.tab) && p.cwd)?.cwd;
  return { cwd: pane?.cwd || f?.cwd || first || space?.folder || null, on: space?.machine ?? null };
}

// 主資料夾：在 worktree 裡也要回到主 checkout，變更清單才完整。
async function projectRoot(cwd, on) {
  const r = await exec("git", ["rev-parse", "--path-format=absolute", "--git-common-dir"], { cwd, on });
  return r.code === 0 ? r.stdout.trim().replace(/\/\.git\/?$/, "") : cwd;
}

// 新分頁開在已經待在這個資料夾的 space，目前的 space 優先。
function spaceFor(folder) {
  const all = spaces();
  const ps = panes();
  const holds = (s) => ps.some((p) => s.tabs.some((t) => t.id === p.tab) && inside(p.cwd, folder));
  return all.find((s) => s.active && holds(s)) ?? all.find(holds);
}

function sendError(result) {
  try {
    return JSON.parse(result.stderr).error.message;
  } catch {
    return firstLine(result.stderr) || `hc 結束碼 ${result.code}`;
  }
}

export default class SpeclinkPanel extends View {
  init(_props, cx) {
    this.place = location();
    this.root = null;
    this.board = null;
    this.error = null;
    this.known = null;
    this.copied = null;
    this.newWork = false;
    this.topicOpen = false;
    this.more = new Set();
    this.sending = new Set();
    this.collapsed = new Set(JSON.parse(localStorage.getItem("collapsed") ?? "[]"));
    this.promotedOpen = localStorage.getItem("promotedOpen") === "true";
    this.topic = InputState.new({ placeholder: "要討論的主題，按 Enter 送出" });
    this.topic.on("submit", (_e, cx) => this.submitTopic(cx));
    this.reload(cx);
    this.loadKnown(cx);
    // worktree 裡的任務進度不在主資料夾的監看範圍內，要靠計時重抓。
    cx.timer.every(10000, (cx) => {
      this.follow(cx);
      this.reload(cx);
    });
  }

  // 焦點或資料夾變了就改跟新的專案。render 會隨 Herdr 的即時資料重跑，所以平常立刻跟上；
  // 視窗沒有畫面更新時（被其他視窗完全蓋住）render 不會跑，由計時補上。
  follow(cx) {
    const place = location();
    if (place.cwd === this.place.cwd && place.on === this.place.on) return;
    if (place.on !== this.place.on) this.loadKnown(cx);
    this.place = place;
    this.reload(cx);
  }

  loadKnown(cx) {
    cx.spawn(async (cx) => {
      this.known = await knownAgents({ on: this.place.on }).catch(() => []);
      cx.notify();
    });
  }

  reload(cx) {
    if (this.loading) {
      this.again = true;
      return;
    }
    const { cwd, on } = this.place;
    if (!cwd) return;
    this.loading = true;
    cx.spawn(async (cx) => {
      try {
        const root = await projectRoot(cwd, on);
        if (root !== this.root) {
          this.root = root;
          this.board = null;
          this.watch(root, on, cx);
        }
        const read = async (...verb) => {
          const r = await exec("speclink", [...verb, "--json"], { cwd: root, on });
          if (r.code !== 0) throw new Error(firstLine(r.stderr) || `speclink ${verb.join(" ")} 結束碼 ${r.code}`);
          return JSON.parse(r.stdout);
        };
        const [list, plan, discussions] = await Promise.all([read("list"), read("plan"), read("discuss", "list")]);
        this.board = buildBoard(list, plan, discussions);
        this.error = null;
      } catch (e) {
        this.error = String(e.message ?? e);
      }
      this.loading = false;
      cx.notify();
      if (this.again) {
        this.again = false;
        this.reload(cx);
      }
    });
  }

  // 規格資料夾有動靜就重抓；內容一律問 CLI，這裡不讀檔。watchFolder 只看得到本機。
  watch(root, on, cx) {
    if (on) return;
    cx.spawn(async (cx) => {
      const found = await exec("ls", ["-d", "speclink", "openspec"], { cwd: root }).catch(() => null);
      const dir = found?.stdout.trim().split("\n")[0];
      if (!dir) return;
      try {
        while (this.root === root) {
          await watchFolder(`${root}/${dir}`);
          if (this.root === root) this.reload(cx);
        }
      } catch {
        // 監看失敗時還有計時重抓。
      }
    });
  }

  context() {
    const f = focused();
    return {
      root: this.root,
      agents: joinAgents(agents(), terminals(), panes()).map((a) =>
        this.sending.has(a.address) ? { ...a, status: "sending" } : a,
      ),
      focus: { tab: f?.tab ?? null, terminal: f?.terminal ?? null },
      known: this.known,
    };
  }

  deliver(delivery, cx) {
    if (delivery.kind === "send") {
      const { address } = delivery.agent;
      this.sending.add(address);
      cx.notify();
      cx.spawn(async (cx) => {
        const r = await hc(delivery.hc);
        this.sending.delete(address);
        if (r.code === 0) run("pane.focus", { terminal: address });
        else notify({ title: "技能沒有送出", body: sendError(r), level: "warning" });
        cx.notify();
      });
    } else if (delivery.kind === "new") {
      const before = new Set(terminals().map((t) => t.address));
      run("tab.new", {
        name: delivery.tabName,
        space: spaceFor(delivery.folder)?.id,
        folder: delivery.folder,
        argv: delivery.argv,
        task: delivery.text,
      });
      if (delivery.change) this.labelNew(before, delivery, cx, 10);
    }
  }

  // 新分頁貼上標籤，之後按這個變更的技能才認得它。tab.new 不回傳終端機位址，要等它出現。
  labelNew(before, delivery, cx, tries) {
    cx.timer.after(300, (cx) => {
      const tabs = new Map(spaces().flatMap((s) => s.tabs).map((t) => [t.id, t]));
      const pane = panes().find((p) => p.terminal && !before.has(p.terminal) && tabs.get(p.tab)?.name === delivery.tabName);
      if (pane) {
        hc(["label", pane.terminal, `speclink.change=${delivery.change}`, `speclink.skill=speclink-${delivery.skill}`]);
      } else if (tries > 1) {
        this.labelNew(before, delivery, cx, tries - 1);
      }
    });
  }

  submitTopic(cx) {
    const topic = this.topic.value().trim();
    if (!topic) return;
    const delivery = planDelivery({ ...this.context(), skill: "discuss", args: topic });
    if (delivery.kind !== "send" && delivery.kind !== "new") {
      notify({ title: "技能沒有送出", body: deliveryHint(delivery), level: "warning" });
      return;
    }
    this.deliver(delivery, cx);
    this.topic.set_value("");
    this.topicOpen = false;
    cx.notify();
  }

  toggleTopic(cx) {
    this.topicOpen = !this.topicOpen;
    if (this.topicOpen) this.topic.focus();
    cx.notify();
  }

  copy(text, cx) {
    cx.write_to_clipboard(text);
    this.copied = text;
    cx.notify();
    cx.timer.after(1200, (cx) => {
      if (this.copied === text) this.copied = null;
      cx.notify();
    });
  }

  toggleSection(id, cx) {
    if (id === "promoted") {
      this.promotedOpen = !this.promotedOpen;
      localStorage.setItem("promotedOpen", String(this.promotedOpen));
    } else {
      if (this.collapsed.has(id)) this.collapsed.delete(id);
      else this.collapsed.add(id);
      localStorage.setItem("collapsed", JSON.stringify([...this.collapsed]));
    }
    cx.notify();
  }

  copyButton(id, text, label, t) {
    return div()
      .id(`copy-${id}`)
      .flex_none()
      .p(3)
      .rounded(4)
      .cursor_pointer()
      .hover((s) => s.bg(t.hover))
      .tooltip(label)
      .on_click((_e, cx) => this.copy(text, cx))
      .child(icon(this.copied === text ? "check" : "copy", 12, t.muted));
  }

  // onClick：不直接送出的鈕（discuss 要先輸入主題）。
  skillButton(id, action, t, onClick) {
    const d = action.delivery;
    const ready = d && (d.kind === "send" || d.kind === "new");
    const tip = action.waitingFor
      ? `等 ${action.waitingFor.join("、")} 先完成`
      : d.text
        ? `${deliveryHint(d)}：${d.text}`
        : deliveryHint(d);
    return h_flex()
      .id(`skill-${id}-${action.skill}`)
      .h(22)
      .px(7)
      .items_center()
      .rounded(5)
      .text_size(11.5)
      .when(action.primary, (el) => el.border_2().border_color(t.accent).font_semibold())
      .when(!action.primary, (el) => el.border_1().border_color(t.input))
      .when(ready, (el) =>
        el
          .cursor_pointer()
          .hover((s) => s.bg(t.hover))
          .on_click((_e, cx) => (onClick ? onClick(cx) : this.deliver(d, cx))),
      )
      .when(!ready, (el) => el.opacity(0.45))
      .tooltip(tip)
      .child(action.skill);
  }

  skillRow(id, actions, hint, t) {
    return v_flex()
      .gap(3)
      .child(
        h_flex()
          .flex_wrap()
          .gap(4)
          .children(actions.map((a) => this.skillButton(id, a, t))),
      )
      .when(Boolean(hint), (el) => el.child(div().truncate().text_size(11).text_color(t.dim).child(hint)));
  }

  agentChip(id, a, t) {
    return h_flex()
      .id(`agent-${id}-${a.address}`)
      .gap(5)
      .px(5)
      .py(2)
      .items_center()
      .rounded(5)
      .text_size(11)
      .cursor_pointer()
      .hover((s) => s.bg(t.hover))
      .tooltip("切到這個 agent")
      .on_click(() => run("agent.focus", { terminal: a.address }))
      .child(div().w(7).h(7).flex_none().rounded_full().bg(t[a.view] ?? t.idle))
      .child(programIcon(a.program, 12, t.muted))
      .child(div().flex_1().min_w_0().truncate().text_color(t.muted).child(`${a.space_name} / ${a.tab_name}`))
      .child(div().flex_none().text_color(t.dim).child(AGENT_STATE[a.view] ?? a.view));
  }

  changeRow(row, stage, ctx, t) {
    const actions = changeActions(row, ctx);
    const blocked = row.blockedBy.length > 0;
    const main = actions.find((a) => a.primary) ?? actions.find((a) => a.delivery);
    const waiting = blocked ? `等 ${row.blockedBy.join("、")}` : "";
    const hint = [waiting, main?.delivery ? deliveryHint(main.delivery) : ""].filter(Boolean).join(" · ");
    const percent = row.total > 0 ? Math.round((row.completed / row.total) * 100) : 0;
    return v_flex()
      .id(`change-${row.name}`)
      .px(10)
      .py(8)
      .gap(5)
      .child(
        h_flex()
          .gap(6)
          .items_center()
          .when(blocked, (el) => el.opacity(0.6))
          .child(div().flex_none().text_size(11).font_semibold().text_color(t.accent).child(String(row.wave)))
          .child(div().flex_1().min_w_0().truncate().child(row.name))
          .when(Boolean(row.worktree), (el) =>
            el.child(
              div()
                .id(`branch-${row.name}`)
                .flex_none()
                .tooltip(`worktree：${row.worktree?.branch}`)
                .child(icon("branch", 12, t.ansi4)),
            ),
          )
          .when(row.total > 0, (el) =>
            el.child(div().flex_none().text_size(11).text_color(t.muted).child(`${row.completed}/${row.total}`)),
          )
          .child(this.copyButton(`change-${row.name}`, row.name, "複製名稱", t)),
      )
      .when(row.total > 0, (el) =>
        el.child(
          div()
            .h(4)
            .rounded(2)
            .bg(t.raised)
            .overflow_hidden()
            .when(blocked, (bar) => bar.opacity(0.6))
            .child(div().h_full().rounded(2).w(`${percent}%`).bg(t[stage.tone])),
        ),
      )
      .children(changeAgents(row, ctx.agents).map((a) => this.agentChip(`change-${row.name}`, a, t)))
      .child(this.skillRow(`change-${row.name}`, actions, hint, t));
  }

  discussionRow(d, ctx, t) {
    const actions = discussionActions(d, ctx);
    const hint = d.cuts.length > 0 ? `還有 ${d.cuts.length} 刀沒轉出` : "";
    return v_flex()
      .id(`discussion-${d.slug}`)
      .px(10)
      .py(8)
      .gap(5)
      .child(
        h_flex()
          .gap(6)
          .items_center()
          .child(
            v_flex()
              .flex_1()
              .min_w_0()
              .child(div().truncate().child(d.slug))
              .child(div().truncate().text_size(11).text_color(t.muted).child(d.topic)),
          )
          .child(this.copyButton(`discussion-${d.slug}`, d.slug, "複製 slug", t)),
      )
      .child(this.skillRow(`discussion-${d.slug}`, actions, hint, t));
  }

  // 超過門檻的列收進「還有 N 個…」。
  overflow(id, rows, t) {
    if (rows.length <= OVERFLOW_LIMIT) return rows;
    const open = this.more.has(id);
    return [
      ...(open ? rows : rows.slice(0, OVERFLOW_LIMIT)),
      div()
        .id(`more-${id}`)
        .px(10)
        .py(6)
        .text_size(12)
        .text_color(t.muted)
        .cursor_pointer()
        .hover((s) => s.bg(t.hover))
        .on_click((_e, cx) => {
          if (open) this.more.delete(id);
          else this.more.add(id);
          cx.notify();
        })
        .child(open ? "收合" : `還有 ${rows.length - OVERFLOW_LIMIT} 個…`),
    ];
  }

  card({ id, iconName, tone, label, rows, open }, t) {
    return v_flex()
      .flex_none()
      .rounded(10)
      .border_1()
      .border_color(t.border)
      .bg(t.surface)
      .overflow_hidden()
      .child(
        h_flex()
          .id(`card-${id}`)
          .h(30)
          .px(10)
          .gap(6)
          .items_center()
          .cursor_pointer()
          .hover((s) => s.bg(t.hover))
          .when(open && rows.length > 0, (el) => el.border_b_1().border_color(t.border))
          .on_click((_e, cx) => this.toggleSection(id, cx))
          .child(icon(iconName, 13, tone))
          .child(div().flex_1().font_semibold().text_size(12).child(label))
          .child(div().px(5).rounded(4).bg(t.raised).text_size(11).text_color(tone).child(String(rows.length)))
          .child(icon(open ? "chevron-down" : "chevron-right", 13, t.muted)),
      )
      .when(open, (el) => el.children(this.overflow(id, rows, t)));
  }

  newWorkPanel(ctx, t) {
    const actions = newWorkActions(ctx);
    const discuss = actions.find((a) => a.skill === "discuss");
    return v_flex()
      .flex_none()
      .px(10)
      .pb(8)
      .gap(6)
      .child(
        h_flex()
          .flex_wrap()
          .gap(4)
          .children(
            actions.map((a) =>
              this.skillButton("new", a, t, a.skill === "discuss" ? (cx) => this.toggleTopic(cx) : undefined),
            ),
          ),
      )
      .when(this.topicOpen, (el) =>
        el.child(
          Input.new(this.topic)
            .h(28)
            .px(8)
            .rounded(6)
            .border_1()
            .border_color(t.input)
            .bg(t.background)
            .text_size(12)
            .text_color(t.foreground),
        ),
      )
      .child(div().text_size(11).text_color(t.dim).child(deliveryHint(discuss.delivery)));
  }

  body(ctx, t) {
    const { stages, discussions, promoted, skipped } = this.board;
    const cards = STAGES.map((stage) =>
      this.card(
        {
          id: stage.id,
          iconName: stage.id,
          tone: t[stage.tone],
          label: stage.label,
          rows: stages[stage.id].map((row) => this.changeRow(row, stage, ctx, t)),
          open: !this.collapsed.has(stage.id),
        },
        t,
      ),
    );
    cards.push(
      this.card(
        {
          id: "discussions",
          iconName: "discussion",
          tone: t.ansi5,
          label: "討論",
          rows: discussions.map((d) => this.discussionRow(d, ctx, t)),
          open: !this.collapsed.has("discussions"),
        },
        t,
      ),
    );
    if (promoted.length > 0) {
      cards.push(
        this.card(
          {
            id: "promoted",
            iconName: "discussion",
            tone: t.muted,
            label: "已轉出",
            rows: promoted.map((d) => this.discussionRow(d, ctx, t)),
            open: this.promotedOpen,
          },
          t,
        ),
      );
    }
    if (skipped.length > 0) {
      cards.push(
        div()
          .flex_none()
          .px(4)
          .text_size(11)
          .text_color(t.blocked)
          .child(`這些變更讀不到 metadata：${skipped.map((s) => s.change).join("、")}`),
      );
    }
    return cards;
  }

  render(cx) {
    const t = theme();
    this.follow(cx);
    const ctx = this.context();
    const overlay = props()?.overlay;
    return v_flex()
      .id("speclink-panel")
      .size_full()
      .bg(t.popover)
      .text_size(12.5)
      .text_color(t.foreground)
      .child(
        h_flex()
          .flex_none()
          .h(38)
          .px(10)
          .gap(6)
          .items_center()
          .child(div().flex_1().min_w_0().truncate().font_semibold().child(this.root ? baseName(this.root) : "Speclink"))
          .child(
            h_flex()
              .id("new-work")
              .h(24)
              .px(7)
              .gap(4)
              .items_center()
              .rounded(5)
              .border_1()
              .border_color(t.input)
              .text_size(11.5)
              .cursor_pointer()
              .hover((s) => s.bg(t.hover))
              .on_mouse_down("left", (_e, cx) => cx.stop_propagation())
              .on_click((_e, cx) => {
                this.newWork = !this.newWork;
                cx.notify();
              })
              .child(icon("plus", 12, t.foreground))
              .child("新工作"),
          )
          .when(Boolean(overlay), (el) =>
            el.child(
              div()
                .id("close")
                .p(4)
                .rounded(5)
                .cursor_pointer()
                .hover((s) => s.bg(t.hover))
                .on_mouse_down("left", (_e, cx) => cx.stop_propagation())
                .on_click(() => closeOverlay(overlay))
                .child(icon("x", 13, t.muted)),
            ),
          ),
      )
      .when(this.newWork && Boolean(this.root), (el) =>
        el.child(div().flex_none().on_mouse_down("left", (_e, cx) => cx.stop_propagation()).child(this.newWorkPanel(ctx, t))),
      )
      .child(
        v_flex()
          .id("speclink-body")
          .flex_1()
          .min_h(0)
          .overflow_y_scrollbar()
          .px(8)
          .pb(8)
          .gap(8)
          // 視窗只能從標頭拖，內容區的按壓留給捲動與按鈕。
          .on_mouse_down("left", (_e, cx) => cx.stop_propagation())
          .when(Boolean(this.error), (el) =>
            el.child(div().flex_none().px(4).text_size(11.5).text_color(t.blocked).child(this.error)),
          )
          .when(!this.board && !this.error, (el) =>
            el.child(div().flex_none().px(4).text_size(11.5).text_color(t.dim).child("載入中…")),
          )
          .when(Boolean(this.board), (el) => el.children(this.body(ctx, t))),
      );
  }
}
