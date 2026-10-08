#!/usr/bin/env bash
# 一個小模組，workflow config 還是 init 的預設值。
source "$(dirname "$0")/../fixtures.sh"
eval_init
mkdir -p src
cat >src/todo.js <<'JS'
const todos = [];

function addTodo(title) {
  todos.push({ title, done: false });
}

module.exports = { addTodo };
JS
