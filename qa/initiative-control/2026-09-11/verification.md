# Workstream-selection verification — 2026-09-11

Class: planning/lifecycle records and existing native setup; no Corbanu runtime
code changed. Authority: Travis's explicit PF-13/accounting/Task Node selection.
Product linkage remains in the three active plans. No merge or release requested.

Source: recovery base `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546` plus scoped
uncommitted management changes. Main and remote PF-13 pins are recorded in
`docs/plans/workstreams-2026-09-11.md`; main and other workers were not changed.

Final edited tree checks before publication:

- `python3 docs/plans/check.py`: PASS, 3/3 active plans.
- `python3 docs/sprints/check.py`: PASS, 110 current / 98 archived.
- Plan unit tests: PASS, 5 tests.
- Sprint unit tests: PASS, 19 tests.
- Initiative-control unit tests: PASS, 33 tests.
- `git diff --check`: PASS.
- Fable 5.1 through the existing Corbanu/tmux Autoreview wrapper, high effort:
  two P3 metadata findings accepted and fixed (remaining accounting owner fields,
  deferred-plan link label); final structured review clean, patch correct.
  Review used an isolated before/after snapshot of this turn, not all inherited
  dirty changes. Receipts are private under the task's `tasknode-setup.gThcI9`
  directory; no credentials or native-link URL were sent in the review bundle.

Native interaction: installed Corbanu 0.1.36 in a real PTY; skipped optional
upgrade without changing the installed package, opened `/tasknode` with literal
text and separate Enter, chose native GitHub linking. Safari reached the account
authorization screen. Authorization has not been accepted by the manager.
No wallet secret was read and no wallet transaction was initiated.

Human acceptance, account equivalence, current progress-contract verification,
remote publisher credentials, task IDs, enrollment and first live progress
receipt remain pending. Automatic progress posting is OFF. Existing runtime
profile fixes need qualification before worker access. No new implementation
agent has been dispatched; selected initiatives are not running-worker claims.

True-TUI feature/release acceptance and TensorCash/Isometric qualification are
not performed by this docs-only change. Existing product/release gates remain.

## Served publication check

Export digest: `6ce44a5de1d7b3872d1dd739e44a6bafec5be4948752b9566b271c2ae0b8ddee`;
268 allowlisted files. Alex-server activation succeeded. Served generation
`build-tb5qi739`, published `2026-09-11T08:23:51+00:00`, reports healthy with
zero warnings. A read-only HTTP check verified the exact three initiative
headings in order, all three human-plan entries and 106 linked HTML documents
returning HTTP 200. Served writeback state says disabled, enrollment unverified
and zero deliveries. No progress event was sent.

The local 8769 tunnel was disconnected at verification. It was not restarted:
the user deferred dashboard-access work. Publication was checked directly over
the existing authenticated SSH path to the server's loopback listener instead.

## Follow-up: account linking, 08:29 UTC

Travis approved continuing as IridiumMaster. The earlier browser callback had
succeeded but its temporary request had expired before terminal collection.
The same installed Corbanu TUI recovered through `/tasknode link`, the approved
GitHub account selection, and `/tasknode status`. A separate `corbanu tasknode
link status --json` invocation confirmed a valid active session, no pending link,
and the expected account. Read-only native status/task-list calls succeeded;
Safari showed the same account ID, handle and wallet association.

No seed or password was needed. The account's two accepted tasks do not match
the three selected workstreams. No task was created, mapped, accepted, modified,
submitted or completed. No credentials were provisioned remotely. Posting and
Campaign Tracker recording remain OFF; task targets and publisher acceptance
remain open. This is operator setup of existing behavior, not a runtime code
change or a release-qualification claim.

## Follow-up: dedicated tasks and beta sprint planning

Travis subsequently authorized creating three workstream tasks and planning a
public Corbanu Desktop beta-testing program. Native Corbanu 0.1.36 created three
personal task requests once each. `requests show` verified all three reached
`proposed` / `offer_published`; `task show` confirmed generated IDs, exact titles,
coordination-only descriptions and Proposed status. [Receipts and mappings](../../../docs/plans/tasknode-workstream-tasks-2026-09-11.md).
The two old Accepted tasks were untouched. No task acceptance, submission,
verification, reward, progress delivery or public beta assignment was performed.

Added PF-79-S01/S02 as drafts within active workstream 3: isolated Desktop beta
channel/test contract, then a bounded recurring public-testing pilot. Pinned
first-party Task Node source distinguishes personal requests from scoped public
Hive/network allocations; live board permissions and Desktop source remain gates.
No branch/release or new scheduler was created. Main and other workers unchanged.

Plan/sprint checks pass at 3/3 active and 112 current / 98 archived. All 57 tests
pass (5 plan, 19 sprint, 33 control); `git diff --check` passes. Draft specifications
are not runtime, Desktop GUI or human beta acceptance evidence. Account secrets,
private transcript and raw task metadata are excluded from planning/review export.
Initial Fable 5.1 high review found one P3 dangling closeout-reference note;
replaced it with this explicit status. A local path audit also corrected S02's
renderer reference to existing `control.py::collect/safe_markdown`. Both are
in-scope documentation fixes. Final Fable review and served-projection verification
for this amendment were pending at that review checkpoint.

### Final amendment review and projection receipt

Fable 5.1 (`claude-fable-5-1-plan`, high) completed through the existing Corbanu
tmux Autoreview harness: **clean, no accepted/actionable findings; patch correct**.
Frozen review scope: 13 documentation/manager-config files, 489 insertions and
21 deletions before the bounded fixes; no runtime changes. Isolated before/after
receipts remain private in `tasknode-beta.Rf2AJJ`; no whole dirty-branch review,
credential or transcript export. Review stopped after the clean corrected pass.

Alex-server generation `build-6te4_aqt`, collected `2026-09-11T08:53:32+00:00`,
published `2026-09-11T08:53:36+00:00`: health OK, zero warnings, 256 rendered
documents and 109 checked internal links returning HTTP 200. Both PF-79 sprint
pages, beta contract and task-target record were verified from the served HTTP
surface; the three active workstreams remain in order. Export: 272 allowlisted
files, digest `e31a5b2560f4007c7d1115a3f70510e7887ddca50967c5b72988a1dcb1448cfa`.

Remote config has eight sprint mappings, `enabled:false`; enrollment unverified
and zero deliveries. Four prepared local goal events were pending with zero
attempts at that check. These include earlier run observations and must be
reviewed as a whole batch before first delivery; no live progress claim follows
from enqueue. No record was discarded or force-retried. The local 8769 tunnel
remains deferred. This is a verified private projection, not a public beta launch.

Final receipt-only status sync: `build-k62rwp42`, collected
`2026-09-11T08:56:34+00:00`, published `2026-09-11T08:56:37+00:00`; health OK,
zero warnings, 256 documents and 109 internal links reverified. Export digest:
`f5e4f342aa81c65c94f67f15cd79d324cdeeb047630e24e12fe52581717ac9c5`.
The additional final manager observation brings the local outbox to five pending
goal events; enrollment remains unverified, posting OFF and deliveries zero.
