# Portfolio validation and Fable 5.1 review

## Historical state — original recovery candidate

This is a retained September 9 review receipt, not validation of the main
integration. Counts, hashes, checkout paths and the word “current” below refer to
that original candidate. See [the September 10 integration record](../2026-09-10-main-integration/README.md)
for receiving-main checks and subsequent review. The original manifests/results
are retained unchanged as historical evidence.

Planning-only candidate: 16 proposed plans, 50 draft sprints, source/dependency/workload portfolio, navigation and read-only QA validation. No product runtime changes, active-plan changes, worker dispatches, Task Node posts or releases.
The first completed Fable review found six issues. The second completed pass judged the overall patch correct with two P3 wording/consistency findings. Both were corrected. **Final focused Fable 5.1 closeout passed: helper exit 0, no accepted/actionable findings.** See [the structured result](fable-closeout.json). No further review was run after that clean result.

## Candidate and checks

- Repository/branch: `/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal`, `recovery/corbanu-drive-2026-09-02`.
- Base HEAD: `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`. Drafts remain uncommitted.
- [Candidate manifest](candidate-manifest.json): 72 planning/navigation/scope/checker inputs with SHA-256 hashes. It deliberately excludes itself and this evolving evidence log to avoid self-referential hashes.
- Current manifest SHA-256: `73c128c796e80e60701809ebe574278225bdf640d1d00c9a2c5ace1be10c3261`.
- Previous full-review manifest SHA-256: `e05a9a4a5758d41c0651c106940e7eedc2c66ec557dcf26f9df28ecc4ea10937`.
- `python3 docs/plans/check.py`: PASS, active 2/2, available slots 0.
- `python3 docs/sprints/check.py`: PASS, current 109, archived 98.
- `python3 -m unittest discover -s docs/plans/tests`: PASS, 4 tests.
- `python3 -m unittest discover -s docs/sprints/tests`: PASS, 19 tests.
- `python3 qa/portfolio/2026-09-09/validate.py`: PASS, 16 plans, 50 sprints, 73 maximum sprint lines, 72 inputs, zero errors. Checks draft/unallocated state, exact source excerpts, links, plan backlinks, feature IDs, navigation entries and exclusion of private recording paths.
- `git diff --check`: PASS.
- MkDocs executable was unavailable; no full rendered-site build is claimed. Navigation paths were checked statically; `ruby -ryaml -e 'YAML.parse_file("mkdocs.yml")'` passed YAML syntax validation.
- Future sprint commands and actual-user qualification were **not run** by this planning pass.

## Exact reviewer and terminal execution

- Requested route: `claude-plan / claude-fable-5-1-plan`, effort `high`, no fallback.
- Provider response reported `claude-fable-5-1`; Corbanu recorded the requested alias and reported identity separately. This is the expected Fable 5.1 identity, not substitution with Fable 5 or another reviewer.
- Engine: established autoreview helper invoking the existing Corbanu binary through its Fable wrapper inside a private tmux server.
- Binary: `/Volumes/CorbanuDrive/Corbanu/.codex-work/cargo-target/debug/corbanu`; version `0.1.38`; SHA-256 `950d42af0ca423ecf24bdde0101363008406776e1f9753136e66fca088ffd7e7`.
- This was an existing review binary, not a build of the recovery checkout. Its exact source-build commit was not established; the binary hash identifies what ran.
- Host: Darwin arm64; tmux 3.7c; 160×48 private session `portfolio-fable`, pane `%0`.
- Private socket: `/Volumes/CorbanuDrive/Corbanu/.codex-work/portfolio-sprints.nMMW9K/tmux.sock`. Existing user tmux sessions were untouched.
- Launch command text was sent literally, captured, then Enter sent separately. `RUST_LOG=trace` was set; trace output was retained in the private runner log.
- Review is read-only with project-config/rule isolation from the established helper; web search disabled; no nested reviewers requested.
- This is **external Corbanu exec-based review inside tmux**, not a true-TUI product acceptance test and not proof the proposed native PF-14 feature exists.

Exact successful-dispatch command (inside the private tmux session):

```bash
python3 /Users/Neo/.codex/skills/autoreview/scripts/autoreview \
  --mode local --engine codex \
  --codex-bin /Volumes/CorbanuDrive/Corbanu/.codex-work/portfolio-sprints.nMMW9K/fable-engine \
  --model claude-fable-5-1-plan --thinking high \
  --no-web-search --stream-engine-output \
  --prompt-file qa/portfolio/2026-09-09/review-scope.md \
  --output /Volumes/CorbanuDrive/Corbanu/.codex-work/portfolio-sprints.nMMW9K/review-2.txt \
  --json-output /Volumes/CorbanuDrive/Corbanu/.codex-work/portfolio-sprints.nMMW9K/review-2.json
```

## Attempt history

| Attempt | Result | Scope/evidence |
| --- | --- | --- |
| Preflight (run-review-1) | Refused before model dispatch | A benign options-plan filename matched the helper's sensitive-name filter. Renamed file/paths, retained safeguard; no credential or recording content was involved. |
| First completed review (run-review-2) | Six findings; helper exit 1 | Full 424,883-character planning packet; Corbanu session `01a089ff-42a4-7981-91d0-cca38dfea3d9`. |
| Corrected candidate (run-review-3) | Overall patch correct; two P3 findings; helper exit 1 | Full 477,249-character packet. Navigation wording and PF-72 missing-authority gate were the only remaining findings. |
| Focused closeout (run-review-4) | CLEAN; helper exit 0; findings [] | 480,897-character context packet, focused on the two corrections and bookkeeping. Session `01a08a11-f0fd-70c0-aef6-47b65bb08c75`; provider reported `claude-fable-5-1`; completed 2026-09-09 23:47 America/Phoenix. |

Raw logs/results are retained in the private local review directory, not copied into public docs; the historical first review contains source-location text that was subsequently removed from the candidate.

## Findings and disposition

| Finding | Classification / action |
| --- | --- |
| Private recording identifier and outside-repo transcript/research links in publishable docs | Accepted in-scope blocker. Replaced with neutral private source IDs and removed outside-repo public links. Private mapping stays outside the repository. No claim is made about whose address the original filename described. |
| Evidence link overstated the pending placeholder | Accepted. This log now records actual completed checks and clearly separates pending re-review. |
| Tester calibration relied on auth recovery it did not own | Accepted. S01 must supply a previously qualified positive control on the pinned candidate; S02 tests it and seeded negatives. If unfinished auth is chosen, its receiving-branch dependency must be declared. PF-75 cannot repair auth. |
| Document-only sprints required nonzero automated test counts | Accepted across all artifact-only sibling records. They now require reviewer, digest, expected/actual results and observed cases, with explicit automation non-applicability. Code/qualification tests retain nonempty-run proof. |
| Generic sprint filename suffixes misrepresented work | Accepted. All 50 files now use descriptive title-based names; backlinks and navigation updated. |
| Distribution citation fragment did not authorize adjacent scope | Accepted. Complete source sentence plus explicit scope/spec-amendment gate added to PF-64/PF-73/PF-74. No draft priority or citation is implementation authority. |
| Sprint inventory/navigation omitted new records (helper filtered as outside changed files) | Verified manually and accepted as an in-scope documentation integration correction. Added inventory note and navigation only; no policy or existing sprint state changed. |
| Benchmark implementation boundary pointed at generic diagnostic scripts | Primary-agent correction. PF-63-S02 now extends the canonical coding runner/tests, preserves fresh-workspace/source-integrity guarantees and names exact offline verification commands. |

All corrections remain inside the original planning task. No product contracts, policies, active work or live state were changed to satisfy review.

Second-pass findings: both accepted as small in-scope documentation corrections. The inventory now accurately describes plan-to-sprint navigation; PF-72 now has the explicit product-spec-amendment/external-record gate used for its adjacent siblings. After two correction cycles, scope was rechecked: no runtime or policy expansion was required. Focused closeout verified the corrections and independently recomputed all 72 candidate hashes with zero mismatches.

Final invocation used the same flags as the command above, plus the focused-closeout instruction in `run-review-4.sh`, with output paths `review-4.txt` and `review-4.json`. This evidence log and the copied structured result are post-review receipts; neither alters the reviewed planning inputs or their manifest.

## Limits and remaining gates

- Source speaker attribution and brainstorming priorities are not binding product decisions.
- Actual receiving checkout, active slots, individual owners, worktree/base allocation, budgets and any missing product authority must be resolved before dispatch.
- Financial/enterprise bets stop at research/paper decisions; no live scope is authorized.
- Agent review and these structural checks are not human acceptance, release readiness or proof of bulletproof testing.
- Cleanup: the task-owned `portfolio-fable` session was closed after exit 0; querying its private socket confirmed no server remained. Existing user sessions were untouched. Private review logs and source mapping were retained locally.
