# Server Deployment

[繁體中文](server-deployment.zh-TW.md) · **English**

The official server is for demonstration and trial. It lets you try remote projects without writing code. For production use, embed the engine in your own system. See [Embed Speclink in your system](../README.md#embedding).

This document shows how to run the official `speclink-server` (the demonstration server) for a trial. It covers the run modes, the configuration, HTTPS, the management commands, and upgrade and rollback. For a first trial, follow [Remote Getting Started](remote-getting-started.md) once. That guide takes you through the first-run setup, the memberships, and the Speclink Desktop and CLI connections.

## <a id="shared-host-kit"></a>The official server and your system share the host kit

`speclink-server` and a host with the embedded engine (your own system) are built from the same host kit and the same store contract. So the CLI and Speclink Desktop work with both. For the differences that remain, see the [known gaps](product-status.md#known-gaps) in the product status.

![Your system and the official server share the host kit: two columns with three layers; the top layer is the routes and login of each side, the middle layer is the host kit and the engine that both sides share, and the bottom layer is the storage of each side](assets/figures/en/shared-host-kit.svg)

Each side has three layers from top to bottom. Only the middle layer is the same:

| Layer | Your system (the host) | The official server |
| --- | --- | --- |
| Top: different | Your one route and your login | The official routes, accounts, and admin pages |
| Middle: the same | The host kit and the engine | The host kit and the engine |
| Bottom: different | Your three store functions and your database | The three store port examples (`sqlite`, `postgres`, `serverfs`) |

The official server already has accounts, login, and admin pages, so you can try it without writing code. If you already have your own backend, embed the engine. See [Embed Speclink in your system](../README.md#embedding) in the README, and the [Node SDK](sdk-node.md). This document covers only how to run the official server.

## <a id="choose"></a>Choose a run mode

| Run mode | Use it for | Data location |
| --- | --- | --- |
| [npx](#npx) | A single-machine trial with Node.js | `./speclink-data/` |
| [Docker](#docker) | One container | `/data` in the container |
| [Docker Compose (SQLite)](#compose) | The default when the server stays on and several people try it together | A named volume |
| [Docker Compose (PostgreSQL)](#compose) | Teams that already operate PostgreSQL | PostgreSQL and a named volume |
| [Build from source](#build-from-source) | Your own binary | The paths in the configuration file |

All run modes use the same binary. The build embeds the browser console (`apps/server-web`) into `speclink-server` at compile time. The console and the API use the same URL. At run time, the server needs no Node.js, no external `dist` folder, no CDN, and no second static file server.

Each storage backend supports only **one** server instance. Do not use `--scale`, and do not point two servers at the same data. SQLite and PostgreSQL do not block a second instance. For details, see the [store port examples](server-store-drivers.md).

After the start, all run modes behave the same:

- `GET /healthz` returns `200` when the process is alive. `GET /readyz` returns `200` when the storage backend is available, and `503` when it is not.
- At the first start (no admin yet), the server prints a one-time `/setup?token=…` link to stdout. The link is valid for 24 hours. When the setup is complete, the setup closes permanently.
- If the configuration has an error (an unreadable file, a bad format, an unknown storage backend), the server opens no port and stops with a non-zero exit code.

## <a id="npx"></a>npx

You need Node.js 18 or later. The supported platforms are macOS (arm64, x64), Linux (x64, arm64), and Windows (x64).

```bash
npx @speclink/server
```

Without arguments, the launcher does three things:

1. It makes `speclink-data/config.yaml` from the [environment variables](#configuration). It writes this file again at each start, so manual edits do not stay.
2. It uses SQLite by default and keeps the data in `./speclink-data/`.
3. It starts the server, which listens only on `127.0.0.1:<SPECLINK_PORT>`.

With arguments, the launcher gives the arguments to the server binary without changes, for example `npx @speclink/server invite --config ./speclink-data/config.yaml …`. To get a fixed `speclink-server` command, run `npm i -g @speclink/server`.

By default, only the local computer can connect to an npx server. To let other computers connect, use Docker Compose with an [HTTPS reverse proxy](#https) in front. If you must use npx, keep your own configuration file and start with `npx @speclink/server --config <file> --addr 0.0.0.0:8080`. With `--config`, the launcher does not change your configuration file.

## <a id="docker"></a>Docker

The official image is `ghcr.io/momochenisme/speclink-server`. It supports `linux/amd64` and `linux/arm64`. Each release pushes three tags: the full version (for example `0.8.0`), the `major.minor` version (for example `0.8`), and `latest`.

```bash
docker run -d --name speclink \
  -p 8080:8080 \
  -v speclink-data:/data \
  ghcr.io/momochenisme/speclink-server:latest
docker logs speclink        # get the one-time setup link
```

The image contains a default configuration. It keeps the store and identity SQLite files in `/data`, and `public_url` is the default `http://localhost:8080`. The server runs as a non-root user with uid 10001. The container health check calls `/healthz`.

The configuration file does not expand environment variables. To change `public_url` or the storage backend, mount your own configuration file over the file in the image:

```bash
docker run -d --name speclink \
  -p 8080:8080 \
  -v speclink-data:/data \
  -v ./server.yaml:/etc/speclink/config.yaml:ro \
  ghcr.io/momochenisme/speclink-server:latest
```

If the port mapping is not `8080:8080`, or the public URL is not `http://localhost:8080`, you must change `public_url`. If you do not, the setup link shows the wrong URL, and the server rejects browser forms because the origin does not match.

## <a id="compose"></a>Docker Compose

The `deploy/` folder of the repo contains two compose files. Both keep `/data` in a named volume and make the configuration from environment variables.

SQLite (default):

```bash
cd deploy
docker compose pull          # get the official image first
docker compose up -d
docker compose logs server   # get the one-time setup link
```

PostgreSQL (two services: server and postgres):

```bash
cd deploy
cp .env.example .env         # set SPECLINK_POSTGRES_PASSWORD; .env stays out of version control
docker compose -f docker-compose.postgres.yml pull
docker compose -f docker-compose.postgres.yml up -d
```

In the PostgreSQL file, the server starts only after postgres is healthy. The account data (identity) stays in a SQLite file under `/data`. For the limits of this storage backend, see the [store port examples](server-store-drivers.md).

The compose files contain both `image:` and `build:`. If you do not run `pull` first and the image is not on the machine, `up` builds the image from source. This needs the full repo, and the Rust build takes several minutes.

After a container restart, the data stays in the volume, and the server does not print the setup link again.

## <a id="build-from-source"></a>Build from source

The build order from source is `npm ci`, then `npm run build -w apps/server-web`, and then the `speclink-server` build. Run these commands in a checkout of the repo:

```bash
npm ci
npm run build -w apps/server-web
cargo build --release -p speclink-server
```

The output is `target/release/speclink-server`. Step 3 embeds the `apps/server-web/dist` output of step 2 into the binary. If `apps/server-web/dist/index.html` is missing, the release build fails at compile time. It never makes a binary with the API but no console. The Docker image uses a multi-stage build with the same order, and the final image contains no Node.js.

Prepare a configuration file, and then start the server:

```bash
./target/release/speclink-server --config /etc/speclink/server.yaml --addr 0.0.0.0:8080
```

The binary always needs `--config`. The default `--addr` is `127.0.0.1:8080`, so only the local computer can connect. To serve other computers, set the bind address. With a process manager such as systemd, set `Restart=on-failure`.

## <a id="configuration"></a>Environment variables and configuration file

### Environment variables

| Variable | Run modes | Default | Description |
| --- | --- | --- | --- |
| `SPECLINK_STORE` | npx | `sqlite` | The storage backend: `sqlite`, `serverfs`, or `postgres`. |
| `SPECLINK_DATA_DIR` | npx | `./speclink-data` | The data folder for the configuration file and the databases. |
| `SPECLINK_PORT` | npx, Compose | `8080` | npx: the port that the server listens on at `127.0.0.1`. Compose: the host port (the container port is always 8080). |
| `SPECLINK_PUBLIC_URL` | npx, Compose | `http://localhost:<SPECLINK_PORT>` | The public URL: the scheme, host, and port in the browser address bar of your users. Setup links, invitation links, and the same-origin check use it. Always set it when other computers connect. |
| `SPECLINK_POSTGRES_URL` | npx | None (required for `postgres`) | The PostgreSQL connection URL. Keep the password out of it. |
| `SPECLINK_POSTGRES_PASSWORD` | All | None (required for the PostgreSQL compose file) | The PostgreSQL password. The server reads this variable itself and adds it to a connection URL without a password. |
| `SPECLINK_CONFIG` | npx | None | Use an existing configuration file. With it, the npx-only variables above have no effect, and the launcher does not set `--addr`, so the server listens on `127.0.0.1:8080`. Subcommands such as `invite` still need their own `--config`. |

When you run the binary directly or with `docker run`, the server reads only `SPECLINK_POSTGRES_PASSWORD`. All other settings are in the configuration file.

### Configuration file

The server reads one YAML configuration file:

| Field | Required | Description |
| --- | --- | --- |
| `store` | Yes | The storage backend (`driver` plus `path` or `url`). See the [store port examples](server-store-drivers.md). |
| `identity` | Yes | The account database: `driver: sqlite` plus `path`. |
| `public_url` | No | The public URL. The default is `http://localhost:8080`. |
| `events` | No | Settings for the live event stream: `retention` (events kept per scope, default 1024), `buffer` (buffer per connection, default 256), and `heartbeat_secs` (heartbeat interval in seconds, default 15). |

Example:

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
public_url: https://speclink.example.com
```

The old `tokens` and `projects` sections are no longer valid. If the file contains one of them, the server does not start, and the error shows what to use instead.

## <a id="https"></a>HTTPS and reverse proxy

The server does not handle TLS. When other computers connect, put a reverse proxy in front of the server to provide HTTPS.

You need this: the sign-in cookie always has the `Secure` attribute, and browsers keep it only over HTTPS or on `http://localhost`. If you deploy with `http://` and a host that is not localhost, the browser does not keep the sign-in. Then nobody can sign in to the console or to the device approval page.

The reverse proxy must do these things:

- Provide HTTPS and send the requests to the server port.
- Set `public_url` to the HTTPS URL that your users open. If the scheme, host, or port is different, the server rejects write requests from the browser (`403`, `same_origin_required`).
- Do not buffer `/api/speclink/v1/projects/<project key>/events`. This is a long-lived event stream (Server-Sent Events). The server sends a heartbeat every 15 seconds, so the idle timeout of the proxy must be longer than that.

## <a id="manage"></a>Management commands

Most console actions also have a command-line version for scripts. Each subcommand needs `--config` with the path of the server configuration file:

| Command | Purpose |
| --- | --- |
| `invite --email <email> --display <name>` | Make a one-time invitation and print the acceptance URL. You can repeat `--project <project key>`; after the invitee accepts, the invitee is an `editor` in that project. `--admin` gives the admin flag. `--expires-in-days` defaults to 7 days. |
| `user suspend --email <email>` / `user reactivate --email <email>` | Suspend or reactivate a user. You cannot suspend the last active admin. |
| `token revoke --token-id <id>` | Revoke an access key. The id starts with `pat_`, and it is not the key itself. The console shows only the key prefix, so it is usually easier to revoke a key on `/admin/credentials`. |
| `project create --key <project key> [--name <name>]` | Make a project. |
| `repo create --project <project key> --key <repository key> [--name <name>]` | Make a repository in a project. |

No command adds a membership to an existing account. Use `/admin/users` for that.

The start of the command depends on the run mode:

```bash
# npx (run it in the folder that contains the data folder)
npx @speclink/server invite --config ./speclink-data/config.yaml \
  --email dev@example.com --display "Dev" --project demo

# Docker Compose (while the server runs)
docker compose exec server speclink-server invite \
  --config /etc/speclink/config.yaml \
  --email dev@example.com --display "Dev" --project demo
```

For the backup, verify, and restore commands, see [Server Backup and Restore](server-backup.md).

## <a id="verify-deployment"></a>Check the deployment

After the deployment and the `/setup`, do these three checks to make sure that the server works.

First, look at the console overview. Right after the setup, the top of the overview shows a "Getting started" block one time. It shows the service URL, the project key, and the repository key:

![The server console overview with the service URL, the project and repository keys, the counts, and the system health](assets/screenshots/en/server-overview.png)

- The service URL must be the same as the URL that your users open. If it is not, change `public_url`.
- "System health" shows "Healthy", and the data schema version has a value.
- The "Needs attention" item "No active credentials — remote workflows cannot connect." is normal. It goes away after a person signs in with the Desktop app or the CLI, or creates an access key.

Second, look at the Users page to check the memberships. A sign-in does not give access to a project. The admin flag and the project membership are separate. The admin from the setup starts with no membership, so the row on the Users page shows "No projects". After you add a membership, the "Memberships" tab of the user detail page lists each project and its role:

![The user detail page in the server console, with the Memberships tab that lists each project and its role](assets/screenshots/en/server-members.png)

Each person who uses the remote must have the target project on the "Memberships" tab of their user detail page. For the steps to add a membership, see [Remote Getting Started](remote-getting-started.md#grant-membership).

Third, run a health check from a different computer:

```bash
curl -o /dev/null -w "%{http_code}\n" https://<your-url>/healthz
```

The check passes only with `200`.

## <a id="upgrade"></a>Upgrade

In most cases, to upgrade, change to the new version and restart. The server runs as one instance, so there is no rolling update.

One case needs more than a version change: the server runs 0.12.0 or earlier, and you move to a version after 0.12.0. In this case, do the three steps in [Upgrade from 0.12.0 or earlier](#upgrade-from-0-12).

The steps for all other cases:

1. Make a backup first, and make sure that `verify-backup` passes. See [Server Backup and Restore](server-backup.md).
2. Change to the new version and restart:
   - Docker Compose: `docker compose pull && docker compose up -d`. The volume does not change.
   - npx: `npx @speclink/server@<new version>`.
   - Your own binary: replace the binary and restart.
3. Make sure that `/healthz` and `/readyz` both return `200`.

If the new version needs a newer account database (identity) structure, the server upgrades the database automatically at start. If the configuration or the data is not compatible, the server stops with a non-zero exit code. It never starts with an error.

### <a id="upgrade-from-0-12"></a>Upgrade from 0.12.0 or earlier

In versions after 0.12.0, the storage backend uses new tables and a new file layout. A new version cannot read data from 0.12.0 or earlier, and it does not convert the data automatically. You move the data with a backup and a restore. There are three steps.

The commands below run the binary directly. To run a subcommand in Docker, see [Back up and restore in Docker](server-backup.md#docker).

**Step 1: Back up with the old version.** Stop the server. Then run this command with the **old** version:

```bash
speclink-server backup \
  --config /etc/speclink/server.yaml \
  --output /var/backups/speclink/before-upgrade.tar
```

**Expected output** (the numbers depend on your data):

```text
備份完成：1 個 scope、2 個成員 → /var/backups/speclink/before-upgrade.tar
```

Then check this backup with the old version:

```bash
speclink-server verify-backup --input /var/backups/speclink/before-upgrade.tar
```

**Expected output**:

```text
備份完整：格式版本 1、2 個成員、1 個 scope
```

A backup from the old version has backup format version 1 (格式版本 1). The new version can read it. You must make the backup with the old version. The new version cannot open the old data, so it cannot back up the old data.

**Step 2: Point the configuration at an empty storage location and an empty account database.** Change the `store` and `identity` sections of the configuration file to locations that contain no data. Leave the original files in their place. Do not delete or move them. Write down the two original locations first, because a rollback points back to them.

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/new/store.db      # was /var/lib/speclink/store.db
identity:
  driver: sqlite
  path: /var/lib/speclink/new/identity.db   # was /var/lib/speclink/identity.db
public_url: https://speclink.example.com
```

This step has no output. An "empty location" for each storage backend:

- `sqlite`: a file path that does not exist. The folder of the file must exist.
- `serverfs`: a folder that does not exist or is empty.
- `postgres`: a new, empty database.
- Docker Compose: the SQLite files are below `/data` in the named volume, so an empty location is a new, empty volume. The PostgreSQL compose file also needs an empty database.
- npx: without arguments, the launcher writes the configuration file again from the environment variables at each start. Set `SPECLINK_DATA_DIR` to a new folder to change to an empty location. `restore` needs a configuration file in `--config`, so first write your own configuration file that points at the new folder. The launcher uses the paths `<folder>/store.db` (`<folder>/store` for `serverfs`) and `<folder>/identity.db`.

For the full definition of "empty", see [Two rules](server-backup.md#rules) in Server Backup and Restore.

**Step 3: Restore with the new version, and check the verification report.** Change to the **new** version. Do not start the server. Run:

```bash
speclink-server restore \
  --config /etc/speclink/server.yaml \
  --input /var/backups/speclink/before-upgrade.tar
```

**Expected output**:

```text
還原完成且驗證通過：1 個 scope 全數比對一致
```

This line is the verification report. After `restore` writes the data, it reads each project and repository again and compares each item with the backup. Start the server only when you see this line and the command returns 0. Then make sure that `/healthz` and `/readyz` both return `200`. The restore includes the account data, so the server does not print a setup link, and the existing accounts and access keys continue to work.

If one item does not match, the command returns non-zero and lists each difference, for example:

```text
還原後驗證發現不符，此目標不可投產：
  - scope demo/backend: 2 document(s), expected 3
```

In this case, do not start the server. Follow [Rollback](#rollback) to go back to the old version. The old data is still in its original place.

### <a id="refused-start"></a>Start the new version without the three steps

If you start the new version directly on the old data, the server refuses to start and does not change the data:

- It stops with a non-zero exit code. It opens no port and prints no setup link.
- The data of the storage backend and the account database stay as they were.
- It prints the message below to stderr. The message contains the three steps above. The location and the configuration file path are yours:

```text
error: the store at /var/lib/speclink/store.db was created by speclink-server 0.12 or earlier.
This version uses a different layout and did not change your data.

To move the data to this version:
  1. With the old version, run:
       speclink-server backup --config /etc/speclink/server.yaml --output <file>
  2. Point the config at an empty store location and an empty identity database.
  3. With this version, run:
       speclink-server restore --config /etc/speclink/server.yaml --input <file>
```

With the `postgres` storage backend, the location in the first line is `the configured PostgreSQL database`.

What to do next: change back to the old version. The data did not change, so the old version starts as before. Then do the three steps from step 1.

The `backup` subcommand of the new version prints the same message on old data and makes no backup file. `restore` also refuses a target that contains old data.

## <a id="rollback"></a>Rollback

To roll back, deploy the **previous** binary or image:

- Docker Compose: change `image:` in the compose file to the previous tag (for example `ghcr.io/momochenisme/speclink-server:0.7.0`), and then run `docker compose up -d`. The volume does not change.
- npx: `npx @speclink/server@<previous version>`.
- Your own binary: check out the previous tag and build again, or use the previous image.

Up to 0.12.0, all versions use the same data format (account database structure version 6, storage format version 1). A rollback between these versions needs no data change. The "Data schema version" on the console "System" page shows the current account database version.

A rollback from a version after 0.12.0 to 0.12.0 or earlier needs one more thing. You must also point the configuration back to the original storage location and account database:

1. Stop the server of the new version.
2. Deploy the previous version.
3. Change the `store` and `identity` sections of the configuration file back to the locations from before the upgrade.
4. Start the server, and make sure that `/healthz` and `/readyz` both return `200`.

During an upgrade with the three steps, the old data stays in its original place and does not change. So the old version works when you point back to it. Data that you wrote to the new location after the upgrade is not in the old location. The old version also cannot read a backup from the new version (backup format version 2), so you cannot take this data back to the old version.

A future version can raise the account database version. In that case, the upgrade changes the database automatically, and an older server refuses to open the newer database. A rollback then needs the backup from before the upgrade, restored into an empty target. So always make a backup before an upgrade.

## <a id="related"></a>Related documents

- [Remote Getting Started](remote-getting-started.md): first-run setup, memberships, and the Speclink Desktop and CLI connections.
- [Store port examples](server-store-drivers.md): how to choose SQLite, serverfs, or PostgreSQL.
- [Server Backup and Restore](server-backup.md): `backup`, `verify-backup`, `restore`, the backup format versions, and schedules.
- [Embed Speclink in your system](../README.md#embedding): the path for production use.
