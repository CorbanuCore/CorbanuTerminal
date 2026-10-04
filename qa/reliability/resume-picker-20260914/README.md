# Resume picker input and provider repair — September 14, 2026

## Scope and authority

Bounded repair, not a new initiative or authorization/storage change. Product
source: **Shipping MVP — LIVE**, “durable mailboxes, supervision, resume, and
recovery” and “Multi-provider inference” including OpenAI and Anthropic/Claude
Plan. User requested implementation, testing, integration-branch merge/push,
local rebuild, existing Developer ID signing and launcher refresh. No main push,
live-profile test, credential migration, active-session restart or model call.

Base: `5748c8d20c58b1ff20e5cdbe21bb46910b15df64`.
Branch: `fix/resume-input-provider-20260914`; worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/resume-input-provider-20260914`.
Receiving branch: `integrate/management-workstreams-20260911`.

## Defects and repair

The in-app `/resume` event awaited a picker that independently polled the shared
terminal broker while the main loop's background drainer continued consuming
input. Real-process evidence showed the picker alive and watchdog events queued
without main-handler progress. The picker now borrows that existing queue and
accounts for handled events. Its subsequent directory-choice prompt shares that
input ownership. Startup pickers retain their own sole input stream.

Local resume lists silently restricted history to the current provider. The
reported saved Fable chat was unarchived in the correct profile, but the new chat
used OpenAI. Resume lists now request all providers within the existing source,
archive and directory restrictions. The existing resume machinery restores saved
provider/model; no credential stores or provider permissions are changed. Fork
and `--last` selection policies are outside this bounded picker repair.

## Verification and limitations

`design-original.md` preserves the independent intent-only test proposals.
`pty_check.py` is an owner-run offline regression using synthetic history and
providers, a private TMUX socket, clean environment, debug native-keyring guard
and (on macOS) additional network/personal-credential denials. It is **not** an
independent permission-isolated agent execution or a live Claude/Fable request.
The explicit binary input permits exact packaged/signed candidate replay.

Private attempt artifacts are retained under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/resume-fix.q96t3X`.
The original signed 0.1.42 baseline failed cross-provider visibility as expected:
only `ALPHA_OPENAI` appeared, not the alternate-provider sessions. Initial
compilation exposed an unavailable non-test `futures` dependency; the queue
adapter now implements the existing Stream trait without adding dependencies.

No claim of full independent functional acceptance, real-provider continuation,
native GUI launch qualification, Windows/Linux validation or full release-suite
qualification follows from these local checks. These remain explicit limitations
under the user's authorized local installation. No benchmark claim is made for
this bounded usability repair. Review and final-candidate results are recorded
below before handoff.

### Final-tree supporting results

- 112 focused Rust tests passed after the runtime review corrections, including
  resume-picker snapshots, input queue/watchdog handoff, model restoration and
  cwd resolution/prompt behavior (`tests-3.log`; 3,995 unrelated tests excluded).
- 28 canonical package-builder tests passed. An earlier wrong discovery pattern
  selected zero tests and is not counted as proof.
- Signed source candidate SHA-256:
  `98c20b808137af5a53576c78e8ca267c362e6e6347d5a58d4606096312067cc4`.
  Owner-run `candidate-inapp-2` and `candidate-startup-select-1` passed:
  cross-provider visibility, directory filtering, no-result/edit/clear search,
  Escape clear then cancel, reopen, cross-provider selection, saved history,
  provider shown by `/status`, editable composer, all-directory selection and
  real keys through its cwd confirmation. All network was denied; providers
  were synthetic, not real Claude Plan sessions. `candidate-inapp-1` failed
  because the synthetic rollout omitted the assistant UI event; its original
  evidence remains. Adding that event repaired the fixture without changing
  runtime code. Startup-select-1's legacy `startup: false` field identifies only
  the cancel mode; its command used `--startup-select`, now recorded separately.
- Runtime code review 1 found an all-provider encoding defect and an incorrect
  test Escape sequence; both were confirmed and repaired. Runtime review 2
  was clean (Astra High). Final harness closeout is recorded privately separately.
- Sprint checker passed: 116 current and 126 archived.

Frozen-design disposition: FT-01/02/03/05/06/07/09 have supporting owner-run
observations, not independent acceptance; FT-04/08 are partial cancel/recovery
coverage, not original-draft/empty-profile proof. FT-10/11 burst/replay/privacy
variants and FT-12 actual-provider continuation remain unqualified. All original
cases remain visible in `design-original.md`; no strict functional gate is
declared complete. Final receiving build, signing and launcher receipts and
fresh exact-package replays are kept with the private installation artifacts.

## Frozen closeout-review scope

Owner boundary: TUI resume picker, input handoff and directory confirmation;
seven Rust files plus bounded regression/evidence files in this directory.
No runtime authorization, protocol, dependency, storage or credential changes.
An independent code review follows formatting; relevant tests repeat after any
accepted repair. Extra unrelated formatter changes are removed from the clean
repair worktree; no unrelated user changes are discarded.
