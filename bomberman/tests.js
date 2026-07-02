"use strict";
// bomberman/tests.js — mocked-DOM test suite for bomberman/index.html
// Run with: node bomberman/tests.js
// Every test maps to a spec scenario (openspec/specs/**). Loads the <script> body of
// index.html into a node vm sandbox and drives it through the window.__bomberman hook.

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const HTML_PATH = path.join(__dirname, "index.html");

// ---------- tiny test runner ----------
let passed = 0, failed = 0;
function test(name, fn) {
  try { fn(); passed++; console.log("PASS  " + name); }
  catch (e) { failed++; console.log("FAIL  " + name + "\n      " + e.message); }
}
function assert(cond, msg) { if (!cond) throw new Error(msg || "assertion failed"); }
function assertEq(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error((msg || "values differ") + ": expected " + JSON.stringify(expected) + ", got " + JSON.stringify(actual));
  }
}

// ---------- mocks ----------
function makeCtx2d(record) {
  const noops = ["clearRect", "fillRect", "strokeRect", "beginPath", "closePath",
    "moveTo", "lineTo", "arc", "fill", "stroke", "save", "restore", "rect"];
  const ctx = {};
  for (const m of noops) ctx[m] = function () {};
  ctx.fillText = function (text) { record.fillText.push(String(text)); };
  return ctx;
}

function makeSandbox() {
  const record = { fillText: [] };
  const ctx = makeCtx2d(record);
  const canvas = { width: 600, height: 560, getContext: () => ctx };
  const sandbox = {
    document: {
      getElementById: () => canvas,
      addEventListener: function () {},
    },
    requestAnimationFrame: function () {},
    Date: { now: () => 42 },
    Math: Math,
    console: console,
  };
  sandbox.window = sandbox;
  vm.createContext(sandbox);
  const html = fs.readFileSync(HTML_PATH, "utf8");
  const m = html.match(/<script>([\s\S]*)<\/script>/);
  if (!m) throw new Error("no <script> block found in index.html");
  vm.runInContext(m[1], sandbox);
  const hook = sandbox.window.__bomberman;
  if (!hook) throw new Error("window.__bomberman hook missing");
  return { hook, record };
}

const EMPTY = 0, HARD = 1, SOFT = 2, TILE = 40;
function center(col, row) { return { x: col * TILE + TILE / 2, y: row * TILE + TILE / 2 }; }

// Pin all enemies to far corners so long step() runs stay deterministic near the spawn.
function pinEnemies(hook) {
  const corners = [[13, 11], [13, 9], [11, 11], [13, 7]];
  const es = hook.state.enemies;
  for (let i = 0; i < es.length; i++) {
    const c = center(corners[i % corners.length][0], corners[i % corners.length][1]);
    es[i].x = c.x; es[i].y = c.y;
  }
}
function tapKey(hook, code) { hook.pressKey(code); hook.releaseKey(code); }

// ================= arena-layout =================

test("arena-layout/外圈與硬柱: outer ring and even-even pillars are HARD", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  const g = hook.state.grid;
  for (let c = 0; c < 15; c++) { assertEq(g[0][c], HARD, "row0"); assertEq(g[12][c], HARD, "row12"); }
  for (let r = 0; r < 13; r++) { assertEq(g[r][0], HARD, "col0"); assertEq(g[r][14], HARD, "col14"); }
  assertEq(g[2][2], HARD, "(2,2)"); assertEq(g[2][4], HARD, "(2,4)"); assertEq(g[10][12], HARD, "(10,12)");
});

test("arena-layout/軟磚分佈可重現: same seed → identical grid", () => {
  const { hook } = makeSandbox();
  hook.reset(123);
  const snap1 = JSON.stringify(hook.state.grid);
  hook.reset(123);
  assertEq(JSON.stringify(hook.state.grid), snap1, "grids differ across same-seed resets");
});

test("arena-layout/出生保留區: (1,1)(1,2)(2,1) EMPTY, player at tile center", () => {
  const { hook } = makeSandbox();
  hook.reset(7);
  const s = hook.state;
  assertEq(s.grid[1][1], EMPTY); assertEq(s.grid[1][2], EMPTY); assertEq(s.grid[2][1], EMPTY);
  assertEq(s.player.x, 60); assertEq(s.player.y, 60);
});

// ================= player-control =================

test("player-control/按鍵位移 Example: (60,60) + ArrowRight ×10 frames → (90,60)", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  hook.pressKey("ArrowRight");
  hook.step(10);
  assertEq(hook.state.player.x, 90); assertEq(hook.state.player.y, 60);
});

test("player-control/硬牆阻擋: holding ArrowLeft never crosses the inner boundary", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  hook.pressKey("ArrowLeft");
  hook.step(30);
  const p = hook.state.player;
  assert(p.x - 13 >= 40, "player crossed into the outer wall: x=" + p.x);
  assert(p.x < 60, "player did not move left at all");
});

// ================= bomb-mechanics =================

test("bomb-mechanics/放置與同格去重: Space twice on one tile → one bomb", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  tapKey(hook, "Space"); tapKey(hook, "Space");
  const bombs = hook.state.bombs;
  assertEq(bombs.length, 1);
  assertEq(bombs[0].col, 1); assertEq(bombs[0].row, 1);
});

test("bomb-mechanics/同時上限: cap 1 + one live bomb → second placement refused", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  tapKey(hook, "Space");
  hook.pressKey("ArrowRight"); hook.step(14); hook.releaseKey("ArrowRight");
  tapKey(hook, "Space");
  assertEq(hook.state.bombs.length, 1, "cap ignored");
});

test("bomb-mechanics/引信計時: no blast at 149 frames, blast at 150", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  tapKey(hook, "Space");
  hook.step(149);
  assertEq(hook.state.bombs.length, 1, "bomb detonated early");
  assertEq(hook.state.blasts.length, 0);
  hook.step(1);
  assertEq(hook.state.bombs.length, 0, "bomb did not detonate at 150");
  assertEq(hook.state.blasts.length, 1);
});

test("bomb-mechanics/走出後實體化: cannot walk back onto own bomb", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  tapKey(hook, "Space");
  hook.pressKey("ArrowRight"); hook.step(14); hook.releaseKey("ArrowRight");
  assertEq(hook.state.bombs[0].solid, true, "bomb did not solidify after walk-off");
  hook.pressKey("ArrowLeft"); hook.step(30); hook.releaseKey("ArrowLeft");
  assert(hook.state.player.x - 13 >= 80, "player re-entered the bomb tile: x=" + hook.state.player.x);
});

// ================= blast-resolution =================

test("blast-resolution/硬牆截斷: flames never include HARD tiles", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  tapKey(hook, "Space");
  hook.step(150);
  const s = hook.state;
  assertEq(s.blasts.length, 1);
  for (const t of s.blasts[0].tiles) {
    assert(s.grid[t.row][t.col] !== HARD, "flame on HARD tile (" + t.row + "," + t.col + ")");
    assert(t.col !== 0 && t.row !== 0, "flame extends into the outer wall");
  }
});

test("blast-resolution/軟磚納入即停: SOFT tile is included, propagation stops behind it", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  s.grid[1][2] = SOFT; s.grid[1][3] = EMPTY;
  tapKey(hook, "Space");
  hook.step(150);
  const tiles = hook.state.blasts[0].tiles;
  assert(tiles.some(t => t.col === 2 && t.row === 1), "SOFT tile not included");
  assert(!tiles.some(t => t.col === 3 && t.row === 1), "flame passed through the SOFT tile");
});

test("blast-resolution/範圍 2 全空傳播 Example: open field blast covers 9 tiles", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  for (const [c, r] of [[7, 7], [6, 7], [5, 7], [8, 7], [9, 7], [7, 6], [7, 5], [7, 8], [7, 9]]) {
    s.grid[r][c] = EMPTY;
  }
  s.player.x = 300; s.player.y = 300; // tile (7,7) center
  tapKey(hook, "Space");
  hook.step(150);
  assertEq(hook.state.blasts[0].tiles.length, 9);
});

test("blast-resolution/敵人被爆風擊殺: enemy in flames is removed", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  tapKey(hook, "Space");
  hook.step(149);
  const e = hook.state.enemies[0];
  e.x = 100; e.y = 60; // tile (1,2) — inside the incoming blast
  hook.step(1);
  assertEq(hook.state.enemiesLeft, 3, "enemy in flames survived");
});

test("blast-resolution/玩家被爆風擊中: player overlapping flames loses a life", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  tapKey(hook, "Space");
  hook.step(150); // player still stands on the bomb tile
  assertEq(hook.state.lives, 2);
});

// ================= destructibles-and-powerups =================

test("destructibles/軟磚被摧毀: SOFT tile becomes EMPTY when flames end", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  s.grid[1][2] = SOFT;
  tapKey(hook, "Space");
  s.player.x = 300; s.player.y = 300; // step out of the blast zone
  hook.step(150 + 24);
  assertEq(hook.state.grid[1][2], EMPTY, "SOFT tile not destroyed");
  assertEq(hook.state.lives, 3, "player should not have been hit");
});

test("destructibles/掉落可重現: same seed + same ops → identical power-up drops", () => {
  function run(seed) {
    const { hook } = makeSandbox();
    hook.reset(seed);
    pinEnemies(hook);
    const s = hook.state;
    s.grid[1][2] = SOFT; s.grid[2][1] = SOFT;
    tapKey(hook, "Space");
    s.player.x = 300; s.player.y = 300;
    hook.step(150 + 24);
    return JSON.stringify(hook.state.powerups);
  }
  let seed = 1;
  while (run(seed) === "[]" && seed < 200) seed++;
  assert(seed < 200, "no seed produced a drop — DROP_RATE broken?");
  assertEq(run(seed), run(seed), "drops differ across identical runs");
});

test("destructibles/拾取 extra-bomb: cap becomes 2 and two bombs can coexist", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  s.grid[1][3] = EMPTY;
  s.powerups.push({ col: 2, row: 1, kind: "extra-bomb" });
  hook.pressKey("ArrowRight"); hook.step(14); hook.releaseKey("ArrowRight");
  assertEq(hook.state.bombCap, 2, "extra-bomb not applied");
  tapKey(hook, "Space");
  hook.pressKey("ArrowRight"); hook.step(14); hook.releaseKey("ArrowRight");
  tapKey(hook, "Space");
  assertEq(hook.state.bombs.length, 2, "second bomb refused despite cap 2");
});

// ================= game-flow =================

test("game-flow/敵人生成距離: every enemy spawns at manhattan ≥ 6 from (1,1)", () => {
  for (const seed of [5, 42, 99]) {
    const { hook } = makeSandbox();
    hook.reset(seed);
    for (const e of hook.state.enemies) {
      const col = Math.floor(e.x / TILE), row = Math.floor(e.y / TILE);
      assert(Math.abs(col - 1) + Math.abs(row - 1) >= 6,
        "enemy too close at (" + col + "," + row + ") seed " + seed);
    }
  }
});

test("game-flow/敵人接觸擊殺+重生: contact costs a life and respawns at (60,60)", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  s.player.x = 300; s.player.y = 300;
  s.enemies[0].x = 300; s.enemies[0].y = 300;
  hook.step(1);
  assertEq(hook.state.lives, 2);
  assertEq(hook.state.player.x, 60); assertEq(hook.state.player.y, 60);
});

test("game-flow/命盡: third hit → phase gameover", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  for (let i = 0; i < 3; i++) {
    s.player.invuln = 0; // expire the respawn window so each hit lands
    s.player.x = 300; s.player.y = 300;
    s.enemies[0].x = 300; s.enemies[0].y = 300;
    hook.step(1);
  }
  assertEq(hook.state.phase, "gameover");
});

test("game-flow/全滅獲勝: killing the last enemy → phase victory", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  const s = hook.state;
  s.enemies.splice(1); // keep exactly one enemy
  tapKey(hook, "Space");
  hook.step(149);
  s.enemies[0].x = 100; s.enemies[0].y = 60;
  hook.step(1);
  assertEq(hook.state.phase, "victory");
});

test("game-flow/R 重開: restart resets lives/cap/range/enemies", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  const s = hook.state;
  s.enemies.splice(1);
  tapKey(hook, "Space");
  hook.step(149);
  s.enemies[0].x = 100; s.enemies[0].y = 60;
  hook.step(1);
  assertEq(hook.state.phase, "victory");
  tapKey(hook, "KeyR");
  const after = hook.state;
  assertEq(after.phase, "playing");
  assertEq(after.lives, 3); assertEq(after.bombCap, 1); assertEq(after.range, 2);
  assertEq(after.enemiesLeft, 4);
});

test("game-flow/暫停凍結: P freezes logic frames, second P resumes", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  tapKey(hook, "KeyP");
  const snap = JSON.stringify([hook.state.player, hook.state.enemies]);
  hook.pressKey("ArrowRight");
  hook.step(60);
  assertEq(hook.state.phase, "paused");
  assertEq(JSON.stringify([hook.state.player, hook.state.enemies]), snap, "state changed while paused");
  tapKey(hook, "KeyP");
  hook.step(1);
  assert(hook.state.player.x > 60, "player did not move after resume");
});

test("game-flow/HUD 更新: picking up extra-bomb updates the HUD immediately", () => {
  const { hook, record } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  hook.state.powerups.push({ col: 2, row: 1, kind: "extra-bomb" });
  hook.pressKey("ArrowRight"); hook.step(14); hook.releaseKey("ArrowRight");
  record.fillText.length = 0;
  hook.render();
  assert(record.fillText.some(t => t.indexOf("Bombs: 2") !== -1),
    "HUD does not show updated bomb cap: " + JSON.stringify(record.fillText));
});

// ================= ingest additions: chain detonation & invulnerability =================

test("blast-resolution/連鎖引爆: flames covering another bomb detonate it the same frame", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  for (const [c, r] of [[7, 7], [8, 7], [9, 7]]) s.grid[r][c] = EMPTY;
  s.bombs.push({ col: 7, row: 7, fuse: 1, solid: true });
  s.bombs.push({ col: 9, row: 7, fuse: 999, solid: true });
  hook.step(1);
  assertEq(hook.state.bombs.length, 0, "bomb B did not chain-detonate");
  assertEq(hook.state.blasts.length, 2, "expected two simultaneous flame groups");
});

test("game-flow/無敵窗: enemy overlap within 60 frames of respawn costs no life", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  s.player.x = 300; s.player.y = 300;
  s.enemies[0].x = 300; s.enemies[0].y = 300;
  hook.step(1);
  assertEq(hook.state.lives, 2, "first hit should cost a life");
  assert(hook.state.player.invuln > 0, "invulnerability window not started");
  s.enemies[0].x = 60; s.enemies[0].y = 60; // overlap the respawn point
  hook.step(30);
  assertEq(hook.state.lives, 2, "hit registered during the invulnerability window");
});

test("game-flow/無敵結束: after 120 frames the player is hittable again", () => {
  const { hook } = makeSandbox();
  hook.reset(42);
  pinEnemies(hook);
  const s = hook.state;
  s.player.x = 300; s.player.y = 300;
  s.enemies[0].x = 300; s.enemies[0].y = 300;
  hook.step(1); // lives 3 → 2, respawn, invuln 120
  for (let i = 0; i < 130; i++) {
    s.enemies[0].x = 60; s.enemies[0].y = 60; // keep the enemy pinned on the respawn point
    hook.step(1);
  }
  assertEq(hook.state.lives, 1, "invulnerability did not expire after 120 frames");
});

// ================= integration =================

test("integration: the test hook exposes every documented member", () => {
  const { hook } = makeSandbox();
  for (const k of ["state", "step", "pressKey", "releaseKey", "reset"]) {
    assert(k in hook, "hook missing " + k);
  }
  const s = hook.state;
  for (const k of ["grid", "player", "bombs", "blasts", "enemies", "powerups",
    "lives", "bombCap", "range", "enemiesLeft", "phase"]) {
    assert(k in s, "state missing " + k);
  }
  assert("invuln" in s.player, "player missing invuln");
});

console.log("\n" + passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
