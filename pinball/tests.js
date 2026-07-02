"use strict";
// pinball/tests.js — mocked-DOM test suite for pinball/index.html
// Run with: node pinball/tests.js
// Mocks: document / canvas 2d context / localStorage / AudioContext / requestAnimationFrame.
// Loads the <script> body of index.html into a node vm sandbox and drives it
// through the window.__pinball test hook.

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
function makeLocalStorage(opts = {}) {
  const store = Object.create(null);
  return {
    getItem(k) { if (opts.throws) throw new Error("storage disabled"); return k in store ? store[k] : null; },
    setItem(k, v) { if (opts.throws) throw new Error("storage disabled"); store[k] = String(v); },
    removeItem(k) { if (opts.throws) throw new Error("storage disabled"); delete store[k]; },
    _store: store,
  };
}

function makeCtx2d(record) {
  const noops = [
    "clearRect", "fillRect", "strokeRect", "beginPath", "closePath", "moveTo",
    "lineTo", "arc", "fill", "stroke", "save", "restore", "rect", "translate",
    "rotate", "scale", "setLineDash",
  ];
  const ctx = {};
  for (const m of noops) ctx[m] = function () {};
  ctx.fillText = function (text) { record.fillText.push(String(text)); };
  ctx.strokeText = function (text) { record.fillText.push(String(text)); };
  return ctx;
}

function makeAudioParam() {
  const p = { value: 0, events: [] };
  p.setValueAtTime = function (v) { p.events.push(v); return p; };
  p.exponentialRampToValueAtTime = function (v) { p.events.push(v); return p; };
  p.linearRampToValueAtTime = function (v) { p.events.push(v); return p; };
  return p;
}

function makeAudioContextClass(record) {
  return class MockAudioContext {
    constructor() {
      this.currentTime = 0;
      this.destination = { name: "destination" };
      this.state = "running";
      record.audioContexts++;
    }
    resume() { return Promise.resolve(); }
    createOscillator() {
      const osc = {
        type: "sine",
        frequency: makeAudioParam(),
        connect(node) { return node; },
        start() { record.oscillators.push(osc); },
        stop() {},
      };
      return osc;
    }
    createGain() {
      return { gain: makeAudioParam(), connect(node) { return node; } };
    }
  };
}

// ---------- environment loader ----------
function createEnv(opts = {}) {
  const html = fs.readFileSync(HTML_PATH, "utf8");
  const m = html.match(/<script>([\s\S]*?)<\/script>/);
  if (!m) throw new Error("no <script> block found in index.html");

  const record = { fillText: [], audioContexts: 0, oscillators: [], raf: [] };
  const listeners = {};
  const ctx2d = makeCtx2d(record);
  const canvas = { id: "game", width: 440, height: 660, getContext: () => ctx2d };
  const sandbox = {
    console,
    document: { getElementById: (id) => (id === "game" ? canvas : null) },
    addEventListener: (type, fn) => { (listeners[type] = listeners[type] || []).push(fn); },
    requestAnimationFrame: (cb) => { record.raf.push(cb); return record.raf.length; },
    localStorage: opts.localStorage || makeLocalStorage(),
    AudioContext: makeAudioContextClass(record),
  };
  sandbox.window = sandbox;
  sandbox.globalThis = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(m[1], sandbox, { filename: "index.html<script>" });

  const env = {
    record, listeners, sandbox,
    get pb() { return sandbox.window.__pinball; },
    press(key, code) {
      (listeners.keydown || []).forEach((fn) => fn({ key, code: code || "", repeat: false, preventDefault() {} }));
    },
    release(key) {
      (listeners.keyup || []).forEach((fn) => fn({ key, code: "" }));
    },
    step(dt) { env.pb.step(dt); },
    stepSeconds(sec) {
      const DT = 1 / 120;
      for (let t = 0; t < sec; t += DT) env.pb.step(DT);
    },
    render() {
      record.fillText.length = 0;
      env.pb.render();
      return record.fillText;
    },
    startedFreqs() { return record.oscillators.map((o) => o.frequency.events[0]); },
    drainBall() {
      const pb = env.pb;
      pb.ball.state = "inPlay";
      pb.ball.x = 220; pb.ball.y = 640; pb.ball.vx = 0; pb.ball.vy = 200;
      env.step(1 / 120);
    },
  };
  return env;
}

// =====================================================================
// 1. Harness self-check + baseline regression (existing behavior)
// =====================================================================

test("harness: script loads and exposes the window.__pinball hook", () => {
  const env = createEnv();
  assert(env.pb, "window.__pinball is missing");
  for (const k of ["step", "render", "serveBall", "resetGame", "launch", "ball", "flippers", "bumpers"]) {
    assert(k in env.pb, "hook is missing member: " + k);
  }
});

test("baseline: ball serves in waiting state and Space launches it", () => {
  const env = createEnv();
  assertEq(env.pb.ball.state, "waiting", "initial ball state");
  env.press(" ", "Space");
  assertEq(env.pb.ball.state, "inPlay", "state after Space");
});

test("baseline: launch is ignored while a ball is already in play", () => {
  const env = createEnv();
  env.press(" ", "Space");
  const vx = env.pb.ball.vx, vy = env.pb.ball.vy;
  env.press(" ", "Space");
  assertEq(env.pb.ball.vx, vx, "vx unchanged by second Space");
  assertEq(env.pb.ball.vy, vy, "vy unchanged by second Space");
});

test("baseline: gravity pulls a ball in play downward", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 200; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
  env.stepSeconds(0.1);
  assert(pb.ball.vy > 0, "vy did not increase under gravity: " + pb.ball.vy);
  assert(pb.ball.y > 300, "ball did not fall: " + pb.ball.y);
});

test("baseline: holding ArrowLeft raises the left flipper, release returns it to rest", () => {
  const env = createEnv(); const pb = env.pb;
  const rest = pb.flippers[0].angle;
  env.press("ArrowLeft");
  env.stepSeconds(0.3);
  assert(pb.flippers[0].angle < rest, "left flipper did not raise");
  env.release("ArrowLeft");
  env.stepSeconds(0.5);
  assert(Math.abs(pb.flippers[0].angle - rest) < 0.05, "flipper did not return to rest");
});

test("baseline: alternate key A raises the left flipper too", () => {
  const env = createEnv(); const pb = env.pb;
  const rest = pb.flippers[0].angle;
  env.press("a");
  env.stepSeconds(0.3);
  assert(pb.flippers[0].angle < rest, "left flipper did not raise with A");
  env.release("a");
});

test("baseline: bumper hit rebounds the ball and increases the score", () => {
  const env = createEnv(); const pb = env.pb;
  const b = pb.bumpers[0];
  pb.ball.state = "inPlay"; pb.ball.x = b.x; pb.ball.y = b.y - b.r - 2; pb.ball.vx = 0; pb.ball.vy = 120;
  const before = pb.score;
  env.stepSeconds(0.05);
  assert(pb.score > before, "score did not increase on bumper hit");
  assert(pb.ball.vy < 0, "ball was not rebounded upward: vy=" + pb.ball.vy);
});

test("baseline: drain consumes a ball; last ball triggers game over", () => {
  const env = createEnv(); const pb = env.pb;
  env.drainBall();
  assertEq(pb.balls, 2, "balls after first drain");
  assertEq(pb.ball.state, "waiting", "new ball served");
  env.drainBall();
  env.drainBall();
  assertEq(pb.over, true, "game over after last drain");
});

test("baseline: R restarts the game after game over", () => {
  const env = createEnv(); const pb = env.pb;
  env.drainBall(); env.drainBall(); env.drainBall();
  assertEq(pb.over, true, "precondition: game over");
  env.press("r");
  assertEq(pb.over, false, "over cleared by R");
  assertEq(pb.balls, 3, "balls reset by R");
  assertEq(pb.score, 0, "score reset by R");
});

// =====================================================================
// 2. audio-feedback (Synthesized Sound Effects, Mute Toggle)
// =====================================================================

test("audio: pressing ArrowLeft starts a synthesized flipper sound (240Hz) within the same event", () => {
  const env = createEnv();
  env.press("ArrowLeft");
  assert(env.startedFreqs().includes(240), "no flipper oscillator started: " + env.startedFreqs());
});

test("audio: alternate key L also triggers the flipper sound", () => {
  const env = createEnv();
  env.press("l");
  assert(env.startedFreqs().includes(240), "no flipper oscillator started for L");
});

test("audio: bumper hit plays 660Hz; slingshot sound is a distinct 330Hz", () => {
  const env = createEnv(); const pb = env.pb;
  const b = pb.bumpers[0];
  pb.ball.state = "inPlay"; pb.ball.x = b.x; pb.ball.y = b.y - b.r - 2; pb.ball.vx = 0; pb.ball.vy = 120;
  env.stepSeconds(0.05);
  assert(env.startedFreqs().includes(660), "no bumper sound: " + env.startedFreqs());
  pb.playSound("sling");
  assert(env.startedFreqs().includes(330), "no slingshot sound: " + env.startedFreqs());
});

test("audio: drain plays a 300Hz descending sound; last-ball drain adds the game-over melody", () => {
  const env = createEnv();
  env.drainBall();
  assert(env.startedFreqs().includes(300), "no drain sound: " + env.startedFreqs());
  env.drainBall();
  env.drainBall();
  assert(env.pb.over, "precondition: game over");
  assert(env.startedFreqs().includes(392), "no game-over melody note: " + env.startedFreqs());
});

test("audio: M mutes all sounds; pressing M again restores them", () => {
  const env = createEnv(); const pb = env.pb;
  env.press("m");
  const countAfterMute = env.record.oscillators.length;
  const b = pb.bumpers[0];
  pb.ball.state = "inPlay"; pb.ball.x = b.x; pb.ball.y = b.y - b.r - 2; pb.ball.vx = 0; pb.ball.vy = 120;
  env.stepSeconds(0.05);
  pb.playSound("sling");
  assertEq(env.record.oscillators.length, countAfterMute, "sounds were synthesized while muted");
  env.press("m");
  pb.playSound("sling");
  assert(env.record.oscillators.length > countAfterMute, "sounds did not resume after unmute");
});

test("audio: index.html references no external audio files", () => {
  const html = fs.readFileSync(HTML_PATH, "utf8");
  assert(!/<audio|\.mp3|\.wav|\.ogg/i.test(html), "external audio reference found");
});

// =====================================================================
// 3. hit-effects (Particle Burst On Hit, Hit Flash)
// =====================================================================

test("particles: bumper hit emits at least 10 particles that fade within 600ms", () => {
  const env = createEnv(); const pb = env.pb;
  const b = pb.bumpers[0];
  pb.ball.state = "inPlay"; pb.ball.x = b.x; pb.ball.y = b.y - b.r - 2; pb.ball.vx = 0; pb.ball.vy = 120;
  env.stepSeconds(0.05);
  assert(pb.particles.length >= 10, "expected >=10 particles, got " + pb.particles.length);
  pb.ball.state = "waiting";
  env.stepSeconds(0.65);
  assertEq(pb.particles.length, 0, "particles did not fade out within 600ms");
});

test("particles: live particle count never exceeds the 150 cap", () => {
  const env = createEnv(); const pb = env.pb;
  for (let i = 0; i < 30; i++) pb.spawnParticles(220, 300, "#fff");
  assert(pb.particles.length <= 150, "cap exceeded: " + pb.particles.length);
});

test("flash: bumper hit sets a flash that lasts >=100ms and clears within 300ms", () => {
  const env = createEnv(); const pb = env.pb;
  const b = pb.bumpers[0];
  pb.ball.state = "inPlay"; pb.ball.x = b.x; pb.ball.y = b.y - b.r - 2; pb.ball.vx = 0; pb.ball.vy = 120;
  env.step(1 / 120);
  assert(b.flash >= 0.1, "flash shorter than 100ms: " + b.flash);
  pb.ball.state = "waiting";
  env.stepSeconds(0.3);
  assert(b.flash <= 0, "flash did not clear within 300ms: " + b.flash);
});

// =====================================================================
// 4. pinball-table additions (Slingshot Rebound And Scoring, Drop Target Bank)
// =====================================================================

// Places the ball just off the left slingshot face, moving into it.
function hitLeftSlingshot(env) {
  const pb = env.pb;
  pb.ball.state = "inPlay";
  pb.ball.x = 80.65; pb.ball.y = 450.6;   // 5px off the face midpoint, field side
  pb.ball.vx = -219; pb.ball.vy = 204;    // moving into the face
  env.step(1 / 120);
}

test("slingshot: two slingshots exist in the lower play field", () => {
  const env = createEnv();
  assertEq(env.pb.slingshots.length, 2, "slingshot count");
});

test("slingshot: hit rebounds the ball with added impulse and awards 75 points", () => {
  const env = createEnv(); const pb = env.pb;
  const before = pb.score;
  hitLeftSlingshot(env);
  assertEq(pb.score - before, 75, "slingshot award");
  assert(Math.hypot(pb.ball.vx, pb.ball.vy) > 200, "ball was not kicked away");
  assert(pb.ball.vx > 0, "ball was not rebounded away from the face: vx=" + pb.ball.vx);
});

test("slingshot: hit flashes, emits particles and plays the 330Hz sling sound", () => {
  const env = createEnv(); const pb = env.pb;
  hitLeftSlingshot(env);
  assert(pb.slingshots[0].flash > 0, "no flash on slingshot");
  assert(pb.particles.length >= 10, "no particle burst on slingshot");
  assert(env.startedFreqs().includes(330), "no sling sound: " + env.startedFreqs());
});

// Drops the ball straight onto target index i, then parks it in waiting state.
function knockTarget(env, i) {
  const pb = env.pb;
  const t = pb.targets[i];
  pb.ball.state = "inPlay";
  pb.ball.x = t.x + t.w / 2; pb.ball.y = t.y - 13; pb.ball.vx = 0; pb.ball.vy = 150;
  env.stepSeconds(0.05);
  pb.ball.state = "waiting"; pb.ball.x = 220; pb.ball.y = 250; pb.ball.vx = 0; pb.ball.vy = 0;
}

test("targets: a bank of three standing drop targets exists", () => {
  const env = createEnv();
  assertEq(env.pb.targets.length, 3, "target count");
  assert(env.pb.targets.every((t) => t.alive), "targets not all standing at start");
});

test("targets: hitting a standing target knocks it down and awards 150 points", () => {
  const env = createEnv(); const pb = env.pb;
  const before = pb.score;
  knockTarget(env, 0);
  assertEq(pb.targets[0].alive, false, "target still standing");
  assertEq(pb.score - before, 150, "target award");
});

test("targets: a downed target no longer collides or scores", () => {
  const env = createEnv(); const pb = env.pb;
  knockTarget(env, 0);
  const before = pb.score;
  knockTarget(env, 0);
  assertEq(pb.score - before, 0, "downed target scored again");
});

test("targets: clearing the bank adds a flat 2000 bonus and resets all targets", () => {
  const env = createEnv(); const pb = env.pb;
  const before = pb.score;
  knockTarget(env, 0);
  env.stepSeconds(3.05);   // > combo window, keeps every hit at x1
  knockTarget(env, 1);
  env.stepSeconds(3.05);
  knockTarget(env, 2);
  assertEq(pb.score - before, 150 + 150 + 150 + 2000, "bank sweep total (spec example)");
  assert(pb.targets.every((t) => t.alive), "targets did not reset after clearing");
});

// =====================================================================
// 5. combo-scoring (Combo Multiplier, Multiplier HUD Display)
//    + modified Bumper Scoring (pinball-table)
// =====================================================================

test("combo: chained hits 1s apart award 100 (x1), 200 (x2), 300 (x3) — spec example", () => {
  const env = createEnv(); const pb = env.pb;
  const a1 = pb.addScore(100);
  env.stepSeconds(1);
  const a2 = pb.addScore(100);
  env.stepSeconds(1);
  const a3 = pb.addScore(100);
  assertEq(a1, 100, "first hit");
  assertEq(a2, 200, "second hit");
  assertEq(a3, 300, "third hit");
  assertEq(pb.score, 600, "total score");
  assertEq(pb.multiplier, 3, "multiplier after chain");
});

test("combo: multiplier resets to x1 after 3000ms without a hit", () => {
  const env = createEnv(); const pb = env.pb;
  pb.addScore(100); pb.addScore(100);
  assertEq(pb.multiplier, 2, "precondition: multiplier x2");
  env.stepSeconds(3.05);
  assertEq(pb.multiplier, 1, "multiplier did not reset after timeout");
  assertEq(pb.addScore(100), 100, "hit after timeout awarded above x1");
});

test("combo: multiplier caps at x5 across six chained hits", () => {
  const env = createEnv(); const pb = env.pb;
  const awards = [];
  for (let i = 0; i < 6; i++) awards.push(pb.addScore(100));
  assertEq(awards.join(","), "100,200,300,400,500,500", "award sequence");
  assertEq(pb.multiplier, 5, "multiplier exceeded the cap");
});

test("combo: bumper scoring goes through the multiplier", () => {
  const env = createEnv(); const pb = env.pb;
  pb.addScore(100);                       // primes the combo window at x1
  const b = pb.bumpers[0];
  pb.ball.state = "inPlay"; pb.ball.x = b.x; pb.ball.y = b.y - b.r - 2; pb.ball.vx = 0; pb.ball.vy = 120;
  env.step(1 / 120);
  assertEq(pb.score, 300, "bumper hit inside the window did not award 100 x2");
  assertEq(pb.multiplier, 2, "bumper hit did not advance the multiplier");
});

test("combo HUD: the current multiplier is displayed on the HUD", () => {
  const env = createEnv(); const pb = env.pb;
  let texts = env.render();
  assert(texts.some((t) => t.includes("x1")), "HUD does not show x1 at start: " + texts.join(" | "));
  pb.multiplier = 2;
  texts = env.render();
  assert(texts.some((t) => t.includes("x2")), "HUD does not show x2: " + texts.join(" | "));
});

// =====================================================================
// 6. high-scores (Persistent High Score Table, High Score Display At Game Over)
// =====================================================================

function seededStorage(scores) {
  const ls = makeLocalStorage();
  ls.setItem("pinball.highscores", JSON.stringify(scores));
  return ls;
}

test("highscores: game over inserts a ranking score into the stored top-3 (spec example)", () => {
  const ls = seededStorage([5000, 3000, 1000]);
  const env = createEnv({ localStorage: ls });
  const pb = env.pb;
  pb.score = 4000; pb.balls = 1;
  env.drainBall();
  assertEq(pb.over, true, "precondition: game over");
  assertEq(ls.getItem("pinball.highscores"), "[5000,4000,3000]", "stored table");
});

test("highscores: a score below the stored top-3 leaves the table unchanged", () => {
  const ls = seededStorage([5000, 3000, 1000]);
  const env = createEnv({ localStorage: ls });
  const pb = env.pb;
  pb.score = 500; pb.balls = 1;
  env.drainBall();
  assertEq(ls.getItem("pinball.highscores"), "[5000,3000,1000]", "stored table");
});

test("highscores: a throwing localStorage degrades silently (game over + R still work)", () => {
  const env = createEnv({ localStorage: makeLocalStorage({ throws: true }) });
  const pb = env.pb;
  pb.score = 1234; pb.balls = 1;
  env.drainBall();
  assertEq(pb.over, true, "game over did not happen with broken storage");
  env.press("r");
  assertEq(pb.over, false, "R restart failed with broken storage");
});

test("highscores: the game-over screen lists the persisted top-3 in order", () => {
  const ls = seededStorage([5000, 3000, 1000]);
  const env = createEnv({ localStorage: ls });
  const pb = env.pb;
  pb.score = 4000; pb.balls = 1;
  env.drainBall();
  const texts = env.render();
  assert(texts.some((t) => t.includes("HIGH SCORES")), "no HIGH SCORES heading: " + texts.join(" | "));
  const list = texts.slice(texts.findIndex((t) => t.includes("HIGH SCORES")) + 1).join(" ");
  assert(list.includes("5000") && list.includes("4000") && list.includes("3000"),
    "top-3 values missing from game-over screen: " + list);
  assert(list.indexOf("5000") < list.indexOf("4000") && list.indexOf("4000") < list.indexOf("3000"),
    "top-3 not listed in descending order: " + list);
});

// =====================================================================
// 7. pause-control (Pause Toggle)
// =====================================================================

test("pause: P freezes the ball and score and shows PAUSED", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 200; pb.ball.y = 300; pb.ball.vx = 50; pb.ball.vy = 0;
  env.press("p");
  assertEq(pb.paused, true, "paused flag");
  const score = pb.score;
  env.stepSeconds(1);
  assertEq(pb.ball.x, 200, "ball x moved while paused");
  assertEq(pb.ball.y, 300, "ball y moved while paused");
  assertEq(pb.score, score, "score changed while paused");
  const texts = env.render();
  assert(texts.some((t) => t.includes("PAUSED")), "no PAUSED overlay: " + texts.join(" | "));
});

test("pause: the combo window is frozen while paused", () => {
  const env = createEnv(); const pb = env.pb;
  pb.addScore(100); pb.addScore(100);
  assertEq(pb.multiplier, 2, "precondition: multiplier x2");
  env.press("p");
  env.stepSeconds(4);
  assertEq(pb.multiplier, 2, "multiplier decayed while paused");
  env.press("p");
  env.stepSeconds(4);
  assertEq(pb.multiplier, 1, "multiplier did not decay after resume");
});

test("pause: resume continues from the same position and velocity, PAUSED disappears", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 200; pb.ball.y = 300; pb.ball.vx = 50; pb.ball.vy = 0;
  env.press("p");
  env.stepSeconds(1);
  env.press("p");
  assertEq(pb.paused, false, "paused flag after resume");
  env.stepSeconds(0.05);
  assert(pb.ball.x > 200, "ball did not resume moving");
  const texts = env.render();
  assert(!texts.some((t) => t.includes("PAUSED")), "PAUSED still shown after resume");
});

test("pause: Space is ignored while paused (launch suppressed)", () => {
  const env = createEnv(); const pb = env.pb;
  assertEq(pb.ball.state, "waiting", "precondition");
  env.press("p");
  env.press(" ", "Space");
  env.press("p");
  assertEq(pb.ball.state, "waiting", "ball launched from a paused game");
});

test("pause: P has no effect on the game-over screen", () => {
  const env = createEnv(); const pb = env.pb;
  env.drainBall(); env.drainBall(); env.drainBall();
  assertEq(pb.over, true, "precondition: game over");
  env.press("p");
  assertEq(pb.paused, false, "paused toggled on the game-over screen");
});

// =====================================================================
// 8. Nudge + TILT (pinball-table: Nudge Impulse, Tilt Lockout)
// =====================================================================

test("nudge: N gives an in-play ball a 90 px/s horizontal impulse toward the field center", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 100; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
  env.press("n");
  assertEq(pb.ball.vx, 90, "left-half nudge (spec example)");
  pb.ball.x = 350; pb.ball.vx = 0;
  env.press("n");
  assertEq(pb.ball.vx, -90, "right-half nudge");
});

test("nudge: N is ignored while the ball waits in the lane or the game is paused", () => {
  const env = createEnv(); const pb = env.pb;
  assertEq(pb.ball.state, "waiting", "precondition");
  env.press("n");
  assertEq(pb.ball.vx, 0, "waiting ball was nudged");
  pb.ball.state = "inPlay"; pb.ball.x = 100; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
  env.press("p");
  env.press("n");
  assertEq(pb.ball.vx, 0, "paused ball was nudged");
});

test("tilt: a fourth nudge within 3s tilts the table, disables flippers and shows TILT", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 100; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
  for (let i = 0; i < 4; i++) env.press("n");
  assertEq(pb.tilted, true, "tilted flag");
  const rest = pb.flippers[0].rest;
  env.press("ArrowLeft");
  env.stepSeconds(0.3);
  assert(Math.abs(pb.flippers[0].angle - rest) < 0.01, "flipper raised while tilted");
  env.release("ArrowLeft");
  const texts = env.render();
  assert(texts.some((t) => t.includes("TILT")), "no TILT on HUD: " + texts.join(" | "));
});

test("tilt: nudges spread more than 3s apart never tilt", () => {
  const env = createEnv(); const pb = env.pb;
  for (let i = 0; i < 4; i++) {
    pb.ball.state = "inPlay"; pb.ball.x = 100; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
    env.press("n");
    pb.ball.state = "waiting";
    env.stepSeconds(1.1);
  }
  assertEq(pb.tilted, false, "table tilted from spread-out nudges");
});

test("tilt: clears when the ball drains; the next ball has working flippers again", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 100; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
  for (let i = 0; i < 4; i++) env.press("n");
  assertEq(pb.tilted, true, "precondition: tilted");
  env.drainBall();
  assertEq(pb.tilted, false, "tilt survived the drain");
  const rest = pb.flippers[0].rest;
  env.press("ArrowLeft");
  env.stepSeconds(0.3);
  assert(pb.flippers[0].angle < rest - 0.1, "flippers still dead on the next ball");
  env.release("ArrowLeft");
  const texts = env.render();
  assert(!texts.some((t) => t.includes("TILT")), "TILT still shown after drain");
});

test("tilt: while tilted, flipper keys make no sound and N adds no impulse", () => {
  const env = createEnv(); const pb = env.pb;
  pb.ball.state = "inPlay"; pb.ball.x = 100; pb.ball.y = 300; pb.ball.vx = 0; pb.ball.vy = 0;
  for (let i = 0; i < 4; i++) env.press("n");
  assertEq(pb.tilted, true, "precondition: tilted");
  const vx = pb.ball.vx;
  env.press("n");
  assertEq(pb.ball.vx, vx, "nudge still applied while tilted");
  const count = env.record.oscillators.length;
  env.press("ArrowLeft");
  assertEq(env.record.oscillators.length, count, "flipper sound played while tilted");
  env.release("ArrowLeft");
});

// =====================================================================
// 9. Integration
// =====================================================================

test("integration: the in-play HUD shows SCORE, BALLS and COMBO together", () => {
  const env = createEnv();
  const texts = env.render();
  assert(texts.some((t) => t.startsWith("SCORE")), "no SCORE on HUD");
  assert(texts.some((t) => t.startsWith("BALLS")), "no BALLS on HUD");
  assert(texts.some((t) => t.startsWith("COMBO")), "no COMBO on HUD");
});

test("integration: the test hook exposes every documented member", () => {
  const env = createEnv();
  const members = ["step", "render", "serveBall", "resetGame", "launch", "playSound",
    "spawnParticles", "addScore", "nudge", "ball", "walls", "bumpers", "flippers",
    "particles", "slingshots", "targets", "score", "balls", "over", "multiplier",
    "comboTimer", "highScores", "paused", "muted", "tilted"];
  for (const k of members) assert(k in env.pb, "hook missing member: " + k);
});

// ---------- summary ----------
console.log("");
console.log(passed + " passed, " + failed + " failed");
if (failed > 0) process.exit(1);
