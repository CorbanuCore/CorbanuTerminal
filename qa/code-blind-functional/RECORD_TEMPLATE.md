# Functional-design handoff record

Keep these JSON files together with their referenced artifacts. Every artifact
reference is `{ "path": "relative/path", "sha256": "64 lowercase hex digits" }`,
relative to its JSON file. Paths must remain inside that record directory.
Replace placeholders with actual values; the examples deliberately do not pass.
The design's raw agent response and input packet are retained, not rewritten to
match implemented tests. Product decisions and review ledgers must name actual
people/agents and link their recorded decisions; invented attestations are invalid.

`design.json`:

```json
{
  "feature": "Feature name / sprint ID",
  "designer": "independent-agent-session-id",
  "fresh_context": true,
  "code_blind": true,
  "results_blind": true,
  "isolation": "instruction-only",
  "brief": {"path": "packet/intent.md", "sha256": "..."},
  "screenshots": [{"path": "packet/screen.png", "sha256": "..."}],
  "proposal": {"path": "original-proposal.md", "sha256": "..."},
  "access_record": {"path": "designer-session.md", "sha256": "..."},
  "cases": [{
    "id": "F01-mac-existing",
    "priority": "blocker",
    "starting_conditions": "Mac; existing configured profile; installed shortcut",
    "actions": ["Open a new window using the shortcut", "Open the feature"],
    "expected": ["The intended feature is usable without repeating completed setup"]
  }]
}
```

`results.json` (one record per final packaged candidate; use the same frozen
design across variants, explicitly dispositioning non-applicable cases):

```json
{
  "design_sha256": "...",
  "implementer": "implementation-agent-session-id",
  "candidate": {
    "version": "version",
    "source": "commit plus dirty-source manifest if applicable",
    "platform": "macOS arm64",
    "binary_sha256": "...",
    "package_manifest": {"path": "candidate-manifest.md", "sha256": "..."}
  },
  "cases": [{
    "id": "F01-mac-existing",
    "disposition": "passed",
    "summary": "Observed outcome, including profile and launch path",
    "candidate_sha256": "...",
    "method": "native-ui",
    "evidence": [{"path": "runs/F01.md", "sha256": "..."}]
  }],
  "evidence_check": {
    "agent": "independent-agent-session-id",
    "verdict": "pass",
    "artifact": {"path": "evidence-check.md", "sha256": "..."}
  },
  "review_budget": {
    "used": 2,
    "limit": 5,
    "ledger": {"path": "review-ledger.md", "sha256": "..."}
  }
}
```

For `failed` or `blocked`, retain the ID, summary and available evidence; the
checker blocks readiness. For `out_of_scope`, replace execution fields with
`"approval": {"by": "product-authority name", "reason": "why excluded",
"artifact": {"path": "scope-decision.md", "sha256": "..."}}`.
When review use exceeds the recorded limit, `review_budget.extension` must have
the same approval shape, recording the explicit budget amendment or authorized
critical-finding exception. It does not turn failed cases into passes.
