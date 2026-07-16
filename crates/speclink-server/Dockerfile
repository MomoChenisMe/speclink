# syntax=docker/dockerfile:1
# 多階段建置：Rust 編譯層 → 最小執行層。build context 是 repo 根：
#   docker build -f crates/speclink-server/Dockerfile .
#
# 基底定案：alpine（musl），非 distroless（glibc）。理由（rusqlite bundled 實測）：
# - rusqlite bundled 的 SQLite C 原始碼在 rust:alpine 加 musl-dev 後編譯通過；
#   server binary 依賴樹無任何 TLS 原生庫（postgres driver 為 NoTls，無
#   openssl/ring/aws-lc），musl 建置沒有其他 C 依賴障礙。
# - HEALTHCHECK 需要容器內指令：alpine 內建 busybox wget 可打 /healthz；
#   distroless 無 shell 無工具，得另外編一支探針 binary，複雜度不值。
# - alpine 保留 shell，容器內執行 backup/restore 等子命令與除錯都更直接。

FROM rust:1-alpine AS build
# musl-dev＋gcc：rusqlite bundled 經 cc crate 編譯 SQLite C 原始碼所需。
RUN apk add --no-cache musl-dev gcc
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p speclink-server

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
