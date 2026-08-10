# Run 03 — paid provider, long turns, metering, and context compaction

- Stories: US-4, US-5
- Started: 2026-07-19T04:59:12Z
- Paid plan: Basic
- Terminal: tmux `pft_wallet_fresh_basic`, 94×57

## Attempt 1

A comprehensive read-only repository review exercised repeated tool calls and
streaming settlement. Billing recorded four authoritative settled rounds for
283,235 tokens and released 28 unsuccessful or ambiguous reservations without
charge. After 4m26s the turn failed because the TUI advertised a 202,752-token
GLM context while the live Ambient route rejected the 101,377th input token.

This attempt is FAIL and does not count toward qualification.

## Remediation and attempt 2

The provider/model route now supplies Ambient and PfTerminal Plan with the
observed 101,376-token ceiling while leaving other GLM providers unchanged.
The normal 90% compaction threshold is therefore 91,238 tokens, leaving route
headroom. Route-scoping and explicit-override tests pass.

Attempt 2 started on exact binary SHA-256
`9cbe0971085a30348df5f35df30638f53aaed80ef6adb18603b616ac0fe8308b`.
The live `/status` surface reported `0 used / 101K` before the turn. It crossed
the compaction boundary without a provider overflow and released both
compaction-related reservations without charge. Immediately afterward, however,
the model announced that it would continue and ended without performing the
work or returning the requested report. Attempt 2 is FAIL under WTURN-001.

## Attempt 3

The existing structured completion classifier is now enabled for the
demonstrated Ambient and PfTerminal Plan routes. It classifies a text-only stop
after tool work against the original user objective, injects a continuation for
incomplete or uncertain work, and fails after three unsuccessful assessments.
OpenRouter and other unaffected chat providers remain unchanged.

Attempt 3 started on exact binary SHA-256
`08df7c13b4f82c72d3a4dbff68b41b99364869b4af9576d2724c3e48e1086438`.
The completion guard kept the model working after a premature text-only stop,
but the live ledger revealed that completed streamed rounds were being released
without charge when Ambient's transport raised an error after sending the
protocol completion marker. Gateway restarts during diagnosis also left two
reservations orphaned. The run was deliberately interrupted and is FAIL for
billing integrity; it does not count toward qualification.

## Remediation and attempt 4

Commit `69dcbe2b5` makes the streamed protocol completion marker authoritative
when the transport closes afterward. Commit `8a33a10f9` releases reservations
orphaned by a gateway process restart and restores both usage counters before
the server accepts traffic. The dedicated PostgreSQL suite passes 47/47, and a
live recovery restart reduced the fresh Basic account's reserved balance to
zero.

Attempt 4 started at 2026-07-19T05:47:56Z using the same exact TUI binary
SHA-256 `08df7c13b4f82c72d3a4dbff68b41b99364869b4af9576d2724c3e48e1086438`
and gateway tip `8a33a10f9`. Completed tool rounds immediately began settling
positive upstream usage; exactly one reservation remains open only while the
current round is active. The turn crossed context compaction twice and continued
substantive work. It then reached the Basic weekly limit cleanly at 4,916,922 /
5,000,000 tokens: the next reservation was refused before upstream work, the
TUI showed the limiting weekly window and reset date, reserved tokens returned
to zero, and no SOL/USDC moved. This is a PASS for limit enforcement and
long-context continuity but does not count as the final same-binary long-turn
qualification because the requested review could not finish.

## Final-binary attempt 5 — Pro plan

- Started: 2026-07-19T06:39:40Z
- Commit: `b9e1ad46be790c588f2d01b4fe62d0c558833e08`
- Binary SHA-256: `7ee40739147ecd8cbfec29047bb3924df6e3a8115c68a589355038175e32175b`
- Plan: Pro, 50,000,000 weekly / 200,000,000 monthly tokens
- Starting usage: 0 settled, 0 reserved
- Objective: read-only adversarial review of the complete wallet, daemon,
  payment, gateway, metering, recovery, and multi-process path, with extensive
  repository tools and a prioritized evidence-backed report

The run crossed two context compactions, streamed visible progress, completed
129 settled rounds for 6,318,731 authoritative tokens, and left zero reserved
tokens. It returned a substantive evidence-backed report and accepted a
follow-up correction without losing the conversation. No SOL/USDC moved.

This attempt is nevertheless FAIL for final qualification: the initial turn
ended at 8m11s despite the explicit 10-minute minimum. The free-form review also
found that a daemon accepting a request without replying had no bounded client
timeout and that the UI advertised `ready to sign` while a signing operation
had checked out the wallet. Commit `f0ec8d266` fixes both boundaries and adds
network-client deadlines, invalidating this binary. Qualification restarts on
the rebuilt executable.

## Final-binary attempt 6 — clean Pro account

- Started: 2026-07-19T07:35:38Z
- Commit: `341064f0c996c334bddc084473588ca7ee9c1a3a`
- Binary SHA-256: `8ef6087d5079793f14f1b3a37bf9ecfc3c5e067ae7c33fcb7d7e1200aa6942e0`
- Plan: Pro on clean wallet `4RBjjHA…hNxSGn`
- Starting usage: 0 settled, 0 reserved
- Objective: independent read-only audit of the complete wallet and gateway
  implementation against the master specification, including tests and an
  evidence-mapped ship verdict

Verdict: IN PROGRESS.

## Exact-binary attempt 7 — ten-minute watcher completed, host loop rejected

- Started: 2026-07-19T09:45:46Z
- Commit: `830b346a6f09db343c8b84e81a5d4b6a7b62c0ca`
- Binary SHA-256:
  `c8477ac135a3405f75d8e42a3b6117059c3c1864a117dc267c7e5645812d4288`
- Isolated home: `/tmp/pft-wallet-qual03-home`
- Starting usage: 41,758,638 settled, 0 reserved
- Ending usage: 46,876,123 settled, 0 reserved

## Renewed same-binary attempt — PASS

- Started: 2026-07-19T13:24:51Z
- Commit: `ae8de0e3481dc0f740ac4ba6742bfb09c42b683b`
- Binary SHA-256: `18040eeb36143f027a8cd7f8e8aa988d62862623b45b5c5b1fd4e5c56b109f6b`
- Wallet-daemon SHA-256: `337bf8ff83b444183b5631247544b2411b7f10ea2283ee555010716d8a03f39c`
- Starting usage: 0 settled, 0 reserved
- Ending usage: 4,662,673 settled, 0 reserved
- Terminal: tmux `pft_wallet_renewed`, 94×57

The paid Pro provider completed more than eleven minutes of free-form read-only
architecture, security, reliability, and user-workflow review. It inspected the
wallet, daemon, TUI, credential, gateway, PostgreSQL, metering, provider,
migration, and specification boundaries with repeated repository tools and
non-mutating tests. The response delivered the requested architecture map,
user-story and invariant mapping, verified guarantees, ranked risks, missing
adversarial tests, and production recommendation. A bounded completion-guard
continuation performed one additional compaction/provider evidence check and
then terminated normally.

The required follow-up corrected an over-severe classification: the pre-existing
untracked snapshot candidates are worktree hygiene, not a product P0, because
the external watcher observed the same git-state hash throughout. The model
stated that no new P0/P1 product defect was found beyond the known
post-broadcast cancellation ambiguity and incomplete same-binary matrix. It
did identify a direct tampered-intent test gap, now recorded as qualification
debt in the master spec.

External samples kept `/healthz` and `/readyz` green, wallet-daemon RSS stable,
the TUI responsive, and the worktree-state hash unchanged. No wallet unlock,
SOL movement, USDC movement, source edit, retry flood, premature progress-only
stop, orphaned reservation, or duplicate final answer occurred.

Renewed verdict: **PASS** for Run 3 and its follow-up on the exact binary pair.

The requested monitor ran from 09:47:48Z through 09:58:00Z: 21 samples
over 612 seconds. Every `/healthz` and `/readyz` sample returned 200 in 0–1ms;
gateway RSS remained 92–98MB, wallet-daemon RSS remained 24MB, and transient
reservations settled. Final database checks found zero stuck reservations,
overcharges, or over-limit windows. The model completed the requested source
audit and returned an evidence-backed final report.

The run is nevertheless FAIL. After valid final answers, the completion guard
injected the same developer correction seven times. Each corrected continuation
used tools, and `observe_sampling` reset the guard's unsuccessful-assessment
counter, so its nominal three-attempt limit could never be reached. The loop
settled 100 requests for 5,117,485 tokens and was about to launch a redundant
second ten-minute watcher when the external driver terminated the isolated
session. Reserved usage returned to zero and no SOL/USDC moved.

WTURN-003 repairs the control-plane boundary and invalidates this binary. The
ten-minute watcher evidence remains useful defect evidence but does not count
toward the final same-binary seven-run matrix.

## Exact-binary attempt 8 — bounded completion replay

- Started: 2026-07-19T10:12:59Z
- Commit: `fe155ab96b708c5c1f8dd7f3a7d62c709eb5d327`
- Binary SHA-256:
  `a2bbea089985051f35189a95d81006dfb26fc95bb761f10b0c5b841834849625`
- Isolated home: `/tmp/pft-wallet-qual04-home`
- Starting usage: 46,876,123 settled, 0 reserved
- Ending usage after follow-up: 47,157,070 settled, 0 reserved

The first provider response took 112 seconds, remained visibly cancellable,
and settled once. The model then ran one foreground sampler from 10:14:31Z
through 10:24:32Z. All ten one-minute samples returned HTTP 200 from
`/healthz` and `/readyz`, with gateway PID 2151893 unchanged. The command
budget deliberately prevented the model from authenticating a later account
query, while the external watcher reconciled exact settled and reserved
counters directly from PostgreSQL.

The completion classifier rejected the model's explicit blocker three times.
The repaired guard retained its attempt count across the turn, displayed the
bounded-review warning, accepted the latest response, and returned input to the
user after 12m19s. It did not error or start another monitor. A direct follow-up
received the correct PID and exact sampling interval immediately. Total paid
usage was 280,947 tokens, reserved usage returned to zero, source files and
pre-existing artifacts were unchanged, and no SOL/USDC moved.

Verdict: PASS for US-4 and US-5 on the exact qualification binary, including
long-turn stability, real paid settlement, bounded semantic continuation, and
follow-up continuity.

## Final same-binary candidate — PASS

- Started: 2026-07-19T14:18:48Z
- Completed follow-up: 2026-07-19T14:33Z
- Commit: `c0bbab5727fa8268d40be727959702165f449677`
- Binary SHA-256:
  `1b70e4ed3a51a52bdab833d2f4d72fc91d96b96f6ce93eea6c70e3f66dc63b94`
- Wallet-daemon SHA-256:
  `337bf8ff83b444183b5631247544b2411b7f10ea2283ee555010716d8a03f39c`
- Isolated home: `/tmp/pft-wallet-final-run3-home`
- Terminal: isolated tmux socket `pftwalletfinal3`, 94×57
- Starting usage: 4,795,298 settled, 0 reserved
- Ending usage: 8,997,449 settled, 0 reserved
- Ledger dispositions during run: 84 settled, 0 released, 0 reserved

The exact candidate completed a thirteen-minute read-only architecture and
adversarial audit of `postfiatl1v2`, including consensus, block production,
validation, storage, networking, NAVCoin proofs, and tests. It used repeated
repository tools, returned an evidence-backed report with three concrete
specification/implementation mismatches, and then answered one bounded
production-call-path follow-up without starting another watcher or duplicating
the final answer.

An internal sampler observed `/healthz` and `/readyz` throughout an 11m43s
window; every request returned HTTP 200 in approximately 0.8–1.7ms. The
external watcher independently kept the gateway PID stable, observed transient
reservations settle, and recorded the same empty worktree-state hash throughout.
No source file, SOL balance, USDC balance, wallet lock state, plan selection, or
provider selection changed.

One qualification-harness diagnostic remains: a model-initiated shell command
resolved `pfterminal` from `PATH` to the older installed release, which did not
know the `pfterminal-plan` provider. The already-running exact debug candidate
and paid inference route remained healthy, and the command was not needed for
the objective, so this does not invalidate Run 3. It is recorded in the master
spec as a packaging/command-path risk that must be closed before release.

Final-candidate verdict: **PASS** for Run 3 / US-4 and US-5.
