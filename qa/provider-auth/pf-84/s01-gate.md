# PF-84-S01 gate evidence (home override hygiene)

Candidate: branch `feat/pf-84-s01-home-override`, code commit `84f2797b62`, base
`051f9747225776a5d85ad00c2e5d0a8f5f4036bf`; macOS arm64 debug build with
`CORBANU_TEST_NO_NATIVE_KEYRING=1` and disposable homes for every run.

| Check | Result |
| --- | --- |
| `just test -p codex-utils-home-dir` | 12 passed (conflict warning, symlink/trailing-slash spelling, three-variable message) |
| `python3 -m unittest scripts.install.test_install_sh` | all passed; new case covers the stable and debug wrappers (`CORBANU_HOME`, `PFTERMINAL_HOME`, `CODEX_HOME`, `CORBANU_DEBUG_HOME`) |
| `sh scripts/dev/test_corbanu_launcher.sh` | 5 cases ok |
| developer-accounting variant | N/A: `codex-utils-home-dir` and the scripts have no such feature |
| Linux clippy `-D warnings` (RTX, `-p codex-utils-home-dir -p codex-cli --tests`) | `CLIPPY_EXIT 0` at `84f2797b6` |
| tmux + GLM 5.3 Flash run (Z.AI default key via the vault helper) | `pf84-home-override`: the GLM agent starts corbanu through the checked-in launcher with `CORBANU_HOME=homeB`; doctor shows homeB; with `CORBANU_HOME=homeA CODEX_HOME=homeB` one warning names homeA |
| Second provider (OpenAI API key) | `pf84-home-override-openai`: homeB holds a fake OpenAI key; `corbanu exec` through the launcher gets `401 Unauthorized` (the default demo home has no OpenAI credential), so homeB's account was used |
| Videos | [home override](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s01-pf84-home-override-84f2797b624b-2026-10-10.mp4), [OpenAI second provider](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s01-pf84-home-override-openai-84f2797b624b-2026-10-10.mp4); leak scan passed |

## Independent review (Opus 5.5 High, read-only `corbanu exec`)

Verdict on `9294007ab`: CHANGES REQUESTED. Dispositions (fixed in `84f2797b62`):

1. Debug wrapper lost its debug home when `CORBANU_HOME` was set: fixed; each wrapper
   checks only the variables its binary honours before `CODEX_HOME`; test added.
2. Sprint not yet `ready`: activation PR #407 merged before this PR.
3. Warning inside Corbanu sessions (tool shells export `CODEX_HOME`): documented in
   `docs/authentication.md` as the expected confirmation; demo shows it.
4. Dev launcher now honours an exported `CODEX_HOME`: documented (matches the release wrapper).
5. Tests: PFTERMINAL_HOME, spelling, missing-path and three-variable cases added. The
   launcher shell test is not wired into CI (workflow changes are out of scope); run it by hand.
6. Advice text: now "To use another home, set CORBANU_HOME to it."
7. Repeated work: the conflict check runs once per process.
8. Launcher overrides renamed to `CORBANU_LAUNCHER_WORK_DIR` / `CORBANU_LAUNCHER_WORKSPACE_DIR`.

## Open

- Independent code-blind functional design and execution: assigned to the acceptance
  step (owner instruction 2026-10-10), not done by the implementer.
- Installing `scripts/dev/corbanu-launcher.sh` as `~/.local/bin/corbanu` on the Mac dev
  host is an operator step after merge.
