# Latest-call amendment evidence — September 10

Scope: [curated call amendments](../../../docs/plans/call-followup-2026-09-10.md),
seven amended proposals, affected sprint checklists, two new PF-70 drafts,
navigation and scoped QA. Source base:
`6ca801add7eeda9feb9723e7f4935fbf6b29a73b`.

This is routine planning-document maintenance of proposed product initiatives,
not implementation. The Corbanu development skill routes it through the existing
plan/sprint process. Product context is cited in each proposal; the general index
API/creator extension remains gated on an explicit spec/scope decision.
No active slot, runtime, credentials, dashboard deployment, automation or live
Task Node state changes. No financial action or public launch is authorized.

## Checks and limits

Run `python3 qa/portfolio/2026-09-10-call-followup/check.py`.
It validates 16 proposals/52 unallocated drafts and compares unchanged
reservations, archives and the exact seven inherited global errors. It is a
no-new-error check, not an execution/release waiver.

The September 9 validator's default remains 50; `--expected-sprints 52` is an
explicit amendment count. Earlier QA manifests and receipts are historical.
The old fixed-publication check is not a gate for the new 52-sprint snapshot.

Final validation and independent review results are recorded below.
Future sprint tests, real Task Node auth/writeback, API implementation,
human acceptance and deployment are not claimed by this docs-only amendment.

## Final checks

| Check | Observed result |
| --- | --- |
| Plan checker | PASS; two active plans, no free slot |
| Global sprint checker | FAIL; exactly seven inherited errors, 112 current / 121 archived |
| Amendment checker | PASS; 16 proposals, 52 drafts, maximum 78 sprint lines; unchanged reservations/archives and protected runtime/policy boundary |
| Explicit-count regression probes | Expected 52 passes; expected 50 and 53 each fail only the inventory check |
| Plan / sprint unit suites | PASS; 4 + 19 tests |
| MkDocs YAML syntax | PASS with Ruby YAML parser |
| Whitespace and curated Markdown privacy-pattern scan | PASS; no matches for the selected credential/private-recording patterns |
| Full rendered MkDocs site | Not run; executable unavailable |

Commands: `python3 docs/plans/check.py`; `python3 docs/sprints/check.py --json`;
`python3 qa/portfolio/2026-09-10-call-followup/check.py`;
`python3 qa/portfolio/2026-09-09/validate.py --expected-sprints N` for 50/52/53;
`python3 -m unittest discover -s docs/plans/tests` and `docs/sprints/tests`;
`ruby -ryaml -e 'YAML.parse_file("mkdocs.yml")'`; `git diff --cached --check`.
See [validation.json](validation.json). The global failure is not waived.

## Independent review

The autoreview skill ran the established helper with `--mode local --engine
codex --model claude-fable-5-1-plan --thinking high --no-web-search
--stream-engine-output`, the Corbanu wrapper as `--codex-bin`, and
[review-scope.md](review-scope.md) as the repo-relative prompt file.
Read-only repository inspection; no nested reviewer or raw private source input.

- Exact route: `claude-plan / claude-fable-5-1-plan`, high, no fallback.
  Provider response metadata confirmed `claude-fable-5-1`.
- Corbanu review binary: 0.1.38, SHA-256
  `950d42af0ca423ecf24bdde0101363008406776e1f9753136e66fca088ffd7e7`.
  Source-build SHA remains unestablished; this is not a build of the candidate.
- Task-owned Darwin arm64 tmux 3.7c, 160×48, session `call-followup`,
  pane `0.0`, private directory
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/call-followup-review.Al3RAi`.
  Launch text was observed before Enter was sent separately.
- Helper exit **0**, findings **[]**. [Structured result](review-result.json).
  No accepted/actionable finding; two harmless formatting/path-list nits were
  not patched. No additional review followed the clean result.
- [Input manifest](review-inputs.json): all 32 hashes matched at closeout;
  588 added / 81 removed lines, zero production runtime lines.
- Only the owned tmux session was closed; its socket reported no server afterward.

This README's closeout additions, input manifest, result and validation receipt
are post-review evidence. No reviewed plan, sprint, navigation or checker changed
after the clean result. Exec-based model review in tmux is not true-TUI product
qualification. Human acceptance, API delivery and live Task Node writeback remain
pending; this amendment starts none of them.
