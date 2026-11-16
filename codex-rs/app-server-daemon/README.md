# codex-app-server-daemon

> `codex-app-server-daemon` is experimental and its lifecycle contract may
> change while the remote-management flow is still being developed.

`codex-app-server-daemon` backs the machine-readable `codex app-server`
lifecycle commands used by remote clients such as the desktop and mobile apps.
It is intended for Codex instances launched over SSH, including fresh developer
machines that should expose app-server with `remote_control` enabled.

## Platform support

The current daemon implementation is Unix-only. It uses pidfile-backed
daemonization plus Unix process and file-locking primitives, and does not yet
support Windows lifecycle management.

## Commands

```sh
codex app-server daemon start
codex app-server daemon restart
codex app-server daemon enable-remote-control
codex app-server daemon disable-remote-control
codex app-server daemon stop
codex app-server daemon version
codex app-server daemon bootstrap --remote-control
```

On success, every command writes exactly one JSON object to stdout. Consumers
should parse that JSON rather than relying on human-readable text. Lifecycle
responses report the resolved backend, socket path, local CLI version, and
running app-server version when applicable.

## Bootstrap flow

For a machine with an existing Corbanu managed standalone installation:

```sh
corbanu app-server daemon bootstrap --remote-control
```

`bootstrap` requires the standalone managed install. It records the daemon
settings under `CODEX_HOME/app-server-daemon/`, starts app-server as a
pidfile-backed detached process, and stops any previous detached updater loop.
Its JSON response reports `autoUpdateEnabled: false`.

## Installation and update cases

Corbanu Terminal disables daemon-managed self-install and automatic updates
until the managed installer supports Corbanu Terminal (owner decision D564,
#2197). The daemon does not download or run an installer. Invoking the internal
`pid-update-loop` command returns an error immediately with manual installation
guidance.

Install or update Corbanu Terminal using the platform-specific commands in the
[installation instructions](../../README.md#install), then explicitly restart
the daemon. The daemon still requires the existing standalone managed binary
at `CODEX_HOME/packages/standalone/current/codex`; installer support for that
layout remains a follow-up. A missing managed binary produces an actionable
error instead of attempting an automatic installation.

`start`, `restart`, `stop`, version reporting and remote-control management
remain available for an existing managed installation. `bootstrap` starts
app-server without starting an updater. A running app-server stays on its
current executable until an explicit restart after a manual update.

Product basis: **Product principles** — “Maintain continuous Codex parity
without removing Corbanu-specific behavior.” D564 authorizes disabling the
upstream installer while Corbanu managed-install support is pending.

## Lifecycle semantics

`start` is idempotent and returns after app-server is ready to answer the normal
JSON-RPC initialize handshake on the Unix control socket.

`restart` stops any managed daemon and starts it again.

`enable-remote-control` and `disable-remote-control` persist the launch setting
for future starts. If a managed app-server is already running, they restart it
so the new setting takes effect immediately.

Top-level `corbanu remote-control` stops any previous updater. It enables
remote control on an existing managed daemon, or bootstraps with
`--remote-control` when the daemon is not running.

`stop` sends a graceful termination request first, then sends a second
termination signal after the grace window if the process is still alive.

All mutating lifecycle commands are serialized per `CODEX_HOME`, so a concurrent
`start`, `restart`, `enable-remote-control`, `disable-remote-control`, `stop`,
or `bootstrap` does not race another in-flight lifecycle operation.

## State

The daemon stores its local state under `CODEX_HOME/app-server-daemon/`:

- `settings.json` for persisted launch settings
- `app-server.pid` for the app-server process record
- `app-server-updater.pid` for the pid-backed standalone updater loop
- `daemon.lock` for daemon-wide lifecycle serialization
