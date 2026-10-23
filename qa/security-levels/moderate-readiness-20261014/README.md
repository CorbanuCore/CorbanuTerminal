# Moderate readiness: tmux spot checks (2026-10-14)

Supporting evidence for [docs/plans/moderate-ships-readiness.md](../../../docs/plans/moderate-ships-readiness.md).
These are coordinator-side spot checks. They are not a code-blind run and not a milestone result.

- **Binary:** the third Aggressive run's packaged candidate, `origin/main` `4b6dd55c85`, macOS arm64 0.1.48,
  `bin/corbanu` SHA-256 `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`.
- **Profile:** a disposable home (`HOME`, `CODEX_HOME`, `CORBANU_HOME`, `PFTERMINAL_HOME` and `TMPDIR` all under the
  run folder), `CORBANU_TEST_NO_NATIVE_KEYRING=1`, the milestone feature flags of
  [`product-config.toml`](../milestones/aggressive-20261013-third/harness/product-config.toml), plus `[security]`
  `version = 1`, `level = "moderate"` in `config.toml`. Moderate can't be chosen in `/security` (see `c1-picker.txt`),
  so config is the only way to reach it.
- **Model:** GLM 5.3 Flash on `zai`. Approvals at the default (`on-request`); sandbox at the default for a trusted
  folder (workspace-write).
- **Fixture:** a git project `ledger-notes` with a synthetic `.env`, and `outside-notes.txt` beside it. For `c6`, a
  dummy file `secrets/probe.txt` (no secret) was placed in the disposable home. Paths are shortened to `<run>` (in the header box too, so its border no longer lines up).
- `c1-*` come from a first session whose home sat under `/tmp`; its later captures were discarded because the
  writable `/tmp` root made every launch refuse ("would let this command write Corbanu's configuration directory").
  `c2`–`c6` come from a second session with the home outside any temp folder.

| Capture | What it shows |
| --- | --- |
| `c1-picker.txt` | `/security` with config Moderate: "Active in this session: Permissive", "Core's level: Moderate", Moderate "not available yet" |
| `c1-inspector.txt` | Inspector at start: "Active this run: Permissive" next to "Live policy: Moderate in force"; the Vault row says the vault store and sign-in file are readable to agent commands; the Sandbox row says approved commands can run outside the sandbox |
| `c2-prompt.txt`, `c2-result.txt` | `echo … >> ../outside-notes.txt`: the prompt offers the outside write; after `y`, "Protected launch refused: this command would run outside the OS sandbox …". The file still held only its original line when checked with `cat` on the host afterwards (not captured) |
| `c3-prompt.txt`, `c3-result.txt` | After untrusted content, `head -3 ~/.corbanu/config.toml`: a fresh-approval prompt, then "Operation not permitted" with no security-level attribution |
| `c4-prompt.txt`, `c4-result.txt` | `corbanu vault list` (not a real subcommand): a fresh-approval prompt for "vault access after untrusted content"; after approval it ran and failed on the subcommand |
| `c5-inspector-after.txt`, `c5-inspector-scrolled.txt` | Inspector after those turns: the same rows; the Nested agents row speaks only of Aggressive |
| `c6-vault-store-read.txt` | New session, first command `cat ~/.corbanu/secrets/probe.txt`: denied ("Operation not permitted") with no attribution, while the Vault row says the store is readable |

No `corbanu vault` subcommand that reads or creates vault entries was run, so no keychain item could be created
or probed.
