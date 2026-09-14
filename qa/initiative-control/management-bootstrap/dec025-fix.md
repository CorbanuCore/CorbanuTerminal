# DEC-025 bounded dashboard fix — dec025-fix-01

- Allocation digest: `8f5af2cafca39f0086b8e2b2fc00f41b25224ba55f694d7bc4d34b254ad70efd`.
  Claim: `bc21009f-0579-457a-8d24-962343e05cd9`.
- Implementer: gpt-6-astra / high; 2026-09-14.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-dec025-20260914`;
  branch: `bootstrap/dec025-20260914`.
- Assigned base: `8320109b126570697f3138099b34c7168ff63fbb`.
  Actual starting HEAD: `a826527bbc0029db4f5e9e81a097e15d2a2215f6`.
  The three allocated Python/CSS files were identical at these commits.
- Classification: **bounded fix** restoring decision scanability.
  Product authority: [Internal delivery control — TO BUILD](../../../../docs/corbanu-product-spec.md#internal-delivery-control--to-build):
  “linked sprint summaries, expandable issue context and an answerable question
  only when a decision is needed.”

Frozen finding: attempt `attempt-61f5jo95` on candidate `c77123a7e9`
expanded Blair's long background/evidence beyond the 1440×900 viewport,
obscuring Avery and Casey. DEC-025 requires distinguishable summaries/owners
and locatable remaining decisions while inspecting one.

The open-question group now starts with a compact overview outside all
expandable cards. Each row shows the summary, explicit owner, approved sprint
links and textual status, with a native link to the existing decision anchor.
Cards repeat those fields and offer a return link to the overview. Long context,
including retained revisions, scrolls inside a panel capped at 40vh; its native
summary and return link remain outside that panel. The panel is a named,
keyboard-focusable region with a visible focus outline. Narrow layouts wrap
instead of truncating the overview. Print styles remove the panel height cap.

Existing decision IDs, native details/summary behavior and safe-text/link
resolution remain; decision cards carry `attention-item`. No script, CSP,
generation attribute, decision schema/semantics or Slack changes.

Validation:
- Added two DEC-025 regression tests: three owners with long background/evidence
  and retained revisions; index/card summaries and approved sprint destinations;
  focusable bounded regions; complete context retention; open/history membership;
  stale projection, escaping and unavailable-context links.
- `git diff --check`: passed.
- Documented SDK suite: **516 tests passed**, 0 failures/errors, exit 0,
  276.497 seconds. This includes 18 attention tests (two new DEC-025 tests).
  The run emitted non-failing ResourceWarnings during implicit cleanup of
  synthetic HTTP 500 and HTTP 429 error fixtures.

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
```

This is an implementation return for manager integration, not an unqualified
human-test handoff. Browser visual/keyboard execution and independent DEC-025
replay on the exact packaged candidate remain with the manager's qualification
lane; no visual pass or human acceptance is claimed here. This change affects
the static browser dashboard, with no terminal interaction changed. No push.

## DEC-001 revision — dec025-fix-02

- Allocation digest: `c144bdefbed236e9ff5538c1c9b96ae63ba5af111d386222bd4f8a3ae09bd89e`.
  Claim: `6e0ba3a7-7919-47fc-8c17-edfb2eebe57e`.
- Implementer: gpt-6-astra / high; 2026-09-14.
- Worktree and branch: same as above.
- Assigned base and actual starting HEAD:
  `fcbc2ad3149f9c89460614a6715006b723919ad2`.
- Classification: **bounded fix** under **Internal delivery control — TO BUILD**:
  “linked sprint summaries, expandable issue context and an answerable question
  only when a decision is needed.” No plan/sprint scope added.

Frozen finding: code-blind DEC-001 on candidate `fcbc2ad31`, attempt
`attempt-tpi7m6j1`, failed because the single-decision overview repeated the
details summary and made the section identifier-heavy. DEC-025 on the same
candidate, attempt `attempt-7s87v6yz`, passed with three decisions.

The compact index now appears only with two or more open decisions. The return
links use the same condition so single-decision cards do not point to a missing
index. The single details card keeps its existing summary, context and selectors.
The 40vh bound, keyboard-focusable panels, multiple-decision return links,
permanent anchors and CSS from `d71a8ef14` are preserved.

Regression coverage checks a single open or acknowledged decision at fresh and
stale times, one open decision alongside resolved history, and the two-open
threshold alongside that history. The existing three-decision long-context test
continues to check owners, retained revisions, links and bounded panels.

Validation:
- Documented Python suite (exact command recorded above): **517 tests passed**,
  0 failures/errors, exit 0, 277.591 seconds; includes 19 attention tests.
- Non-failing ResourceWarnings occurred during implicit cleanup of synthetic
  HTTP 500 and HTTP 429 error fixtures.
- `git diff --check`: passed. CSS is unchanged.

This revision is an implementation return to the Fable manager. Independent
DEC-001 and DEC-025 execution/evidence review on the revised exact package remain
in the manager's qualification lane; the prior candidate's results above are
historical evidence, not passes for this revision. Browser acceptance and human
sign-off are not claimed. Terminal/live-repository qualification and benchmarks
are outside this bounded browser-rendering assignment; no release is requested.
No push.

## DEC-002 revision — dec025-fix-03

- Allocation digest: `0935453563b2d91adf56cb11d6adbe5c6b702c927eff7a60b81f8adddf002cae`.
  Claim: `7f00950a-6ae9-4e26-8d37-61fa25681220`.
- Implementer: gpt-6-astra / high; 2026-09-14.
- Worktree and branch: same as above.
- Assigned base and actual starting HEAD:
  `d8d8e3516ecd6e8f74d0f484db0abf29102cc662`.
- Classification: **bounded fix** under **Internal delivery control — TO BUILD**:
  “linked sprint summaries, expandable issue context and an answerable question
  only when a decision is needed.” No plan/sprint scope added.

Frozen finding: code-blind DEC-002 on candidate `d8d8e3516`, attempt
`attempt-ie37kvk2`, failed because the expanded body was clipped at 40vh and
required inner scrolling unavailable to the page-scroll/key actor. Choices,
recommendation, question and evidence were not visually exposed. DEC-025 on
that candidate, attempt `attempt-q27z3wg0`, passed with the compact overview.

Expanded content now occupies normal page flow: removed the body's max-height,
overflow and scrollbar-gutter declarations. The existing
`decision-body-bounded` selector is retained for compatibility but no longer
bounds height or creates an inner scroll region. Its focusability, named region,
focus outline, return links and all other selectors remain unchanged.
The compact overview still appears only for at least two open decisions; CSS
`position:sticky;top:0` keeps it at the viewport top while scrolling its
open-decisions group. Its opaque background and stacking order keep it legible.
Print uses a static index. Rendering logic and CSP are unchanged; no JavaScript
was added.

Regression coverage checks the absence of body height/overflow constraints,
preserved focus styling, sticky positioning and the index/return-link threshold
with zero, one, two and three open decisions. Existing long-context assertions
retain complete evidence and revision content, owners, links and focus targets.

Validation:
- Documented Python suite (exact command recorded above): **543 tests passed**,
  0 failures/errors, exit 0, 293.061 seconds.
- Non-failing ResourceWarnings occurred during implicit cleanup of synthetic
  HTTP 500 and HTTP 429 error fixtures.
- `git diff --check`: passed.

This is an implementation return to the Fable manager. Independent browser
execution of DEC-002 and DEC-025 (including page scrolling and index navigation)
and evidence review on the revised exact package remain in the manager's
qualification lane. Prior attempts are preserved as historical results, not
passes for this revision. No browser acceptance or human sign-off is claimed.
No terminal flow changed; live-repository qualification and benchmarks remain
outside this bounded browser assignment. No release or push.
