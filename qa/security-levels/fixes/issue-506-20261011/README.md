# #506: under Aggressive, agent commands cannot list the Corbanu home (2026-10-11)

Travis's decision (2026-10-11): listing the Corbanu home is denied under Aggressive, not only reading its protected
files, from the moment Aggressive is confirmed and after a restart, explained where it is denied as in #464. Corbanu
itself keeps working.

## Live check (tmux, GLM 5.3 Flash)

Setup: a private tmux socket, a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, the "Aggressive ships"
feature flags, and the real Z.AI key from the vault helper on the launch command only. Scripts:
[launch.sh](scripts/launch.sh), [case506.sh](scripts/case506.sh).

Steps: confirm Aggressive and press Esc on screen 07 (no restart); ask for `ls -la "$CORBANU_HOME"`; restart into
Aggressive with `r` in `/security`; ask again; ask for `ls "$CORBANU_HOME/skills"`.

Builds (macOS arm64, debug): **before** `origin/main` `2e2e2f8d82`; **after** this branch at `0a3cf8cd1d` (the later
commits change the Linux sandbox, the explanation's attribution and tests, not this path).

| Step | Before | After |
| --- | --- | --- |
| Listing, before the restart | the home's names are listed (`config.toml`, `history.jsonl`, databases …) ([capture](captures/before-list-before-restart.txt)) | `Operation not permitted`, and the warning "Security level Aggressive blocked listing `$CORBANU_HOME` …" ([capture](captures/after-list-before-restart.txt)) |
| Listing, after the restart | names listed ([capture](captures/before-list-after-restart.txt)) | denied with the same warning ([capture](captures/after-list-after-restart.txt)) |
| `ls "$CORBANU_HOME/skills"` (a runtime entry) | runs ([capture](captures/before-skills-after-restart.txt)) | still runs ([capture](captures/after-skills-after-restart.txt)) |

## Linux

Real bubblewrap on glitch (Ubuntu): `linux-sandbox/tests/suite/landlock.rs::issue_506_*` (a read-only and a
full-write base; a `.codex` home inside a writable workspace) and the `bwrap::tests::issue_506_*` unit tests. Before the
change the real-bwrap test lists the canary (`ls=0`).

## Not covered

- Windows ignores the entry (the home stays listable there; its protected entries stay denied). Why it stays so:
  [#554](../issue-554-20261011/README.md#windows).
- A listing from a parent folder (`find ~`, `ls -R ~`) is denied for the home; #554 explains it.
