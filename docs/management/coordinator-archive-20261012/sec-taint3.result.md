I merged both sprints to main. PF-30-S02 is finished and archived. PF-30-S03 is under way: its first slice is merged and the sprint stays `in_progress`. Both are behind the default-off `source_envelopes` flag; with the flag off, behaviour is unchanged.

## What shipped
**PR #198 (PF-30-S02, merge `743a7c22da`):**
- **Sub-agent lineage:** a task one agent hands another (spawn, `send_input`, the automatic reviewer's prompt) is never treated as human input. It is trusted only while the sender has seen nothing untrusted; otherwise it arrives labelled as agent data. A sub-agent's result reaches its parent as agent data. `/review` prompts stay human.
- **Export/import:** saved origin records are now signed with a key kept in the Corbanu home (`source-origin.key`, owner-only). A session file opened in another home, an edited record, an old unsigned record or a home without a key restores nothing, so that content comes back labelled. Imported external-agent sessions never carry records.
- **Token-budget MCP thread hint:** now sent as its own message labelled as coming from MCP, not as part of the trusted host context.
- **New tests:** capacity, edited content losing its standing, a one-off approval not clearing taint, and a real paginated-thread fork plus cold resume through the app server.
- **Memory summarisation bound to the session level:** PF-30-S04 had already done this. I added a test confirming the flag doesn't reopen it.
- **Moved instead of left open:** letting memory summarisation run under Moderate on labelled input is new work, so I moved it to PF-23-S01 and noted it there. Tainted follow-on actions are PF-30-S03's job.
- **Records:** sprint record archived; index, plan, mkdocs and reconciliation links updated.

**PR #204 (PF-30-S03 slice 1, merge `38516a5b22`):**
- **When it applies:** once a session has taken in untrusted content (tool output, MCP, agent, memory), any shell, exec or patch action touching the vault, credentials or security policy needs a fresh human approval of that exact action.
- **What can't stand in for that approval:**
  - a cached "for this session" approval (it counts once only)
  - a hook's allow
  - the automatic reviewer
  - patch pre-approval
- **What the prompt does:**
  - It says why it is asking.
  - It no longer offers "don't ask again".
- **When the action is refused or re-asked:**
  - With `approval_policy = never` the action is refused rather than run.
  - New untrusted content arriving while the prompt is open refuses the action.
  - Running outside the sandbox after a block asks again.
- **Same-response calls:** commands the model issues in the same response as the untrusted read are not gated, because it hadn't seen that content yet.

## Gate evidence
- **Tests, PF-30-S02:** `just test -p codex-core pf_30_s0` passed 75/75. Thread-fork tests passed 25/25. codex-core, codex-protocol and external-agent-migration passed 4,207 of 4,210; the 3 failures were the same baseline ones recorded in slice 1.
- **Tests, PF-30-S03:** `pf_30_s0` passed 84/84. All of codex-core passed 3,788 of 3,792: the same 3 baselines, plus `config_schema_matches_fixture`, which also fails on current main because #201 left the schema fixture out of date.
- **CI:** all checks passed on both PRs. One macOS credential canary on #198 failed on its proxy-injection probe, passed when rerun, and the PR-triggered run of the same job passed.
- **Reviews (Opus 5.5 High):**
  - PF-30-S02 took 3 rounds and ended APPROVE. Round 1 found a High: agent text could be recorded as human input when an image was attached. I fixed it.
  - PF-30-S03 took 4 rounds and ended APPROVE. All findings were ways around the command classifier, all fixed.
  - Outputs are in `.codex-work/workers-20261002/pf30s02b-review{1,2,3}/` and `pf30s03-review{1..4}/`.
- **GLM 5.2 tmux runs and videos:** six runs, all behaving as expected. They are listed in `qa/demos/index/PF-30-S02.md` (3, at `833be22574`) and `PF-30-S03.md` (3, at `d31a15ae53`):
  - a session moved to another home comes back labelled
  - a one-off approval keeps the tool output labelled
  - a sub-agent's task arrives labelled
  - a vault command after untrusted content stops at an approval prompt
  - with approvals off it is refused
  - ordinary commands are unchanged
- **Gate records:** `qa/security-levels/sprints/PF-30-S02/slice-2-gate.md` and `qa/security-levels/sprints/PF-30-S03/post-taint-gate.md`.

## Still open
- **PF-30-S03:** the check sees only shell, exec and patch actions; MCP calls, typing into running processes and code mode go to PF-23-S01. It also still needs:
  - ties to policy changes, not just new taint
  - end-to-end tests for memory recall and for a tricked child agent
  - approval counts for PF-26
  - the review's listed test gaps

  The classifier works by matching command text, so indirect routes such as `python -c` or symlinks get past it; the gate record lists them.
- **Accepted behaviour:** under Moderate, after any tool output, the documented `corbanu vault auth-helper` pattern needs approval, and is refused when approvals are off.
- **Shared files:** three files (`security/mod.rs`, `tools/sandboxing.rs`, `tools/runtimes/apply_patch.rs`) are also in PF-27-S02's scope. I listed them in PF-30-S03's integration note instead of claiming them, following PF-27-S02's own pattern.
- **Small doc follow-up:** the PF-30-S03 record doesn't yet cite #204's merge commit.
- **Not my code, needs an owner:** the out-of-date config schema fixture on main.