# PF-83-S01 campaign 35: independent code-blind execution of F01–F11 (2026-10-05/06)

**Outcome: 3 passed, 1 failed, 7 blocked.**

| Disposition | Cases |
|---|---|
| Passed | F01, F02, F06 |
| Failed | F05 |
| Blocked | F03, F04, F07, F08, F09, F10, F11 |

No case is waived. The isolated executor and the schema-2 receipts exist and were verified. The functional gate itself is not met. See [dispositions](dispositions.md), [evidence checks](review/) and [results.json](results.json).

## Environment

- Sealed guest "macOS 3" (`agent@192.168.64.3`, pinned host key). "macOS 2" stayed stopped.
- Pinned package: `corbanu 0.1.42`, binary `e552180a…e758f6`, manifest `510a7286…4958`, source `b4513f6`.
  - It was staged root-owned and read-only at `/opt/pf83/pkg`.
  - Digests were verified at provisioning and again in every preflight and by every executor.
- Frozen cases: `original-F01-F11.md`, `c97bf701…1726`, unchanged.

## Isolated executor

Provisioned by [harness/provision.sh](harness/provision.sh). Effective root-captured policy: [common/effective-policy.txt](common/effective-policy.txt).

**Accounts and launch**
- Three hidden executor accounts, `pf83x1`–`pf83x3` (uid 602–604, gid 601).
  - No password, not admin, not staff, no SSH.
  - Homes are mode 700.
- The coordinator launches only through `sudo -u LANE /opt/pf83/bin/pf83-launch`, which uses `env_reset`.

**Network**
- Per-uid pf rules allow these accounts only `127.0.0.1:18443`. All other tcp/udp, including loopback services, is dropped.
- `set skip on lo0` was suspended for the campaign so the loopback rules could take effect.
- Drop counters grew to 1947, 2079 and 1460 packets.

**Mediated inference**
- [harness/pf83_mediator.py](harness/pf83_mediator.py) runs as the coordinator. It accepts CONNECT only to `api.z.ai:443` and only the inference paths.
- It terminates TLS with a campaign-local synthetic CA. The product trusts that CA through its supported `CODEX_CA_CERTIFICATE`/`HTTPS_PROXY` path, not a weakened feature.
- It injects the Z.AI key. Executors hold only a placeholder.
- Totals: 1136 forwarded requests, all 200. Denied CONNECTs went to github, raw, codeload, api.github, chatgpt and example.com.

**Removed from executor reach for the campaign**
- Other homes.
- An unrelated PostgreSQL socket.
- The VirtIOFS host share (unmounted).

**Probes**
- [harness/pf83-probe](harness/pf83-probe) checks, with synthetic canaries:
  - repository read/list
  - history and prior findings
  - symlink escape
  - credentials (real key opened without reading)
  - cross-run IPC
  - unapproved network
  - package modification
  - child escape (sudo, grandchild)
- It also runs positive controls: packet digest, binary digest, launch, and PTY input echo.
- It ran three times per attempt: from the launcher, from a tmux child, and by the executor itself. Every attempt had 0 failed probes.

## Execution

- Each attempt used a fresh GLM 5.2 executor (`corbanu exec`, provider `zai`), receiving only:
  - [executor-prompt.md](packet/executor-prompt.md)
  - neutral [navigation](packet/navigation.md)
  - the frozen cases
  - an environment/report assignment ([packet/prompts](packet/prompts))
- The executor drove the product only through [pf83-tui](harness/pf83-tui), which logs every key, text and screen.
- The full executor transcript was captured by the coordinator, outside executor reach.
- 14 attempts:
  - 11 originals.
  - F03, F04 and F10 replayed once each, with unchanged prompts and fresh contexts.
- 4 earlier attempts were aborted by a coordinator harness defect before any case action ([runs-aborted](runs-aborted)).
- Per-attempt receipts: `runs/*/isolation.json`, `probes.json`, `mediator-window.jsonl`.
- Mediator windows cover preflight to executor end and include rows from concurrent lanes, because all lanes shared one mediator; the full log is `common/mediator-campaign.jsonl`.
- F10 attempt 2 met every observed outcome but stays blocked: its variants were chained in one session, restart was a tmux kill, the offered Esc recovery route for pending states was not exercised, and the (c) pre-stop Full Access header claim has no screen. These are deviations without product-authority acceptance.

## Independent evidence review

- Reviewer: `claude-opus-5-5-plan`, high effort, read-only, record directory only. Isolation was instruction-only for source.
- r1 verdict: **fail**, on seven evidence defects. It overturned F07 to blocked and corrected the bases for F04, F05 and F11.
- r2 verdict: **fail**. Six r1 defects were fixed and one partly fixed. Four new representation defects were found; on its basis F10 moved to blocked.
- All r1 and r2 corrections are applied.
- r3 verdict: **pass** ([review/evidence-check-r3.md](review/evidence-check-r3.md)), with six non-blocking follow-ups. Items 2, 4, 5 and 6 were applied afterwards as wording/receipt additions only; item 1 (PF83-DEF-015 lives outside this record) and item 3 (model binding comes from the launch command in the ledger) remain as stated.
- The review ledger is at [review-ledger.md](review-ledger.md). The budget extension is recorded there for manager confirmation.

## Product findings (factual, pinned 0.1.42)

1. **Declined commands render as run.** A declined or cancelled approval shows "✗ You canceled…" followed by "• Ran … (no output)". Seen in F02, F03, F04, F05, F07, F08 and F09. Fixtures show no effect. This is tracked in the sprint record as PF83-DEF-015; that record states a later source correction, which this campaign did not test.
2. **The approval modal treats typed text as option shortcuts.** Typing `/permissions` approved a pending request with "don't ask again" scope (F05).
3. **"Enable full access?" defaults to Cancel.** Enter silently returns to the picker (F03 attempt 2).
4. **Mid-turn input is steered into the running turn.** Text submitted during an active turn runs under the old authority (F04 attempt 1). The probe used was non-discriminating.
5. **The `zai-anthropic` route fails every turn.** The error is "model glm-5.2 has no catalogued maximum output token limit".
6. **GLM 5.2 refused out-of-workspace writes even under Full Access** (F03 attempt 1, verbatim).

## Gaps and limits

- **Process visibility:** other accounts' process command lines are visible, and the lanes ran concurrently. F03 attempt 2 ran `ps aux`.
- **Executor PATH:** the executor shell PATH includes system and Homebrew paths, because `bash -lc` runs path_helper.
- **Driver evidence location:** driver evidence files sit in executor-writable space. They were cross-checked against the coordinator-captured transcripts.
- **Designer provenance:** unattested, because no designer packet or screenshots were ever supplied. `check.py` therefore cannot pass on design fields alone.
- **OpenAI route:** not provisioned.

## Restore

Ran [harness/restore.sh](harness/restore.sh); output is in [common/restore-record.txt](common/restore-record.txt).

**Reverted**
- Original anchor restored and `lo0 (skip)` back.
- No per-user rules, accounts, group, sudoers drop-in or `/opt/pf83` remain.
- Home, socket and moved-script modes are back to original.
- The VirtIOFS share is remounted (automount tag).
- Mediator, logger and canaries are removed, as are the campaign directories.
- The guest egress gate was re-measured after restore: `api.z.ai` returns 301; github, raw and codeload get curl rc 6. This is appended to `common/restore-record.txt`.

**Residual**
- Empty SIP-protected `/var/folders` skeleton directories for the deleted uids remain.
- Pre-existing prior-increment directories in `agent`'s home were left untouched.
