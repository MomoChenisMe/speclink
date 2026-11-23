# @speclink/server

The official demonstration server of [Speclink](https://github.com/MomoChenisMe/speclink), a Spec-Driven Development (SDD) engine written in Rust.

The server holds the specs of remote projects, so that a team shares one set of specs. The Speclink CLI and the desktop app connect to it. It has accounts, login, and an admin console, so you can try remote projects without writing code.

The server is for demonstration and trial. For production use, embed the engine in your own system with [`@speclink/engine`](https://www.npmjs.com/package/@speclink/engine).

## Start

```bash
npx @speclink/server
```

You need Node.js 18 or later. The supported platforms are macOS (arm64, x64), Linux (x64, arm64), and Windows (x64).

Without arguments, the launcher does three things:

1. It makes `speclink-data/config.yaml` from the environment variables below. It writes this file again at each start, so manual edits do not stay.
2. It uses SQLite by default and keeps the data in `./speclink-data/`.
3. It starts the server, which listens only on `127.0.0.1:8080`.

On the first start, the server prints a one-time `/setup?token=…` link. Open it to create the admin account and the first project. The link is valid for 24 hours. A normal restart does not print it again.

To get a fixed `speclink-server` command, run `npm i -g @speclink/server`.

## Environment variables

| Variable | Default | Description |
| --- | --- | --- |
| `SPECLINK_STORE` | `sqlite` | The storage backend: `sqlite`, `serverfs`, or `postgres` |
| `SPECLINK_DATA_DIR` | `./speclink-data` | The data folder for the configuration file and the databases |
| `SPECLINK_PORT` | `8080` | The port that the server listens on at `127.0.0.1` |
| `SPECLINK_PUBLIC_URL` | `http://localhost:<SPECLINK_PORT>` | The URL in the browser address bar of your users. Setup links and invitation links use it |
| `SPECLINK_POSTGRES_URL` | None | The PostgreSQL connection URL. You must set it for `postgres` |
| `SPECLINK_POSTGRES_PASSWORD` | None | The PostgreSQL password. Keep it out of the connection URL |
| `SPECLINK_CONFIG` | None | Use your own configuration file. Then the other variables have no effect, except `SPECLINK_POSTGRES_PASSWORD` |

With arguments, the launcher gives them to the server binary without changes, for example `npx @speclink/server invite --config ./speclink-data/config.yaml …`.

## Let other computers connect

By default, only the local computer can connect. To let other computers connect, use Docker Compose with an HTTPS reverse proxy in front. See [HTTPS and reverse proxy](https://github.com/MomoChenisMe/speclink/blob/main/docs/server-deployment.md#https).

The official Docker image is `ghcr.io/momochenisme/speclink-server`:

```bash
docker run -d -p 8080:8080 -v speclink-data:/data ghcr.io/momochenisme/speclink-server:latest
```

Each storage backend supports only one server instance. Do not point two servers at the same data.

## Check the server

- `GET /healthz` returns `200` when the process is alive.
- `GET /readyz` returns `200` when the storage backend is available, and `503` when it is not.

## Documentation

- [Remote getting started](https://github.com/MomoChenisMe/speclink/blob/main/docs/remote-getting-started.md): from the first start to a connected desktop app and CLI
- [Server deployment](https://github.com/MomoChenisMe/speclink/blob/main/docs/server-deployment.md): npx, Docker, Compose, the configuration file, upgrades, and rollbacks
- [Server backup and restore](https://github.com/MomoChenisMe/speclink/blob/main/docs/server-backup.md): `backup`, `verify-backup`, and `restore`
- [Store port examples](https://github.com/MomoChenisMe/speclink/blob/main/docs/server-store-drivers.md): how to choose SQLite, PostgreSQL, or the file system
- [Changelog](https://github.com/MomoChenisMe/speclink/blob/main/CHANGELOG.md)

## License

[MIT](https://github.com/MomoChenisMe/speclink/blob/main/LICENSE)
