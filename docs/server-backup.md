# Server Backup and Restore

[繁體中文](server-backup.zh-TW.md) · **English**

The official `speclink-server` has three subcommands for backup, verification, and restore:

| Subcommand | What it does | Stop the server? |
| --- | --- | --- |
| `backup` | Packs the project data and the account data into one backup file | Yes |
| `verify-backup` | Checks only the integrity of a backup file, with no restore | No |
| `restore` | Restores a backup into an **empty** target, and then compares each item of the result | Yes |

The backup file does not depend on the storage backend. You can back up from `sqlite` and restore into `serverfs` or `postgres`. For the steps to change the storage backend, see [Server Storage Backends](server-store-drivers.md#switch).

## <a id="rules"></a>Two rules

**1. Stop the server before a backup.** No writes can occur during a backup. If writes occur, the backup can be inconsistent. There is no online backup of a running server. The procedure is always: stop the server → `backup` → start the server.

**2. A restore accepts only an empty target.** "Empty" means:

- The account database contains no users, and
- The target storage backend contains no content for each project and repository in the backup.

If the target is not empty, `restore` refuses. It lists the existing content in the target and writes nothing. There is no option to force an overwrite. The simplest way is to prepare a new data folder or an empty database, and not to complete `/setup` before the restore. Also keep the target server stopped during the restore.

## <a id="contents"></a>What a backup file contains

A backup is one uncompressed tar file. It contains:

- `manifest.json`: the backup format version, the UTC creation time, the engine version, the source storage backend, the account database version, the document count for each project and repository, and the hash of each member file.
- `manifest.json.sha256`: the hash of `manifest.json` itself, which shows that nobody changed the list.
- `bundles/<n>.json`: one export file for each project and repository. It comes from the export function of the storage backend. It is not a direct copy of the database file.
- `identity.db`: a complete snapshot of the account database at the time of the backup.

The backup file contains no plaintext passwords or keys. The server keeps passwords as argon2id hashes, and access keys and sessions only as SHA-256 hashes. But the backup file still contains account and project data, so keep it as a confidential file.

The "Export" area of the console "System" page lets you download the export file of one project and repository. It contains no account data, so it cannot replace `backup`.

## <a id="backup"></a>Back up

```bash
speclink-server backup \
  --config /etc/speclink/server.yaml \
  --output /var/backups/speclink/speclink-$(date -u +%Y%m%dT%H%M%SZ).tar
```

`--config` tells the command where the storage backend and the account database are. The account database must be `sqlite`. You choose the output file name. The examples in this document use a UTC timestamp.

**Expected output**:

```text
備份完成：1 個 scope、2 個成員 → /var/backups/speclink/speclink-20261007T084144Z.tar
```

A "scope" is one project and repository pair. A "member" (成員) is one data file in the tar. After a successful backup, the server adds a record to the account database. To see it, filter `/admin/audit` in the console by the action "Backup recorded".

## <a id="verify"></a>Verify a backup

`verify-backup` reads only the backup file. You do not need to stop the server, and you do not need an empty target. The command compares the list with the hash of each member file, checks the structure of the export files, and makes sure that this server knows the backup format version.

```bash
speclink-server verify-backup --input /var/backups/speclink/speclink-20261007T084144Z.tar
```

**Expected output**:

```text
備份完整：格式版本 1、2 個成員、1 個 scope
```

If all checks pass, the command returns 0. If one bit of the file changed, or the format version is unknown, it returns non-zero and shows the reason, for example:

```text
backup integrity check failed: digest mismatch for bundles/0.json
```

With `--config`, the command also writes the result to the backup records of that account database.

Run `verify-backup` after each regular backup.

## <a id="restore"></a>Restore

First, make sure that the target is empty (see [Two rules](#rules)) and that the target server is stopped.

```bash
speclink-server restore \
  --config /etc/speclink/target.yaml \
  --input /var/backups/speclink/speclink-20261007T084144Z.tar
```

`restore` does four things in this order:

1. It checks the integrity of the backup file, the same as `verify-backup`. This step stops a changed backup or an unknown format.
2. It puts the account database snapshot at the target location.
3. It imports the content of each project and repository into the target storage backend.
4. It compares each item: the content hashes and the document count of each project and repository, and the record counts and the version of the account data, must match the backup.

**Expected output**:

```text
還原完成且驗證通過：1 個 scope 全數比對一致
```

If one item does not match, `restore` returns non-zero, lists each difference, and says that you must not use this target.

If the target is not empty, the output looks like this, and the target does not change:

```text
restore target is not empty (restore only into an empty target): 1 user(s) exist
```

After the restore, start the server. The account data is back, so the server does not print a setup link, and the existing accounts can sign in.

## <a id="docker"></a>Back up and restore in Docker

The entry point of the image is `speclink-server`, so the arguments after `docker compose run` are the subcommand. Run the examples below in the `deploy/` folder with the SQLite compose file.

The container runs as uid 10001. This uid must be able to write to the mounted host folder. If it cannot, you see `backup io failed: Permission denied (os error 13)`. On a Linux host, run this once:

```bash
mkdir -p backups && sudo chown 10001:10001 backups
```

Back up (the server stops for a short time):

```bash
docker compose stop server
docker compose run --rm -v ./backups:/backups server \
  backup --config /etc/speclink/config.yaml \
  --output /backups/speclink-$(date -u +%Y%m%dT%H%M%SZ).tar
docker compose start server
```

Verify (no downtime):

```bash
docker compose run --rm -v ./backups:/backups server \
  verify-backup --input /backups/speclink-20261007T084144Z.tar
```

Restore into a new environment: first make the container and the empty volume, but do not start the server and do not open `/setup`.

```bash
docker compose create server
docker compose run --rm -v ./backups:/backups server \
  restore --config /etc/speclink/config.yaml \
  --input /backups/speclink-20261007T084144Z.tar
docker compose start server
```

## <a id="schedule"></a>Scheduled backups

A backup needs downtime, so a schedule always uses this procedure: stop the server → back up → verify → start the server. The script below contains this procedure. A systemd timer or cron then runs the script. The script assumes that the server is a systemd service with the name `speclink-server`.

`/usr/local/bin/speclink-backup.sh`:

```sh
#!/bin/sh
set -eu
CONFIG=/etc/speclink/server.yaml
DIR=/var/backups/speclink
F="$DIR/speclink-$(date -u +%Y%m%dT%H%M%SZ).tar"

systemctl stop speclink-server
# Start the server again when the script ends, after a success or a failure.
trap 'systemctl start speclink-server' EXIT
speclink-server backup --config "$CONFIG" --output "$F"
speclink-server verify-backup --input "$F" --config "$CONFIG"
# Keep only the backups of the last 14 days.
find "$DIR" -name 'speclink-*.tar' -mtime +14 -delete
```

Run `chmod +x /usr/local/bin/speclink-backup.sh`. If a step fails, the script stops with a non-zero code, and the server still starts again.

### systemd timer

`/etc/systemd/system/speclink-backup.service`:

```ini
[Unit]
Description=Speclink server offline backup

[Service]
Type=oneshot
ExecStart=/usr/local/bin/speclink-backup.sh
```

`/etc/systemd/system/speclink-backup.timer`:

```ini
[Unit]
Description=Nightly Speclink backup

[Timer]
OnCalendar=*-*-* 03:30:00 UTC
Persistent=true

[Install]
WantedBy=timers.target
```

To turn it on, run `systemctl enable --now speclink-backup.timer`.

### cron

Add one line to the root crontab (`crontab -e`) to run the script each day at 03:30. The time uses the time zone of the host:

```cron
30 3 * * * /usr/local/bin/speclink-backup.sh
```

### Off-site copies and restore drills

The script writes backup files only on the local host. To copy them to a different host or site, use your existing deployment tools. Also, on a regular basis, restore a backup into an empty target on a different host. This makes sure that the backups really work.
