# Server Backup and Restore

[繁體中文](server-backup.zh-TW.md) · **English**

The official server is for demonstration and trial. It lets you try remote projects without writing code. For production use, embed the engine in your own system. See [Embed Speclink in your system](../README.md#embedding).

The official `speclink-server` (the demonstration server) has three subcommands for backup, verification, and restore:

| Subcommand | What it does | Stop the server? |
| --- | --- | --- |
| `backup` | Packs the project data and the account data into one backup file | Yes |
| `verify-backup` | Checks only the integrity of a backup file, with no restore | No |
| `restore` | Restores a backup into an **empty** target, and then compares each item of the result | Yes |

The backup file does not depend on the storage backend. You can back up from `sqlite` and restore into `serverfs` or `postgres`. For the steps to change the storage backend, see the [store port examples](server-store-drivers.md#switch).

The current backup format is version 2. Version 0.12.0 and earlier versions make version 1, and a new version can also verify and restore it. See [Backup format versions](#format-versions). An upgrade from 0.12.0 or earlier also uses a backup and a restore. For the steps, see [Server Deployment](server-deployment.md#upgrade-from-0-12).

## <a id="rules"></a>Two rules

**1. Stop the server before a backup.** No writes can occur during a backup. If writes occur, the backup can be inconsistent. There is no online backup of a running server. The procedure is always: stop the server → `backup` → start the server.

**2. A restore accepts only an empty target.** "Empty" means:

- The account database contains no users, and
- The target storage backend contains no content for each project and repository in the backup.

If the target is not empty, `restore` refuses. It lists the existing content in the target and writes nothing. There is no option to force an overwrite. The simplest way is to prepare a new data folder or an empty database, and not to complete `/setup` before the restore. Also keep the target server stopped during the restore.

## <a id="contents"></a>What a backup file contains

A backup is one uncompressed tar file. It contains:

- `manifest.json`: the backup format version, the UTC creation time, the engine version, the kind of the source storage backend, the account database version, the document count for each project and repository, the record counts of the account data, and the hash of each member file.
- `manifest.json.sha256`: the hash of `manifest.json` itself, which shows that nobody changed the list.
- `bundles/<n>.json`: one export file for each project and repository. The file identifies each document by its key, and it includes the content and the hash of the content. The server reads the documents one by one to make this file. It is not a direct copy of the database file.
- `identity.db`: a complete snapshot of the account database at the time of the backup.

An export file looks like this:

```json
{
  "formatVersion": 2,
  "scopeVersion": 7,
  "documents": [
    {
      "key": "changes/add-auth/proposal.md",
      "content": "…",
      "digest": "sha256:…"
    }
  ]
}
```

A key is the name of a document in the storage backend, for example `changes/add-auth/proposal.md`, `specs/accounts/spec.md`, or `config.yaml`. `digest` is the SHA-256 hash of `content`. `scopeVersion` is the version number of this project and repository at the time of the export.

The backup file contains no plaintext passwords or keys. The server keeps passwords as argon2id hashes, and access keys and sessions only as SHA-256 hashes. But the backup file still contains account and project data, so keep it as a confidential file.

### <a id="scope-export"></a>The export download in the console

The "Export" area of the console "System" page lets you download the export file of one project and repository. The file name is `<project key>__<repository key>.bundle.json`.

The download has the same content as `bundles/<n>.json` in a backup file. It identifies each document by its key, and it includes the content and the hash. The `GET /export` route of the API returns the same content. Version 0.12.0 and earlier versions give a download in the old format, which identifies each document by its document kind. Change your scripts that read this file.

The export file contains no account data, so it cannot replace `backup`.

## <a id="format-versions"></a>Backup format versions

The backup format version is in `manifest.json`, and the output of `verify-backup` shows it. There are two versions:

| Backup format | Made by | How an export file identifies a document |
| --- | --- | --- |
| Version 1 | 0.12.0 and earlier versions | By its document kind |
| Version 2 | Versions after 0.12.0 | By its key |

The backups that each server version can read:

| Server version | A version 1 backup | A version 2 backup |
| --- | --- | --- |
| Versions after 0.12.0 | Can verify and restore | Can verify and restore |
| 0.12.0 and earlier versions | Can verify and restore | Cannot read |

When a new version restores a version 1 backup, it first changes the kind of each document into a key. Then it imports the documents. The change is complete before the command writes data. If a document has no key, `restore` returns non-zero and writes nothing. The message starts with `this backup cannot be restored, and nothing was written:` and then shows the project and repository and the document.

When an old version gets a version 2 backup, `verify-backup` and `restore` both return non-zero:

```text
backup integrity check failed: incompatible backup format version 2 (this build reads 1)
```

A new version also refuses a version that it does not know:

```text
backup integrity check failed: incompatible backup format version 3 (this build reads 1 and 2)
```

So after an upgrade to a version after 0.12.0, only a new version can read the new backups. Keep the backup that you made with the old version before the upgrade. It is the only backup that the old version can read.

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

If the storage location contains data from 0.12.0 or earlier, the `backup` subcommand of a new version refuses to run. It makes no backup file and prints the three upgrade steps. Back up this data with the old version. See [Server Deployment](server-deployment.md#upgrade-from-0-12).

## <a id="verify"></a>Verify a backup

`verify-backup` reads only the backup file. You do not need to stop the server, and you do not need an empty target. The command compares the list with the hash of each member file, checks the structure of the export files, and makes sure that this server knows the backup format version.

```bash
speclink-server verify-backup --input /var/backups/speclink/speclink-20261007T084144Z.tar
```

**Expected output**:

```text
備份完整：格式版本 2、2 個成員、1 個 scope
```

"格式版本" is the backup format version. For a backup from 0.12.0 or earlier, this line shows "格式版本 1".

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

`restore` does six things in this order. The first two read only the backup file and do not touch the target:

1. It checks the integrity of the backup file, the same as `verify-backup`. This step stops a changed backup or an unknown format.
2. It prepares the export file of each project and repository, and it makes sure that it can import each document. For a version 1 backup, this step changes each document kind into a key.
3. It makes sure that the target is empty. If the target is not empty, the command stops, and the existing content of the target does not change.
4. It puts the account database snapshot at the target location.
5. It imports the content of each project and repository into the target storage backend. Each project and repository is one commit, so the command never writes half of it.
6. It compares each item: the keys, the content hash of each document, and the document count of each project and repository, and the record counts and the version of the account data, must match the backup.

**Expected output**:

```text
還原完成且驗證通過：1 個 scope 全數比對一致
```

If one item does not match, `restore` returns non-zero, lists each difference, and says that you must not use this target.

Step 5 is not one transaction for all projects and repositories. If the import fails between two of them, `restore` returns non-zero. It lists the projects and repositories that it restored and the one that failed, and it says that the target is incomplete and that you must not use it. In this case, restore again into a different empty target.

If the target is not empty, the output looks like this, and the target does not change:

```text
restore target is not empty (restore only into an empty target): 1 user(s) exist
```

If the target storage location contains data from 0.12.0 or earlier, `restore` also refuses and prints the three upgrade steps. See [Server Deployment](server-deployment.md#refused-start).

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
