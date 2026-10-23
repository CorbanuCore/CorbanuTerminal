# PF-84-S03 independent source-blind acceptance, plus a re-check of S02-3b, S02-9b and S02-14 (2026-10-10)

Scope:
- [PF-84-S03](../../../../docs/sprints/current/unified-provider-auth/pf-84-s03-account-selection-and-propagation.md):
  choosing an account per session, with `corbanu exec --account`, for spawned agents, and on resume. Behind the
  default-off `named_accounts` flag.
- A re-run of the three PF-84-S02 criteria that [failed on 2026-10-10](../independent-acceptance-20261010/README.md):
  S02-3b, S02-9b and S02-14. Their issues #414, #415 and #416 were fixed by PR #422.

Gate under test: [s03-gate.md](../s03-gate.md). Independent review: [REVIEW.md](REVIEW.md).

## Method

- **Source-blind, not the milestone code-blind VM gate.** I read only:
  - the S02/S03 sprint records and the plan's PF-84 section;
  - `s02-gate.md` and `s03-gate.md`;
  - the previous acceptance's README and REVIEW, plus its scripts for the three failed criteria (to repeat them exactly);
  - the PF-84 demo indexes, `docs/authentication.md`, `docs/config.md` (custom providers, `[security] level`),
    `docs/features/{subagents,spawn-orchestration}.md`, `docs/tmuxHarness.md`, `sec-common.md`, and `--help` output.

  I did not read `codex-rs/**`, `scripts/**`, PR diffs, the #422 fix evidence (`acceptance-fixes-20261010/`),
  worker logs or reviews. I ran a copy of the checked-in `scripts/dev/corbanu-launcher.sh` without reading it.
  There was no separate designer and no handoff-checker run, so this does not tick the sprints'
  "code-blind handoff checker" box.
- **Candidate.** `origin/main` at `ee530a0e26e21dac65e0bb9d946e5549c0892806` (merge of #422). Built as a macOS arm64
  debug build (`cargo build --locked --offline -p codex-cli --bin corbanu`, toolchain 1.95.0, its own target dir).
  This is not the packaged release.
- **Baseline.** A pre-PF-84 build of `44b527f11f`, built the same way. It was used for the version-skew re-check
  and to create the "existing single-account" homes.
- **Safety.**
  - Every candidate and baseline process ran with `env -i`, a fake `HOME`, a disposable `CORBANU_HOME` and
    `CORBANU_TEST_NO_NATIVE_KEYRING=1`. The vault key went to `secrets/keyring-fallback/`.
  - Real keys came from the operator vault through the installed signed backup binary
    (`X="$(… corbanu.bak-20261010 vault auth-helper <label>)" <consumer>`), only inside the consuming command. They
    were never put on a command line or printed.
  - The canary proxy fetches its key inside its own command ([proxy-run.sh](scripts/s03/proxy-run.sh)).
  - The pre-PF-84 onboarding paste used a tmux buffer that was deleted on paste.
  - A `security(1)` PATH shim recorded 0 calls during the Claude runs.
  - The real `~/.local/bin/corbanu` and the real home were never used.
- **Providers.** Z.AI `glm-5.3-flash` (`provider/zai_api_key`), Kimi `kimi-k3` on `kimi-code`
  (`provider/kimi_api_key`), OpenAI `gpt-5.4` (`openai-api-key`, API-key login) and Claude Plan
  `claude-opus-5-5-plan` (`claude-plan-test-token`).
- **Per-account attribution.**
  - **Canary proxy.** [canary_proxy.py](scripts/canary_proxy.py) is a local proxy for a custom chat provider
    `zpx`. Each account in the test homes holds a synthetic canary, so the proxy can name the account behind every
    request (default, main, work, fake …). It forwards default/main/work to the real Z.AI with the real key and
    answers 401 for every other canary. It also records whether any canary or the real key appears in a request
    body, which is what the model sees.
  - **Spawn tests.** Custom chat providers get no spawn tool, so in-process spawn ran on the built-in `zai`. In
    home hS the `default` account is a synthetic invalid key, while `main` and `work` hold the real key and `fake`
    a synthetic one. Any silent fallback to `default` therefore shows up as a 401.
  - **The `fake` override.** To prove that a `fake` child really used `fake`, SP3 and SP7 were re-run with a
    *valid* default and a valid parent account. A 401 then has no other source.
  - **Limit.** I have one Z.AI key, so `main` and `work` share it. Which valid account an in-process child used
    (for example `work` rather than the parent's `main`) is shown only by the thread's recorded
    `provider_account` in the rollout ([rollout_view.py](scripts/rollout_view.py)). The proxy proves it for the
    out-of-process and TUI-worker paths.
  - **Brokered requests.** Later runs in hS went through the credential broker (errors show
    `http://api.z.ai:443`). That came from a security-level state I had created by hand (see
    [D3 states](captures/s03-tui-d3-security-states.txt)). It does not change which account was used.
  - **`auth.command` provider.** A provider `zcmd` gets its credential from a mock
    ([authcmd.sh](scripts/s03/authcmd.sh)) that logs `CORBANU_PROVIDER_ACCOUNT` and prints that account's canary.
- **TUI.** Driven through a private tmux socket, following `docs/tmuxHarness.md`
  ([tmuxlib.sh](scripts/tmuxlib.sh)). Text and Enter are sent separately, every wait needs two identical
  captures, and each session exits with `/exit`. The step-by-step runs are listed in
  [tui-interactive-steps.md](scripts/s03/tui-interactive-steps.md).

## Verdicts: PF-84-S03

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| S03-1 | The thread config carries the account. Spawned children, including role children and TUI `/spawn` workers, inherit it unless overridden. | PASS | [spawn matrix](captures/s03-spawn-exec-matrix.txt). SP1: the coordinator is on `--account main` and the child is spawned without `account`. The child records `provider_account=main` and answers `child-ok`, although the default account is invalid. SP6: an `explorer` role child inherits `main`. SP8: an account chosen by `[provider_accounts]` is inherited (`work`). Which valid account the child used comes from the rollout (see Method). Nazgul and Orc role children were not run. TUI `/spawn troll` worker under `corbanu --account work`: the proxy saw only the `work` bearer, and `auth.command` was asked for `work` ([preflight](captures/s03-tui-worker-preflight.txt)). |
| S03-1b | App-server `thread/spawnAgent` workers inherit the parent's account. | NOT VERIFIABLE | There is no user-facing surface I could identify as `thread/spawnAgent` without reading source. The TUI `/spawn` worker above is the closest observable path. `thread/start` and `thread/resume` are exercised through `exec`, whose errors come from `thread/start` and `thread/resume`. |
| S03-2a | Spawn `account` accepts only configured names of the child's provider. | PASS | SP4 `ghost` and SP5 `kimionly` (configured for `kimi-code` only) are refused before any child exists. The refusal lists the configured `zai` accounts and says not to retry on another account without consent. |
| S03-2b | The spawn result shows `account`. | PASS | The tool result is `{"agent_id":…,"nickname":…,"account":"work"}` (SP2, SP3, SP7, TUI), matching the child's recorded `provider_account`. With the flag off there is no `account` field (SP9). The TUI spawn cell does not render the account. That is a documented known limit, so the plan row "Spawn event shows `work`" is only met in the tool result. |
| S03-2c | D3: under Aggressive (the level in force), choosing an account needs the human's approval. | PASS | [TUI](captures/s03-tui-spawn-aggressive-d3.txt) (`security_level=aggressive`, `approval_policy=on-request`): "Allow the spawned agent to run on account `work` of `zai` instead of the parent's account?". **Allow once** led to a child on `work` answering `child-ok`. **Cancel** led to "The user declined …" with no child. An inherited account and an explicit `account` equal to the parent's are not prompted (SP11, SP12). Those children were created, but their `child-ok` replies are not proof that they finished, because the wait tool was gated. Permissive gets no prompt ([TUI](captures/s03-tui-spawn-permissive.txt), SP13). An unreadable or missing stored level counts as Aggressive and asks ([states](captures/s03-tui-d3-security-states.txt): sec6, sec7). One gap: in the "Aggressive enforced, boundary unverified" state (preflight features off), the TUI says Aggressive, but core records Permissive and the switch is not gated (sec1, sec3). This needs a product decision: [#428](https://github.com/CorbanuCore/CorbanuTerminal/issues/428). |
| S03-2d | D3: refused when approvals are off. | PASS | [exec under Aggressive](captures/s03-spawn-exec-d3-config-level.txt) (`approval_policy=never`), SP10: "… needs the user's approval under the Aggressive security level, and approvals are off." No child was created. The level was set with `[security] level`, as the docs describe. Writing only `security_level.toml` left the level at Permissive ([note](captures/s03-spawn-exec-d3.txt)). |
| S03-3a | `--account` works on exec, the TUI and resume. | PASS | Real Z.AI exec X1–X10 ([zai](captures/s03-exec-zai-matrix.txt)). Kimi K1–K7 ([kimi](captures/s03-exec-kimi-matrix.txt)). Claude Plan `real` pong, `fake` 401 ([claude](captures/s02-recheck-claude-414-415.txt)). `cfgacct` (a Claude Code login folder with a synthetic token) gets its own token, then fails closed after 5 reconnects and 27 token-command calls: "Claude Code OAuth refresh token is missing. Run `claude /login` again". No real login folder was tested, so that kind has no positive proof. TUI `--account main` pong and `fake` 401 ([real](captures/s03-tui-account-real-zai.txt)). TUI `--account work` and `fake` were canary-attributed ([proxy](captures/s03-tui-proxy.txt)). `corbanu resume --account main` and `exec resume --account main` also work. |
| S03-3b | Missing accounts are refused at thread start. | **FAIL** (low–medium) | Exec, exec resume, spawn, the out-of-process worker and the TUI worker preflight all refuse with recovery text, and no request is sent. Two gaps: **(1)** `--account <other-provider>:<missing>` is accepted silently, and the session runs on the default account of its own provider (X12, K9, K12): [#425](https://github.com/CorbanuCore/CorbanuTerminal/issues/425). **(2)** TUI `--account ghost` (also `corbanu resume --account ghost`, run t4d) opens a default-key onboarding loop that never names the account and never reaches the chat. Nothing is saved, and on `zpx` the proxy saw 0 requests (run p3) ([ghost](captures/s03-tui-missing-account-ghost.txt)): [#426](https://github.com/CorbanuCore/CorbanuTerminal/issues/426). |
| S03-4 | Resume uses the recorded account, and a removed account blocks with recovery text. | PASS | Exec: R1 (recorded `fake`, 401 while env and default are valid), R3 (recorded `main` beats config `fake`), R4 (a config-chosen `fake` is recorded), K5/K7 on Kimi, and the Claude Plan resume of `fake` (401). R9: after `account remove zai gone`, resume fails with "account `gone` … is not configured; add it with …". R10: `--account main` recovers it ([resume](captures/s03-resume-zai.txt)). TUI resume, canary-attributed: recorded `fake` gives a `fake` 401, recorded `work` gives `work` 200, and `--account main` gives `main` 200 ([proxy](captures/s03-tui-proxy.txt)). Messaging gap: TUI resume shows the default-credential hint ([#427](https://github.com/CorbanuCore/CorbanuTerminal/issues/427), low). |
| S03-5 | The TUI worker preflight checks the worker's own account (the S02 known limit). | PASS | `/spawn troll` under `--account work` runs `auth.command` with `CORBANU_PROVIDER_ACCOUNT=work`. After `account remove zcmd work`, the next `/spawn troll` is refused with "Cannot run a native Corbanu Terminal worker on provider `zcmd`: account `work` … is not configured; add it with …", and the command runs 0 times ([preflight](captures/s03-tui-worker-preflight.txt)). |
| S03-6 | Precedence: explicit > session/recorded > `[provider_accounts]` > `default`, and no silent fallback. | PASS | X7 (config `fake`, `--account main`): pong. X8 (config `main`, `--account fake`): 401. X9: `--account default` beats config. X10: config alone. R3 and R5: the recorded account beats config. R6: a thread with nothing recorded falls through to config. SP8. |
| S03-7 | Worker on another account: a coordinator on `default` runs an out-of-process tmux worker with `corbanu exec --account …`. The worker uses its own account, the coordinator stays on `default`, and a failure never falls back. | PASS | [workers](captures/s03-out-of-process-workers.txt). The TUI coordinator (GLM 5.3 Flash through the proxy) ran the checked-in launcher. The proxy saw: coordinator `default` ×4; W1 `work` 200; W2 `fake` 401 (one request); W4 (resume, no flag) `fake` 401; W5 (resume `--account main`) `main` 200. W3 `ghost` was refused with 0 requests. W6 (no account) gets no env key, because the shell strips it. |
| S03-8 | Flag off: a named `--account` is an error, `--account default` works, and behaviour is otherwise unchanged. | PASS | X14/X17 and TUI t3b refuse with "named accounts need the `named_accounts` feature …". X15 and X16 pong. SP9: the model omitted `account` although asked, which suggests the tool does not offer it. The proxy could not check the schema, because custom providers get no spawn tool. The result has no `account` field, and no `provider_account` is recorded. M1/M8/O1 are unchanged. One note: a flag-off resume of a thread recorded on a named account (R7; the recorded account is `zai:main`, re-recorded by R2) fails closed with text that blames `--account` ([#427](https://github.com/CorbanuCore/CorbanuTerminal/issues/427)). |
| S03-9 | The model sees account names, never values, and no env var carries a credential to children. | PASS | [proxy summary](captures/s03-proxy-summary.txt): 0 of 23 request bodies contained any canary value or the real key. Worker env: `ZPX_API_KEY=unset`. The Claude token child gets `internal-claude-oauth-token --account <name> --enable named_accounts` plus `CORBANU_PROVIDER_ACCOUNT=<name>` (a name, not a value) ([argv log](captures/s02-recheck-claude-414-415.txt)). [Leak scan](captures/leak-scan.txt) of `$S` and this directory (the random `fake-*` keys went straight into the vault and cannot be searched for): the only real-key hit is the OpenAI key in `hO/auth.json`, where the pre-PF-84 `login --with-api-key` stores it today. Synthetic values appear only in my own fixture files. |
| S03-10 | Child isolation with canaries: B's failure never retries on A. | PASS | Proxy: W2 and TUI p2 each made exactly one `fake` request and none on another canary. SP3 and SP7 re-run with a valid default and the parent on the real `main` ([valid default](captures/s03-spawn-exec-fake-valid-default.txt)): the `fake` child (and the `explorer` role child) ends with a 401 and `completed: null`. With `main` and `default` both valid, only `fake` can produce that 401, and it did not fall back. |
| S03-11 | An existing single-account home carries over untouched. | PASS | [migration](captures/s03-migration-existing-homes.txt) ([how hM was made](captures/s03-migration-onboard-pre.txt)). Z.AI home hM was created and onboarded by the pre-PF-84 build. Across M1–M8 (flag off, flag on, `--account default`, `account list`, resuming pre-PF-84 exec and TUI sessions) the hashes of `config.toml`, `local.age` and the vault key file stay the same, and default answers pong. M4 and M7 (`main` not enrolled) are refused, not defaulted. M9: the pre build still resumes its session. OpenAI home hO: `auth.json` is unchanged, `gpt-5.4` pongs with the flag off and with `--account default`, and named OpenAI accounts are refused (descoped to S04). |
| S03-12 | Second provider, Claude Plan and the auth child. | PASS | Kimi K1–K7. Claude Plan: see S03-3a and S02-3b. The token child receives `--account` as an argument. |

Not verdicts: there is no user-facing doc for named accounts yet (`docs/provider-accounts.md` is in S03's write
scope but does not exist). `--help` matches what I observed, except for #425.

## Verdicts: PF-84-S02 re-check (the same steps as 2026-10-10)

| # | Criterion | Then | Now | Evidence |
| --- | --- | --- | --- | --- |
| S02-3b | Claude Plan resolver: per-account token, never a fallback. | FAIL (#414) | **PASS** | [claude](captures/s02-recheck-claude-414-415.txt). The session selects account `fake`, and `CLAUDE_CODE_OAUTH_TOKEN` holds the real default token. With the candidate first on PATH: 401. With the pre-PF-84 build first on PATH: "provider auth command `corbanu` predates named accounts, so account `fake` was refused instead of using the default account; update it or put a current `corbanu` first on PATH". No pong in either case. The same refusal applies to `--account real` and `cfgacct` under skew (fail closed). With the candidate on PATH, `real` gives pong, `fake` gives 401 and `ghost` is refused. |
| S02-9b | With the flag off, every code path is today's. | FAIL (#415) | **PASS** | With the flag off, `internal-claude-oauth-token --account fake` and `--account cfgacct`, and `CORBANU_PROVIDER_ACCOUNT=fake` (with or without an env token), exit 1 with "named accounts are off; enable them …" and print nothing. Exec with `provider_accounts.claude-plan="fake"` warns "ignored" and answers pong on the default token. Difference from the pre-PF-84 binary: a stray `CORBANU_PROVIDER_ACCOUNT` is now refused instead of ignored, which fails closed. The flag-on rows and the env exception (default-only) are unchanged. |
| S02-14 | Account isolation: a visible 401 with recovery. | FAIL (#416) | **PASS** | [TUI](captures/s02-recheck-416-tui.txt). The same selector as before (`-c provider_accounts.zai="fake"`) plus `--account fake`: "account `fake` was rejected. Replace its key with `corbanu account add zai fake` (reads stdin), or choose another account. The default Z.AI credential is unchanged." The canary run p2 ([proxy](captures/s03-tui-proxy.txt)) shows exactly one `fake` request, so there is no retry. Remaining gap: the TUI **resume** path still shows the old default-credential hint ([#427](https://github.com/CorbanuCore/CorbanuTerminal/issues/427), low). |

Also fixed, from the earlier run's other finding (#417): a home whose only Z.AI credentials are named accounts now
reaches the chat with `--account main` and with `-c provider_accounts.zai="main"`
([named-only](captures/s03-tui-named-only-home.txt)).

## Issues filed

- [#425](https://github.com/CorbanuCore/CorbanuTerminal/issues/425) (low–medium): `--account <other-provider>:<missing>` is accepted silently, and the session runs on the default account.
- [#426](https://github.com/CorbanuCore/CorbanuTerminal/issues/426) (low–medium): TUI `--account <missing>` opens a default-key onboarding loop instead of refusing.
- [#427](https://github.com/CorbanuCore/CorbanuTerminal/issues/427) (low): resume messaging. TUI resume shows the default-credential hint, and a flag-off resume blames `--account`.
- [#428](https://github.com/CorbanuCore/CorbanuTerminal/issues/428) (product decision): D3 is not applied in the "Aggressive enforced, boundary unverified" state, because core stays Permissive.

## Recommendation

- **S01:** accept, as recommended on 2026-10-10. #418 was closed by #422 but was not re-run here.
- **S02:** accept behind the flag. S02-3b, S02-9b and S02-14 now pass with real Claude Plan and Z.AI. Travis
  should note one behaviour change: with the flag off, a stray `CORBANU_PROVIDER_ACCOUNT` is now refused rather
  than ignored, which fails closed. S02-15 (sibling-route correction) stays NOT VERIFIABLE until product names a
  route pair.
- **S03:** limited acceptance behind the flag. Selection, precedence, inheritance, out-of-process workers,
  resume, the D3 approval and per-account isolation all pass with real providers and canary attribution.
  Fix #425 and #426 before the flag is removed. Neither leaks a credential or crosses providers, but both break
  "a missing account is an error". Decide #428 (should D3 follow the TUI's displayed level?). #427 is polish. S03-1b (`thread/spawnAgent`) needs a product-supplied
  check or the full code-blind run.

## Reproduce

- Scripts are in [`scripts/`](scripts/). Set `S` (default `/Volumes/CorbanuDrive/Corbanu/tmp/pf84s03acc`) and
  put the builds at `$S/bin/corbanu-{main,pre}`.
- Order:
  1. `exec-zai.sh` → `resume-zai.sh` → `exec-kimi.sh`;
  2. `proxy-setup.sh` (starts the proxy in the private tmux server);
  3. `spawn-setup.sh` → `spawn-exec.sh`;
  4. `s02/recheck-claude.sh` → `s02/claude-exec-candidate.sh`;
  5. the TUI scripts and the [interactive steps](scripts/s03/tui-interactive-steps.md);
  6. `authcmd-setup.sh`;
  7. `migration-onboard.sh` → `migration.sh`;
  8. `leakscan.py`.
- The scripts hard-code this host's helper path. Scratch homes, the proxy, the target directories and the
  worktrees were deleted after the run.
