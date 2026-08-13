# Telegram Integration Implementation Report

Date: 2026-07-22  
Branch: `feat/telegram-connector-hardened`  
Pull request: [agtico/PfTerminal#59](https://github.com/agtico/PfTerminal/pull/59)  
Candidate binary SHA-256: `dd43427a94989d26383b7ec28c382b5049796564ab5c85193048ce023fd3d1eb`

## Verdict

The implementation work in Phases 2–5 of
`TELEGRAM-INTEGRATION-WITH-PFTERMINAL.md` is present and passes its scoped
automated gates. One private-chat replay has now run against a real bot and
exposed a first-message reconciliation loop; that defect was fixed and the
same durable pending update then completed without a repeated app-server
error. The connector remains **experimental** because the full live matrix,
three-session gate, native final-head packages, and durability soak have not
run.

## Implemented boundaries

| Boundary               | Implementation evidence                                                                                                                                                                                                                                                                                                                                            |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Conversation identity  | One `ConversationKey` carries chat and optional forum-topic ID through sessions, replies, queues, cancellation, streaming, and approvals. Old chat-only state keys still load.                                                                                                                                                                                     |
| Authorization          | Chats are default-deny. Group and forum messages require both an allowed chat and an allowed actor. Approval callbacks bind to the exact conversation and the actor who started the turn.                                                                                                                                                                          |
| Durable delivery       | Raw updates enter a bot-keyed bounded inbox before dispatch. Atomic persistence is serialized, failed writes roll memory back, completed IDs are bounded, and corrupt state is isolated.                                                                                                                                                                           |
| App-server idempotency | User turns and steering use deterministic `telegram:<bot-id>:<update-id>` client IDs. Pending replay checks thread history before resubmission.                                                                                                                                                                                                                    |
| First-message recovery | Fresh threads skip history reconciliation until a first message materializes them. A replayed first message treats the app-server's structured “not materialized” response as an empty history, while unrelated errors still fail. Persisted threads are replaced only for the structured missing-rollout state; transient resume failures do not discard context. |
| Natural follow-ups     | Active turns use `turn/steer`; non-steerable cases enter a 16-item/256-KiB per-conversation FIFO without blocking later Telegram updates behind the queued handler.                                                                                                                                                                                                |
| Approval recovery      | A failed app-server resolution leaves the approval pending. Successful state changes are not replayed merely because the Telegram confirmation message failed.                                                                                                                                                                                                     |
| Attachments            | Images remain native image inputs. Bounded text/source, JSON, PDF, XML, and YAML files receive stable hashed paths and sidecar metadata. Archives, executables, and opaque binary media are rejected. Cleanup enforces age and total-byte caps.                                                                                                                    |
| Operation              | `/status` reports plain-language runtime state, queue depth, last contact/error, and next action. `telegram --health` checks authorization, storage, workspace, provider credentials, sandbox viability, and Bot API identity.                                                                                                                                     |
| Services               | Setup preserves existing chat/user policy, rejects command-line tokens, runs health before service install, and emits systemd, launchd, and Windows current-user service definitions.                                                                                                                                                                              |
| Installed setup        | PFTerminal package archives now carry the setup script and all service templates. `pfterminal telegram --setup` locates the packaged copy and runs it without requiring a source checkout.                                                                                                                                                                         |
| Log privacy            | Structured connector logs use a stable redacted conversation identifier. Authorization and media logs no longer emit raw Telegram chat/user IDs or paths containing them.                                                                                                                                                                                          |

## Verification completed

- `just test -p codex-telegram`: **111 passed, 0 failed**.
- Package-helper tests: **15 passed, 0 failed**, including PFTerminal-only
  Telegram resources and an unchanged stock Codex package layout.
- `cargo clippy -p codex-telegram --all-targets --no-deps`: passed.
- `just fix -p codex-telegram`: passed.
- `just fmt`: passed.
- `just bazel-lock-check`: passed.
- `cargo build -p codex-cli --bin pfterminal`: passed; candidate hash is
  recorded above.
- `pfterminal telegram --help`: exposes `--health` and documents the readiness scope.
- Clean-home setup dry run: generated valid configuration and a systemd unit with absolute executable and environment-file paths.
- Built-package setup dry run: the extracted PFTerminal binary found and
  launched `codex-resources/telegram/setup-telegram.sh`; the package also
  contained the systemd, launchd, Windows Task, and `AGENTS.md` templates.
- Missing-token health run: exited nonzero with the exact environment/vault remediation and no hang.
- LaunchAgent template: parsed as XML. Windows installer was inspected, but PowerShell is unavailable on this Linux host.
- `just test -p codex-cli`: 519/520 passed. The sole
  `debug_clear_memories_resets_memories_db_without_state_db` failure reproduces
  on the clean `c378b230a` base checkout and is unrelated to Telegram.

### Live evidence completed

- Real bot identity: `@a65123_bot`; one explicitly authorized private chat.
- A clean qualification home and workspace were created under
  `/tmp/pfterminal-telegram-qual-29929282255` without copying live Telegram
  state.
- The first `/start` update completed, but the next text update exposed a
  deterministic retry loop: `thread/read includeTurns=true` ran before the
  new thread had its first user message, returned `-32600`, and released the
  durable update for another attempt. Restart then exposed the adjacent
  missing-rollout state.
- After repair, the exact pending update `570445838` moved to the completed
  set, the pending count became zero, and a delivered assistant item was
  persisted. The captured replay log contains zero `thread/read`,
  `not materialized`, `no rollout`, or app-server failure entries.
- Replay evidence:
  `/tmp/pfterminal-telegram-qual-29929282255/evidence/replay-after-unmaterialized-fix.log`.

The first nonpublishing package workflow at
`https://github.com/agtico/PfTerminal/actions/runs/29929282255` passed Linux
x64, Linux ARM64, macOS ARM64 (including DMG), and macOS Intel (including
DMG). Its Windows job hit the workflow's exact 90-minute timeout while still
compiling; it did not report a compiler or smoke-test failure. The timeout is
now 150 minutes. That workflow predates the live defect repair and therefore
does **not** qualify the final head. A second nonpublishing run was cancelled
once its head became obsolete. Neither run published a release or changed
Latest.

Two workspace tools could not provide a Telegram verdict:

- `just argument-comment-lint -p codex-telegram` is blocked because its pinned
  Rust 1.92 nightly cannot compile the workspace's SQLx 0.9 dependency, which
  requires Rust 1.94.
- Dependency-inclusive `cargo clippy -D warnings` stops on a pre-existing
  `codex-api` warning. Package-only Clippy is green.

## Unrun release gates

The following requirements remain deliberately unclaimed:

1. Three fresh sessions on the exact packaged artifact, with at least 50
   accepted updates and one non-implementer driver.
2. Real Telegram text, images, documents, approvals, cancellation, replay,
   restart, polling conflict, and network-loss fault injection with an invariant
   observer.
3. Independent adversarial review of group/topic authorization and callback
   isolation.
4. Native packaged execution on the final head on all five targets. Earlier
   Linux and macOS packages passed; Windows timed out before smoke tests.
5. The seven-day, 100-turn durability soak required for a stable claim.

The real bot and private chat are configured only in the isolated qualification
home. An external live driver and additional fresh qualification homes are the
remaining inputs for gates 1–2.

## Release rule

Merging this report does not graduate the connector. PR #59 may be described as
disabled experimental code only after its packaging CI and independent review
pass. The private-testing and stable labels remain governed by the release table
in `TELEGRAM-INTEGRATION-WITH-PFTERMINAL.md`.
