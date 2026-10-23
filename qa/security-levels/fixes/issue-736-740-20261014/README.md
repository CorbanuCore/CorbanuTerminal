# #736, #737, #740: third Aggressive run defects (AGG-10, AGG-T03, AGG-11/T04/R13/R18/T06)

Found by the third "Aggressive ships" run (`../../milestones/aggressive-20261013-third/`, runs G06, G27, G30, G31).

| Issue | Before (candidate 4b6dd55c85) | After |
| --- | --- | --- |
| #736 | `security find-generic-password -s probe` ran after approval; the prompt talked about credential files and the vault; the result (`…could not be found`, exit 44) had no attribution | Same result (the sandbox never reached the keychain), now explained: the prompt says the keychain is out of reach of agent commands and that this is never granted; the result says the "could not be found" comes from Aggressive, not from the keychain's contents; no Grant once; listed in the inspector |
| #737 | `git config --global core.hooksPath …` offered Grant once; denial without the start-up-file wording | No Grant once; prompt and denial: `~/.gitconfig` is a start-up file outside the current folder, not even once. `npm config set`, `pip config set`, `gh config set` are recognised too; reads are not writes |
| #740 | Grant once offered for `echo … >> ../outside-notes.txt`, `.git/hooks/pre-commit`, `git config core.hooksPath`, `mkdir .agents`, `cat .codex/notes.md`, `ln -s … && cat …` | Not offered when nothing the command writes is liftable (outside the folder, the project's `.git`, `.codex`/`.agents`, or a command whose text shows every write); prompt and denial say why. Commands whose writes the text can't show keep the offer |

## #736: is the keychain reachable? No.

Seatbelt allows the keychain service (`com.apple.SecurityServer`) only in `sandboxing/src/seatbelt_network_policy.sbpl`,
which is appended only with network on or a network proxy; Aggressive has network off. `~/Library/Keychains` is also a
denied read. Checked with a disposable keychain file (made with `HOME`/`CFFIXED_USER_HOME` pointed at a temp folder;
the login keychain and the search list were never touched, compared before and after):

| Where `security find-generic-password -s probe736 -w <temp keychain>` ran | Result |
| --- | --- |
| outside any sandbox | the canary |
| workspace-write sandbox, network on | the canary |
| workspace-write sandbox, network off (Aggressive's) | `The specified item could not be found` (exit 44), although the keychain file itself was readable |

The regression test `core/tests/suite/issue_736_740.rs::issue_736_keychain_is_out_of_reach_and_explained` repeats this
through a real Aggressive session. The keychain text is shown only on macOS with network off and no network proxy: with an
enforced managed proxy the sandbox opens the keychain service with the network rules, and Corbanu then claims nothing.

## Evidence

- Regression tests: `core/tests/suite/issue_736_740.rs` (real Seatbelt on macOS; the #737/#740 cases also on glitch with
  bubblewrap) fail before (Grant once offered in all three) and pass after; unit tests in
  `security::protected_reads::writes_tests`.
- tmux replays (GLM 5.3 Flash, debug build of a79ff79b39, disposable home and HOME, through `scripts/demo_video.py`):
  approval prompts `prompt-736.txt`, `prompt-737.txt`, `prompt-740-outside.txt`, `prompt-740-git.txt` (no Grant once),
  results `after-736.txt`, `after-737.txt`, `after-740.txt`.
- Videos: [#736](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-736-issue736-keychain-explained-a79ff79b392a-2026-10-13.mp4),
  [#737](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-737-issue737-gitconfig-no-grant-a79ff79b392a-2026-10-13.mp4),
  [#740](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-740-issue740-no-grant-where-useless-a79ff79b392a-2026-10-13.mp4).
- Limits: decided from the command's text. A command whose writes it can't show keeps Grant once (unless it also writes a
  start-up file outside the folder, `.codex`/`.agents` or Corbanu's files); keychain commands joined without spaces or in a
  subshell get no keychain text (they are never granted anyway). Not on Windows (#554).
