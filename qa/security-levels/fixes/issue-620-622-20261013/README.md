# #620, #621, #622: Aggressive re-run defects (AGG-18/R17, AGG-R07, AGG-R09)

Found by the "Aggressive ships" re-run (`../../milestones/aggressive-20261011-rerun/`, runs G10, G24, G28).

| Issue | Before (candidate 6ebfa1f47c) | After |
| --- | --- | --- |
| #620 | Grant once offered for `echo '# grant-once-1' >> ~/.zshrc`; accepted, the write still failed with "operation not permitted" | No Grant once offered; the prompt and the denial say why: a start-up file outside the current folder, which agent commands can never write under Aggressive, not even once |
| #621 | `python3 -c "…os.listdir('<home>')…"` offered Grant once; no never-grantable reason | Paths inside interpreter code (`python -c`, `node -e`, `perl -e`, `ruby -e`) are recognised: no Grant once, "cannot be granted, not even once"; a path built at run time is explained from the denial line |
| #622 | Inspector "Recent denials: None" after home-listing denials | Explained sandbox denials (protected read, home listing, start-up file), Aggressive "blocked this command" notices, and additional-permission refusals are listed |

## Why #620 is "no grant" rather than "the write succeeds"

The ~/.zshrc append was not stopped by the start-up-file rule that Grant once lifts. It was stopped by
Aggressive's base sandbox: writes only in the current folder, "approved commands stay inside it too" (the
confirmed Sandbox row; decision 2 of 2026-10-06; #442 decision 9 of 2026-10-12: describe Grant once as what it
actually lifts). The start-up-file rule only applies where the sandbox could write, so a grant could never reach
~/.zshrc. Letting a grant open a file outside the folder would move the sandbox boundary: Decisions for Travis
in the PR.

## Evidence

- Regression tests (fail before, pass after): `core/tests/suite/issue_620_622.rs` (real Seatbelt on macOS, bubblewrap
  on glitch), `security::protected_reads::tests::issue_620_*`, `issue_621_*`.
- tmux (GLM 5.3 Flash, debug build of 30cb180947, disposable home and HOME): `after-620.txt`, `after-621.txt`
  (approval prompts without Grant once), `after-622.txt` (the inspector row).
- Videos: [#620](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-620-issue620-zshrc-no-grant-30cb18094768-2026-10-12.mp4),
  [#621](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-621-issue621-python-listing-never-granted-30cb18094768-2026-10-12.mp4),
  [#622](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-622-issue622-inspector-lists-denials-30cb18094768-2026-10-12.mp4).
- Limits: the write-target parsing is text-based (quotes inside a script are not kept, `/usr/bin/env` or `sudo -u`
  prefixes and newline-separated commands are not looked into); a miss offers Grant once as before, a false match only
  withholds it. Listing denials are not enforced on Windows, so nothing is said there.
