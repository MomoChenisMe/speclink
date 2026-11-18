# Store port examples

[繁體中文](server-store-drivers.zh-TW.md) · **English**

The store port is three functions that the host implements: list, read, and commit. This repo has three finished examples. They keep the data in SQLite, in PostgreSQL, and on the file system. All three pass the same conformance tests.

This document is for two readers:

- **Integrators who embed Speclink in their own system.** Find the example for your kind of storage. Read its tables, how it writes one commit, and how it records events. Then write your own three functions the same way. For the full picture of embedding, see [Embed Speclink in your system](../README.md#embedding).
- **Administrators of the demonstration server.** The storage backend of the demonstration server is one of these three examples. Start at [How to choose](#choose).

The examples are for you to read and to copy. They are not production stores. [What the examples do not do](#limits) lists what they leave out on purpose.

## <a id="examples"></a>Where the three examples are

| Example | Location | Data location |
| --- | --- | --- |
| SQLite | [`crates/store/speclink-store-port-sqlite`](../crates/store/speclink-store-port-sqlite/src/lib.rs) | One SQLite file |
| PostgreSQL | [`crates/store/speclink-store-port-postgres`](../crates/store/speclink-store-port-postgres/src/lib.rs) | One PostgreSQL schema |
| File system | [`crates/store/speclink-store-port-fs`](../crates/store/speclink-store-port-fs/src/lib.rs) | One folder |

The code of each example is in one file, `src/lib.rs`. The comment at the top of the file describes the layout and the steps of a commit. The tests are in `tests/it/`.

The contract is in [`crates/store/speclink-store-port`](../crates/store/speclink-store-port/src/lib.rs). It has the definition of the three operations, the map from each kind of document to its key (`src/keys.rs`), the conformance tests (`src/conformance/`), and an implementation that keeps the data in memory, for tests.

The conformance tests have 16 checks. All three examples pass the 16 checks and skip none. To see how to connect an implementation to the conformance tests, read `tests/it/conformance.rs` of each example.

A host that embeds Speclink with Node.js writes the three functions in JavaScript:

- For the inputs, the outputs, and the three guarantees, see [Implement the three store functions](sdk-node.md#store-port) in the Node SDK document.
- Then run the same checks. See [Check your implementation with the conformance tests](sdk-node.md#conformance).
- There is no JavaScript example at this time. Use the three Rust examples as a reference for the tables and the transaction. The [known gaps](product-status.md#known-gaps) in the product status list this item.

The three examples are also the storage of the demonstration server. When the demonstration server starts, it opens the one that its configuration names.

## <a id="choose"></a>How to choose

This section is for administrators of the demonstration server. Know these three points first:

- The server keeps accounts and memberships in a separate SQLite file (the `identity` section of the configuration file). This file does not change with the storage backend.
- The three examples pass the same conformance tests, so the API and the CLI behave the same on each one. A change of storage backend does not change how you use Speclink.
- With every storage backend, the demonstration server supports only **one** instance.

| Storage backend | Data location | Use it for | Watch out for |
| --- | --- | --- | --- |
| `sqlite` | One database file | Most cases. npx and the Docker image use it by default | Keep it on a local disk. Nothing blocks a second instance on the same file |
| `serverfs` | One folder | Plain files, with no database to operate | The file system of the folder must support file locks |
| `postgres` | One PostgreSQL schema | Teams that already operate and back up PostgreSQL | The connection has no encryption (no TLS). Nothing blocks a second instance on the same schema |

Set the storage backend in the `store` section of the configuration file. `sqlite` and `serverfs` use `path`. `postgres` uses `url`:

```yaml
store:
  driver: sqlite
  path: /var/lib/speclink/store.db
```

```yaml
store:
  driver: serverfs
  path: /var/lib/speclink/store     # a folder, not a file
```

```yaml
store:
  driver: postgres
  url: postgres://speclink@db.internal:5432/speclink   # no password here, see below
```

- **The PostgreSQL password** goes in the environment variable `SPECLINK_POSTGRES_PASSWORD`. When this variable has a value, the server connects with it and does not use a password from the connection URL. A password in the URL also works, but the server prints a warning to stderr at startup. People often copy, compare, and paste configuration files into issues, so the files must not contain passwords. No error message shows the connection URL or the password.
- **With npx**, the environment variable `SPECLINK_STORE` selects the storage backend (`sqlite`, `serverfs`, or `postgres`; the default is `sqlite`). The data goes under `SPECLINK_DATA_DIR`: the file `store.db` for `sqlite`, and the folder `store` for `serverfs`. For `postgres`, also set `SPECLINK_POSTGRES_URL`.
- **If `driver` has an unknown name**, or `path` or `url` is missing, the server does not start. The error names the configuration file and the cause. A misspelled `serverfs` never falls back to a different storage backend.
- **`memory`** is the in-memory implementation from the contract. It is for test configurations only. The data goes away when the process stops, and npx does not accept it.

For all configuration fields, the environment variables, and the run modes, see [Server Deployment](server-deployment.md#configuration).

## <a id="layout"></a>Tables and file layout

The three examples store the same things:

- **A scope** is one set of documents that is separate from all other sets. On the demonstration server, a scope is one project and repository pair. Its name is `<project key>/<repository key>`, for example `demo/backend`. Each scope has its own version. The version starts at 0, and each successful commit adds 1.
- **A document** is one key with one content. The key is a string. It is the same as the relative path in the spec folder of a local project, for example `changes/<change name>/proposal.md`, `specs/<capability>/spec.md`, `discussions/<slug>.md`, `config.yaml`, and `LANGUAGE.md`. There are two exceptions. The key of the board order is `board/order`. Archived changes and discussions are under `archive/changes/` and `archive/discussions/`. Each document records the scope version and the time of the commit that last wrote it.
- **Events** come with each commit. See [The event log and its sequence](#events).

Each example writes a marker in a fixed place: `format`, with the value `speclink-store-port`, and the layout version `layout_version`, with the value `1`. When an example opens, it accepts only this pair:

- The location is empty: the example makes the layout.
- The marker and the layout version match: the example opens the data as it is and changes nothing.
- Anything else: the example refuses to open. It makes and changes no table and no file. The error shows the marker and the layout version that it found.

When an example refuses to open, the demonstration server does not start. If `speclink-server` 0.12 or earlier wrote the data, the error lists three steps to move the data. See [the upgrade section of Server Deployment](server-deployment.md#upgrade).

The SQLite and PostgreSQL examples use the same four tables:

| Table | Columns | Contents |
| --- | --- | --- |
| `meta` | `key`, `value` | Two rows: `format` and `layout_version` |
| `scopes` | `scope`, `version`, `event_seq`, `event_floor` | One row for each scope: the scope version, the sequence of the last event, and the event floor |
| `documents` | `scope`, `key`, `content`, `version`, `modified_at` | One row for each document. The primary key is `scope` plus `key`. `modified_at` is in milliseconds (Unix time) |
| `events` | `scope`, `seq`, `version`, `name`, `payload` | One row for each event. The primary key is `scope` plus `seq`. `version` is the scope version after that commit |

### <a id="sqlite"></a>SQLite

- Text columns are `TEXT`, and number columns are `INTEGER`. The event `payload` is JSON text in a `TEXT` column.
- If the file does not exist, or the file has no table, the example makes the four tables. If the file has tables but no `meta` table, the example refuses to open.
- The database uses the SQLite WAL mode, so work files with names that end in `-wal` and `-shm` appear next to the database file. When you move or copy the data, include them, or use [`backup`](server-backup.md).
- Keep the file on a local disk. The WAL mode does not support network file systems such as NFS or SMB.
- One store object has one connection. If it waits more than 5 seconds for the write lock, it reports "temporarily unavailable", not a conflict.
- Two store objects can open the same file, and the version check stays correct: the SQLite locks make their commits take turns. So the demonstration server does **not** block a second instance on the same file. You must prevent it yourself.
- The availability check makes sure that the file is still at its path and that a write works. If someone moves or deletes the file, `/readyz` of the demonstration server returns `503`.

### <a id="postgres"></a>PostgreSQL

- The example makes the four tables in the **current schema** of the connection. Set `search_path` in the connection string to choose the schema. Several stores can then share one database.
- If the schema has no `meta` table, the example makes the four tables. Other tables in the schema stay as they are. If the schema already has a table with one of the four names (for example `documents`) but no `meta` table, the open fails and the example makes nothing.
- The `scope` and `key` columns are `text COLLATE "C"`. The contract orders keys by their bytes, and the default collation of a database follows its locale.
- `content` is `bytea`. A `text` column cannot hold a NUL character, and the contract permits one in a document.
- `payload` is JSON text in a `text` column, not `jsonb`. `jsonb` changes how numbers are written and the order of keys.
- All number columns are `bigint`.
- One store object has one connection, and there is no connection pool. If the connection is gone, the next call connects again one time and then runs. While the database is not reachable, requests to the demonstration server return `unavailable` (HTTP 503), and `/readyz` returns `503`. When the connection comes back, the same server continues. You do not need to restart it.
- One case does not run again: the example sent `COMMIT`, and the connection broke before the answer came. The example cannot know whether the database applied the commit, so it reports "temporarily unavailable". When the caller tries again, the version check rejects writes that the database already applied, so nothing is written twice.
- The connection has no encryption (no TLS). A hosted PostgreSQL that accepts only TLS does not work. Keep the database on a trusted internal network.
- Two store objects can connect to the same schema, and the version check stays correct. So the demonstration server does **not** block a second instance on the same schema.
- CI runs the tests of this example on PostgreSQL 15.

### <a id="serverfs"></a>File system

The configuration of the demonstration server calls this example `serverfs`. The folder contains these files:

```text
<folder>/
├── meta.json
├── lock
└── scopes/
    └── <folder name of the scope>/
        ├── index.json
        ├── blobs/<number>
        └── events/<sequence>.json
```

| File | Contents |
| --- | --- |
| `meta.json` | The two fields `format` and `layout_version` |
| `lock` | The lock file. The store object holds an exclusive lock on it for as long as the object lives |
| `scopes/<scope>/index.json` | The only source of truth for the scope. Its fields are `version`, `event_seq`, `event_floor`, `next_blob`, and `documents`. For each key, `documents` records `blob` (the number of the content file), `version`, and `modified_at` |
| `scopes/<scope>/blobs/<number>` | The content of one document. It never changes after the example writes it |
| `scopes/<scope>/events/<sequence>.json` | One event, with the fields `name`, `payload`, and `version` |

- **The folder name of a scope**: lowercase ASCII letters, digits, `-`, and `_` stay as they are. Every other byte becomes `%XX`. The folder of the scope `demo/backend` is `demo%2Fbackend`. A document key never becomes a file name.
- **A read trusts only `index.json`.** A content file that `index.json` does not name does not exist for a read. An event file with a sequence above the one in `index.json` does not exist for a read either. Each open and each commit removes such files.
- **There is no history of old versions.** After a commit replaces or deletes a document, the example deletes the old content file at the end of that commit.
- **The file lock**: the operating system holds the lock. When the process ends (also when it is killed or crashes), the system releases the lock. The next open can then take over, and the `lock` file can stay in place. If a second store object opens the same folder, it fails at once and reports that the folder is in use. It does not wait, and it does not take the lock. The demonstration server then does not start.
- Before you deploy, make sure that the file system of the folder supports file locks. Use a local disk. File locks on network file systems such as NFS are not reliable in some setups.
- File modification times on the file system have no meaning. `index.json` records the last-modified time of each document.
- Three temporary file names appear for a short time: `index.json.new`, `meta.json.new`, and `probe.tmp`.
- The folder name of a scope can be three times as long as the scope name. A long scope name can then be more than the file system permits for one name, and the commit fails. On Windows, a scope with a device name such as `con` or `nul` possibly cannot get its folder.
- The folder format is private to this example. Do not edit it by hand. For an export that people can read, use [`backup`](server-backup.md).

## <a id="commit"></a>The steps of one commit

When the engine runs a verb, it asks the store only for the documents that the verb uses. In one run, it lists a prefix at most one time and reads a key at most one time. When it needs several documents, it gets them in one read. It does not load the whole scope into memory. The engine keeps the writes of the verb in memory. When the verb succeeds, all writes, deletes, and events go to the store as **one commit**. A query does not commit.

Each write in a commit carries an expectation: the document "must not exist", or it "must be at this version". Each delete also carries the expected version. The store must give three guarantees:

1. A commit applies all of its operations or none of them.
2. One expectation that does not hold rejects the whole commit. The store reports a conflict: the key, the expectation of the caller, and the current version of the document. A rejected commit changes no document, does not move the scope version, and keeps no event.
3. The events are in the same commit as the writes.

The PostgreSQL example writes one commit as one transaction. This is the SQL that it runs, in order (`BEGIN` and `COMMIT` stand for the start and the commit of the transaction):

```sql
BEGIN;

-- 1. The row of the scope: insert it if it is not there, then lock it
INSERT INTO scopes (scope, version, event_seq, event_floor)
VALUES ($1, 0, 0, 0) ON CONFLICT (scope) DO NOTHING;
SELECT version, event_seq FROM scopes WHERE scope = $1 FOR UPDATE;

-- 2. One query for the current version of each operation; compare all of them before the first write
SELECT version FROM documents WHERE scope = $1 AND key = $2;

-- 3. The writes and deletes; a written document gets the new scope version
INSERT INTO documents (scope, key, content, version, modified_at)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (scope, key) DO UPDATE SET
    content = excluded.content,
    version = excluded.version,
    modified_at = excluded.modified_at;
DELETE FROM documents WHERE scope = $1 AND key = $2;

-- 4. The events; the sequences continue after event_seq
INSERT INTO events (scope, seq, version, name, payload)
VALUES ($1, $2, $3, $4, $5);

-- 5. Update the row of the scope, then commit
UPDATE scopes SET version = $2, event_seq = $3 WHERE scope = $1;
COMMIT;
```

What each step does:

1. **Lock the row of the scope.** `SELECT … FOR UPDATE` makes the commits on one scope take turns, from every connection and every process. Commits on different scopes do not wait for each other. If the wait for the lock is longer than 5 seconds (`lock_timeout`), the example reports "temporarily unavailable".
2. **Compare the versions first, then write.** The example checks the expectation of every operation before the first write. If one does not hold, the example rolls the transaction back and reports the conflict. So a commit is never half written. Do not check and write one operation at a time.
3. **Write and delete.** The new scope version is the old version plus 1. Each document that this commit writes gets this new version as its `version`, and the time that the commit carries as its `modified_at`.
4. **Write the events in the same transaction.** If the transaction does not commit, the events do not exist. A rejected commit uses no sequence.
5. **Update the scope and commit.** `COMMIT` is the one point where the commit takes effect. The return value is the new scope version.

A list and a read each run in one read-only transaction (`REPEATABLE READ`). So the result of one list or one read comes from one completed commit. It never mixes the content of two commits.

The other two examples do the same thing:

- **SQLite**: the whole commit is one `BEGIN IMMEDIATE` transaction. It takes the write lock at the start, so nothing can change the versions between the check and the write. The order of the steps is the same as above.
- **File system**: the example reads `index.json` and checks every expectation. Then it writes the new content files and event files and syncs them to the disk. Then it writes `index.json.new`, syncs it, and renames it to `index.json`. This rename is the one point where the commit takes effect. If the example stops before the rename, a read sees the state before the commit. After the rename, the example deletes the content files that the commit replaced. One case has no sure answer: the sync of the folder fails after the rename. The commit is then visible, but it is possibly not there after a power loss. The example reports a store failure. The caller must list the scope again and then decide whether to send the commit again.

## <a id="events"></a>The event log and its sequence

Each commit can carry events (for example, an artifact was written, or a change was archived). All three examples record the events in their own event log:

- The events of each scope have their own **sequence**. It starts at 1 and adds 1 for each event. The events of one commit get consecutive sequences, in their order in the commit.
- Each event records its name, its payload, and the scope version after that commit.
- A rejected commit keeps no event and uses no sequence.
- A commit with only events and no writes is also valid, and it moves the scope version.

The three operations of the contract do not read events back. The host decides whether its store keeps events. Outside the contract, each example has four more methods for the host that owns the store:

| Method | What it does |
| --- | --- |
| `events_after(scope, after, limit)` | Returns the events with a sequence larger than `after`, smallest first, at most `limit` of them. Each event comes with its sequence |
| `event_floor(scope)` | Returns the floor: the largest sequence that was removed. It is 0 when nothing was removed |
| `trim_events(scope, through)` | Removes the events with a sequence up to and including `through`. It stops at the largest sequence in use. Later events continue the numbering and do not use a removed sequence again. It changes no document and no version |
| `probe()` | Checks that the store can answer: a read works, and a write works |

The examples never remove events by themselves. If nothing calls `trim_events`, the events stay. To learn whether you missed events, compare your last sequence with the floor. If your sequence is smaller than the floor, some events between them are gone.

The demonstration server uses the four methods in this way:

- The event stream `GET /events` (see [Event stream](verb-contract.md#events) in the verb and flag document) reads events with `events_after`. The id of each pushed event is its sequence. When a client connects again, it sends the last id that it received, and the server sends the events after it.
- After the server pushes events, it removes older events with `trim_events`. It keeps the newest `events.retention` events of each scope (the default is 1024; set it in the configuration file).
- `/readyz` and the "System" page of the console use `probe` to tell whether the storage backend can answer.

A host that needs "at least once" delivery does what the examples do. In the same transaction as the commit, it writes the events to its own event table. Then its own program sends them from that table. See [The four steps of a verb](sdk-node.md#four-steps) in the Node SDK document.

## <a id="limits"></a>What the examples do not do

The three examples leave out these things on purpose. When you write your own store from an example, add what your environment needs:

- **A connection pool.** One store object has one connection, and all calls take turns on it.
- **TLS.** The PostgreSQL example always uses a connection with no encryption.
- **Migration between layout versions.** An example opens only data of layout version `1`. It refuses every other version and does not move the data by itself.
- **Event delivery between several instances.** After one instance commits, nothing tells another instance that there are new events.
- **A history of old document versions.** After a commit replaces a document, the old content is gone.
- **Automatic removal of events.** The host decides whether to remove events and how many to keep.

Because of these limits, the demonstration server supports only one instance. Do not use `--scale`, and do not point two servers at the same data. `serverfs` blocks a second instance. `sqlite` and `postgres` do not. When two instances open the same data, the data does not break. But after one instance commits, nothing tells the other instance, so the clients on that instance do not get the update at once.

## <a id="switch"></a>Change to another storage

The three examples have different layouts, so you cannot move the files or copy the tables directly. To change the storage backend, use a backup file:

1. Stop the server. Run `speclink-server backup --config <old config> --output <backup file>` with the old configuration.
2. Prepare the new configuration. Point `store` at an empty location, and point `identity` at a new, empty account database.
3. Run `speclink-server restore --config <new config> --input <backup file>` with the new configuration. Then start the server with the new configuration.

The backup file does not depend on the storage backend, and `restore` compares each item of the result. For the details of the commands and the rule that `restore` accepts only an empty target, see [Server Backup and Restore](server-backup.md).
