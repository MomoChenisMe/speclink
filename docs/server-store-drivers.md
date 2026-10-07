# Server Storage Backends

[繁體中文](server-store-drivers.zh-TW.md) · **English**

The storage backend (store driver) decides where the server keeps the project data (specs, changes, discussions, and more). In most cases, use the default `sqlite`.

Know these points first:

- The server keeps accounts, memberships, and credentials in a separate SQLite file (the `identity` section of the configuration file). This file does not change with the storage backend.
- All three storage backends pass the same conformance tests. The API and the CLI behave the same on each backend, so a change of backend does not change how you use Speclink.
- Each storage backend supports only **one** server instance.

For the run modes and all configuration fields, see [Server Deployment](server-deployment.md).

## <a id="choose"></a>How to choose

| Storage backend | Data location | Use it for | Watch out for |
| --- | --- | --- | --- |
| `sqlite` (default) | One database file | Most cases | Use a local disk only. It does not block a second instance. |
| `serverfs` | One data folder | Plain files, with no database to operate | The file system must support `flock` file locks. |
| `postgres` | A PostgreSQL database | Teams that already operate and back up PostgreSQL | PostgreSQL 15 or later. No TLS connections. It does not block a second instance. |
| `memory` | Memory | Tests only | The data goes away when the process stops. The npx launcher does not accept it. |

If the configuration names an unknown storage backend, the server does not start. The error shows the supported names. A misspelled `serverfs` never falls back to a different backend.

With npx, the environment variable `SPECLINK_STORE` selects the backend. Other run modes use the `store` section of the configuration file, as the sections below show.

## <a id="sqlite"></a>sqlite (default)

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

- The database uses the SQLite WAL mode, so the `store.db-wal` and `store.db-shm` work files appear next to it. When you move or copy the data, include them, or use [`backup`](server-backup.md).
- Keep the file on a local disk. The WAL mode does not support network file systems such as NFS or SMB.
- Only one server can use the file. The server does **not** block a second instance on the same file, so you must prevent it yourself.

## <a id="serverfs"></a>serverfs

```yaml
store:
  driver: serverfs
  path: /var/lib/speclink/store     # a folder, not a file
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

`path` points to a **data folder**. If the folder does not exist, the server makes it. The server sets up an empty folder as a new store.

### File lock

serverfs uses an operating system file lock (`flock`), so only one server writes at a time. The operating system holds the lock. If the server stops by force, crashes, or loses power, the system releases the lock. The next server can then take over. You never need to remove a lock by hand.

If a second server points to the same folder, it does not start and reports `unavailable`. It does not wait, and it does not take the lock.

Before you deploy, make sure that the file system of the data folder supports `flock`. All local disks support it. Network file systems such as NFS are not reliable in some setups, so do not use them.

### When the server refuses to start

In these cases, the server does not start, and it changes nothing in the folder:

- The folder is not empty, and serverfs did not make it (for example, a typo in the path points to a different folder).
- The version data in the folder is damaged, or its version is newer than this server supports.
- A different storage backend marked the folder.

### Other limits

- The folder format is private to serverfs. Do not edit it by hand. For an export that people can read, use [`backup`](server-backup.md).
- File modification times have no meaning. If a backup tool changes them, nothing changes in the behavior.
- Each write adds a new version file for each changed document. The server removes old version files only at the **next start**. If the server runs for a long time without a restart, the disk usage grows with the writes. There is no compression or history trimming.

## <a id="postgres"></a>postgres

```yaml
store:
  driver: postgres
  url: postgres://speclink@db.internal:5432/speclink   # no password here, see below
identity:
  driver: sqlite
  path: /var/lib/speclink/identity.db
```

The server makes four tables in the current schema of the connection: `documents`, `history`, `outbox`, and `meta`. So with a different `search_path`, several stores can share one database. The server sets up an empty schema.

### Password

Put the password in the environment variable `SPECLINK_POSTGRES_PASSWORD`. If the connection URL has no password, the server adds the password from this variable.

A password in the URL also works, but the server prints a warning to stderr. People often copy, compare, and paste configuration files into issues, so the files must not contain passwords. If the URL contains a password, the environment variable does not replace it.

### Limits

- **PostgreSQL 15 or later.** CI runs the full test set on this version.
- **No TLS connections.** The server always uses an unencrypted connection. A hosted PostgreSQL that accepts only TLS does not work. Keep the database on a trusted internal network.
- **One server only.** The server runs writes to the same scope one at a time, so the data does not break. But the server does **not** refuse a second instance on the same database. Several servers on one database are not supported.
- **The table format is private.** Do not edit it by hand. For an export that people can read, use [`backup`](server-backup.md).

### When the server refuses to start

In these cases, the server does not start, and it changes nothing in the database:

- The schema already contains tables that Speclink did not make (for example, a typo in the URL points to a different database).
- The version in `meta` is newer than this server supports.
- The authentication fails, or the database does not exist. The error includes the PostgreSQL message.

A lost connection while the server runs is not data damage. Requests return `unavailable`, and `/readyz` returns `503`. When the connection comes back, the same server continues. You do not need to restart it.

## <a id="switch"></a>Change the storage backend

You cannot move files directly between storage backends. Use a backup file:

1. Stop the server, and run `backup` with the old configuration.
2. Prepare a new configuration that points to a new, empty store.
3. Run `restore` with the new configuration, and then start the server with the new configuration.

The backup file does not depend on the storage backend, and `restore` compares each item of the result. For the commands and the rules, see [Server Backup and Restore](server-backup.md).
