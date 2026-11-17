# Sync ledger reconciliation — November 17, 2026

Routine internal evidence audit; no product behavior changed. Source inspected at
`5b5c85143884c903cdcceecca276c06f6f9eee94` on
`sync/upstream-rust-v0.167.0`; main comparison
`71c71f1e42409cf6b145dad3dbd33f13aaecb438`. This is landing-plan step 1,
not SG2/SG3 approval or a release qualification.

## Every previously open row

All **96** previously open rows have a dated disposition appended to their
existing note, an accountable owner role and implementation/tracking links.
Historical notes are preserved; the final dated note supersedes their stale
pending-decision/failure counts. No previously resolved row was changed.
`status` stays compatible with existing tooling (`open`/`resolved`);
original `resolution` values are retained. Dated notes distinguish held/tracked
dispositions and preserve the historical owner alongside the accountable role.

| Ledger suffix | Before resolved / open | After resolved / open |
| --- | ---: | ---: |
| `.tsv` | 1 / 29 | 5 / 25 |
| `-resolved.tsv` | 1,006 / 0 | 1,006 / 0 |
| `-s1c.tsv` | 202 / 0 | 202 / 0 |
| `-s3b.tsv` | 547 / 27 | 547 / 27 |
| `-s4a.tsv` | 136 / 0 | 136 / 0 |
| `-compile.tsv` | 187 / 12 | 192 / 7 |
| `-main-merges.tsv` | 636 / 18 | 643 / 11 |
| `-main-merges-2.tsv` | 938 / 10 | 939 / 9 |
| **Total** | **3,653 / 96** | **3,670 / 79** |

The total is **3,749 rows**, excluding previews, not unique defects. The landing
plan's older 3,679-row cut predates 70 subsequently resolved entries. This audit
resolves 17 rows; 10 are held by the real CLI in
[#2274](https://github.com/CorbanuCore/CorbanuTerminal/pull/2274), and 69 remain
open under precise residual inventories:

- [#2562: security decisions and parity evidence](https://github.com/CorbanuCore/CorbanuTerminal/issues/2562), 43 rows; security owner.
- [#2563: core/carry tests and Telegram](https://github.com/CorbanuCore/CorbanuTerminal/issues/2563), 15 rows; core/TUI/Telegram maintainers and integrator as individually named.
- [#2564: API/config/release guidance and obsolete MCP remnants](https://github.com/CorbanuCore/CorbanuTerminal/issues/2564), 11 rows; app-server maintainer and integrator.

Those issues enumerate ledger data ordinals and paths. A composite row stays
open when only some obligations are proved. An obsolete implementation does
not close a replacement security-contract or leftover-file cleanup obligation.
No new undecided D-number was identified among these rows. D522/D523/D525/D530,
D603–D618 and D624/D625 are accepted; missing implementation/evidence remains
open. D679–D685 were accepted as recommended on November 17 at 06:31Z in the
coordinator's `todo.md` and the [public owner-decision record](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6940669510):
D679/D680 #2496; D681 #2499; D682 #2498; D683 #2497; D684 #2500; D685 #2501.
D690 extends copied-key protection in #2522. None is still awaiting a choice.

## Evidence and limits

Each resolved row names its implementation and the current symbol/test inspected.
The security review of this docs PR must explicitly assess those closures and
receive a `corbanu-security` approval naming its exact head. Historical broad
passes are supporting evidence, never substitutes for unresolved boundary reviews.

The current `codex-rs/core` and `codex-rs/tui` trees are unchanged from
`e0391e82dd9ad1db0b256dab753daf886f4a5483`, the source of the fresh full Linux
library results on [#2296](https://github.com/CorbanuCore/CorbanuTerminal/issues/2296)
(4,882 passed, 10 existing ignored) and
[#2312](https://github.com/CorbanuCore/CorbanuTerminal/issues/2312)
(7,492 passed, 9 existing ignored). No Rust build was run for this audit. Integration-specific closure evidence is
separate: #2437 records the two repaired websocket redaction cases in its affected
run ([details](https://github.com/CorbanuCore/CorbanuTerminal/issues/2296#issuecomment-6925700052));
#2442 records original app-server permissions 8/8 plus the new background-grant
negative 1/1 on its exact head, with those test files unchanged here. D603's two
Linux rows remain open for explicit grant-semantics and non-skipped native-run evidence.
The CLI integration exclusions, macOS/native Windows, final SG2 and interactive
qualification remain separate. In particular, idle-lifecycle flakiness retains
its own follow-up despite a passing full library run.

Checks run on the inspected source: Bazel isolation metadata script **2/2**;
Windows nextest grouping script **12/12**; `docs/sprints/check.py` passes
(103 current / 167 archived); release-note `check` passes **152 fragments**, and
`preview` succeeds. These static checks do not establish native runtime parity.

## PF-85-S01–S05 status reconciliation

This PR is restricted to ledgers plus this summary. The actual record/plan
updates below remain an explicit integrator follow-up under #2564; no sprint is
relabelled complete. Main still has all five records draft/unallocated. Sync:

| Sprint | Recorded state | Evidence-backed status and remaining record work |
| --- | --- | --- |
| S01 | in_progress; historical #2101 contributor worktree | #2101 and Tier-1 ports landed; refresh the completed slice evidence and current assignment. #2122/#2051 still require the real CLI; migration/rollback qualification is incomplete. Do not archive. |
| S02 | draft; UNALLOCATED | Core ports and main carries exist; #2296 records a full Linux library pass, not the final full integration/platform gate. Record actual slice work and retained #2274 exclusions, with a coordinator-approved assignment matching the plan. |
| S03 | draft; UNALLOCATED | S3a ports and S3b reviews occurred, but 27 S3b rows remain open and #2562 has exact security residuals. Record those facts; Travis's exact-head SG2 sign-off on the dedicated needs-security-sign-off PR is still required. |
| S04 | draft; UNALLOCATED; early-start authorization recorded | TUI/app-server/exec ports exist; #2312 closes the Linux TUI library inventory. #2274, API docs, release guidance, full CI restoration, true-TUI QA and demos remain. Record those limits. |
| S05 | draft; UNALLOCATED | Audit preparation has begun, not qualification completion. Split pre-merge SG1/SG2/SG3/platform/rollback gates from later final merge and 48-hour installed canary; its 'last main merge #4' wording is stale after main carry #10. |

The active plan already records the integrator's sync worktree and original base
`781b935ff783178f146d02815f8fc01332ae21a4`; that is not authorization to silently
replace every sprint's assignment or bypass unfinished dependencies.

## Release notes, workflows and migration audit

There are **152** fragments. Coverage found: provider/auth restoration in
`2312-sync-provider-agents.md`; security copied-key changes in
`sync-snapshot-brokered-copies.md` and `2502-model-snapshot-copies.md`; D681 in
`sync-cursor-path-recovery.md`; D684 in `1532-skill-catalog-reconnect.md`.
D682 is a test-contract replacement, not a new product capability. Gaps for #2564:

- D679 explicit effort on cold resume, D680 frozen instructions after revert/model change and D683 queue-only completion recipients have no dedicated fragments in their merged PRs; confirm equivalent existing coverage, then add only verified user-facing wording.
- `2015-detached-review-parent-settings.md` still promises removed detached-review behavior. Reconcile the replacement thread/fork contract before writing claims. Document the removed `mcp-server` entry point and upgrade impact once the actual CLI lands.
- D620 doctor desktop-row removal belongs to held #2274. Its PR description still says those rows are kept. Do not claim the removal shipped yet.
- Accepted but unfinished D604–D612/D616–D618/D624 security work must not be described as delivered. Align notes with the eventual verified implementation.
- Preserve main carry #9's intentional omission of a duplicate #2443 mail-recovery fragment: the D659–D664 content is already in `2443-agent-mail-recovery.md`; `2443-spawn-authorization-guidance.md` is a separate change.

Workflow inventory is still **51 sync / 47 main**; **26 shared files differ** at
the recorded SHAs (the landing plan's older cut said 24). The four sync-only files
remain `sync-package-ci.yml`, `python-sdk-build.yml`,
`python-sdk-cli-release.yml`, `rust-release-provisioned-macos.yml`. Integrator
must remove temporary sync CI through `scripts/upstream_sync/ci_mode.py`, review
SDK/publishing/signing triggers and permissions, and restore final full lanes.
This audit changes no workflows and grants no publishing permission.

All **53 files under `codex-rs/state/migrations/`** present at pre-sync base
`781b935ff7` are byte-identical on the inspected head; the **52 files** in that
directory at released `rust-v0.1.48` (`e4e8d9b2232`) are also unchanged. `state/MIGRATIONS.md` records upstream 0047–0062 mapped to
0054–0069; `state/tests/state_db_upgrade.rs` verifies preserved applied checksums
and repeat-open behavior for released/main fixtures. Those tests were inspected,
not rerun. The same migration document explicitly says the old released binary
**cannot reopen an upgraded DB** and rollback requires a backup, conflicting with
the landing plan's reopen gate. Keep S01/S05 blocked on an explicit owner-backed
rollback disposition and actual final-candidate upgrade/restore evidence; do not
claim checksum preservation alone satisfies downgrade compatibility. Scope also
includes seven additions outside that directory: `memory_migrations/0002`, new
queue DB `queue_migrations/0001`–`0002`, and `thread_history_migrations/0005`–`0008`.
These are absent from the pre-sync base and compared main and are not covered by
that state-directory checksum statement. Extend upgrade/backup/restore evidence
and migration documentation to every affected database (#2564).
