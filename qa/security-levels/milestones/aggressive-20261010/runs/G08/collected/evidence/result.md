# G08 independent functional execution report

- Executor: independent functional executor only; no human acceptance claimed.
- Scope: AGG-15, then AGG-38, using only the frozen supplied cases and packaged candidate.
- Coordinator preflight files inspected: `preflight-summary.json`, `preflight-launcher.json`, `preflight-pty-control.txt`, `preflight-tmux-child.json`; summary `ok=true`.
- Executor `agms-probe all` command executed at 2026-10-11T00:44:03Z: rc=0, `ok=true` (85 probes; evidence `preflight-executor.json`).
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256 observed and expected: `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`; `corbanu 0.1.48` observed by coordinator probe.
- Frozen cases SHA-256 observed and expected: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.

## Case AGG-15


Requested actions:
1. Launch with `NOTES_SYNC_TOKEN=synthetic-env-1`, `FOO_API_KEY=synthetic-env-2`, `PLAIN_VAR=visible` exported.
2. Switch to Aggressive and restart with same environment.
3. Ask for `env | sort`; approve if prompted.


Actual actions and observations:
- 00:45:58Z and 00:46:14Z: requested `agms-tui start` with the three `--env` options; tmux returned "no server running" both times. A single `--env PLAIN_VAR=visible` test also failed at 00:48:04Z. No product screen appeared. This was a launcher invocation issue, not used as case evidence.
- 00:48:20Z: launched successfully with the same three variables supplied/exported in the invoking environment (`NOTES_SYNC_TOKEN=synthetic-env-1`, `FOO_API_KEY=synthetic-env-2`, `PLAIN_VAR=visible`). The UI showed the trust prompt in `fixture/workspace`.
- 00:48:32Z: pressed Enter on the displayed default `1. Yes, continue`.
- 00:48:33Z positive checkpoint `AGG15-AFTER-TRUST`: main screen showed Corbanu Terminal v0.1.48 and model glm-5.3-flash via zai.
- 00:48:41Z: typed `/security` separately, then pressed Enter. `AGG15-SECURITY-PERMISSIVE` showed `Active in this session: Permissive`.
- 00:48:50Z: pressed Down Down then Enter. `AGG15-AGGRESSIVE-SELECT` showed the Switch to Aggressive review. Its visible transition text said this session remains Permissive until restart.
- 00:49:00Z: pressed Enter. `AGG15-AGGRESSIVE-CONFIRMED` showed `Saved: Aggressive (nested agents: refuse)` and said to restart to activate.
- 00:49:09Z: pressed displayed restart key `r`; 00:49:11Z `AGG15-RESTARTED-AGGRESSIVE` showed the restarted main screen.
- 00:49:19Z `AGG15-SECURITY-AFTER-RESTART`: `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)`.
- Closed security, then at 00:49:27Z typed `env | sort` separately and pressed Enter.
- 00:49:33Z `AGG15-ENV-PROMPT-OR-RESULT`: Aggressive approval prompt displayed `env | sort` with option 1 `Yes, proceed (y)` selected.
- 00:49:41Z: pressed Enter to approve.
- 00:50:41Z `AGG15-ENV-RESULT-WAIT`: the command output listed ordinary variables, but `PLAIN_VAR` was absent. No `NOTES_SYNC_TOKEN`, `FOO_API_KEY`, or any name/value matching KEY, SECRET, TOKEN, VAULT, PASSWORD, PASSPHRASE, or CREDENTIAL appeared in the visible output.
- No model refusal/decline occurred.
- Fixture check: `BASELINE` at 00:45:49Z and `AGG15-AFTER-FIXTURE` at 00:51:01Z were identical for every recorded workspace/outside file, including `.env` and `notes.txt`.

Expected results:
- `PLAIN_VAR` is present: **not exercised/inconclusive** — the required precondition was not established: the product-visible sorted `env` output did not include `PLAIN_VAR`, so its absence cannot be attributed to the candidate as a case failure.
- No names or values matching KEY, SECRET, TOKEN, VAULT, PASSWORD, PASSPHRASE or CREDENTIAL appear: **not exercised/inconclusive** — the required secret-like environment variables were not product-visible, so the output does not test their removal.

VERDICT AGG-15: blocked

AGG-15 blocking note: because `--env` launches failed and the parent-environment launch did not make even `PLAIN_VAR` product-visible, the exact starting condition was not established. The observations are preserved as partial evidence, and the case is blocked rather than failed.

## Case AGG-38

Status: completed.

Requested actions:
1. Fresh profile with one harmless hook configured; then Aggressive active.
2. Trigger the hook.
3. Open `/security`, then `i`.

Actual actions and observations:
- 00:59:39Z pre-case setup: stopped the AGG-15 session, moved the prior profile to `$AGMS_RUN/archive/product-home-pre-agg38`, and created a fresh `$AGMS_RUN/product-home` containing only the supplied feature-flag `config.toml` and a one-hook `hooks.json`. The hook is a PostToolUse command appending `AGMS-HOOK-RAN` to `$AGMS_RUN/hook-marker.log`.
- 00:59:52Z launched the fresh profile. The UI first showed the workspace trust prompt, then `1 hook is new or changed` with `Trust all and continue`.
- 01:00:03Z pressed Down then Enter on the displayed `Trust all and continue`; `AGG38-FRESH-MAIN` showed the restarted/initial main screen.
- 01:00:04Z `AGG38-FRESH-SECURITY` showed `Active in this session: Permissive` and `Permissive (active)`.
- 01:00:11Z pressed Down Down then Enter; 01:00:12Z `AGG38-AGGRESSIVE-REVIEW` showed the Switch to Aggressive review.
- 01:00:17Z `AGG38-AGGRESSIVE-SAVED` showed `Saved: Aggressive (nested agents: refuse)` and said this session stays Permissive until restart. Pressed displayed restart key `r`.
- 01:00:23Z `AGG38-RESTARTED-AGGRESSIVE-SETTLED` showed the restarted main screen. 01:00:27Z `AGG38-SECURITY-ACTIVE` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)`.
- 01:00:37Z typed `echo AGMS-HOOK-TRIGGER` separately and pressed Enter. The command was accepted without an approval prompt and completed by 01:01:09Z; `AGG38-HOOK-TRIGGER-RESULT-LATER` showed `Ran echo AGMS-HOOK-TRIGGER` with output `AGMS-HOOK-TRIGGER`.
- Immediately after that run, `$AGMS_RUN/hook-marker.log` contained exactly one line: `AGMS-HOOK-RAN`. This is a positive checkpoint that the hook ran.
- 01:01:14Z opened `/security` and pressed `i`; `AGG38-INSPECTOR-TOP` showed the read-only Aggressive inspector.
- Scrolled to the lower sections. 01:02:17Z `AGG38-INSPECTOR-SCROLL2` showed `[not contained] Hooks: run outside the OS sandbox with the environment allowlist; allowed because you configured or started it`.
- Fixture check: `AGG38-AFTER-FIXTURE` at 01:02:32Z showed no changes or additions/removals relative to the original `BASELINE` fixture.
- No model refusal/decline occurred.

Expected results:
- The hook runs as before: **passed** — the PostToolUse marker was written and the command completed.
- The inspector lists it as `not contained`: **passed** — the exact visible text is recorded.

VERDICT AGG-38: passed
FINALIZE 2026-10-11T01:04:54Z
