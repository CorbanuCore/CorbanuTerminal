# PF-23-S04 frozen independent functional design

Designer: Schrodinger (`01a1c964-ea15-7cb0-b52b-cbfd03d720cf`), requested Astra High.
Fresh context contained only user intent and isolation constraints, not source,
implementation rationale, test results or prior findings. Original response is
preserved below. This is design evidence only: all cases remain unexecuted.

```text
## Frozen functional test plan — design only

**Scope:** Bounded, code-blind review authorized by coordinator in #2106. No files, tools, history, implementation, or results inspected. No execution or runtime assurance claimed.

### Shared prerequisites

- Use the **actual Corbanu Terminal TUI**, exclusively inside a disposable guest. Never invoke `chroot` on the operator host.
- Prepare synthetic profiles, a working guest-local chroot fixture, harmless executables, disposable target files, and execution markers. No real secrets, host mounts, shared directories, or production credentials.
- Use guest-local fixture privileges where necessary; distinguish OS privilege failures from Terminal approval decisions.
- Freeze the existing permission configuration. Exercise each existing protected mode separately; use a separate synthetic Permissive profile for compatibility comparisons.
- Start each case from a clean snapshot unless continuation is specified. Observe the TUI and independently inspect synthetic fixture state inside the guest.
- Establish equivalent direct-command behavior as the baseline. Do not attribute ordinary permission-policy prompts to chroot recognition.
- If the TUI does not produce the intended command form, record the case as **not exercised**, not passed.

### Numbered cases

**1. P0 — Recognized protected action; approval happy path**
**Start:** Protected mode; synthetic target present; equivalent direct destructive action requires approval.
**Action:** Request that action through a working `chroot`; inspect the prompt before approving once.
**Observable:** Approval precedes target mutation or payload execution. Prompt meaningfully identifies the protected action. Approval permits the intended synthetic change, subject to unchanged permission policy.
**Isolation:** Unique target and execution marker; fresh snapshot per mode.

**2. P0 — Explicit refusal**
**Start:** Case 1 fixture, untouched.
**Action:** Deny the approval request.
**Observable:** Target and marker remain unchanged; TUI reports refusal without executing through another route or repeatedly seeking equivalent approval.
**Isolation:** Dedicated fixture; inspect after the turn settles.

**3. P0 — Cancel before approval**
**Start:** Protected chroot action awaiting approval.
**Action:** Cancel using the TUI’s supported cancellation control.
**Observable:** No payload execution or target change; TUI returns to a usable state. A later unrelated harmless request succeeds.
**Isolation:** Cancel only while approval is pending; do not infer rollback guarantees after execution begins.

**4. P0 — Effects cannot be judged**
**Start:** Protected mode; synthetic payload whose behavior is unavailable to the approval decision—for example, a fixture script with no available contents.
**Action:** Request its execution through `chroot`; first refuse, then repeat from a reset fixture and approve.
**Observable:** Uncertainty triggers approval rather than silent execution. Refusal leaves markers untouched. Approval does not override any independent policy denial.
**Isolation:** Payload can affect only disposable synthetic targets. If its effects become available before the decision, the ambiguity condition was not exercised.

**5. P0 — Permission-policy preservation**
**Start:** Synthetic profiles representing existing allow, ask, and deny rules where supported; record direct-command baselines.
**Action:** Request equivalent chroot-launched actions.
**Observable:** Existing prohibitions remain prohibitions; approval cannot bypass them. Additional protection for risky or indeterminate effects does not rewrite saved policy.
**Isolation:** Separate profile copies; no approval option that intentionally changes persistent policy.

**6. P1 — Ordinary harmless workflow**
**Start:** Valid fixture; protected mode; harmless baseline such as printing a literal or reading an ordinary synthetic text file.
**Action:** Request the equivalent operation through `chroot`, then another routine TUI task.
**Observable:** Correct output and continued usability; no blanket block or unnecessary additional prompt merely because chroot is present, beyond existing policy.
**Isolation:** Read-only payload; pre/post target-state comparison.

**7. P1 — Argument and root-context distinctions**
**Start:** Two guest-local roots containing different synthetic targets at the same internal path.
**Action:** Separately exercise a harmless literal containing risky-looking text, a protected action with a quoted filename containing spaces, and the protected action in the other root.
**Observable:** Literal text is not mistaken for an executed action; the protected action still prompts; execution affects only the selected fixture target.
**Isolation:** Reset between variants. Coverage is limited to these forms, not all launchers or syntax.

**8. P1 — Failure and recovery**
**Start:** Missing root, missing guest executable, or intentionally insufficient guest privileges; then a valid fixture.
**Action:** Request execution, decline any elevation request, observe failure, correct the fixture, and retry.
**Observable:** Clear failure, no unintended target changes, no silent fallback outside the selected root; corrected protected action follows normal approval behavior.
**Isolation:** Guest-only repair; no host privilege escalation.

**9. P1 — Resume with pending or refused action**
**Start:** Pending approval, and separately an explicitly refused action.
**Action:** Exit without approving; resume through the supported TUI workflow.
**Observable:** Resume alone does not authorize execution. Pending work follows the existing resume contract; refusal is not converted into approval. A fresh harmless request remains usable.
**Isolation:** Preserve only the synthetic session/profile; inspect markers before further interaction.

**10. P1 — Permissive compatibility**
**Start:** Separate Permissive profile and an established pre-change behavioral baseline.
**Action:** Repeat harmless, protected, and indeterminate chroot requests.
**Observable:** Existing Permissive prompting, execution, denial, and cancellation semantics remain unchanged.
**Isolation:** Fresh fixtures; without a baseline, report compatibility as unresolved.

**11. P1 — Cross-platform applicability**
**Start:** Disposable guests for supported platform/configuration combinations.
**Action:** Run applicable cases where chroot is supported; otherwise exercise the normal unavailable-command path.
**Observable:** Supported combinations satisfy the same behavioral criteria. Unsupported combinations fail clearly without unintended effects; they do not count as recognition coverage.
**Isolation:** Never substitute operator-host execution or treat one guest environment as evidence for another platform.

### Product ambiguities requiring resolution

- Which protected modes and action categories are in scope?
- What qualifies as insufficiently judgeable effects, and what must the prompt disclose?
- Which chroot invocation forms and platforms are officially supported?
- What are the existing approval scope, persistence, and resume contracts?
- What authoritative baseline establishes unchanged policy and Permissive behavior?

**Limit:** This plan tests approval decisions and observed synthetic effects—not full containment or exhaustive launcher coverage.
```

## Coordinator disposition

Apply existing Moderate/Aggressive post-taint policy, without changing Permissive
or approval persistence. The supported change is direct/wrapper-chain chroot
classification; arbitrary launcher coverage and operating-system containment
are not claimed. A non-root or unknown root must still ask, including a harmless
command whose executable cannot be judged. Thus case 6's no-additional-prompt
control uses the known `/` root inside the disposable guest, not an arbitrary
replacement root. Existing main at `152f62bf6e305d74c4deab00013520bb75d3636e`
is the compatibility reference. No case is deleted or counted as passed by this
disposition. Functional execution, separate evidence review and demo are pending.
