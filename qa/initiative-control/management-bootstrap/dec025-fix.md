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
