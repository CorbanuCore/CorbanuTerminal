I finished and archived PF-30-S03; PR #212 is merged as `55339d5b24`. Step 0 needed no PR, and I didn't start PF-23-S01. Everything is behind the default-off `source_envelopes` flag; with the flag off, or under Permissive, nothing changes.

## Step 0: config schema fixture (no PR)
`config_schema_matches_fixture` passes on current main. #197 (merged via #203) had already run `just write-config-schema`, and running it again changes nothing, so a fixture PR would have been empty. I put the explanation in #212's description instead.

**Why CI didn't catch it:** the checks that run on a PR don't include codex-core's Rust tests.
- `rust-ci` on PRs is format, shear and lint only.
- The Bazel Linux test job is skipped on PRs and only runs nightly.
- The Windows Bazel test shards (Bazel's MinGW Windows target) did nothing on #201: they skip without the BuildBuddy key.

The job that caught it was the post-merge run, `postmerge-ci` → `rust-ci-full` → "Tests — ubuntu-24.04 x86_64-unknown-linux-gnu, shard 2/4" (run 37460150785). That job, or the Bazel Linux test job, should have run on #201.

## Step 1: what shipped in PR #212
- **Ties to policy changes:** a post-taint approval is now tied to the session's security policy as well as its taint (policy version, revocations, kill switch, security level, agent lineage).
  - If any of these change while the prompt is open, the action is refused. This applies to the first prompt and to the retry outside the sandbox.
  - With the kill switch on, the action is refused without asking.
- **Approval counts for PF-26:** each decision is logged with its kind, outcome and how long the human took. In a test research workflow (25 ordinary commands plus 5 protected ones), there were exactly 5 prompts. Classification takes about 1 ms per action in tests and 1–4 ms in the live demo runs.
- **Indirect routes:** the command checker now sees through:
  - quoting and expansion tricks, variables, folder changes and symlinks
  - inline code (`python -c` and similar), encoded payloads, and the options of each language's interpreter
  - script files, including scripts written by an earlier patch
- **New category:** code the host can't read before it runs (a download piped into a shell, an unknown variable, a script it can't read, or any internal limit hit). The prompt calls it "running code the host cannot read first", and it is gated the same way as vault, credential and policy access.
- **Tests:** these cover the reviewer's slice-1 gaps and each review round's bypasses and false prompts:
  - taint or a policy change arriving while the prompt is open
  - the kill switch
  - a hook's allow being ignored
  - patch pre-approval being ignored
  - the sandbox retry asking again
  - end to end: recalled memory, and a child agent tricked by its parent
- **Moved to PF-23-S01:**
  - MCP calls, typing into running processes, and code mode
  - typed resources and OS-level blocking of home-folder reads
  - the reviewer's remaining minor findings
  - disclosure requests and value transfers driven by untrusted content

## Gate evidence
- **Tests:**
  - `just test -p codex-core pf_30_s03` passes 27 tests.
  - `pf_30_s0` passes 102 after merging main.
  - Full codex-core: 3,810 of 3,813 pass. The 3 failures are the same ones recorded in earlier slices (`skills_append_to_developer_message`, `skills_use_aliases_in_developer_message_under_budget_pressure`, `remote_compact_trim_estimate_uses_session_base_instructions`).
  - `just fmt` and `just fix` are clean, and the Bazel lockfile is unchanged.
- **Independent review (Opus 5.5 High):** six rounds; rounds 1–5 requested changes, all to the command checker, and round 6 approved at `8d09e0345d`. Outputs are in `.codex-work/workers-20261002/pf30s03b-review{1..6}/`.
- **CI:** everything passed before the merge. Three failures were flaky and passed on rerun:
  - a Windows platform-contract job (`boot_identity_unavailable`); its sibling run on the same PR passed
  - a Windows credential canary that timed out in its harness
  - a Windows argument-comment lint failure, which was real (three missing comments) and fixed
- **GLM 5.2 tmux runs, recorded at `8d09e0345d`:**
  - [Inline Python that builds the `.ssh` path asks the human](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s03-pf30s03-inline-code-needs-human-8d09e0345d65-2026-10-06.mp4)
  - [`cat greet.sh | sh` asks the human](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s03-pf30s03-piped-shell-needs-human-8d09e0345d65-2026-10-06.mp4)
  - [Recalled memory makes the first protected action ask](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s03-pf30s03-memory-recall-gates-8d09e0345d65-2026-10-06.mp4)
  - [A tricked sub-agent's vault command is refused](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s03-pf30s03-tricked-child-refused-8d09e0345d65-2026-10-06.mp4); the log confirms the refusal came from the host (`refused_approvals_off`)
- **Model refusals during recording:**
  - GLM refused a base64 decode-and-run command itself, so that video uses `cat greet.sh | sh` instead.
  - One run of the inline-Python video was refused by GLM, so it was re-recorded with the request reworded.
- **Records:** `qa/security-levels/sprints/PF-30-S03/slice-2-gate.md`; the sprint record is archived, and the index, PF-23-S01, plan, reconciliation and mkdocs links are updated.

## Still open
- **PF-23-S01 not started:** CI took several hours of reruns, so there was no time. Its sprint file now lists everything moved from PF-30-S03.
- **Accepted extra prompts after untrusted content** (refused outright when approvals are off):
  - a command run through an environment variable that is never set in that command, such as `$EDITOR file`
  - `eval "$(ssh-agent -s)"`
  - `cat x | sh`
  - the real `gradlew` and `mvnw` wrapper scripts
  - a CLI name followed by `vault` in one command, such as a commit message that contains both
- **Plan text left alone:** the plan file's front-matter still lists the old slice-1 worktree; the plan worker owns that text.