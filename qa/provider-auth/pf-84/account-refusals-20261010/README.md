# PF-84 named accounts: fixes for #425, #426 and #427 (2026-10-10)

Bounded fix. Product spec **Shipping MVP — LIVE**: "Encrypted `/vault`, masked entry, metadata-only inspection, and
operational credential use without placing raw values in chat." (PF-84 rule: a missing account is an error, never the
default account.)

Defects from the [S03 independent acceptance](https://github.com/CorbanuCore/CorbanuTerminal/pull/429)
(`independent-acceptance-s03-20261010/`). #428 (D3 under "Aggressive, boundary unverified") is waiting on a
product decision and is not touched here.

## What changed

| Issue | Change |
| --- | --- |
| #425 | `--account <other-provider>:<name>` is checked at thread start like the session provider's account: a missing one is refused with the same "not configured … `corbanu account add …`" text. Meaning (now in `--help` and [docs/provider-accounts.md](../../../../docs/provider-accounts.md)): it selects that provider's account for whatever in the session runs on it (spawned agents and workers given that provider, a TUI switch to it). `corbanu exec` cannot switch providers, so it refuses such an account when no spawned agent can run there (`agents.enabled = false`, or `agents.provider_allowlist` excludes it); `<provider>:default` is never refused. The account a resumed thread recorded (or a worker's parent runs on) now has its own override slot, so another provider's `--account` no longer drops it. |
| #426 | The TUI checks the selected accounts (session provider and an explicit other provider) after loading its config and exits with that text before any onboarding screen, also on resume (onboarding cannot use the recorded account; `--account` is the recovery). A remote app server checks its own home at thread start instead. |
| #427 | `thread/resume` returns the account the thread runs on (`providerAccount`, experimental). The TUI uses it for the 401 hint, so a resumed thread on a recorded named account names that account. A flag-off resume of a thread recorded on a named account now says "the thread's recorded account zai:main: … resume with `--account default` …" instead of blaming `--account`. |

## Live checks

Fresh debug builds of `origin/main` `3b21152316` (before) and this branch (after), disposable homes, `env` emptied
except the provider key, `CORBANU_TEST_NO_NATIVE_KEYRING=1`. Real keys from the installed helper only inside the
consuming process ([live-env.sh](scripts/live-env.sh), [tui-lib.sh](scripts/tui-lib.sh)). GLM 5.3 Flash on `zai` and
`kimi-k3` on `kimi-code`.

Attribution: in `hZ` the only Z.AI credentials are the default key (env) and named `main`/`fake`; with no Z.AI
`--account`, a `pong` comes from the default key. `hK` has only the default Kimi key. For the positive spawn check,
`kimi-code:kmain` holds the real Kimi key and **no default Kimi key exists**, so a kimi-code child that answers used
`kmain` (control without `--account`: the child fails on the missing `KIMI_API_KEY`).

| Run | Before | After |
| --- | --- | --- |
| X12 zai session, `--account kimi-code:main` (kimi-code has no accounts) | `pong` from the default Z.AI key, exit 0 | refused: "account `main` of provider `kimi-code` is not configured; add it with …", exit 1 |
| X12b `--account kimi-code:ghost` | `pong`, exit 0 | refused, same text |
| K9 kimi-code session, `--account zai:ghost` | `pong` from the default Kimi key | refused, same text |
| Z3 `--account ghost` (reference) | refused | refused (unchanged) |
| P1 `--account kimi-code:kmain` (configured) | `pong` | `pong` (session stays on zai default) |
| P2 same with `agents.provider_allowlist=["zai"]` | `pong` (account silently unused) | refused: "nothing in it can run on `kimi-code` (`agents.provider_allowlist` …)" |
| P3 same with `agents.enabled=false` | `pong` (silently unused) | refused: "(`agents.enabled` is false, so no agent can be spawned)" |
| P4 `--account kimi-code:default`, `agents.enabled=false` | `pong` | `pong` (`default` is never refused) |
| R8 `exec resume --account kimi-code:kmain <thread recorded on fake>` | `pong` from the **default** Z.AI key (recorded `fake` dropped) | 401 on `fake` (recorded account kept) |
| S2 `--account kimi-code:kmain`, `multi_agent_v2` spawn on `kimi-code`/`k3` | – | `child-ok`; child rollout `model_provider=kimi-code provider_account=kmain` |
| S3 control: same spawn without `--account` | – | child fails: "Missing environment variable: `KIMI_API_KEY`" |
| R7 `exec resume --disable named_accounts` of a `main` thread | "--account zai:main: named accounts need …" | "the thread's recorded account zai:main: … resume with `--account default` …" |
| R7b same with `--account default` | `pong` | `pong` |
| T1 TUI `--account ghost` | default-key onboarding screen | exits 1 with the "not configured" text |
| T2 TUI `resume --account ghost <id>` | onboarding screen | exits 1, same text |
| T3 TUI `--account kimi-code:ghost` | chat on zai default | exits 1 with the kimi-code text |
| T4 TUI `resume <thread recorded on fake>`, send `ping` | 401 + "Z.AI (zai) credential was rejected. Open /providers …" | 401 + "Z.AI (zai) account `fake` was rejected. Replace its key with `corbanu account add zai fake` …" |
| T5/T6 TUI `--account fake` / no flag | – | reach the chat; default answers `pong` |
| T7 TUI `-c provider_accounts.zai="ghost"` | – | exits 1 with the "not configured" text |
| T8 same, `resume <thread recorded on default>` | – | exits 1 (onboarding would otherwise loop) |
| T9 same with `--account default` | – | resumes, `pong` |

Captures: [exec before](captures/425-427-exec-before.txt), [exec after](captures/425-427-exec-after.txt),
[spawn](captures/425-spawn-other-provider-account-after.txt), [spawn control](captures/425-spawn-control-no-account-after.txt),
[TUI before](captures/425-426-427-tui-before.txt), [TUI after](captures/425-426-427-tui-after.txt),
[TUI positive](captures/tui-positive-after.txt), [TUI config table](captures/tui-config-table-after.txt).
Review: [REVIEW.md](REVIEW.md). Leak scan of the diff, these captures and the tmux wrappers against
both real keys: 0 hits.

Known limit: with `multi_agent` v1 (the default spawn tool), a child of a `zai` parent cannot move to another
provider at all ("spawn_agent cannot switch from provider `zai` …"); the other-provider account reaches spawns through
`multi_agent_v2` `model_provider` (S2), TUI workers and a TUI provider switch.
