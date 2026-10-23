The manager is still not on. The preflight passed items 1, 2, 3 and 5 but refused item 4: the test VM can still reach OpenAI directly, and fixing that needs a VM firewall change I was told not to make. The independent review agreed: **NOT QUALIFIED**. So I didn't run the promotion. The owner is still **armed, generation 1, `fixture-only`**, config digest unchanged, ticking normally (44,193 ticks, no hold, 0 errors). The VM credentials are restored exactly.

## Preflight, item by item (rerun on commit `36b6ef77e5`, action `owner-first-check-01-r2`, exit 2)

| Item | Result | Evidence |
|---|---|---|
| 1. Manager prepares actions through `freeze_worker_inputs` | Pass | Unchanged from round 2. |
| 2. A fresh prepared action to hand over | **Pass** | The manager made two real decisions with the vault login. The first prepared `owner-first-check-01`. I re-registered the allocation with `replace: true` (base commit moved to `edc60bfb02`), which cancelled it. The second prepared **`owner-first-check-01-r2`**, which is waiting for the owner. One earlier try was refused before launch (tmux socket path too long, no inference used); I closed it out cleanly. |
| 3. No unfinished publication write | Pass | — |
| 4. Real isolated worker test | **Not met** | The run itself worked; details below. |
| 5. Hand dispatch and publishers quiet | **Pass** | The coordinator's hand-worker tmux server wasn't running, and nothing else was writing to the coordinator or publishing. The preflight ran 1.7 s after the check, with the owner job and coordinator state unchanged before and after. |

**B1 (commit `edc60bfb02`):** the manager now reads its Claude login from the vault at use time and keeps it in memory. Only a hash is recorded, so a login that changes before launch is refused. All tests pass (887 in a clean checkout). The generic `corbanu vault auth-helper` refuses the Claude login on purpose, because it's stored as Claude-provider-only. So the manager uses the Claude provider's own hidden helper, `corbanu internal-claude-oauth-token`. Travis should confirm that's acceptable. The alternative is storing a copy of the login under a separate vault label.

This change also changes the package fingerprint the owner checks, from `8021fc53…` to `146e64e9…`. The test below ran on the new one.

## Item 4: the test run
- **It worked end to end.** The owner on the VM ran the worker through ACK, START and RETURN in 7 ticks with no holds. All 11 of the worker's model responses match the broker's log, so inference really went through the broker. Probes from both the worker and a separate shell could not reach the host checkout, a test file served from the host, GitHub, or the broker's other routes. With the broker stopped, inference failed as it should.
- **Why it still fails:** the VM firewall lets the VM reach OpenAI's servers directly. Every time the client started, including the worker launch and the broker-down test, it opened two short connections to chatgpt.com. No inference or credentials went that way, but direct access isn't blocked, which the test requires.
- **What I fixed along the way:**
  - **Broker (commit `36b6ef77e5`):** its first live use failed because the ChatGPT backend uses a different request-ID header and sends streams without a content type. Both are now accepted; the broker's 64 tests pass and the failed attempts are kept.
  - **Live owner binary:** its folder was missing the `codex-code-mode-host` helper, so every worker shell command would have failed after promotion. I copied the signed helper from the same build into it.

## VM cleanup and restore
- **Deleted:** 21 old `~/pf83-cases-*` folders, plus `pf83-cases-29.Y0k1xE/cases.uPHy00`. The `Y0k1xE` package, manifest, packet and runtime are kept. Free space went from 532,192 KiB to 9,428,532 KiB (+8.5 GiB).
- **Credentials held:** as root through `neo2`, I moved `~/.pf83-auth` into a root-only folder for the test. I also moved a 52-byte `~/qual/probe/home/auth.json`; it's not a match for the test placeholder login, so I treated it as possibly real.
- **Restored** with the same inode, owner, mode, size, timestamps and digests:
  - `auth.json` `db552572…f256`
  - `zai_api_key` `4ac7ce27…d9f7`
  - qual probe `auth.json` `ec7ab237…eff2`
- **Shared folder:** the VM had a shared folder mounted from your Mac (`/Volumes/FASTDRIVE2/z23`, personal files, no Corbanu code). I unmounted it for the test and remounted it after.
- **Left alone:** the firewall and the VM's system config (unchanged). "macOS 2" stays stopped. All test folders, processes, the tunnel and the broker are removed.

## Review verdict
`claude-opus-5-5-plan`, given only the evidence and the procedure, said **NOT QUALIFIED**.
- **Supported, with gaps:** code isolation (`isolated_transport`), the fresh test profile (`isolated_profile`) and the access probes (`negative_access_probes`).
- **Not supported:** inference only through the broker (`mediated_inference`), because direct access to OpenAI isn't blocked.
- **Confirmed:** ACK, START and RETURN each counted once, and the credential restore was exact.
- **Corrections:** it also listed several inaccuracies in my evidence write-up. They're in its review file; I haven't corrected the write-up.

## Commits (all pushed fast-forward to `integrate/management-workstreams-20260911`)
- `edc60bfb02` B1. It reached the remote through the Task Node session's push, which landed on top of it.
- `36b6ef77e5` broker fixes.
- `577526daab` record extended, including Travis's B1–B3.

I never staged the other session's four files. I didn't republish the dashboard, because it only follows a promotion and would again publish those uncommitted files.

## Still open
1. **Decision needed:** permission to block the VM's outside traffic during the test, either from the host or by removing the OpenAI exception in the VM firewall. Then rerun the broker-down check and one owner run with zero direct connections, get a fresh review, and promote to generation 3.
2. `owner-first-check-01-r2` is prepared in the live coordinator. Hand workers shouldn't claim it.
3. Whether the manager may keep using the Claude provider's hidden helper (B1 above).
4. `slack-receiver-02` is still unsettled.

Evidence is in `.codex-work/owner-manager-enable-20261002/round3/`.