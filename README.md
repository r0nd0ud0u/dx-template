# dx-template

A [Dioxus](https://dioxuslabs.com) 0.7 fullstack app template: a single Rust
codebase that builds to a web app (WASM client + Axum server), a desktop app,
and a mobile (Android) app, sharing the same UI, auth, and realtime
(websocket) logic across all of them.

Included out of the box:

- **Auth** — username/password (optional, `USE_PASSWORD`) or anonymous login,
  session cookies via `axum_session`/`axum_session_auth`, SQLite-backed users
  (`src/auth_manager/`).
- **Realtime state** — a websocket connection pushing server events (e.g. the
  live "connected users" list) to every client; a minimal demo of the
  broadcast pipeline you can replace with your own app state
  (`src/websocket_handler/`).
- **Component library** — a themed wrapper around
  [dioxus-primitives](https://github.com/DioxusLabs/components) (button,
  input, select, tabs, tooltip, sidebar, sheet, alert dialog, ...) in
  `src/components/`.
- **i18n** — English/French via `dioxus-i18n`, toggled from the navbar
  (`src/i18n/`).
- **Multi-platform native clients** — desktop and Android builds that talk to
  a remote web server instance over the same server-fn/websocket protocol.

## Prerequisites

- Rust (stable, edition 2024) — install via [rustup](https://rustup.rs)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/CLI/) matching the version in
  `Cargo.toml`/CI: `cargo binstall dioxus-cli@0.7.9` (or `cargo install
dioxus-cli@0.7.9`)
- For desktop builds on Linux: `libwebkit2gtk-4.1-dev libgtk-3-dev
libayatana-appindicator3-dev libxdo-dev` (see `.github/workflows/test.yml`)
- For Android builds: Android SDK/NDK + `adb` on `PATH`

## Getting started

```bash
cp .env_template .env   # then edit values as needed
dx serve --platform web
```

or, for the exact same thing with LAN access enabled (so a native client on
another device can reach it):

```bash
./scripts/dev_web.sh
```

The app is fullstack by default (`Dioxus.toml` → `default_platform =
"fullstack"`): one process serves both the Axum API/websocket and the
hydrated WASM client.

### Native clients (desktop / Android)

Native clients don't infer a server origin the way a browser page does, so
they need one explicit (`SERVER_URL`, default `http://127.0.0.1:8080`):

```bash
./scripts/dev_desktop.sh
./scripts/dev_android.sh
```

## Environment variables

See `.env_template` for the full list with explanations (`IP`, `PORT`,
`DATABASE_URL`, `SERVER_URL`, `INSECURE_ACCEPT_INVALID_CERTS`, `LOG_LEVEL`,
`USE_PASSWORD`, `IS_MAIN_SERVER`).

## Development workflow

```bash
dx fmt                              # format rsx! blocks
cargo fmt --all                     # format Rust source
cargo clippy --all-targets --all-features
cargo test --all-features
dx build --web                      # verify the fullstack build compiles
```

These are the same checks run in CI (`.github/workflows/test.yml`) — run
them before pushing.

## Project layout

```
src/
  main.rs                 launch (web/desktop/mobile/server) + App() root
  lib.rs                  module declarations
  common.rs                shared Route/context types
  i18n/                    en-US / fr-FR Fluent locale files
  auth_manager/            login/register/session logic, SQLite user store
  websocket_handler/       server-pushed realtime events
  components/              themed dioxus-primitives wrappers (one dir each)
  pages/                   routed pages (home, login) + navbar layout
scripts/                   dev/bundle/docker helper scripts (see each file)
deploy/                    docker-compose + nginx production deployment docs
android/                   debug keystore for reproducible Android signing
```

## Bundling / deployment

- `scripts/bundle_app.sh` — release web bundle
- `scripts/bundle_desktop.sh`, `scripts/bundle_mobile.sh` — native bundles
- `deploy/NGINX.md`, `deploy/FAIL2BAN.md` — production VPS setup notes
- `deploy/docker-compose.yml` — containerized deployment

## Tests

Unit tests live next to the code they cover (`#[cfg(test)] mod tests` at the
bottom of the relevant file), e.g. `src/websocket_handler/event.rs`,
`src/auth_manager/server_fn/auth.rs`, `src/pages/navbar.rs`, `src/i18n.rs`.
Run them with `cargo test`.

