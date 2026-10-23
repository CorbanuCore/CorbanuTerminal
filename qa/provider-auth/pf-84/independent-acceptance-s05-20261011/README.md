# PF-84-S05 independent source-blind functional acceptance (2026-10-11)

Scope: [PF-84-S05](../../../../docs/sprints/current/unified-provider-auth/pf-84-s05-per-account-usage-attribution.md),
per-account usage and rate-limit attribution behind `named_accounts`. Gate under test:
[s05-gate.md](../s05-gate.md). Independent review and executor dispositions: [REVIEW.md](REVIEW.md).

## Method

- **Source-blind, not the milestone code-blind VM gate.** I read only these:
  - the S05 sprint record (including its code-boundary paths);
  - the plan's PF-84 section, `s05-gate.md` and the demo index;
  - the S01/S02 independent acceptance README and REVIEW (for method);
  - `docs/provider-accounts.md`, `docs/authentication.md`, `docs/config.md` and `docs/tmuxHarness.md`;
  - `sec-common.md` and `--help` output.

  I did not read `codex-rs/**`, `scripts/**`, PR diffs, worker logs, `qa/provider-auth/pf-84/s05/` or reviews. I found the
  state DB tables only by opening the run homes with `sqlite3`. This run does not tick the sprint's "code-blind handoff
  checker" box.
- **Candidate.** `origin/main` at `d93f5e639d` (merge of #469, S05). Three builds were made in my own worktree, each with
  its own target dir and toolchain 1.95.0:
  - CAND: debug, `--features codex-core/developer-accounting,codex-tui/developer-accounting`;
  - REL: `cargo build --release` with default features;
  - BASE: the pre-S05 baseline `bddf6292c4` (the merge's first parent), debug with developer-accounting.
- **Safety.**
  - Every candidate and baseline process ran with a minimal environment ([`cleanexec.py`](scripts/cleanexec.py)), a
    disposable `CORBANU_HOME`, a fake `HOME` and `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
  - Real keys came only from the installed signed helper `/Users/Neo/.local/bin/corbanu.bak-20261010` (`corbanu 0.1.48`),
    named by the owner's task brief for this run. It ran with `vault auth-helper <label>` inside the consuming command.
    The helper reads the operator vault (and therefore its keychain item), so "no keychain" applies to the candidate and
    baseline only.
  - Keys went to stdin (`account add`, `login --with-api-key`) or into a process environment, never into argv. Keyed
    TUIs ran on dedicated tmux servers started with the key in their environment. Every keyed server was killed after
    its session (`kill-server`); none were left running.
  - `PATH` held the candidate as `corbanu`, so Claude Plan's token command was the candidate (see #414).
  - Scratch homes, logs and target dirs were deleted after the run. Scripts hard-code this host's paths
    ([`env.sh`](scripts/env.sh)).
- **Providers.**
  - Z.AI `glm-5.3-flash` (`provider/zai_api_key`): `default`, `work` (the same real key under a second name), `fake`.
  - Kimi `kimi-k3` on `kimi-code`: `default`, `alt` (the real key under a second name).
  - Claude Plan `claude-opus-5-5-plan`: named `cp` (real token) and `cpfake`. The default Claude login was never used.
  - OpenAI `gpt-5.4` (API-key login, default only; named OpenAI accounts are S06).
- **Provider-reported usage, three sources.**
  1. **Wire, built-in route (h6).** [`mitmmock.py`](scripts/mitmmock.py) runs in forwarding mode as an `HTTPS_PROXY`
     with a throwaway CA (`SSL_CERT_FILE`). It intercepts only `api.z.ai` (`MITM_HOSTS`), relays to the real Z.AI and
     logs every reply's `usage`. It keys each request by its `thread-id` header and records an HMAC of the bearer under
     a per-run random salt. Replies the client abandons are logged too (status, bytes, usage seen).

     The 4 TLS failures per process in early probes were the candidate's other connections (`github.com`,
     `chatgpt.com`). They are now refused and logged as `connect_refused`.
  2. **Wire, custom provider (h2).** [`usageproxy.py`](scripts/usageproxy.py) is a plain forward proxy, used as the
     custom provider `zproxy`. Its early log used an unsalted bearer hash; the committed copy replaces it with
     pseudonyms.
  3. **Rollout (h1, hc).** Each thread's last cumulative `token_count`. The candidate writes this itself, so it is a
     self-consistency check, not an independent one.

  [`reconcile.py`](scripts/reconcile.py) joins the sources with the ledger. It normalises NativeAnthropic input
  (input + cache read + cache write) and can exclude OpenAI websocket prewarm attempts, which rollouts don't count.
  Both adjustments were added after a first pass printed DIFF; the raw output is kept above the adjusted run in
  [providers](captures/providers.txt).
- **Throttle probes.** These use synthetic canary keys only: `mitmmock.py` in mock mode answers as `api.z.ai` itself
  (429, slow, or `pong` per key), driven by [`throttle.sh`](scripts/throttle.sh), [`lease.sh`](scripts/lease.sh) and
  the Retry-After probe.
- **TUI.** I drove the TUI through private tmux sockets per `docs/tmuxHarness.md`: text and Enter are sent separately,
  and every session ends with `/exit`. `send_line` verifies the composer and aborts without Enter if the text never
  shows. Before that fix, one flag-off candidate session lost its first word; it was rerun. Launch lines are in
  [tui-invocations](captures/tui-invocations.txt) and in each capture's header.

## Verdicts

| # | Criterion | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | An accounted attempt on a named account records `account: {name, fingerprint}` (12 hex) through ledger format 4. | PASS | Fingerprints: h1 Z.AI `work` `9ef7707e5fe8`, Kimi `alt` `5a4334eda677`, `fake` `c18cc965da52`; hc Claude Plan `cp` `bbf4a6e822ee`; h6 `work` `96829868dc8d`. The format rows become `[2, 3, 4]` on the first named attempt ([h1](captures/h1-reconcile-final.txt), [providers](captures/providers.txt), [h6](captures/h6-wire-reconcile.txt), [migration](captures/migration.txt)). `/providers` shows the same fingerprint (`fp f8c1d2a7de8d` for h6 `fake`). |
| 2 | The fingerprint is salted per home, holds no credential, and is stable per key. | PASS | The same key as `zai/same` gives `b68b222e8d72` in one home and `11069f57a072` in the other. Replacing the key gives `33702e7de6df`; restoring it gives `b68b…` again. 0 files hold the canary ([fingerprint](captures/fingerprint.txt)). |
| 3 | Default-account attempts omit the field, keep their bytes, read as `default`, and never upgrade the ledger. | PASS | Home hm (baseline-made, format 1, no format tables): after candidate runs with the flag off, then on, then `--account default`, there are still no format tables and 0 account fields. The baseline rows' sha256 is unchanged (`359efd1b…`), and `config.toml` and the vault are untouched. Default rows show as `account default` in mixed views ([migration](captures/migration.txt), [h1 /cost](captures/tui-h1-session-work.txt)). |
| 4 | Older ledgers still read; an older build handles a format-4 ledger. | PASS | The candidate reads baseline format-1 ledgers: `/cost` counts the baseline's requests ([default-only view](captures/tui-default-only-view.txt)). In home hb, the first named attempt moves the ledger to format 4 and the baseline rows keep their exact bytes (`2ae1bb24…`). The baseline then turns accounting off by name ("cost ledger was written by a newer Corbanu build … /cost does not include them") and still answers `pong`, also on `--account work` ([concurrent](captures/vault-write-concurrent.txt)). |
| 5a | Two accounts of one provider reconcile **exactly** with **independently** observed provider usage. | PASS | **Built-in Z.AI (h6, wire):** all 9 threads match exactly, as `PROXY vs LEDGER exact per thread: ALL MATCH` ([reconcile](captures/h6-wire-reconcile.txt), [requests](captures/h6-wire-requests.jsonl)). The threads are: exec default and work 7,425 each; fake 401 at 0; the coordinator's 6 requests at 39,762, including one reply aborted with Esc (`200/client-aborted`, no usage relayed before the abort, so the ledger has no usage either); spawned `work` 4,622; spawned `fake` 401; the `work` session with `/compact` plus `exec resume`, 4 requests at 27,233; and the pre-switch default session 7,614 with the switched `work` session 7,649. **Custom route (h2, wire):** tokens match on every thread (work 4,616 + 4,616 = 9,232; default 9,420; fake 0). The script prints `MISMATCH` only on the coordinator's attempt count, 10 vs 9: the Esc-interrupted request, which that earlier proxy version did not log ([reconcile](captures/h2-proxy-reconcile.txt)). What the provider billed for an aborted reply is unknowable from the client. |
| 5b | The ledger equals the rollouts on built-in routes (self-consistency). | PASS | h1: all 10 threads match, including Kimi default 11,421 and alt 11,357, Z.AI work 27,072 and default 46,913 ([h1](captures/h1-reconcile-final.txt)). hc: Claude Plan `cp` 8,492 (input 4 + cache write 8,484 + output 4), `cpfake` 0, OpenAI 10,934 with the 9,465-token prewarm excluded ([providers](captures/providers.txt)). |
| 6 | `/cost` groups rows, pages and request links by provider, account and model: this conversation (coordinator and spawned agents) and other conversations (`exec` workers) in the same home. | PASS | h1: the coordinator is on `default`, and the spawned agents are `(… · account work)` → pong and `(… · account fake)` → 401 ([spawn](captures/tui-h1-spawn.txt)). This conversation: default 5 / 39,488, fake 1 / not reported, work 1 / 4,616 ([nav](captures/tui-h1-cost-overview-nav.txt)). The Open list has per-account provider pages (`Account: work`) and request links. A technical page shows `Account: fake (fingerprint c18cc965da52)` ([page](captures/tui-h1-cost-page-work.txt), [request](captures/tui-h1-cost-request-fake-tech.txt)). Final h1 state on a `work` session: this conversation work 7,614; others default 46,913, work 19,458 (7,425 + 4,616 + 7,417), Kimi alt and default, fake 2 ([session](captures/tui-h1-session-work.txt)). h6: the same pattern ([default](captures/tui-h6-cost-default.txt), [work](captures/tui-h6-cost-work.txt)). The top rows don't open on Enter even though the footer says "Enter open"; the pages are under "Open" (UX note). |
| 7 | A fake-key account records the failure and no spend. | PASS (with #475) | `exec` and spawn on `fake` get a 401 even though a valid default key is in the environment. Each records 1 request with no usage, no estimate and no spend. `/cost` shows it as "usage incomplete — check the bill", the totals as "at least $…", and the 401 itself is not shown ([#475](https://github.com/CorbanuCore/CorbanuTerminal/issues/475), low). |
| 8 | Throttle state (429 cooldown, request lease) is per (provider, account); B's 429 never blocks A. | **PARTIAL**: keying PASS, behaviour NOT VERIFIABLE | Rows are keyed `account:<name>` next to `env:ZAI_API_KEY:<hash>` for Z.AI, Kimi and Claude Plan, in debug and release ([throttle](captures/throttle-cand-zai.txt), [release](captures/release-build.txt)). After `b`'s 429s, `a` and default answered at once. But no cooldown or lease is ever recorded, for any key, provider, path or build, the baseline included ([baseline vs candidate](captures/throttle-retry-after-base-vs-cand.txt), [lease](captures/lease-cand.txt)). `Retry-After: 120` is ignored: 5 retries within about 3.5 s. So B's 429 doesn't block even B. Separately, the plan keys leases by account **fingerprint**, but S05 keys by **name** (gate note 5); that is a product decision for Travis ([#476](https://github.com/CorbanuCore/CorbanuTerminal/issues/476), medium). |
| 9 | `/status` names the session's account. | PASS | `Z.AI - https://api.z.ai/api/paas/v4 · account default` / `· account work` (debug and release), and `Z.AI via QA proxy … · account default` ([h1](captures/tui-h1-status.txt), [release](captures/tui-release-work.txt), [h2](captures/tui-h2-status.txt)). With the flag off there is no suffix. The footer's `via zai default` is the reasoning effort, not the account (S01/S02 acceptance, `tui-status-line-effort-not-account.txt`). |
| 10 | Session switch, resume and compaction keep attribution. | PASS | `/providers` → `Z.AI · work` → "Use for this session" starts a **new** session ("requests to zai use account `work`"). `/status` shows `account work`, and `/cost` puts the new conversation under `work` and the earlier one under default ([session](captures/tui-h6-switch-session.txt), [/cost](captures/tui-h6-switch-cost.txt)). No user path rebinds a running thread. On a `work` session, `/compact` records a `compact:` attempt on `work`, and `exec resume <thread>` without `--account` records its attempt on `work` ([resume](captures/h6-resume.txt)). All of these match the wire (row 5a). |
| 11 | Flag off: behaviour unchanged. | PASS | Candidate and baseline match on identical homes. `account add` is refused, `exec` answers `pong`, and `--account work` is refused with the same text. There is no format-4 table, no account field and only the `env:` throttle key. `/status` and `/cost` look the same, with no account naming. The candidate's TUI session was rerun after the dropped-keystroke fix ([flag-off](captures/flag-off.txt), [cand](captures/tui-flagoff-cand.txt), [base](captures/tui-flagoff-base.txt)). |
| 12 | An existing single-account home carries over untouched. | PASS | hm: see row 3. `config.toml` hash unchanged, no vault created, original ledger rows byte-identical, and the baseline still works on it afterwards. |
| 13 | A missing or wrong account fails closed and never falls back; each account's own key is sent. | PASS | `exec --account ghost` fails at thread start (Z.AI, zproxy, release). The TUI `--account ghost` exits 1 before any UI. A spawn on `ghost` is refused ("configured: default, fake, work") and no attempt names `ghost` ([ghost](captures/ghost.txt), [workers](captures/exec-h1-workers.txt)). Per-key proof comes from the mock, which saw each canary under its own label: `b` → 429, `a` and `default` → 200 ([mock log](captures/mock-cand-zai.jsonl), [lease log](captures/lease-cand.jsonl)). On the wire, `fake`'s bearer differs from the default/work key ([h6 requests](captures/h6-wire-requests.jsonl)). Named OpenAI is refused. |
| 14 | No credential value in the state DB, logs or rollouts. | PASS | [Leak scan](captures/leak-scan.txt): real keys and synthetic values checked across every scratch file and this directory. Real-key hits: only the OpenAI key in `auth.json`, where `login --with-api-key` has always stored it. Synthetic hits: only my own scripts. No export command was exercised, so the sprint's "exports" is not claimed. |
| 15 | Release build matches the docs. | PARTIAL | The S05 limits hold. REL: `/status` shows `· account work`; `/cost` is "Unrecognized command"; there is no ledger table; per-account throttle rows exist; ghost fails closed ([release](captures/release-build.txt), [tui](captures/tui-release-work.txt)). These match the gate's "Known limits", but `docs/provider-accounts.md` says nothing about usage. Also, the docs say "a rejected key names the account and its `corbanu account add` recovery". The TUI does ([fake](captures/tui-h1-session-fake.txt)); `exec`, in release and debug, prints a bare 401 ([#480](https://github.com/CorbanuCore/CorbanuTerminal/issues/480), low, S03 area). |
| 16 | Other providers: Kimi, Claude Plan, OpenAI. | PASS | Kimi `alt`: rows 5b and 6. Claude Plan `cp` answers pong, with `account:cp`, its own fingerprint, and `/cost` "Claude Plan · account cp … 8,492 tokens". `cpfake` gets a 401. OpenAI default is named `account default` in a mixed view ([/cost](captures/tui-providers-cost.txt)). |
| 17 | The fingerprint read is limited to 2 s, with a name-only fallback. | NOT VERIFIABLE | It can't be induced from outside without slowing the vault the key itself is read from. Needs a product-supplied fault hook. |
| 18 | Rate-limit snapshots carry the account. | N/A (S06) | The gate says `/status` snapshots come only from ChatGPT sign-ins, which get named accounts in S06. Travis must accept this deferral. |
| 19 | Requests reported by another client (Claude panes bridge) record no account. | NOT VERIFIABLE | The Claude panes bridge was not available in this setup (no default Claude Code login, by the safety rule). Needs a run with a Claude Code login and panes. |

### Other findings

- [#474](https://github.com/CorbanuCore/CorbanuTerminal/issues/474) (low): on a vault created before S05, the first
  named-account request rewrites the vault once (621 → 737 bytes). The gate says the read never creates a salt.
  - Two concurrent first uses gave one consistent fingerprint, and the baseline still reads the vault afterwards
    ([capture](captures/vault-write-on-first-named-use.txt), [concurrent](captures/vault-write-concurrent.txt)).
- [#481](https://github.com/CorbanuCore/CorbanuTerminal/issues/481) (low, S03): the spawn wait reports a child that
  failed with 401 as "Completed".
- [#482](https://github.com/CorbanuCore/CorbanuTerminal/issues/482) (low, pre-existing): the default account's throttle
  key is an unsalted sha256 prefix of the API key, so it can be correlated across machines.

## Recommendation

**Limited acceptance of S05 behind the default-off flag.** It is conditional on Travis accepting these dispositions:
- row 8 (throttle behaviour unverifiable; name-keyed rather than fingerprint-keyed, #476);
- rows 17–19 (not verifiable or deferred);
- the follow-ups #474, #475, #480, #481 and #482.

Usage attribution itself is solid:
- every count reconciles exactly with what Z.AI sent on the wire, for two accounts on the built-in route, including
  spawned agents, compaction, resume and a switched session;
- the ledger migration is byte-exact;
- flag-off and existing homes are unchanged.

Before relying on per-account rate limiting, #476 must be explained or fixed. The lane owner should qualify the
sprint's ticked throttle item until then.

## Reproduce

Scripts are in [`scripts/`](scripts/). They need these builds:

- `…/.codex-work/targets/pf84-s05-indep-{acc,base,rel}-20261011` (build commands under Method);
- a throwaway CA in `$S/certs`, created with `openssl`: a CA plus a leaf for `api.z.ai` and the other API hosts.

Run order:

1. `exec-workers.sh` (h1)
2. the h2 proxy run (`usageproxy.py proxy` with `UPSTREAM=https://api.z.ai/api/paas/v4`)
3. `tui-h6.sh` (with `mitmmock.py` in `MITM_MODE=proxy`)
4. `throttle.sh`, `lease.sh`
5. `migration.sh`, `flagoff.sh`, `tui-snap.sh`
6. `providers.sh`, `fingerprint.sh`
7. `leakscan.py`

The TUI steps for h1/h2 (spawn, `/cost` navigation) were typed interactively with `tmuxlib.sh`/`pagedump.sh`. Their
prompts are in the captures.
