#!/usr/bin/env bash
# src/ 下兩個小模組（購物車與登入），還沒有任何規格。
source "$(dirname "$0")/../fixtures.sh"
eval_init
mkdir -p src
cat >src/cart.js <<'JS'
const items = [];

function addItem(name, priceCents) {
  items.push({ name, priceCents });
}

function totalCents() {
  return items.reduce((sum, item) => sum + item.priceCents, 0);
}

module.exports = { addItem, totalCents };
JS
cat >src/auth.js <<'JS'
const sessions = new Set();

function login(user, password) {
  if (password.length < 8) throw new Error('password too short');
  sessions.add(user);
}

function logout(user) {
  sessions.delete(user);
}

module.exports = { login, logout };
JS
