# Task Node

`pfterminal tasknode` is an agent-facing JSON helper for GitHub-linked Task Node
terminal sessions. Every command emits a JSON envelope (`{"ok": true, ...}` /
`{"ok": false, "error": "...", ...}`) and exits non-zero on failure, so it can be
driven from a script or by an agent with shell access.

```
pfterminal tasknode status        # linked account, wallet, server flags, task counts
pfterminal tasknode tasks         # tasks by tab (defaults to outstanding)
pfterminal tasknode task <id>     # inspect or mutate one task
pfterminal tasknode verification  # respond to verification requests
pfterminal tasknode balance       # linked-wallet PFT balance
pfterminal tasknode rewards       # recent rewarded tasks
pfterminal tasknode request       # create a task request
pfterminal tasknode chat          # Task Node chat
pfterminal tasknode context       # read or save the context document
```

## Headless bootstrap

Every command above needs a `tasknode/session` credential in the vault. Create it
with `tasknode link`. The flow needs a browser only to *authorize* — the terminal
itself never needs a TTY, a display, or D-Bus, so this works over SSH, in a
container, or from an agent.

```console
$ pfterminal tasknode link
{"ok":true,"state":"pending","verificationUrl":"https://.../auth/terminal/...","requestId":"..."}
```

Open that URL anywhere — including on a phone — and authorize with GitHub. Then:

```console
$ pfterminal tasknode link --poll --timeout 300
{"ok":true,"state":"linked","githubUsername":"...","accountId":"..."}
```

That's it. The session persists in the vault and every other command now works.

| Flag | Meaning |
| --- | --- |
| `--poll` | Poll a pending link to completion. Re-runnable; a timeout leaves the pending session intact. |
| `--status` | Report `linked` / `pending` / `unlinked`. Always exits 0. |
| `--relink` | Discard an existing session and start over. Required to overwrite a valid session. |
| `--timeout <secs>` | Bound `--poll` (default 300). Exits non-zero on timeout. |
| `--no-browser` | Never open a browser. Implied automatically when stdout is not a TTY. |
| `--json` | Accepted for scripts; this helper always emits JSON. |

Failures are distinguishable rather than collapsed into "not linked":

```json
{"ok":false,"error":"tasknode_unlinked","state":"unlinked","message":"..."}
{"ok":false,"error":"tasknode_link_pending","state":"pending","verificationUrl":"..."}
{"ok":false,"error":"tasknode_vault_unavailable","message":"..."}
```

`link` additionally reports `tasknode_already_linked` (use `--relink`),
`tasknode_no_pending_link`, and, from `--poll`, one of `tasknode_link_timeout`
(keep waiting — re-run `--poll`), `tasknode_link_rejected` (definitive: the request
was denied or expired server-side, start over), or `tasknode_link_poll_failed`.
A `tasknode_session_corrupt` means the stored session could not be parsed.

## Sharing a session with a service

The credential is keyed to `CODEX_HOME`. Any process running as the same user with
the same `CODEX_HOME` reads the same session — so linking once from a shell also
links the `pfterminal-telegram` connector, provided its unit sets
`Environment=CODEX_HOME=...` (or an `EnvironmentFile` that does).

Link once, and the Task Node loop — list, inspect, accept, submit evidence, answer
verification, check balance — is drivable from wherever that service is reachable.

## Where the credential actually lives

`Vault` stores secrets in `$CODEX_HOME/secrets/local.age`, encrypted with a
passphrase held in the OS keyring. When no OS keyring is available — a headless
container, a box with no `secret-service` — it falls back to a private file:

```
$CODEX_HOME/secrets/keyring-fallback/<sha256>.key   # mode 0600
```

Two consequences worth knowing:

1. **Back up `secrets/local.age` and `secrets/keyring-fallback/` together.** They are
   useless apart. A backup that skips `keyring-fallback/` — it looks like a cache —
   cannot be restored.
2. **Don't move or symlink `CODEX_HOME` once the vault holds anything.** The keyring
   account is derived from the *canonicalized* path, so the passphrase would not be
   found under the new path.

On a host using the file fallback, the vault is protected by filesystem permissions
rather than an OS keyring. Anyone who can read that user's files can read the Task
Node token. Treat it like any other `0600` credential file.
