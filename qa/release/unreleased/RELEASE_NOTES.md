# Corbanu Terminal: notes for the next release

Notes for changes on `main` that are not released yet. The release worker moves
them into `qa/release/<version>/RELEASE_NOTES.md` and empties this file.

## Protocol change: `refused` command status

- **A refused command now says "Refused", not "Approval declined".** When a
  policy or setup check stops a command without anyone being asked to approve
  it, the command's cell shows `└ Refused: <reason>` (the transcript says
  `✗ Refused`), and `corbanu exec` prints `refused:` followed by the reason
  (#822, #851, #900).
- **New status value.** The app-server protocol's `commandExecution` item can
  now have `status: "refused"` (alongside `inProgress`, `completed`, `failed`
  and `declined`); `corbanu exec --json` reports `"refused"` too. `declined`
  now means only that an approval request ended without permission (denied,
  timed out or cancelled). `refused` usually means the command did not run,
  but a network-policy denial can arrive after it started, so don't assume it
  never ran. The refusal reason is in `aggregatedOutput` (`aggregated_output`
  in `exec --json`); the exit code is a placeholder, not a process exit. File
  changes (`fileChange`) have no `refused` status.

### Compatibility

- **App-server clients and SDKs generated before this change** don't know
  `refused` and can fail to read an `item/completed` that carries it. Regenerate
  from this release's schema (`codex-rs/app-server-protocol/schema`). The
  TypeScript SDK in `sdk/typescript` (which reads `corbanu exec --json`)
  already includes it.
- **The Python SDK (`sdk/python`) does not include it.** It stays on its pinned
  upstream runtime (`openai-codex-cli-bin==0.144.4`), which has no `refused`.
  A Python client pointed at a Corbanu binary (`CodexConfig.codex_bin`) can
  fail on a refused command.
- **Saved history.** Default sessions don't save command status, so older
  builds resume them as before. Threads started with the experimental
  app-server `historyMode: "paginated"` (and their subagents) do: an older
  build can't read the history of such a thread once it has a refused command
  (`thread/resume` with turns, `thread/turns/list` with full items,
  `thread/items/list` and `thread/turns/items/list` fail with "failed to
  deserialize stored thread item"). Newer builds read older history unchanged.

## `/cost`

- **Codex Auto Review is named as unpriced.** On an OpenAI API key, requests
  from the automatic approval reviewer still show `no price available` (no
  estimate, never $0), and the next step now adds "OpenAI doesn't publish a
  price for Codex Auto Review." (#913).
- **Editing the first message counts as a reopen.** Editing the first message
  of a saved conversation (Esc Esc) now sends its warm-up as a reopen warm-up,
  as editing a later message already did: `/cost` lists it as "Reopen
  warm-up", Corbanu says once it has been sent, and `skip_reopen_warmup = true`
  skips it (#1038).

## `/security` wording

- **Revoke all names its real scope.** After "Revoke all active authority",
  the confirmation now says it was revoked "in every session of this Corbanu
  Terminal on this Corbanu home", as its review says, instead of "in this
  session". A saved kill switch, on or off, also says that other Corbanu
  Terminals on the home take it within about a second (#1035).
- **A stale inspector view is not called degraded.** When the inspector's facts
  are more than 30 seconds old and nothing else is wrong, its top line reads
  "◌ Moderate set; this view is stale (status observed 33s ago)" instead of
  "▲ Moderate degraded"; press `r` to refresh. A stale view still never shows
  the green "Protected" badge (#1036).

## `/security`: windows without a session yet, and remote app servers

- **Moderate before the window's session starts.** Confirmed in a window whose
  session hasn't started, Moderate's Saved screen no longer says the session
  stays Permissive until a restart: that session starts under Core's Moderate
  rules; the credential broker still waits for the restart (#1047).
- **Remote app server windows.** In a window whose sessions run on an app
  server outside Corbanu Terminal (a remote server or the local daemon),
  `/security` no longer says the session starts here, says the level reaches
  that server only if it runs on this Corbanu home, and the level picker
  doesn't offer "restart now", since Corbanu Terminal starts against such a
  server only under Permissive (#1047).

## Named accounts (behind `named_accounts`)

- **App servers in homes with only the default login no longer wait for a
  conversation.** With named accounts on, a standalone app server or daemon
  held back its own ChatGPT requests (apps, plugins, rate limits, the model
  catalog) until a conversation started, even in homes without named
  accounts. It now does that only when the home has a named OpenAI sign-in
  account or `[provider_accounts]` selects one. A server started without one
  starts holding them from the first conversation on an account added while it
  runs, and never goes back to the default login (#1085).
- **`/status` names the account only where named accounts exist.** With named
  accounts on, the provider line read "OpenAI · account default" (or
  "Z.AI · account default") in every home. It now names the account only when
  that provider has a named account in the home, or the session is on or used
  one; otherwise the line is what it was without the feature (D86).

## `corbanu login status`

- **No part of an API key is shown.** With an API-key login it used to print
  the key's first 8 and last 5 characters. It now prints
  `Logged in using an API key - API key configured`, plus
  `(fingerprint <12 hex>)` when the home's vault has its fingerprint salt. That
  is the same per-home fingerprint a named OpenAI account holding the key
  shows (REG-02).
