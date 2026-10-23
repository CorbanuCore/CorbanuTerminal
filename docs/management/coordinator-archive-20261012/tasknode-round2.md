You are the Task Node integration worker for Corbanu Terminal (sprint PF-80-S01), round 2.

Read your round-1 report first: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/tasknode-enable.result.md
and its record qa/initiative-control/management-bootstrap/tasknode-writeback-live-20261002.md in
/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911. Also read the tasknode-usage skill at
/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/ (SKILL.md and references/).

## Travis's decisions (2026-10-03); add them verbatim to decision `tasknode-writeback-20261002`
- A1: "We don't need to submit anything against a task that has been recognized as closed. If we erred in giving
  that task too much scope, we should create some new tasks that reflect the actual breakdown of our work and
  submit tasks and evidence against those. We should have a way of mapping this back to our actual work."
- A2: A reviewed CLI command that reads Travis's Task Node session from the vault at send time; nothing is written
  to disk.
- A3: "Our wallet address is 5oUcE2vkXK7ffn7pxJXCawq8UYD62hoadLLr6zGDRDtC, I've put the private key in the vault
  corresponding to CorbanuAPIKey. … Alternately, just copy the credentials from the RTX box."
- A4: Enroll `corbanu-initiative-control` through the delivery-control enroll command. No TUI recording.

## Facts already checked
- `corbanu vault auth-helper CorbanuAPIKey` in this terminal's home returns `no credential labeled "CorbanuAPIKey"`.
  The entry may be in a different Corbanu profile or home (for example `~/.corbanu`), or under a different label.
- **Wallet private keys are not API keys.** The vault helper refuses seed phrases and private keys by design. Never
  read, export, print, copy or use a wallet private key. If the Corbanu API key has to be derived through a wallet
  signature, use only the product's own supported wallet flow (the wallet daemon or TUI) with Travis present, or
  stop and report.
- The RTX box (ssh ambient@100.99.88.49, Corbanu home ~/corbanu-rtx/home) may already hold a working Corbanu API
  key, because RTX `/cost` showed Corbanu API usage. Travis authorized copying it. Do it only as an
  operational-credential copy:
  - read it there with the RTX `corbanu vault auth-helper <label>` piped straight over SSH into this Mac's
    `/vault` storage path, in the correct Corbanu home (the one the Task Node send will use);
  - never echo it or write it to a plain file;
  - find the label from metadata only.

## Do
1. **Task breakdown (A1).** Use the supported Task Node CLI/API (chat or task-request paths per the skill) to
   create new tasks covering the real PF-80 breakdown that is done and verifiable. For example: native delivery
   control bootstrap deliverables, the receipted Slack alert path, owner recurrence install, the Task Node
   writeback pipeline, and today's manager `prepare_worker` change (commit 66a5b66840).
   - Keep the scope honest; claim only verified work.
   - Write a mapping file `qa/initiative-control/management-bootstrap/tasknode-task-map.md` from each Task Node
     task id to its commits, QA records and sprint ledger lines. That mapping is the required link back to the
     actual work.
   - If the Task Node, not us, has to accept or assign the tasks, request them and record their states. Never
     move stages yourself.
2. **A2.** Implement the vault-backed send command in the integration branch's Task Node CLI/Python sender
   (whichever is the supported path). It reads the session token through the vault helper at send time, keeps it
   in memory only, and never logs it. Add tests with synthetic tokens. Commit and push fast-forward.
3. **A3.** Get a working Corbanu API key into this Mac's vault, preferring the RTX copy above. Confirm it resolves
   (exit code and length only).
4. **A4.** Enroll `corbanu-initiative-control` through the delivery-control enroll command.
5. Send evidence for one new, assigned task through the new command with a fixed idempotency key. Then re-send it
   to prove there is no duplicate. If no new task is assigned yet, stop after steps 1–4 and report the state.
6. Update the record and the PF-80-S01 ledger, then commit and push fast-forward. Never stage the four
   uncommitted files in scripts/initiative_control/ that belong to another session.

## Never
Never touch wallet private keys. Never print credentials. Never move task stages, accept rewards, enable batch
posting or turn on TUI recording. If something needs Travis (a native prompt, 2FA, a wallet unlock), stop and say
exactly what.

## Report (your final message)
The tasks created and the map, the A2 commit, the A3 result, the A4 result, the send and re-send receipts, and
anything still blocked.
