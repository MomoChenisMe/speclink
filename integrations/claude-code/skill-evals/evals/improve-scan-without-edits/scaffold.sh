#!/usr/bin/env bash
# 「價格顯示」這一個概念散在 src/price/ 的三個小模組，cart.js 與 checkout.js 各自重拼同一串呼叫
# （friction 訊號 1：理解一個概念要在多個小模組之間跳）。
source "$(dirname "$0")/../fixtures.sh"
eval_git_init
eval_init
mkdir -p src/price
cat >src/price/round.js <<'JS'
module.exports = { roundCents: (value) => Math.round(value) };
JS
cat >src/price/currency.js <<'JS'
module.exports = { symbolFor: (code) => ({ USD: '$', EUR: '€', TWD: 'NT$' })[code] ?? code };
JS
cat >src/price/format.js <<'JS'
module.exports = { formatCents: (cents) => `${Math.floor(cents / 100)}.${String(cents % 100).padStart(2, '0')}` };
JS
for caller in cart checkout; do
  cat >"src/$caller.js" <<'JS'
const { roundCents } = require('./price/round');
const { symbolFor } = require('./price/currency');
const { formatCents } = require('./price/format');

function displayTotal(items, currency) {
  const cents = roundCents(items.reduce((sum, item) => sum + item.priceCents * item.quantity, 0));
  return `${symbolFor(currency)}${formatCents(cents)}`;
}

module.exports = { displayTotal };
JS
done
eval_commit 'price display'
