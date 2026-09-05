# PF20S03 review ledger

New genuine track budget: maximum five invocations. PF30S01 remains exhausted
five/five and is not re-reviewed here. Do not reset either ledger.

## Review1 — Astra High

- Frozen HEAD `3c3e6d7a6730d9c341f90394ab2df4dcc6e0104f`, base
  `601602fa7e53fcb5b41753a0b3607addd45d4415`; app Codex0.153.1.
- Structured helper engine `codex`, model `gpt-6-astra`, thinking `high`, branch
  mode, `review-scope.md`; exact text and JSON retained alongside this file.
- Exit1, one P2: blocking native connect could wait on a full listener backlog
  before authenticated frame deadlines begin. This is not a clean review.
- Verified the real `connect_system` path and Linux's `unix_stream_connect`
  backlog branch in [kernel source](https://raw.githubusercontent.com/torvalds/linux/master/net/unix/af_unix.c).
- Classification: in-scope blocker in newly added bounded native bootstrap,
  not a new deployment/API/owner-boundary requirement.
- Remediation: private nonblocking socket establishment fails closed immediately
  if not connected; no pending-connect retry/reconnect or public constructor
  changes. Existing fixed-path/root-peer checks remain. A real saturated-backlog
  fixture and existing post-exec child fixture exercise failure and success.
- Scoped remedy `8d6967179` passed leaf18/18 and Core17/17, scoped fix and full
  formatting on RTX before committing/pushing. Public APIs and deployment scope
  are unchanged. Exact patch was transferred to the disposable remote for proof
  before pushing the review-triggered correction.
- Immutable remediated CLI hash `42fe2d0168a4f9079f6faa4f4a7b6aa41deabe6f1322d0e921b8c40b4cfc4076` also passed profiles2/2 and same-home restart. Correct-cwd profiles were refreshed under the lock and passed again; exact final captures are committed. No production installation was performed.

## Review2 — Fable5.1 High through Corbanu/private TMUX

- Frozen HEAD `84cad22fa9d88482a57e675900ee86e92e276ac4`, same base; final code
  `8d6967179`. Structured helper engine `codex`, approved Corbanu wrapper
  `.codex-work/security-round5/review-fable-high`, model
  `claude-fable-5-1-plan`, thinking `high`, branch mode and `review-scope.md`.
- Executed in private TMUX socket/session; exact JSON/text and completed exit1
  pane are retained. This is structured Corbanu exec review inside TMUX, not an
  interactive-chat test. The fixture was closed only after verdict/capture.
- Overall verdict **patch is correct**, no code blockers; Astra P2 correction
  verified. Helper exit1 reflects two nonblocking P3 notes, not a clean run.
- P3 coordinator ledger/progress text: verified stale in this branch's inherited
  coordination row and HTML. This is coordinator-owned integration follow-up;
  exact paths/lines were sent to root. Do not mutate those shared surfaces from
  this lane or misreport them as already corrected. The final merged guide must
  report2/5, Astra P2 fixed, Fable no code blockers/two nonblocking notes.
- P3 TMPDIR portability: the required ext-family/XFS gate is intentional and
  already recorded in QA. Accepted the documentation option: new crate README
  gives the explicit supported-filesystem scratch requirement and exact RTX
  command. Rejected silently skipping native qualification on unsupported
  filesystems. No runtime/test behavior changed; no unsupported platform is
  claimed passing.

No further source changes or review invocation follow these documentation-only
dispositions. Current budget **2/5 used**. No active source blocker remains; root
still owns shared ledger updates and combined integration qualification. The
unused three invocations are a ceiling, not a target.
