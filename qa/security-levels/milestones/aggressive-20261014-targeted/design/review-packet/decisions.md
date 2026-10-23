# Owner decision comments and the decided behaviour (verbatim)

## #447: qa: milestone "Aggressive ships" code-blind VM run record (not passed; sign-off requested)

### IridiumMaster 2026-10-14T07:04:57Z

Owner decision (Travis, 2026-10-14): third run: hold sign-off until #736 is assessed and #737 fixed (both done in #752), then a targeted re-run of the affected cases rather than a full fourth run. Amend AGG-01 and AGG-25 to today's behaviour (#440/#438 decisions). Model-decline rule accepted for Aggressive and Moderate: a case passes when the model declines a step but the product's text still meets the expected wording; blocked otherwise. Amend AGG-R10 to use rg. The sixth evidence check is accepted as happened, but the rule going forward: going over the review budget is allowed only for a major defect AND with explicit authorization from Travis (recorded); workers may not self-authorize.

## #662: Aggressive: approval for every command from the next turn after confirmation (#505)

### IridiumMaster 2026-10-13T22:14:12Z

Owner decision (Travis, 2026-10-13): approved in principle; finish after the third Aggressive milestone run (fix: managed reviewer override; update each thread's stored settings at confirmation), then final review before merge.

### IridiumMaster 2026-10-14T07:04:01Z

Owner decision (Travis, 2026-10-14): security sign-off — merge. Keep the file-size allowlist entry for now (split later); the two wording points are follow-ups.

## #505: Aggressive: apply restart-only protections at confirmation

### IridiumMaster 2026-10-11T20:06:28Z

Owner decision (Travis, 2026-10-11): restart-only protections (sandbox, approval for every command, network off, environment stripping, model-key broker, child-agent rules) **stay applied at restart** for the "Aggressive ships" milestone. No code change; this issue records what applying each one at confirmation would take.

### IridiumMaster 2026-10-13T17:10:33Z

Owner decision (Travis, 2026-10-13): option B — the third Aggressive milestone run keeps the restart-only list as designed; #662 (approvals at confirmation) is finished after that run.

### IridiumMaster 2026-10-14T04:43:41Z

agg-505 worker, 2026-10-13: **#662 is finished and ready for review.** Its head is 8c9168a1a8, all 31 PR checks pass, and the title still ends "— needs security sign-off". It is **not merged**: Travis gives the final look.

**What it does:**
- **Approval for every command.** When a person confirms Aggressive in `/security`, every Corbanu pane and child agent of this Corbanu Terminal requires a person's approval for every command that isn't known read-only, from its next turn. Panes started later in the same process, before the restart, do too.
- **Stored settings.** The approval change goes into each thread's stored settings at confirmation, not into a per-turn override. Steering, child-agent messages, MCP approvals and `/status` therefore all see it, and a later settings update can't lower it before the restart.
- **Managed requirements win.** If a managed approval-policy or reviewer requirement rules this out, that thread keeps its own settings and is told.
- **Unchanged:** the sandbox, network, environment, credential broker and child-agent rules stay restart-only, and screen 07 says so. Permissive is unchanged.

**Evidence:**
- **Fail-before/pass-after tests** (`core/tests/suite/issue_505.rs` plus unit and picker tests). These fail against the round-1 head: the approval test, steering during a later turn, a child-agent message during a later turn, running children and later threads, and the managed reviewer. Against main, the approval and children tests fail. All pass at the head.
- **Focused `just test`** (core and tui, TMPDIR=/tmp): 1682 of 1682 pass without developer-accounting, and 1682 of 1682 with it.
- **Clippy `-D warnings`:**
  - macOS: clean, plain and with developer-accounting.
  - Linux (glitch): clean, plain and with developer-accounting.
  - Windows (`rust-ci-full` run 38507299010): clean.
  - The Linux and Windows runs were on 5680e2b420, whose codex-rs is identical to the head.
- **Independent review (Opus 5.5 High):** APPROVE in r2, r3 and r4. r4 is on the exact head. The remaining nits are listed in the PR.
- **tmux check** (disposable home, GLM 5.3 Flash on zai), starting with approvals off:
  - After confirming, the agent's write command waits for approval.
  - A `/new` thread shows "untrusted" and asks as well.
- **Video:** https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-505-issue505-approvals-at-confirmation-5549054ff93f-2026-10-13.mp4

**CI note:** `core/src/session/tests.rs` is already over the blob-size limit on main (512109 bytes, since #687). #662 adds it to the blob-size allowlist; the file should be split later.

## #440: Aggressive (AGG-15): ordinary environment variables are removed too; review says only secret-like ones

### IridiumMaster 2026-10-11T12:14:03Z

Fixed by #458. Behaviour is unchanged: the PF-27-S02 allowlist stays. The text now matches it.

- **Review row:** the Aggressive review's Vault row now says that with `secretless_agent_launch` on, every other variable outside an allowlist is removed too (for example PATH, HOME, locale, terminal, proxy and tool locations). It also says others can be set explicitly in `shell_environment_policy.set`.
- **Inspector:** the inspector says that only allowlisted environment variables reach agent commands.
- **Before and after (tmux, GLM 5.3 Flash):** before, the row said only secret-like variables are removed; after, it states the allowlist.
  - Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-440-issue440-review-says-allowlist-abf6c06251ca-2026-10-11.mp4
- **Test:** `issue_440_vault_row_says_the_environment_is_allowlisted_under_secretless_launch` checks the row and checks that the launch removes `PLAIN_VAR`.
- **Open for Travis:** to let ordinary variables pass instead, the allowlist would have to become a denylist. That would weaken PF-27-S02, so it needs your decision.

### IridiumMaster 2026-10-11T19:55:31Z

Owner decision (Travis, 2026-10-11): keep the agent environment allowlist (PF-27-S02 design; wording fixed in #458). No change.

## #438: Aggressive preflight (AGG-25/E1): -c cli_auth_credentials_store="file" flagged as an unmanaged secret and blocks Aggressive

### IridiumMaster 2026-10-11T11:55:28Z

Fixed by #448 and #457 (both merged). The preflight no longer treats the top-level `cli_auth_credentials_store`, `mcp_oauth_credentials_store` and `model_auto_compact_token_limit_scope` settings as secrets when they hold one of their enum values; the same name under another table (e.g. `shell_environment_policy.set.CLI_AUTH_CREDENTIALS_STORE`) or a non-choice value still blocks (independent-review finding, fixed in #457).

Evidence:
- Tests: `pf_29_s01_credential_store_modes_are_not_secrets` (fails on the old code), plus the nested/non-choice cases.
- tmux, GLM 5.3 Flash, disposable home, `-c 'cli_auth_credentials_store="file"'` (AGG-25 steps): main showed `Preflight blocked … launch flags cli_auth_credentials_store: unmanaged secret`; the fix shows `Preflight passed` and saves Aggressive.
- Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agg-438-agg-438-store-mode-preflight-cdb62e96c6d7-2026-10-11.mp4
- Linux clippy -D warnings (plain and developer-accounting) clean on glitch; independent review (Opus 5.5 High) of #457: approve.
