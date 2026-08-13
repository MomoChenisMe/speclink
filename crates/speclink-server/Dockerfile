# syntax=docker/dockerfile:1
# 多階段建置：Node 產 SPA 資產 → Rust 內嵌編譯 → 最小執行層。build context 是 repo 根：
#   docker build -f crates/speclink-server/Dockerfile .
#
# 交付定位：單一 speclink-server binary 同源提供 API 與 SPA（設計 D5／D7）。SPA 資產
# 於 Rust 編譯期內嵌進 binary，runtime 不需要 Node、外部 dist volume、CDN 或第二個
# 靜態檔服務；最終 image 只有 server binary 與 non-root 使用者。
#
# 基底定案：alpine（musl），非 distroless（glibc）。理由（rusqlite bundled 實測）：
# - rusqlite bundled 的 SQLite C 原始碼在 rust:alpine 加 musl-dev 後編譯通過；
#   server binary 依賴樹無任何 TLS 原生庫（postgres driver 為 NoTls，無
#   openssl/ring/aws-lc），musl 建置沒有其他 C 依賴障礙。
# - HEALTHCHECK 需要容器內指令：alpine 內建 busybox wget 可打 /healthz；
#   distroless 無 shell 無工具，得另外編一支探針 binary，複雜度不值。
# - alpine 保留 shell，容器內執行 backup/restore 等子命令與除錯都更直接。

# --- Stage 1：Node — 以 lockfile 安裝並產出 apps/server-web 的 production dist。 ---
# 釘在建置機原生架構（$BUILDPLATFORM 是 buildx 內建變數）：Vite 產出的 dist 與
# 架構無關，而 Node/V8 的 JIT 在 QEMU 模擬 arm64 下會偶發 SIGILL（exit 132），
# v0.1.1 與 v0.1.2 的首跑都因此紅在這一階段（任務 2.7）。
FROM --platform=$BUILDPLATFORM node:20-alpine AS web
WORKDIR /src
COPY . .
# npm ci 依 package-lock.json 做乾淨安裝（.dockerignore 已排除 host node_modules），
# 再對 server-web 做 Vite production build。dist 於此階段內生成，不依賴 host build——
# 內嵌來源永遠對齊 image 這次的 source revision。
RUN npm ci && npm run build -w apps/server-web

# --- Stage 2：Rust — 編譯 speclink-server，內嵌上一階段產出的 dist。 ---
FROM rust:1-alpine AS build
# musl-dev＋gcc：rusqlite bundled 經 cc crate 編譯 SQLite C 原始碼所需。
RUN apk add --no-cache musl-dev gcc
WORKDIR /src
COPY . .
# 帶入 Node stage 現建的 SPA 資產供 compile-time asset embedding；缺 index 或
# manifest 時 Rust 內嵌會 fail closed（見 crates/speclink-server），不產出半成品。
COPY --from=web /src/apps/server-web/dist ./apps/server-web/dist
RUN cargo build --release --locked -p speclink-server

# --- Stage 3：runtime — 只有 server binary 與 non-root 使用者，無 Node runtime。 ---
FROM alpine:3.21 AS runtime
# ca-certificates：binary 目前無出站 TLS，但 CA bundle 極小，先備妥（設計決策 1）。
# 固定 uid/gid 10001：bind mount 情境下權限可預期。
RUN apk add --no-cache ca-certificates \
    && addgroup -g 10001 -S speclink \
    && adduser -u 10001 -G speclink -S -h /data speclink
COPY --from=build /src/target/release/speclink-server /usr/local/bin/speclink-server

# 預設組態：store 與 identity 兩個 SQLite 檔都落在 /data（volume）。
# 組態 YAML 不做環境變數展開——要改 public_url 等欄位，掛載自訂檔覆蓋本路徑
# （deploy/docker-compose.yml 以 configs.content 示範經環境變數插值產生組態）。
COPY <<'EOF' /etc/speclink/config.yaml
store:
  driver: sqlite
  path: /data/store.db
identity:
  driver: sqlite
  path: /data/identity.db
EOF

VOLUME /data
EXPOSE 8080
USER speclink
# /healthz 回程序存活（200 空 body）；就緒檢查另有 /readyz。
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
    CMD wget -q --spider http://127.0.0.1:8080/healthz || exit 1

# 唯一 ENTRYPOINT 是 server binary；backup/restore/invite 等子命令以
# docker run/exec 覆寫參數執行（見 docs/server-deployment.zh-TW.md）。
# --addr 預設 127.0.0.1（loopback），容器內必須綁 0.0.0.0 對外。
ENTRYPOINT ["/usr/local/bin/speclink-server"]
CMD ["--config", "/etc/speclink/config.yaml", "--addr", "0.0.0.0:8080"]
