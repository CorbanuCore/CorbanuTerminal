All four follow-ups are merged into main as separate PRs. The trace-log check found a real leak, and I fixed it. I didn't touch the Aggressive sandbox design, and nothing needs a decision from Travis.

## What shipped

| Item | PR | Status |
|---|---|---|
| 1. Provider key in the trace log | #201 (closes #196) | merged |
| 2a. `/security` description | #200 | merged |
| 2b. Review screen shows current values (B-06) | #202 | merged |
| 2c. Launch check loads the full exec policy | #199 | merged |

**1. Provider key in the log.**
- **The picker's finding was already fixed.** Every PF-24-S03 run that found the key used a build from before #183. Its final runs on #183 code were clean, and so was my replay on main with a fake key.
- **A new regression test found a different leak.** The "Configuring session" DEBUG line printed the whole provider definition, including any bearer token, static header values and query values. Request URLs also logged query values (`?key=…`).
- **Fix:**
  - The provider definition's debug output now hides those values.
  - Request URLs and errors in the logs show query values and logins as `REDACTED`.
  - Auth header values are marked sensitive.
- **Impact before the fix:** these values reached the log database and the `/feedback` buffer at any log level. Keys read from an environment variable, like `ZAI_API_KEY`, were not affected.
- **Issue:** I filed #196 with the security label.

**2a. Description.** With the flag on (or a level stored), the popup now says "choose a security level; takes effect when you restart". Otherwise it keeps the read-only text.

**2b. Current values.** Each row of the Aggressive review now shows what this session has, above the Aggressive value.
- **Accuracy fixes from the review:** the first review asked for changes because rows could claim protection that isn't in force. The rows now say plainly that:
  - approved commands can run outside the sandbox;
  - network is on for those commands;
  - an approved permission request can widen network or vault access.
- **Scrolling:** the review now scrolls with ↑/↓. At 80x24 the "Unchanged" and restart notes were cut off before.
- **Behaviour change:** Aggressive now also refuses to start with custom roles that set `sandbox_workspace_write` or `profile`.

**2c. Exec policy.** Launch now loads every `.rules` file the way a session does. It refuses to start if one doesn't parse, if `corbanu vault` isn't forbidden, or if a `host_executable` rule narrows a vault program.
- **Before the fix (tmux run on main):** a broken project rules file still showed "Aggressive active", and `corbanu vault list` ran after an ordinary approval.
- **Dependency:** the TUI now depends on `codex-execpolicy`, which adds one `Cargo.lock` entry. Updating the Bazel lockfile produced no change, as the PR says.

## Gate evidence
- **Tests:** focused `just test` runs, after `just fmt` and `just fix`, all passed:
  - #201: 458 + 10;
  - #200: 218;
  - #202: 37;
  - #199: 34.

  A local merge of all four with current main passed 247 TUI/http-client tests. CI was green on every PR. I didn't run the full `codex-tui` suite.
- **tmux runs on GLM 5.2:** before/after captures for each fix are under `qa/security-levels/pf24-followups/`.
- **Opus 5.5 High reviews:**
  - #199 approve with fixes;
  - #200 approve;
  - #201 approve with fixes;
  - #202 request changes, then approve with fixes on a second pass.

  All findings are fixed or listed in each PR's disposition file. The fixes for #201 went in without a second review.
- **Videos:** published to the `demos` prerelease and linked in each PR:
  - `issue-196-provider-key-not-logged`
  - `pf24-followups-desc-pf24-security-description`
  - `pf24-followups-review-pf24-review-shows-current`
  - `pf24-followups-pf24-broken-rules-refused`

## Still open
- **#197:** a `.rules` file broken *after* launch still drops the vault rule for threads started later. Agent commands can't write those files under Aggressive, so only the human or another process could cause this. The recommended fix is a core setting that makes rule parse errors fatal under Aggressive.
- **Remaining from #183:** old leaked rows in users' log databases, and response-header (`set-cookie`) logging.
- **Not covered by #201:** user-visible error messages that include the configured URL.
- **Known limit, unchanged:** prefix rules don't match `./corbanu` or `env corbanu`. The unreadable vault store still blocks those.