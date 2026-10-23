You are a Task Node delivery worker for Corbanu Terminal. Owner: Travis.

Task: `task_96e88f7…` — "Private inference user flow spec for PF Terminal". It is in the Accepted state.

1. Read the tasknode-usage skill (/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/SKILL.md).
   Run the Task Node CLI as:
   env CORBANU_HOME=$HOME/.corbanu CODEX_HOME=$HOME/.corbanu PFTERMINAL_HOME=$HOME/.corbanu CORBANU_TASKNODE_PROFILE=null /Volumes/CorbanuDrive/Corbanu/.codex-work/integration-build-20260922a/integration-package-dev/bin/corbanu tasknode ...
   Read the full task (title, description, acceptance/verification criteria) read-only first. The task's own criteria define done.
2. Do the work. Write the spec as docs/specs/private-inference-user-flow.md on a new branch off origin/main in a new worktree under /Volumes/CorbanuDrive/Corbanu/worktrees/. Ground it in what Corbanu Terminal actually ships today (rented GPUs via Vast.ai/RunPod, /providers, vault, Corbanu Plan) and in docs/sprints/current/portfolio-private-inference-research. Open a PR to main; don't merge.
3. Self-check against every criterion. Then have one independent reviewer subagent (spawn tool) check it against the criteria; fix what it finds.
4. Submit evidence only if the work genuinely meets the criteria: use the supported evidence command with a fixed idempotency key and an honest scope; read back the receipt. Never move stages or claim rewards; Task Node does that.
5. Record the submission in /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/qa/initiative-control/management-bootstrap/tasknode-task-map.md
   only via a separate commit on a fresh branch + PR (never edit or stage in that worktree directly).

Never print credentials or touch wallet keys. Never use rm -f or force-push. If something needs Travis, stop and say exactly what.

Final message: the deliverable's path/URL, criteria met (table), evidence receipt, and anything open.
