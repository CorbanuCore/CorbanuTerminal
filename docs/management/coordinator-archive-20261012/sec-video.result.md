PR #177 is merged into main as merge commit `6bb61c4ac949`; all 27 checks passed, with 11 skipped. Sprint workers can now record a feature demo video from the real TUI with one command, and two demos recorded on main with GLM 5.2 are uploaded to the new `demos` prerelease.

**The command**
```sh
python3 scripts/demo_video.py record qa/demos/specs/<demo>.toml \
  --bin "$CARGO_TARGET_DIR/debug/corbanu" --sprint <SPRINT> --publish
```
- It builds on the existing tmux procedure in `docs/tmuxHarness.md` and the AGENTS.md interactive-proof rules; it isn't a parallel harness. Each run uses a private tmux socket, sends real keys with text and Enter separately, waits on what's visible, and leaves through `/exit`.
- asciinema records the session and keeps the `.cast`. The tool compresses idle and model-wait gaps, adds a title card (feature, expected result, commit, date, model), and renders an H.264 MP4 that plays in QuickTime.
- It refuses any video over 90 seconds and saves title, middle and final frames for checking.
- With `--publish` it uploads the MP4 and cast to the `demos` prerelease and adds a row to `qa/demos/index/<sprint>.md`. `python3 scripts/demo_video.py publish <run-dir>` publishes a run after you've reviewed it.
- Secrets: every run gets a throwaway profile, home and workspace. Keys are fetched from the vault only at launch, so they never reach the script, the command line or the screen. If the recording contains a key value or anything shaped like a key, nothing is rendered or published.

**Files**
- `scripts/demo_video.py` and `scripts/test_demo_video.py` (9 unit tests pass; they need Python 3.11+, so they aren't in CI, whose Ubuntu 22.04 Python is 3.10)
- `qa/demos/README.md` (the SOP), `qa/demos/specs/permissions-full-access.toml`, `qa/demos/specs/approval-outside-write.toml`
- `qa/demos/index/demo-tooling.md`
- `docs/tmuxHarness.md` now points to the SOP. MP4s, casts and run folders under `qa/demos/out/` are git-ignored.

**The two demos** (main `cb78550a31cd`, GLM 5.2 via `zai`)
1. `/permissions` to Full Access and back, 37 s: GLM writes `../outside/full.txt` with no approval prompt, then switching back makes `/status` show "Workspace (Ask for approval)".
   https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/demo-tooling-permissions-full-access-cb78550a31cd-2026-10-05.mp4
2. Approval prompt for a write outside the workspace, 31 s: the prompt appears, `y` approves it, and `!cat ../outside/note.txt` prints `hello`.
   https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/demo-tooling-approval-outside-write-cb78550a31cd-2026-10-05.mp4

I checked frames and contact sheets of both: the text is legible and the last frame shows the result. The uploaded files match the local renders byte for byte.

**Product bugs found (not fixed here)**
1. **API key written to logs:** a `!` shell command logs its whole environment at trace level, including the Z.AI API key. It ended up in `codex-tui.log` and the logs database. The recording tool now overwrites the key in those private files and flags it; nothing reached any video. This belongs in the TUI lane.
2. **Old "Codex" wording:** the approval line says "You approved codex to run…" and the Full Access confirmation says "When Codex runs with full access…".

**Still open**
- **Model flakiness:** in one re-run with the original wording, GLM declined to attempt the outside write and the recording timed out. I changed the prompt to say "requesting approval if needed", and the next run passed. Workers should expect the occasional rerun.
- **No review:** no independent review was run, since this is tooling rather than a sprint.
- **PR size:** about 1,300 added lines, over the 800-line guideline. It's all new standalone tooling and docs with no product code, as the PR explains.
- **`sec-common.md` decision 5** still says to use asciinema until the SOP lands. It can now point to `qa/demos/README.md`; I left that file for the plan worker.