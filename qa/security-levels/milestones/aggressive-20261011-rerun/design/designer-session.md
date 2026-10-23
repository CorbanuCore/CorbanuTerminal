# Amendment reviewer and new-case designer (access record)

One fresh agent did both jobs the brief asks of "that same fresh agent": review the integrator's amendment table
against the owner decisions only, and design new cases for the decisions' new behaviours.

| Field | Value |
| --- | --- |
| Session (thread) | `01a12e74-b702-7402-b93d-63d03a499d02` (pass 1, then one resumed confirmation pass) |
| Runtime | installed `corbanu exec` 0.1.48 (`~/.local/bin/corbanu`, SHA-256 `f141adf3…84c5e5`) |
| Model | `claude-opus-5-5-plan`, provider `claude-plan`, reasoning effort high |
| Flags | pass 1: `--json --skip-git-repo-check -s read-only -C <packet dir> -o …`; confirmation: `exec resume <thread> … -c sandbox_mode="read-only"` |
| Context | new process, no inherited conversation; prompts: [prompt-as-sent.md](prompt-as-sent.md), [confirm-prompt-as-sent.md](confirm-prompt-as-sent.md) |
| Working directory | a directory holding only the packet (`.codex-work/aggressive-rerun-20261011/design-r/packet`), outside any source checkout |
| Time | pass 1 2026-10-12 04:52:43Z to 04:58:24Z, exit 0; confirmation 05:02Z to 05:03Z, exit 0 |
| Tokens | pass 1: 196,516 input (140,936 cached), 35,567 output; cumulative after confirmation: 397,802 input, 42,274 output |

## Packet

- The first run's frozen packet: `intent.md`, `screens/*.txt` (the design-time screens) and the executor copy of the
  frozen cases (`frozen-cases.md` = first run's `executor-packet/v2/design-cases.md`, SHA-256 `cebdc554…cda4`).
- [packet/owner-decisions.md](packet/owner-decisions.md): only the owner-decision comments, verbatim, plus the
  coordinator brief's restatement. Implementation and evidence comments were left out.
- `amendments.md` (v1, kept as [amendments-v1-as-reviewed.md](amendments-v1-as-reviewed.md)); in the confirmation
  pass `amendments-v2.md` (kept as [amendments-v2-as-reviewed.json](amendments-v2-as-reviewed.json)) and
  [questions-and-answers.md](questions-and-answers.md).
- [packet/intent-addendum.md](packet/intent-addendum.md), [packet/claude-availability.txt](packet/claude-availability.txt)
  and 17 text screens of this candidate ([packet/new-screens](packet/new-screens)), captured by the coordinator in the
  guest's capture lane `agmx0` on 2026-10-12 04:44–04:51Z (never an executor lane).

## Isolation: instruction-only

The read-only sandbox stopped writes but not reads; no OS-level read restriction was applied. Recorded as
instruction-only, as in the first run.

## What it actually accessed

Event streams: [reviewer-designer-events.jsonl](reviewer-designer-events.jsonl), [confirm-events.jsonl](confirm-events.jsonl).
Pass 1 ran four commands, all inside the packet directory (`ls -R` and `cat` of the packet files and screens). The
confirmation pass ran one (`cat amendments-v2.md questions-and-answers.md`). No other path, no network, no code,
tests, commit messages, plans or results.

## Outcome

- Pass 1: agreed with AGG-16 and AGG-37; asked for changes to 11 rows and proposed three missing amendments
  (AGG-02, AGG-26 wording, AGG-30 advisory). Most notably it moved the grant cases from `notes.py add` (where a
  grant cannot be observed) to the protected write `~/.zshrc`, which the decision names as the target. All were
  applied in v2. It asked nine questions; answers in [questions-and-answers.md](questions-and-answers.md).
- Confirmation pass: agreed with every v2 row except one clause of AGG-22, removed in the final table
  ([amendments.json](amendments.json), [amendments.md](amendments.md)); revised AGG-R01, R03, R04, R05, R07 and R08
  after the answers. Final new cases: [new-cases.json](new-cases.json) (18; AGG-R06 needs Claude Code, which the guest
  lacks).
