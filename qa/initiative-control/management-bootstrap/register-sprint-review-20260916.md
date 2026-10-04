# register_sprint: add-only holds, document agreement does not

**Fable, 2026-09-16.** Independent Opus review of `4f81ca3cf`. **Not received.**
Two P2 blockers, and one of them has already left a defect in live state.

## What held, and held well

The reviewer could not break add-only. Case twins (`PF82`/`pf82`) are distinct
keys and each needs its own matching document. Whitespace, unicode confusables
and oversized ids die in `ident()`. There is no normalisation step, so no id can
collide after normalisation. `source_path` traversal is read-only, with
`resolve(strict=True)` and `is_file()` rejecting directories, `/dev/null` and
FIFOs. `_reference` uses `INSERT OR IGNORE` so evidence rows cannot be clobbered.
Every refusal path rolls back, verified by diffing all six tables. Re-open,
un-archive, re-parent and re-status are structurally unreachable: the only write
is a new key with `status="draft"`.

`git show --numstat` is 92/0, 142/0, 142/0 — **zero deletions anywhere**.
`complete_sprint`, `archive_sprint`, `activate_successor`, `put_allocation` and
the reservation logic are unchanged. No secrets.

## P2-1: document agreement is with a file, not with the repository

`source_path` is unconfined. Any readable file anywhere on disk satisfies the
agreement check, so the guard proves the caller's chosen file matches the
caller's asserted metadata — not that the **repository document** does. That is
the precise bug this method exists to close, so it is a blocker for relying on
it, not a nicety.

## P2-2: the defect is already in live state

The registered `PF-60-S03` row carries an **absolute** `source_path` pointing
into `management-workstreams-20260911`, a dated worktree, while all 22 other
sprint rows use repository-relative paths. If that checkout is moved or removed,
the row points at nothing.

**It cannot be repaired through this API, because the API is add-only.** That is
the correct design and it is also the trap: I used a method before its review to
write a row I now cannot fix with it.

## P3 and a missing test

The `plan_file` to workstream map is hard-coded in the method, so a plan rename
or archival silently breaks registration, and any workstream outside the three
known keys can never register.

Of 17 mutants roughly 11 discriminate. The weak ones are worth naming: two are
detected only by a `TypeError` — a crash, not a guard; the self-edge cycle mutant
is unreachable in valid state because a self-dependency cannot pre-exist; one is
only reachable through harness-injected corrupt state the coordinator cannot
itself produce; two are caught only by exact message specificity while a later
guard still refuses.

**Nothing tests the bug it exists to close:** register a sprint, then confirm
`put_allocation` against it succeeds. That is the one behaviour that proves the
method solved the problem.

## Disposition

Not received. The corrections are clear and small: confine `source_path` to the
repository and store it relative, derive the workstream rather than hard-coding
it, replace the weak mutants, and add the end-to-end test. The `PF-60-S03` row
stays as it is until then, and **nothing should be dispatched against it on the
strength of a registration whose agreement check is this loose.**
