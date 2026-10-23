Two of the three re-run criteria now pass (#287 only on Linux) and #288 still fails. The evidence and review are merged as PR #311 (merge commit `886f19c873`), with all checks green.

I built `origin/main` at `b294864916` after waiting for #306 to merge at about 15:57Z, so #306 is included. That covers macOS and Linux developer-accounting builds plus a default Linux build. Every displayed total I checked matched my own recomputation from provider-reported usage, to the micro-dollar. The real GLM 5.2 calls cost about $0.40 at list price.

| Criterion | Verdict | Evidence |
| --- | --- | --- |
| **15 / #287** (clock behind the ledger checkpoint) | **PASS on Linux; NOT VERIFIABLE on macOS** | Linux: turns 58 s and 6 days behind complete and are recorded. The 6-day one appears on the faked day, 2026-10-02, at exactly $0.01963924. A turn 95 days behind completes and is reported in the turn output as an accounting gap. Nothing names Anthropic. macOS: libfaketime is available, but any process with its clock behind real time hangs at startup, even in a fresh home with no ledger. Accounting is ruled out, but no other cause was isolated. |
| **9 / #286** (deleted history) | **PASS on macOS and Linux** | The day page no longer claims "No other conversation recorded requests on this day". It now says "Deleted conversations or subagents sent N request attempts on this day…", with N of 7 (Linux), and 5 and 6 (macOS). These match the deleted sessions' request counts, subagents included. The macOS days were faked *ahead* of real time, and that delete only succeeded on a retry after a partial failure (#308). |
| **8 / #288** (no-usage) | **FAIL, unchanged** | The no-usage conversation's own pages give no next step for its own requests. The only next-step line still names a different provider (`zaiproxy`). Missing usage still gets the same "no price available" reason as a missing price. |
| **#289** (observations only) | **Mostly fixed** | Fixed: fresh-home wording, component lines, week and month buckets reaching today, and request numbering (in order, including a conversation with a subagent). Leftover: in the default Linux build, `/usage requests <yesterday>` opens an "accounting ledger not installed" page with a next step that leads nowhere, and `<tomorrow>` is refused as a future date. |

I commented on #286 to #289 with this evidence and filed two new issues:
- **#308:** with the clock as little as 58 s behind the checkpoint, `corbanu delete --force` exits 1. By then it has already removed the session, so `resume` says "No saved session found", while its spend stays recorded. Reproduced on Linux twice and on macOS. A retry once the clock is ahead again succeeds.
- **#310:** a key supplied as the `ZAI_API_KEY` environment variable is readable by the model's shell commands. My test printed only "PRESENT", never the key itself.

One reviewer, run as specified (code-blind, read-only), concluded "Supported with corrections" with nothing blocking. It recomputed every headline number with no discrepancy, and I applied its corrections. The reviewer did not see four items I added afterwards: the second #308 reproduction, the subagent numbering check, a full recapture of one screen and the environment-variable test.

A key scan of the committed evidence and of all raw scratch logs on both hosts found nothing. The first run's RTX directory had already been removed, so I recreated it at the same path, along with libfaketime. My scratch homes, build target, worktree and branch are removed on both hosts.

For Travis:
- **Decision needed:** should a default build show any accounting page at all? `/usage requests` opening one may conflict with developer-only activation (criterion 13).
- **Fix #308 before wider developer activation.** Either tolerate the clock skew as #299 does for model requests, or make deletion all-or-nothing.
- **#288 is still open** and blocks this criterion.
- **Unrecorded requests are invisible in `/cost`.** When a turn's requests go unrecorded (clock more than 90 days behind), only the turn output says so. That log line also contains test wording ("in this fixture").
- **Criterion 15 on macOS** needs a VM with a clock you can set back.