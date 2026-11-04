// Speclink 面板：目前 space 的專案有哪些變更與討論、各自做到哪裡，以及按一下把技能交給對的
// agent。資料只來自 speclink CLI 的 --json；判斷邏輯都在 model.js。
import { View, div, image, svg } from "gpui-kit";
import { h_flex, v_flex } from "gpui-base";
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
  setShared,
  spaces,
  terminals,
  theme,
  watchFolder,
} from "herdr";
import {
  attachPlan,
  buildBoard,
  changeActions,
  changeAgents,
  deliveryHint,
  discussionActions,
  isUninitialized,
  joinAgents,
  logoOpenAttempts,
  speclinkUrl,
  startChoices,
  waveToStart,
} from "./model.js";
import ColumnSync from "./column-sync.js";
import { tr, watchLang } from "./i18n.js";
import SkillPicker from "./skill-picker.js";
import { codeRemaining, firstLine, location, openTabs, placeInInput, projectRoot, sendError, spaceFor, speclinkJson } from "./shared.js";

// 這個 mod 的其他畫面。speclink-new 開跳窗、dock-panel.mjs 放浮動區塊時，用 props 的 view 指定。
const VIEWS = { "skill-picker": SkillPicker, "column-sync": ColumnSync };
const ICONS = "mods/speclink/icons";
// 溢出摺疊的門檻與桌面 app 的系統匣面板相同。
const OVERFLOW_LIMIT = 5;
const STAGES = [
  { id: "proposed", label: "提案中", tone: "ansi3" },
  { id: "in-progress", label: "進行中", tone: "ansi4" },
  { id: "ready", label: "已就緒", tone: "ansi2" },
];
const AGENT_STATE = { working: "執行中", blocked: "在等你回答", done: "已完成", idle: "閒置" };

const icon = (name, size, color) => svg(`${ICONS}/${name}.svg`).size(size).flex_none().text_color(color);
const baseName = (path) => path.split("/").filter(Boolean).pop() ?? path;

// program.icon 是相對於 Herdr 設定資料夾的路徑：tint 表示單色圖示，其餘是圖片。
function programIcon(program, size, color) {
  if (!program?.icon) return div();
  return program.tint
    ? svg(program.icon).size(size).flex_none().text_color(color)
    : image(program.icon).w(size).h(size).flex_none();
}

export default class SpeclinkPanel extends View {
  init(_props, cx) {
    watchLang(exec, cx);
    const Other = VIEWS[props()?.view];
    if (Other) {
      this.other = new Other();
      this.other.init(cx);
      return;
    }
    this.place = location();
    this.root = null;
    this.board = null;
    this.error = null;
    this.uninitialized = false;
    this.known = null;
    this.copied = null;
    this.attaching = new Map();
    this.choosing = null;
    this.more = new Set();
    this.sending = new Set();
    this.starting = new Set();
    this.worktreePolicy = false;
    this.collapsed = new Set(JSON.parse(localStorage.getItem("collapsed") ?? "[]"));
    this.promotedOpen = localStorage.getItem("promotedOpen") === "true";
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
    const { cwd, on } = this.place;
    this.place = location();
    if (this.place.cwd === cwd && this.place.on === on) return;
    if (this.place.on !== on) this.loadKnown(cx);
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
          this.uninitialized = false;
          this.watch(root, on, cx);
        }
        const read = (...verb) => speclinkJson(verb, root, on);
        const [list, plan, discussions, config] = await Promise.all([
          read("list"),
          read("plan"),
          read("discuss", "list"),
          read("workflow-config", "show"),
        ]);
        this.board = buildBoard(list, plan, discussions, await codeRemaining(list, plan, root, on));
        this.worktreePolicy = config.worktree === true;
        this.error = null;
        this.uninitialized = false;
      } catch (e) {
        const message = String(e.message ?? e);
        this.uninitialized = isUninitialized(message);
        this.error = this.uninitialized ? null : message;
        // 切到沒有 Speclink 的分支時，舊的變更清單不能留著。
        if (this.uninitialized) this.board = null;
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
      space: f?.space ?? null,
      on: this.place.on,
      board: this.board,
      known: this.known,
      worktreePolicy: this.worktreePolicy,
      starting: this.starting,
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
        if (r.code === 0) {
          // 面板才知道這個 agent 接下來在做什麼。
          hc(["label", address, ...delivery.labels]);
          run("pane.focus", { terminal: address });
        } else notify({ title: tr("技能沒有送出"), body: sendError(r), level: "warning" });
        cx.notify();
      });
    } else if (delivery.kind === "new") {
      const before = new Set(terminals().map((t) => t.address));
      run("tab.new", {
        folder: delivery.folder,
        argv: delivery.argv,
        task: delivery.text,
        name: delivery.tabName,
        space: spaceFor(delivery.folder)?.id,
      });
      if (delivery.change) {
        this.labelNew(before, delivery, cx, 10);
        this.markStarting(delivery.change, cx);
      }
    }
  }

  // 開工和技能跳窗的開新分頁相同：開在目前的工作區，畫面留在原分頁。
  startTabs(plans, cx) {
    this.choosing = null;
    for (const plan of plans) this.markStarting(plan.change, cx);
    cx.spawn(async (cx) => {
      const failed = await openTabs(plans, cx);
      if (failed) notify({ title: tr("開工失敗"), body: sendError(failed), level: "warning" });
    });
  }

  // agent 開好並回報狀態要幾秒，這段時間列上還看不到它，再按一次開工會多開一個。
  markStarting(change, cx) {
    this.starting.add(change);
    cx.notify();
    cx.timer.after(15000, (cx) => {
      this.starting.delete(change);
      cx.notify();
    });
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

  // 叫 Speclink 桌面 app。open 只有 macOS 有；沒裝 app 時 open 也會失敗。
  openInApp(args, cx) {
    this.openFirstInApp([args], cx);
  }

  // 依序試 open，成功就停；全部失敗才跳通知，內容是最後一次的錯誤。
  openFirstInApp(attempts, cx) {
    cx.spawn(async () => {
      let stderr = "";
      for (const args of attempts) {
        const r = await exec("open", args).catch((e) => ({ code: 1, stderr: String(e.message ?? e) }));
        if (r.code === 0) return;
        stderr = r.stderr;
      }
      notify({ title: tr("開不了 Speclink"), body: firstLine(stderr), level: "warning" });
    });
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
      .on_click((_e, cx) => {
        cx.stop_propagation();
        this.copy(text, cx);
      })
      .child(icon(this.copied === text ? "check" : "copy", 12, t.muted));
  }

  chip({ id, label, ready, primary, tip, click }, t) {
    return h_flex()
      .id(id)
      .h(22)
      .px(7)
      .items_center()
      .rounded(5)
      .text_size(11.5)
      .when(primary, (el) => el.border_2().border_color(t.accent).font_semibold())
      .when(!primary, (el) => el.border_1().border_color(t.input))
      .when(ready, (el) => el.cursor_pointer().hover((s) => s.bg(t.hover)))
      .when(!ready, (el) => el.opacity(0.45))
      // 灰掉的鈕也要擋下點擊，否則會變成點到列本身。
      .on_click((_e, cx) => {
        cx.stop_propagation();
        if (ready) click(cx);
      })
      .when(Boolean(tip), (el) => el.tooltip(tip))
      .child(label);
  }

  skillButton(id, action, t) {
    const d = action.delivery;
    const tip = action.waitingFor
      ? tr("等 {0} 先完成", action.waitingFor.join(tr("、")))
      : d.text
        ? `${deliveryHint(d)}：${d.text}`
        : deliveryHint(d);
    return this.chip(
      {
        id: `skill-${id}-${action.skill}`,
        label: tr(action.skill),
        ready: d && (d.kind === "send" || d.kind === "new"),
        primary: action.primary,
        tip,
        click: (cx) => this.deliver(d, cx),
      },
      t,
    );
  }

  // 開工的一種做法。
  startChoice(id, { label, plan }, t) {
    const ready = plan.kind === "tab";
    return this.chip(
      {
        id: `start-${id}-${label}`,
        label: tr(label),
        ready,
        tip: ready ? tr("在目前的工作區開新分頁並執行：{0}", plan.text) : plan.reason,
        click: (cx) => this.startTabs([plan], cx),
      },
      t,
    );
  }

  // 只有一種做法就直接開工，有兩種時按下去才展開選項。
  startButton(id, target, choices, t) {
    if (choices.length === 1) return this.startChoice(id, choices[0], t);
    return this.chip(
      {
        id: `start-${id}`,
        label: tr("開工"),
        ready: true,
        tip: tr("選擇用 worktree 或直接 apply"),
        click: (cx) => {
          this.choosing = target;
          cx.notify();
        },
      },
      t,
    );
  }

  // 貼上要約 0.7 秒，所以按下去要先有反應。貼上中再按一次不算，否則名稱會貼兩次。
  attach(plan, name, cx) {
    if (this.attaching.has(name)) return;
    this.attaching.set(name, "pasting");
    cx.notify();
    cx.spawn(async (cx) => {
      const failed = await placeInInput(plan);
      if (failed) {
        this.attaching.delete(name);
        notify({ title: tr("沒有放進輸入框"), body: sendError(failed), level: "warning" });
      } else {
        this.attaching.set(name, "done");
        cx.timer.after(1200, (cx) => {
          this.attaching.delete(name);
          cx.notify();
        });
      }
      cx.notify();
    });
  }

  // 把名稱接到目前分頁的輸入框，不送出。
  attachButton(id, name, ctx, t) {
    const plan = attachPlan({ name, agents: ctx.agents, focus: ctx.focus });
    const ready = plan.kind !== "none";
    const state = this.attaching.get(name);
    return div()
      .id(`attach-${id}`)
      .flex_none()
      .p(3)
      .rounded(4)
      .when(ready, (el) => el.cursor_pointer().hover((s) => s.bg(t.hover)))
      .when(!ready && !state, (el) => el.opacity(0.45))
      .tooltip(ready ? tr("附加到目前分頁的輸入框") : plan.reason)
      .on_click((_e, cx) => {
        cx.stop_propagation();
        if (ready) this.attach(plan, name, cx);
      })
      .child(
        state === "pasting"
          ? icon("spinner", 12, t.working).spin(900)
          : icon(state === "done" ? "check" : "message-plus", 12, t.muted),
      );
  }

  // 其他技能在跳窗裡：開跳窗，並先選好這個技能和這一列。
  moreButton(id, skill, target, t) {
    return h_flex()
      .id(`more-${id}`)
      .flex_none()
      .h(22)
      .px(5)
      .items_center()
      .rounded(5)
      .text_color(t.muted)
      .cursor_pointer()
      .hover((s) => s.bg(t.hover))
      .tooltip(tr("其他技能"))
      .on_click((_e, cx) => {
        cx.stop_propagation();
        setShared("speclink.pick", { skill, target, when: Date.now() });
        run("mod.open", { name: "speclink-new" });
      })
      .child("⋯");
  }

  // 每列只留建議的下一步。能開工的列多一顆開工。
  skillRow({ id, actions, start, target, hint }, t) {
    // 開工要選做法時，這一列只放選項。
    if (start?.length > 1 && this.choosing === target) {
      return h_flex()
        .gap(4)
        .items_center()
        .children(start.map((choice) => this.startChoice(id, choice, t)))
        .child(
          this.chip(
            {
              id: `start-${id}-cancel`,
              label: tr("取消"),
              ready: true,
              click: (cx) => {
                this.choosing = null;
                cx.notify();
              },
            },
            t,
          ),
        );
    }
    const next = actions.find((a) => a.primary) ?? actions[0];
    return h_flex()
      .gap(4)
      .items_center()
      .when(Boolean(start), (el) => el.child(this.startButton(id, target, start, t)))
      .child(this.skillButton(id, next, t))
      .child(this.moreButton(id, next.skill, target, t))
      .when(Boolean(hint), (el) =>
        el.child(div().flex_1().min_w_0().truncate().text_size(11).text_color(t.dim).child(hint)),
      );
  }

  // 和 Herdr 內建的 Agents 清單同一套：做事中是轉動的圓弧，閒置是空心圈，做完是實心點，等回答是膠囊。
  agentStatus(view, t) {
    if (view === "working") return icon("spinner", 12, t.working).mx(1).spin(900);
    if (view === "blocked") {
      return div()
        .flex()
        .flex_none()
        .h(18)
        .px(6)
        .items_center()
        .rounded(5)
        .bg(`${t.blocked}26`)
        .text_size(10.5)
        .font_semibold()
        .text_color(t.blocked)
        .child(tr(AGENT_STATE.blocked));
    }
    if (view === "idle") return div().size(7).mx(3).flex_none().rounded(4).border(1.5).border_color(t.idle);
    return div().size(7).mx(3).flex_none().rounded(4).bg(t[view] ?? t.dim);
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
      .tooltip(tr("切到這個 agent（{0}）", tr(AGENT_STATE[a.view] ?? a.view)))
      .on_click((_e, cx) => {
        cx.stop_propagation();
        run("agent.focus", { terminal: a.address });
      })
      .child(programIcon(a.program, 12, t.muted))
      .child(div().flex_1().min_w_0().truncate().text_color(t.muted).child(`${a.space_name} / ${a.tab_name}`))
      .child(this.agentStatus(a.view, t));
  }

  // 點列本身叫 Speclink 開詳情。列上的鈕都會擋下點擊。別台機器的專案路徑在這台開不了。
  openOnClick(el, url, t) {
    if (this.place.on) return el;
    return el
      .cursor_pointer()
      .hover((s) => s.bg(t.hover))
      .on_click((_e, cx) => this.openInApp([url], cx));
  }

  waveNumber(row, t) {
    const number = div()
      .id(`wave-${row.name}`)
      .flex_none()
      .text_size(11)
      .font_semibold()
      .text_color(t.accent)
      .child(String(row.wave));
    if (this.place.on) return number;
    return number
      .px(3)
      .rounded(3)
      .cursor_pointer()
      .hover((s) => s.bg(t.raised))
      .tooltip(tr("在 Speclink 開路線圖"))
      .on_click((_e, cx) => {
        cx.stop_propagation();
        this.openInApp([speclinkUrl("roadmap", this.root)], cx);
      });
  }

  changeRow(row, stage, ctx, t) {
    const actions = changeActions(row, ctx);
    const start = startChoices(row, ctx);
    const blocked = row.blockedBy.length > 0;
    const hint = blocked ? tr("等 {0}", row.blockedBy.join(tr("、"))) : "";
    const percent = row.total > 0 ? Math.round((row.completed / row.total) * 100) : 0;
    const el = v_flex()
      .id(`change-${row.name}`)
      .px(10)
      .py(8)
      .gap(5)
      .child(
        h_flex()
          .gap(6)
          .items_center()
          .when(blocked, (el) => el.opacity(0.6))
          .child(this.waveNumber(row, t))
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
          .child(this.attachButton(`change-${row.name}`, row.name, ctx, t))
          .child(this.copyButton(`change-${row.name}`, row.name, tr("複製名稱"), t)),
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
      .child(this.skillRow({ id: `change-${row.name}`, actions, start, target: row.name, hint }, t));
    return this.openOnClick(el, speclinkUrl("change", this.root, row.name), t);
  }

  discussionRow(d, ctx, t) {
    const actions = discussionActions(d, ctx);
    const hint = d.cuts.length > 0 ? tr("還有 {0} 刀沒轉出", d.cuts.length) : "";
    const el = v_flex()
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
          .child(this.attachButton(`discussion-${d.slug}`, d.slug, ctx, t))
          .child(this.copyButton(`discussion-${d.slug}`, d.slug, tr("複製 slug"), t)),
      )
      .child(this.skillRow({ id: `discussion-${d.slug}`, actions, target: d.slug, hint }, t));
    return this.openOnClick(el, speclinkUrl("discussion", this.root, d.slug), t);
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
        .child(open ? tr("收合") : tr("還有 {0} 個…", rows.length - OVERFLOW_LIMIT)),
    ];
  }

  card({ id, tone, label, rows, open }, t) {
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
          .child(div().w(8).h(8).flex_none().rounded_full().bg(tone))
          .child(div().font_semibold().text_size(12).child(tr(label)))
          .child(div().px(6).rounded_full().bg(t.raised).text_size(11).text_color(t.muted).child(String(rows.length)))
          .child(div().flex_1())
          .child(icon(open ? "chevron-down" : "chevron-right", 13, t.muted)),
      )
      .when(open, (el) => el.children(this.overflow(id, rows, t)));
  }

  // 整波開工不再多問一次：同一列已經寫出會開哪些變更。
  waveBar({ wave, plans }, t) {
    return h_flex()
      .flex_none()
      .gap(8)
      .px(10)
      .py(6)
      .items_center()
      .rounded(10)
      .border_1()
      .border_color(t.border)
      .bg(t.surface)
      .child(
        div()
          .flex_1()
          .min_w_0()
          .text_size(11.5)
          .text_color(t.muted)
          .child(tr("第 {0} 波可以一起開工：{1}", wave, plans.map((p) => p.change).join(tr("、")))),
      )
      .child(
        h_flex()
          .id("start-wave")
          .flex_none()
          .h(22)
          .px(7)
          .items_center()
          .rounded(5)
          .border_1()
          .border_color(t.input)
          .text_size(11.5)
          .cursor_pointer()
          .hover((s) => s.bg(t.hover))
          .tooltip(tr("開 {0} 個新分頁，各跑一個 agent", plans.length))
          .on_click((_e, cx) => this.startTabs(plans, cx))
          .child(tr("整波開工")),
      );
  }

  // 開新分頁跑 speclink init，不交給 agent：它要在真的終端機逐題問裝哪些技能，答完才寫入檔案。
  // 寫出資料夾，按之前看得到會裝在哪裡。
  initCard(t) {
    return v_flex()
      .flex_none()
      .px(10)
      .py(8)
      .gap(6)
      .rounded(10)
      .border_1()
      .border_color(t.border)
      .bg(t.surface)
      .child(div().font_semibold().text_size(12).child(tr("這個專案還沒有 Speclink")))
      .child(div().text_size(11).text_color(t.muted).child(this.root))
      .child(
        h_flex().child(
          h_flex()
            .id("init")
            .h(22)
            .px(7)
            .items_center()
            .rounded(5)
            .border_2()
            .border_color(t.accent)
            .font_semibold()
            .text_size(11.5)
            .cursor_pointer()
            .hover((s) => s.bg(t.hover))
            .tooltip(tr("在新分頁執行 speclink init"))
            .on_click(() => run("tab.new", { name: "speclink init", folder: this.root, argv: ["speclink", "init"] }))
            .child(tr("初始化")),
        ),
      );
  }

  body(ctx, t) {
    const { stages, discussions, promoted, skipped } = this.board;
    const wave = waveToStart(stages, ctx);
    const cards = wave ? [this.waveBar(wave, t)] : [];
    for (const stage of STAGES) {
      cards.push(
        this.card(
          {
            id: stage.id,
            tone: t[stage.tone],
            label: stage.label,
            rows: stages[stage.id].map((row) => this.changeRow(row, stage, ctx, t)),
            open: !this.collapsed.has(stage.id),
          },
          t,
        ),
      );
    }
    cards.push(
      this.card(
        {
          id: "discussions",
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
          .child(tr("這些變更讀不到 metadata：{0}", skipped.map((s) => s.change).join(tr("、")))),
      );
    }
    return cards;
  }

  headerButton(id, iconName, label, t, onClick) {
    return h_flex()
      .id(id)
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
      .on_click((_e, cx) => onClick(cx))
      .child(icon(iconName, 12, t.foreground))
      .child(label);
  }

  // Logo 也是叫出 Speclink app 的鈕，讀得到這台機器上的專案時會切過去。別台機器的 space 和
  // 沒初始化的專案也能按，只叫出 app。
  logoButton(t) {
    const attempts = logoOpenAttempts({ root: this.root, hasBoard: Boolean(this.board), on: this.place.on });
    return div()
      .id("open-app")
      .flex_none()
      .p(3)
      .rounded(5)
      .cursor_pointer()
      .hover((s) => s.bg(t.hover))
      .tooltip(attempts.length > 1 ? tr("在 Speclink app 開啟這個專案") : tr("開啟 Speclink app"))
      .on_mouse_down("left", (_e, cx) => cx.stop_propagation())
      .on_click((_e, cx) => this.openFirstInApp(attempts, cx))
      .child(image(`${ICONS}/logo.svg`).w(21).h(24));
  }

  render(cx) {
    if (this.other) return this.other.render(cx);
    const t = theme();
    this.follow(cx);
    const ctx = this.context();
    const overlay = props()?.overlay;
    return v_flex()
      .id("speclink-panel")
      .size_full()
      // 沒有 overlay 就是停靠在版面裡，底色和側欄一樣，卡片才浮得出來。
      .bg(overlay ? t.popover : t.chrome)
      .text_size(12.5)
      .text_color(t.foreground)
      .child(
        h_flex()
          .flex_none()
          .h(46)
          .px(10)
          .gap(6)
          .items_center()
          .child(this.logoButton(t))
          .child(
            v_flex()
              .flex_1()
              .min_w_0()
              .child(div().truncate().font_semibold().child(this.root ? baseName(this.root) : "Speclink"))
              .child(
                div()
                  .truncate()
                  .text_size(11)
                  .text_color(t.muted)
                  .child([tr("目前的工作區"), this.place.branch].filter(Boolean).join(" · ")),
              ),
          )
          // 讀不到專案（資料夾沒有 speclink）時不給鈕：網址會跳到桌面 app 的初始化確認框。
          .when(Boolean(this.board) && !this.place.on, (el) =>
            el.child(
              this.headerButton("roadmap", "roadmap", tr("路線圖"), t, (cx) =>
                this.openInApp([speclinkUrl("roadmap", this.root)], cx),
              ),
            ),
          )
          // 還沒初始化時，這些技能在這個專案還不存在。
          .when(!this.uninitialized, (el) =>
            el.child(
              this.headerButton("skills", "zap", tr("技能"), t, () => run("mod.open", { name: "speclink-new" })),
            ),
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
          .when(this.uninitialized, (el) => el.child(this.initCard(t)))
          .when(!this.board && !this.error && !this.uninitialized, (el) =>
            el.child(div().flex_none().px(4).text_size(11.5).text_color(t.dim).child(tr("載入中…"))),
          )
          .when(Boolean(this.board), (el) => el.children(this.body(ctx, t))),
      );
  }
}
