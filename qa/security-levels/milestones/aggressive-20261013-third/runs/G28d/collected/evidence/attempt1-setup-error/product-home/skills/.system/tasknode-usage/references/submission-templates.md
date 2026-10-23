# Task Node Submission Templates

Use these templates when drafting live Task Node submissions. Keep them compact; include only relevant sections.

## Task Request

```text
Objective:
<One concrete outcome.>

Context:
<Why this matters, current repo/product/user context, relevant constraints.>

Scope:
<What should be included and excluded.>

Expected Deliverable:
<PR, code change, spec, investigation note, terminal verification, etc.>

Acceptance Criteria:
- <Observable criterion 1>
- <Observable criterion 2>
- <Observable criterion 3>

Evidence Plan:
<What evidence should be submitted if the task is completed.>
```

## Initial Evidence

```text
Summary:
<What was completed.>

Artifacts (durable and reviewable only — see note below):
- <PR URL, commit hash, file path, task id, screenshot, route probe, or generated text.>

Note: The submission parser scans the evidence body for URLs and registers every URL it finds as a formal evidence item. Cite only durable, reviewable artifacts here — real file paths, commit hashes, task IDs, event IDs, or production URLs. Never include localhost, 127.0.0.1, deliberately-broken origins, or throwaway stub-server endpoints; a reviewer cannot reach them and they cannot be removed after submission. Describe test scenarios under Verification below using the exact command run and the observed output, never by citing a transient endpoint as an artifact.

Verification:
- <Command/check/probe and result.>

Requirement Mapping:
- <Task requirement> -> <proof it was satisfied>

Residual Risk:
<None, or honest limitations/follow-up needed.>
```

## Verification Response

```text
Verification Request:
<Restate the verifier's specific ask in one sentence.>

Response:
<Directly answer the ask. If they requested complete text, paste the complete text.>

Evidence:
- <Artifact or command output summary>
- <PR/commit/file reference if applicable>

Pass/Fail:
<Explicit pass/fail or partial status, with one or two sentences explaining why.>
```

## Context Document Edit Summary

Show this to the user after saving a context edit:

```diff
Context document saved.

- <old line or block removed>
+ <new line or block added>
```

If the change is an insertion, show enough unchanged surrounding heading context to make the insertion clear.

## Task Node Chat Prompt

```text
I need a Task Node context judgment.

Decision:
<Choice to make.>

Current context:
<Relevant facts from this work session.>

Options:
1. <Option A and tradeoff>
2. <Option B and tradeoff>

Constraints:
<Time, reward, user preference, product direction, risk.>

Please recommend the best next action and name any critical caveat.
```
