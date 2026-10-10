# PF-84-S01 / PF-84-S02 independent source-blind functional acceptance (2026-10-10)

Scope: [PF-84-S01](../../../../docs/sprints/current/unified-provider-auth/pf-84-s01-home-override-hygiene.md)
(home override hygiene) and
[PF-84-S02](../../../../docs/sprints/current/unified-provider-auth/pf-84-s02-account-registry-and-resolution.md)
(named accounts behind `named_accounts`). Gate evidence under test:
[s01-gate.md](../s01-gate.md), [s02-gate.md](../s02-gate.md). Independent review: [REVIEW.md](REVIEW.md).

## Method

- **Source-blind, not the milestone code-blind VM gate.** As the owner instructed, I read only the
  sprint records (including their code-boundary references), the plan's PF-84 section, the two gate
  files, the demo indexes, `docs/authentication.md`, `docs/tmuxHarness.md`, `sec-common.md` and
  `--help` output. I did not read `codex-rs/**`, `scripts/**`, PR diffs, worker logs or reviews.
  The scripts were only run, never read. One stray `sh -x` run of the launcher, against an empty work
  dir, showed that it execs `<work dir>/bin/corbanu`. That is the only internal detail I used.
  This run has no separate designer and no handoff-checker run. It does not tick the sprints'
  "code-blind handoff checker" box; that needs the full isolated run or Travis's limited-testing
  agreement.
- **Candidate.** `origin/main` at `0fa45b54f0` (merge of #409, which includes S01 #408). This was the
  head when the run started. #410 (sandbox command-log redaction, not PF-84) landed during the run and
  is not included. The build is macOS arm64 debug: `cargo build --locked --offline -p codex-cli --bin corbanu`
  with toolchain 1.95.0 and its own target directory. It is not the packaged release.
- **Baseline.** A pre-PF-84 build of `44b527f11f` (before #408/#409), built the same way, used to create
  the "existing single-account home" and to compare home precedence.
- **Safety.**
  - Every candidate and baseline run used `env -i`, a disposable `CORBANU_HOME`, a fake `HOME` and
    `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
  - Real keys came from the operator vault, through the installed signed backup binary:
    `X="$(… corbanu.bak-20261010 vault auth-helper <label>)" <consumer>`. That path is allowed by
    `sec-common.md`, and it reads the real vault (and therefore its keychain item). So "no keychain"
    applies to the candidate and baseline processes only. Keys went to stdin or to a tmux paste buffer
    that was deleted on paste.
  - Claude Plan runs always selected a named account, so the default-account Claude Code lookup was
    not exercised on purpose. A `security(1)` PATH shim recorded 0 calls from the candidate.
  - Scratch homes, the install prefix and the target directory were deleted after the run.
- **Providers.** Z.AI `glm-5.3-flash`, Kimi `kimi-k3` on `kimi-code`, OpenAI `gpt-5.4` (API-key
  login) and Claude Plan `claude-opus-5-5-plan`.
- **Isolation probes.** A local mock provider ([`mock.py`](scripts/mock.py),
  [canaries](scripts/fixtures/make-canaries.py), provider configs in [`fixtures/`](scripts/fixtures/))
  records which synthetic bearer arrived and answers 401. A mock `auth.command`
  ([`authcmd.sh`](scripts/authcmd.sh)) logs `CORBANU_PROVIDER_ACCOUNT`.
- **TUI.** I drove the TUI through a private tmux socket using the `docs/tmuxHarness.md` rules
  ([`tmuxlib.sh`](scripts/tmuxlib.sh)): text and Enter are sent separately, every wait needs two
  identical matching captures, and each session exits with `/exit`. Each run's home config, account
  list and invocation are in [tui-setup](captures/tui-setup-homes-and-invocations.txt). The status-line
  word after `via zai` is the reasoning effort, not the account ([evidence](captures/tui-status-line-effort-not-account.txt)).
- **S01 wrappers.**
  - Dev launcher: I ran the checked-in `scripts/dev/corbanu-launcher.sh` with
    `CORBANU_LAUNCHER_WORK_DIR`/`CORBANU_LAUNCHER_WORKSPACE_DIR` pointing at a scratch dir. That dir
    holds an [`activate.sh` stand-in](scripts/fixtures/activate.sh.standin) that exports a coordinator
    home unconditionally (the multiacct1 pitfall) and a `bin/corbanu` that links to the candidate.
  - Release wrapper: I installed it with `scripts/install/install.sh` into a scratch prefix. The
    release binary was first replaced by an env-printing stub, then by the candidate.

## Verdicts: PF-84-S01

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| S01-1 | Conflicting home variables give one stderr warning that names the winner. It prints paths only, once per process. Equal paths (trailing slash, symlink) give no warning. | PASS | [bare-binary matrix](captures/s01-matrix-bare-binary.txt) cases 3–9. A full `exec` run printed exactly 1 warning line ([launcher 401](captures/s01-launcher-openai-401.txt)). |
| S01-2 | Precedence is unchanged: `CORBANU_HOME` > `PFTERMINAL_HOME` > `CODEX_HOME`. | PASS | All 9 cases resolve to the same home on the pre-PF-84 build and the candidate. The baseline is silent in every case ([baseline](captures/s01-matrix-pre-pf84-binary.txt)). |
| S01-3 | `scripts/dev/corbanu-launcher.sh` is checked in and keeps a caller-set `CORBANU_HOME`/`CODEX_HOME`/`PFTERMINAL_HOME` after sourcing `activate.sh`. | PASS | [launcher matrix](captures/s01-matrix-launcher.txt): L1 uses the coordinator home. L2–L4 use the caller's home. L5–L7 conflicts are reported. The repo launcher test passes 5 of 5 ([log](captures/repo-launcher-shell-test.txt)). |
| S01-4a | The release wrapper `corbanu` never overrides a home the caller set. | PASS | [stub](captures/s01-release-wrapper.txt) W1–W5 and [candidate](captures/s01-release-wrapper-candidate.txt) V1–V4 (V4 warns once). The repo installer tests pass 23 of 23 ([log](captures/repo-install-wrapper-test.txt); this needs `TMPDIR` set, otherwise 2 unrelated pruning tests fail on `/tmp` vs `/private/tmp`). |
| S01-4b | The installed `corbanu-debug` reports a conflicting home instead of ignoring it. | **FAIL** (low) | [candidate](captures/s01-release-wrapper-candidate.txt): `CORBANU_HOME` is silently ignored in V2, V4 and V6 (no warning). Caller-set `CODEX_HOME`/`CORBANU_DEBUG_HOME` are kept. [#418](https://github.com/CorbanuCore/CorbanuTerminal/issues/418). |
| S01-5 | Regression: a launcher worker with `CORBANU_HOME=homeB` uses homeB's account (fake OpenAI key gives 401). | PASS | [R1–R4](captures/s01-launcher-openai-401.txt): the coordinator home (real key) answers `pong`. `CORBANU_HOME=homeB`, `CODEX_HOME=homeB` and the tool-shell shape each get a 401 that echoes `…6bb0`, which is homeB's synthetic key suffix (last line of the capture). |
| S01-6 | Wrapper/home override flow (plan): with an inherited `CORBANU_HOME`, a worker never silently runs on the coordinator's home. | PASS | TUI on GLM 5.3 Flash ([tui-t5](captures/tui-t5.txt)): the tool shell inherited `CORBANU_HOME=CODEX_HOME=<coordinator>`. The launcher worker landed on homeB with one warning and got homeB's 401. L7: a worker that sets only `CODEX_HOME` stays on the coordinator's home, with a warning. |
| S01-7 | Documented in `docs/authentication.md`. | PASS | Lines 68–77 match the observed behaviour, including the in-session warning. `corbanu-debug` is not covered (#418). |

## Verdicts: PF-84-S02

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| S02-1 | Names match `[a-z0-9][a-z0-9-]{0,31}` and `default` is reserved. | PASS | [account CLI](captures/s02-account-cli.txt): 32 characters accepted, 33 rejected. Uppercase, `_`, space, non-ASCII, empty and a leading `-` are rejected. `default` is refused. |
| S02-2 | Labels are `provider/<id>/accounts/<name>/<kind>` in the existing vault, and the default labels are unchanged. | PASS | [`/vault list`](captures/tui-vault-list-home-G.txt) shows `provider/zai/accounts/{main,fake}/api_key` next to the untouched `provider/zai_api_key` written by the baseline build. `secrets/` keeps the same three entries. Whether named accounts add keychain items is NOT VERIFIABLE under the no-keychain rule. |
| S02-3a | API-key resolvers use only the account's key. Named accounts ignore provider env vars and never fall back. | PASS | Mock canaries ([routing](captures/s02-canary-routing-matrix.txt)): C1 default gets the `env` key, C2 gets `acct-a`, C3 gets `acct-b`. C4 `ghost` fails closed with 0 requests. Real Z.AI ([zai](captures/s02-zai-real-matrix.txt)): Z6 `fake` with a valid `ZAI_API_KEY` in the env still gets 401. Z7 `ghost` with a valid env key fails closed. |
| S02-3b | Claude Plan resolver: per-account token, never a fallback. | **FAIL** (medium) | With the candidate on PATH it works: [P1](captures/s02-claude-plan-real-matrix.txt) `real` answers pong, P2 `fake` gets 401 and P3 `ghost` fails closed. But the token command is the `corbanu` found on PATH, and the account travels only in `CORBANU_PROVIDER_ACCOUNT`. With a pre-PF-84 `corbanu` on PATH, a session on account `fake` answered **pong from the default account** ([skew](captures/s02-claude-plan-path-version-skew.txt)). This is a silent fallback: [#414](https://github.com/CorbanuCore/CorbanuTerminal/issues/414). |
| S02-4 | `auth.command` receives `CORBANU_PROVIDER_ACCOUNT`, and an unenrolled account never runs the command. | PASS | [Q1–Q4](captures/s02-authcommand-matrix.txt): default gets `<unset>`, `work` gets `work` (bearer `cmd:work`), `ghost` runs 0 commands and sends 0 bearers. Minor: 2 unauthenticated `GET /models` requests were still sent ([#419](https://github.com/CorbanuCore/CorbanuTerminal/issues/419) item 2). |
| S02-5 | `internal-claude-oauth-token --account` works for a token or a Claude Code dir, and an unknown account fails closed. | PASS | [internal token](captures/s02-internal-claude-oauth-token.txt): `faketok` and `cfgacct` return the expected sha12s. `ghost` exits 1 with no output. The env token stays default-only on the candidate. Minor: relative or missing config dirs are accepted at add time (#419 item 4). |
| S02-6 | `named_accounts` is off by default. `[provider_accounts]` (config table and `-c`) works. `corbanu account list\|add\|remove` reads secrets from stdin only. | PASS | `features list` shows `named_accounts … false`. With the flag off, `account` refuses and writes nothing. `--value`, empty stdin, whitespace and a TTY are refused ([account CLI](captures/s02-account-cli.txt)). Config table: `[provider_accounts] zai = "fake"` gives 401, and `-c …="main"` overrides it to pong ([table](captures/s02-config-table-selector.txt)). |
| S02-7 | `corbanu account` is refused under Aggressive. | PASS (agent path) | In a session showing "Aggressive is enforced, but the protected boundary is unverified", an approved agent command `corbanu account add` was refused with exit 1 and nothing was saved ([tui](captures/tui-aggressive-account-add.txt)). A user `!` command and a directly started CLI still succeed ([cli](captures/s02-aggressive-account-cli.txt)); `/security` says self-started commands are "not covered yet". Product call needed if the criterion meant those too. |
| S02-8 | Descoped OpenAI sign-ins and AWS fail closed. | PASS | [openai](captures/s02-openai-real-matrix.txt): O2 `openai` and O3 `amazon-bedrock` selectors are refused at config load. `account add openai\|amazon-bedrock` is refused. O1 shows the OpenAI default (`gpt-5.4`) still works with the flag on. |
| S02-9a | With the flag off, the exec and TUI paths behave as today. | PASS | C7, Z8, O4, Q4 and [tui-t4](captures/tui-t4.txt): the selector is ignored and the default key is used (`pong`). The only change is a new "ignored" warning, printed twice (#419 item 1). |
| S02-9b | With the flag off, every code path is today's. | **FAIL** (low) | `internal-claude-oauth-token --account faketok` and `CORBANU_PROVIDER_ACCOUNT=faketok` still return the named token with the flag off: [#415](https://github.com/CorbanuCore/CorbanuTerminal/issues/415). |
| S02-10 | An existing single-account home carries over untouched. | PASS | [migration](captures/s02-migration-existing-home.txt): the home was created and onboarded by the baseline build. With the candidate, flag off then on (exec, `account list`, explicit `default`), `local.age` and `config.toml` hashes stay unchanged until the first named write (M5). Default still answers pong afterwards. The baseline build still reads the home after the named write (M8). After `remove`, the vault is back to 617 bytes but re-encrypted (new hash). |
| S02-11 | Isolation: canaries never cross, and a missing account never yields the default. | PASS | S02-3a and S02-4 matrices, plus [tui-t2](captures/tui-t2.txt) and its log: one request id for the `fake` 401, so no retry on another account. |
| S02-12 | Only names and fingerprints appear; no secret value lands in any artifact. | PASS | [leak scan](captures/leak-scan.txt): 4 real keys and 9 canaries checked against every scratch file (homes, logs, rollouts, sqlite, captures) and this directory. The only hits are the OpenAI key in `auth.json`, which is where `login --with-api-key` stores it today, and the synthetic config-dir canary in my own scripts. Whole-value match only. |
| S02-13 | tmux + GLM 5.3 Flash: `main` answers pong, `fake` gets 401. A second provider works too. | PASS | TUI: [t1](captures/tui-t1.txt) pong, [t2](captures/tui-t2.txt) 401, [t3](captures/tui-t3.txt) `ghost` fails closed (invocations in [tui-setup](captures/tui-setup-homes-and-invocations.txt)). CLI: [Kimi](captures/s02-kimi-real-matrix.txt) K1 pong, K2 401, K3 fails closed. Claude Plan and OpenAI are covered above. |
| S02-14 | Account isolation flow (plan): a visible 401 with recovery. | **FAIL** (UX) | The 401 is visible and fails once. But the hint points to `/providers … press r`, which manages the **default** credential, not the failing named account: [#416](https://github.com/CorbanuCore/CorbanuTerminal/issues/416). |
| S02-15 | A model correction to a sibling route keeps the account only when both routes use the same key. | NOT VERIFIABLE | Source-blind, I could not tell which routes count as siblings. Needs a product-supplied route pair. |
| S02-16 | Named subscription labels are refused by `vault auth-helper`. | PASS | [helper](captures/s02-vault-auth-helper-named-labels.txt): the named Claude token exits 1 with empty stdout. A `claude_config_dir` label returns its directory path (not a secret). Named API keys resolve like default API keys do. |

Other finding (outside S02's listed criteria; probably S03/S04): when the only Z.AI credentials are named
accounts, the TUI forces default-key onboarding even with `-c provider_accounts.zai="main"`. The chat
cannot be reached ([t0](captures/tui-t0-named-only-home-onboarding.txt); home in
[tui-setup](captures/tui-setup-homes-and-invocations.txt);
[#417](https://github.com/CorbanuCore/CorbanuTerminal/issues/417)).

## Issues filed

[#414](https://github.com/CorbanuCore/CorbanuTerminal/issues/414) (medium; silent fallback under version skew; blocks flag removal),
[#415](https://github.com/CorbanuCore/CorbanuTerminal/issues/415) (low),
[#416](https://github.com/CorbanuCore/CorbanuTerminal/issues/416),
[#417](https://github.com/CorbanuCore/CorbanuTerminal/issues/417),
[#418](https://github.com/CorbanuCore/CorbanuTerminal/issues/418) (low),
[#419](https://github.com/CorbanuCore/CorbanuTerminal/issues/419) (polish: duplicate warning, unauthenticated `/models`, 28 auth-command spawns, config-dir validation, grammar).

## Recommendation

- **S01: accept.** Every PASS was proven against real OpenAI 401s, in the TUI and through both main
  wrappers. The one FAIL is the low-severity `corbanu-debug` follow-up #418.
- **S02: limited acceptance behind the default-off flag only.** API-key and `auth.command` isolation is
  solid, and the existing-home migration is byte-exact. This is conditional on Travis accepting #414,
  #415 and #416 as tracked follow-ups, and on S02-15 staying open.
- **Before removing the flag, and before relying on named Claude Plan accounts, #414 must be fixed.**

## Reproduce

Scripts are in [`scripts/`](scripts/) and fixtures in [`scripts/fixtures/`](scripts/fixtures/). Set
`S` to a scratch directory and put the two builds in `$S/bin/corbanu-{main,pre}`. The scripts hard-code
this host's helper path (`/Users/Neo/.local/bin/corbanu.bak-20261010`). Home G was onboarded by the
baseline TUI with masked paste ([tui-setup](captures/tui-setup-homes-and-invocations.txt)).
