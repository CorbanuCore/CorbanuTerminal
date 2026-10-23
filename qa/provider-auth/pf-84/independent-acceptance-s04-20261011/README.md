# PF-84-S04 independent code-blind acceptance (2026-10-11)

Scope: [PF-84-S04](../../../../docs/sprints/current/unified-provider-auth/pf-84-s04-provider-account-management-ui.md),
account management in `/providers` and onboarding, plus the duplicate-row fix. Behind the default-off
`named_accounts` flag, except the duplicate-row fix. Gate under test: [s04-gate.md](../s04-gate.md).
Independent review: [REVIEW.md](REVIEW.md).

## Method

- **Code-blind.** I read only:
  - the S04 sprint record and the plan's PF-84 section, with its acceptance rows "Existing single-account
    migration", "Add second account", "Session account switch" and "Account isolation";
  - `s04-gate.md`, the PF-84 demo indexes, and the S03 acceptance README (for its method);
  - `docs/provider-accounts.md`, `docs/authentication.md`, `docs/tmuxHarness.md`, `sec-common.md`, and `--help`
    output ([help](captures/help.txt)).

  I did not read `codex-rs/**`, `scripts/**`, PR diffs, demo specs, worker logs or reviews. I reused two
  tester-written helpers from the S03 acceptance (`canary_proxy.py` and the tmux driver pattern).
  Code-blindness is self-attested; no file-access log was kept.
- **What this does not close.** One agent designed and ran the cases on a debug build, not the packaged
  release. So the record's "code-blind handoff checker" item and the milestone code-blind VM gate both stay
  open. This is the lighter per-sprint independent functional acceptance.
- **Candidate.** `origin/main` at `c7e6e07cbc` (merge of #430). Built as a macOS arm64 debug build:
  `cargo build --locked --offline -p codex-cli --bin corbanu`, toolchain 1.95.0, in its own target directory.
  This is not the packaged release.
- **Baseline.** `2f95b4fc9a` (main just before #430), built the same way. I used it to reproduce the duplicate
  row and to create the existing single-account home.
- **Safety.**
  - Every run used `env -i`, a fake `HOME`, a disposable `CORBANU_HOME` and `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
    The vault key went to `secrets/keyring-fallback/`.
  - A `security(1)` PATH shim never wrote its log, so it recorded 0 calls
    ([esc-rerun.log](captures/esc-rerun.log), last line). It cannot see direct keychain framework calls; the
    keychain guard here is `CORBANU_TEST_NO_NATIVE_KEYRING=1` (the vault key file appears under
    `keyring-fallback/` in every home).
  - Real keys came from the installed signed backup helper, inside the consuming command only. They went into
    the TUI through a tmux buffer that was deleted on paste, and were never printed.
  - The real `~/.local/bin/corbanu` and the real home were never used.
- **Providers.**
  - Z.AI `glm-5.3-flash` (`provider/zai_api_key`).
  - Kimi `kimi-k3` on `kimi-code` (`provider/kimi_api_key`).
  - Claude Plan (`claude-plan-test-token`), model `claude-opus-5-plan` as onboarding set it.
  - OpenAI `gpt-5.4` (`openai-api-key`, API-key login).
- **Attribution.** Home hP runs the custom chat provider `zpx` through a local canary proxy
  ([canary_proxy.py](scripts/canary_proxy.py), [run](scripts/proxy-run.sh)):
  - Every account holds a synthetic canary, pasted through `/providers`. The proxy logs which account each
    request carried.
  - `pdefault`, `pmain` and `pwork` are forwarded to the real Z.AI. Any other canary gets a 401.
  - Result: [10 requests](captures/proxy-requests.jsonl): `pmain` 200 ×3, `pwork` 200 ×4, `pdefault` 200 ×2,
    `pfake` 401 ×1. No request body contained a canary or the real key.
  - The log has no timestamps; the order matches the transcripts (`p1`–`p3`). `zpx` runs with
    `request_max_retries = 0` and `stream_max_retries = 0`, so "no retry on another account" is shown with
    transport retries off.
  - On the real Z.AI, Kimi and Claude endpoints, only the 401 text and `/status` name the account; there is no
    per-request attribution. Z.AI `main` and `default` hold the same real key, so the inherited spawn's
    `child-ok` shows the label, not which key the child used.
- **TUI.**
  - The TUI ran through a private tmux socket following `docs/tmuxHarness.md` ([lib.sh](scripts/lib.sh)).
    Text and Enter were sent separately, and each wait needed two identical captures.
  - The sessions are listed in [tui-steps.md](scripts/tui-steps.md). Full screen transcripts are in
    [captures/transcripts/](captures/transcripts/).
  - **Evidence.** Most results are in captures, transcripts or saved re-run logs. Output that was only on my
    terminal at the time (Z.AI `config.toml` changes, the first Esc hash check, rule texts) is copied verbatim
    in [console-excerpts.txt](captures/console-excerpts.txt). I then re-ran the Esc hash check and the
    migration check with their output saved ([esc-rerun.log](captures/esc-rerun.log),
    [migration-rerun.log](captures/migration-rerun.log)).
  - Two tester slips:
    - Text typed while a list popup was open was read as list keys. It started a ChatGPT device-code login,
      which I cancelled (no `auth.json` was written).
    - A paste into the list hit the `r` hotkey and opened the default-key entry. I left it with Esc, and the
      default key still answered pong.

## Verdicts

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| S04-1 | `/providers` shows exactly one row per (provider, account), with kind, fingerprint and "this session" / "default for new sessions" markers. | PASS | [Z.AI](captures/providers-zai-real.txt): `Z.AI · default`, `Z.AI · fake  Named account · API key · fp de063d3d0b35`, `Z.AI · obfake`. The other providers have one row each. Claude ([rows](captures/providers-kimi-claude-openai.txt)) shows `default`, `cfg` (Claude Code login), `fake` and `work` (subscription token). Kimi and `zpx` look the same. The markers move with each action. The fingerprint stays the same across a restart and a rename. The default row shows no kind or fingerprint. |
| S04-2a | Add an account in `/providers` with a masked key or Claude token; adding never replaces an existing account. | PASS | Entry is masked (`•••`). Saved: Z.AI (`fake`, `main`), Kimi (`fake`), `zpx` (`main`, `work`, `fake`, `temp`, `quota`), Claude Plan (`fake`, plus `cfg` as a path). An existing name is refused at the name step: "zai already has an account `fake`. Choose another name, or remove it first." An invalid name gets the rule text, and an empty value saves nothing. Renaming to an existing name is also refused ([vault-index](captures/vault-index.txt)). Rule texts are in [console-excerpts](captures/console-excerpts.txt). |
| S04-2b | Add a Claude Code login (config dir) or Claude sign-in account. | NOT VERIFIABLE | `cfg` was saved as a path (`claude_config_dir`) and shows as `Claude Code login · fp 61539973a31e`. I never used it for a request, and I had no real login folder. It was accepted without checking that it holds a login. Adding through Claude sign-in was not run. |
| S04-3 | Rename: credential unchanged; refused for this session's account; a default follows. | PASS | `zpx` `main`→`spare` keeps fp `27f1c1496097`, and the proxy still saw `pmain` for `spare`. Z.AI `fake`→`spare`: "It stays the default for new sessions", and `config.toml` changed to `zai = "spare"`. Renaming the session's account is refused: "This session uses zai account `fake`. Use another account for this session first, then rename it." |
| S04-4 | Remove deletes only that account; an account in use needs a replacement first. | PASS | Removing an account that is in use (or is the default) asks "Choose the account that replaces it first" with a *Replace with …* option per other account and *Keep it*. Replace with `obfake` makes it the new default. Replace with `default` clears `[provider_accounts]`. Replace with `work` moves this session to `work` (the proxy saw `pwork`). *Keep it* changes nothing. After each removal, `corbanu account list` still shows the other accounts and the defaults keep answering. Vault index ([vault-index](captures/vault-index.txt), `/vault` → *View credentials*, metadata only): before, `provider/zpx/accounts/{aa1,aa2,quota}/api_key` (3 credentials); after removing `aa1` in `/providers`, only `aa2` and `quota` (2). The Z.AI `config.toml` changes are in [console-excerpts](captures/console-excerpts.txt). |
| S04-5 | *Use for this session* starts a new session on that account, requests use it, and the default is unchanged after a restart. | PASS | "New session: requests to zpx use account `main`" → the proxy saw `pmain` 200. *Use default account for this session* → `pdefault`. `config.toml` is unchanged by a session switch, and after a restart the configured default (`work`) is back. Z.AI `fake`, Kimi `fake` and Claude `fake` all got 401s (S04-11). |
| S04-6 | *Make default* persists across a restart. | PASS | "New sessions use zpx account `work`. This session is unchanged." The same session's next request was still `pmain`. `[provider_accounts] zpx = "work"` was written. After a restart, `/status` showed `account work` and the proxy saw `pwork`. Z.AI: the default `fake` survived a restart (`/status account fake`, 401). |
| S04-7 | Duplicate-row fix: one Claude row after Claude Plan setup; no duplicate provider rows. | PASS | [onboarding](captures/onboarding-one-row.txt). Baseline: `3. Provider: Anthropic Claude Account` and `4. Provider: Claude Account`, both "Configured · active · ready". Candidate: only `3. Provider: Claude Account`, with the flag on and off, and still one row after adding `work` and `fake`. Z.AI also has one onboarding row. `/providers` has no duplicates (S04-1). |
| S04-8 | Onboarding: one row per configured provider, with *Use configured credentials* / *Add another account* (flag on) / *Replace* behind it. Accounts added there persist. | **FAIL** (medium-low, [#461](https://github.com/CorbanuCore/CorbanuTerminal/issues/461)) | The chooser is as described (Z.AI, Claude), and focus stays on the configured row. Saved from onboarding: `obfake` (Z.AI, later visible in `/providers` after a restart, fp `511a22afe776`), and Claude `work` (real token) and `fake`. A duplicate name is refused, though the wording is odd ("invalid credential label: zai already has an account `obfake`"). *Replace* followed by Esc is inert. **Defect:** typing `q` in the onboarding *name* or *key* field quits the TUI with exit 0 and saves nothing ([captures](captures/onboarding-q-quits.txt)). The default-key field and `/providers` accept `q`. Bracketed paste is not affected. A real Claude token pasted without bracketed paste stopped after 32 characters and the TUI exited. |
| S04-9 | `/status` and the spawn line show the active account. | PASS | `/status`: `Z.AI - https://api.z.ai/api/paas/v4 · account fake`, also `· account default`, `· account work` (`zpx`), Kimi `· account fake`, and Claude Plan `· account fake`. Spawn ([capture](captures/spawn-line.txt)): `Spawned … (glm-5.3-flash max · account main)` when inherited (child answers `child-ok`), and `(glm-5.3-flash max · account fake)` when explicit (child 401). With the flag off, `/status` has no account suffix (a flag-off spawn was not run). The footer reads "via zai default" whatever the account. That is not a criterion, but it is easy to misread. |
| S04-10 | Esc changes nothing. | PASS | Esc in the account actions, at the Rename name, at the Remove confirmation, at the add-name and masked-key steps (in `/providers` and onboarding), in the onboarding chooser and in *Replace*. `account list` was unchanged each time. Saved re-run ([esc-rerun.log](captures/esc-rerun.log)): Esc in the actions, in Rename (with `renamed` typed), in Remove, and after a masked paste in Add; `config.toml` and `local.age` hashes unchanged (diff exit 0); `/status` still `account default`. *Keep it* is also inert. |
| S04-11 | A fake-key account gets a 401 naming it, with no retry on another account. | PASS | Z.AI: "Z.AI (zai) account `fake` was rejected. Replace its key with `corbanu account add zai fake` (reads stdin), or choose another account. The default Z.AI credential is unchanged." Kimi and Claude Plan look the same (Claude also mentions the config directory). Proxy: exactly one `pfake` request and no other canary in that turn. |
| S04-12 | Masked entry; no value appears in the viewport, scrollback, logs, rollouts or artifacts. | PASS for `/providers`; **FAIL** for onboarding (#461) | [Leak scan](captures/leak-scan.txt): 82,690 files across the homes (sqlite logs and rollouts included), the logs, the captures, the proxy log and this directory. Values checked: 4 real keys and 13 canaries. The only hit is the OpenAI key in `hC2/auth.json`, where the pre-existing API-key login stores it today. The canaries pasted in the TUI never appear. A 16-character-window scan for partial real keys ([partial-scan](captures/partial-scan.txt)) also finds only `auth.json`. That covers the one real Claude token cut short by #461 (c1). But #461 breaks masked entry in onboarding: after the TUI quits, the rest of the key arrives in clear on the terminal (a synthetic value is echoed after `TUI_EXITED` in [onboarding-q-quits](captures/onboarding-q-quits.txt)). |
| S04-13 | With the flag off, behaviour is unchanged (except the duplicate-row fix). | PASS | [flag off](captures/flag-off.txt): one `Z.AI` row; the actions are only *Deactivate* and *Replace with API key*; `/status` has no account; `-c provider_accounts.zai="fake"` shows "`[provider_accounts]` is ignored because the `named_accounts` feature is off" and the default answers pong; the onboarding chooser has no *Add another account*; `corbanu account list` is refused. |
| S04-14 | An existing single-account home carries over untouched. | PASS | [migration](captures/migration.txt) and the saved re-run [migration-rerun.log](captures/migration-rerun.log) (fresh home hM2, both hash diffs empty, exit 0). Home hM was onboarded by the baseline. On the candidate, with the flag off and on, the hashes of `config.toml`, `local.age`, the vault key file and `provider-eligibility.json` stay the same. There is one plain `Z.AI` row; the `· default` label appears only after a named account exists, and `/status` shows `· account default`. The plan row's "one `default` account row" is met in substance; product may want the label. It answers pong. Adding and then removing a named account changes only `local.age`, and the default keeps working. |
| S04-15 | A missing or wrong account fails, never falls back. | PASS | Choosing a stale row after `account remove zpx temp`: "thread/start failed: account `temp` of provider `zpx` is not configured …", then "the account selection was not changed". The session stayed on `work`: the 10th proxy line is `pwork` 200. The reply did not render in the transcript (`p3`). The configured default removed from the CLI → the TUI exits before start with the same text; 0 requests. `--account openai:work`: "provider `openai` does not support named accounts yet". |
| S04-16a | Provider-generic. | PASS | Z.AI, Kimi, Claude Plan and the custom `zpx` all work. Claude's two named accounts survive a restart ([vault-index](captures/vault-index.txt), c4). |
| S04-16b | Named ChatGPT and AWS accounts stay refused (D4 → S06). | NOT VERIFIABLE (partly seen) | OpenAI (API key configured, `gpt-5.4` pong) offers no *Add another account*, and `--account openai:work` is refused. ChatGPT sign-in and Bedrock were not run. |
| S04-17 | Reuses the shared PF-50 controller. | NOT VERIFIABLE | Internal. The add flows look and behave the same across providers. |

## Issues filed

- [#461](https://github.com/CorbanuCore/CorbanuTerminal/issues/461) (medium-low): typing `q` in onboarding's
  *Add another account* name or key field quits the TUI.
- [#462](https://github.com/CorbanuCore/CorbanuTerminal/issues/462) (low, also on the baseline, not an S04
  regression):
  - `/status` says "OpenAI API key configured" in a Z.AI-only home;
  - onboarding loops when `config.toml` already names the provider.

## Notes (not verdicts)

- Not tested:
  - the "one session selection per provider" refusal (a known limit);
  - multi-agent v2 spawns (also a known limit).

- The 401 recovery text points to `corbanu account add`, which still replaces an account (a documented S02 limit).
- The default row has no kind or fingerprint. The footer does not name the account.
- `docs/authentication.md` still says to choose "Provider: Anthropic Claude Account". That is still the
  unconfigured label; once configured, the single row reads "Provider: Claude Account".
- A Claude Code login directory is accepted without validation.

## Recommendation

Accept S04 behind the flag, with one fix first. Two items stay open: the record's code-blind
handoff-checker item, and the milestone VM gate. Everything in `/providers` passes with real Z.AI, Kimi and
Claude Plan keys and canary attribution: add, rename, remove, switch, make default, Esc, durability, 401s that
name the account, fail-closed, the spawn line and `/status`. The duplicate Claude row is gone, and an existing
home is untouched.

Fix #461 before the flag is removed, and treat it as security-relevant. A `q` typed in onboarding's *Add
another account* quits the TUI, and the rest of a typed (or non-bracketed-pasted) key then appears in clear on
the terminal.

## Reproduce

Set `S` (default `/Volumes/CorbanuDrive/Corbanu/tmp/pf84s04acc`) and put the builds at
`$S/bin/corbanu-{main,base}`. Run `proxy-run.sh` in the private tmux server, then follow
[tui-steps.md](scripts/tui-steps.md) with the `lib.sh` helpers. Run `leakscan.py` last, with the real keys in
env. The helper path is specific to this host. The scratch homes, the proxy and the target directories were
deleted after the run.
