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
  "schema_version": 2,
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
    "execution": {
      "agent": "fresh-executor-session-id",
      "run_id": "unique-attempt-id",
      "model": "actual model and effort",
      "machine": "isolated machine identity",
      "profile": "fixture version and fresh/existing state",
      "launcher": "actual packaged launcher",
      "fresh_context": true,
      "code_blind": true,
      "results_blind": true,
      "packet": {"path": "runs/F01/packet.md", "sha256": "..."},
      "access_record": {"path": "runs/F01/session.md", "sha256": "..."},
      "actions": {"path": "runs/F01/actions.json", "sha256": "..."},
      "isolation_record": {"path": "runs/F01/isolation.json", "sha256": "..."}
    },
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

Each `isolation.json` has the following shape. Artifact paths are relative to
`results.json`, including those nested here. A group run uses distinct case-bound
receipts referencing the same actual preflight evidence. Populate only from
observed enforced controls; never copy these booleans as acceptance.

```json
{
  "enforcement": "os-enforced",
  "agent": "fresh-executor-session-id",
  "run_id": "unique-attempt-id",
  "case_id": "F01-mac-existing",
  "design_sha256": "...",
  "candidate_sha256": "...",
  "policy": {"path": "runs/F01/effective-policy.txt", "sha256": "..."},
  "tool_inventory": {"path": "runs/F01/tools.json", "sha256": "..."},
  "probe_evidence": {"path": "runs/F01/probes.json", "sha256": "..."},
  "source_denied": true,
  "history_denied": true,
  "symlink_escape_denied": true,
  "credentials_denied": true,
  "cross_run_ipc_denied": true,
  "network_restricted": true,
  "package_readonly": true,
  "children_confined": true,
  "packet_readable": true,
  "candidate_launchable": true,
  "actual_input_available": true
}
```

Executor differs from implementer and designer; reviewer differs from implementer
and every executor. The designer may review evidence. Preserve original results
and distinct fresh replay attempts. Legacy records fail the current checker;
do not backfill fictitious identities/isolation to make historical evidence pass.

For `failed` or `blocked`, retain the ID, summary and available evidence; the
checker blocks readiness. For `out_of_scope`, replace execution fields with
`"approval": {"by": "product-authority name", "reason": "why excluded",
"artifact": {"path": "scope-decision.md", "sha256": "..."}}`.
When review use exceeds the recorded limit, `review_budget.extension` must have
the same approval shape, recording the explicit budget amendment or authorized
critical-finding exception. It does not turn failed cases into passes.
