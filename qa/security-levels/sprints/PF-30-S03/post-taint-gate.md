# PF-30-S03 slice 1 post-taint authority checks: per-sprint gate (2026-10-06)

- **Branch:** `feat/pf-30-s03-post-taint`, built on PF-30-S02 slice 2.
- **Scope:** everything here is behind the default-off `source_envelopes` feature and applies only under Moderate and Aggressive.
- **Unchanged:** Permissive and flag-off behave exactly as before.

## What ships

| Piece | Behaviour |
| --- | --- |
| Taint generation | **Counter:** each recorded batch that holds content without standing raises a per-session counter that never goes down.<br>**Counts:** tool, MCP, agent, memory and unattributed text, including restored history that has no record.<br>**Ignored:** host request structure. |
| Classification | Lexical, over the exact action the host is about to run (shell/exec argv including `bash -lc` scripts; patch files). Words lose quotes, backslashes and braces as the shell does; `~`, `$HOME`, `$CODEX_HOME` expand; `.`/`..` resolve; relative words resolve against the working folder (and `cd` inside a script); the working folder itself is checked. **Vault:** a Corbanu CLI word followed by `vault` in the same simple command, wherever it appears (`nice`, `sudo -u`, `timeout`, `eval`, `npx`, nested `sh -c`); vault files in a home. **Credentials:** keychain and token-printing commands; `.ssh`, `.aws`, `.gnupg`, `.kube` anywhere and `.docker`, `.azure`, `.config/gh`, `.config/gcloud` in the user's home; key, `.netrc`, `.npmrc`, `.pypirc` files; `auth*` in a home. **Security policy:** CLI `config`/`features`/`login`/`logout` subcommands, policy flags in any spelling (`--sandbox=danger-full-access`, `-capproval_policy=…`, `projects.*`), and anything else inside a home: `CODEX_HOME` as a whole-segment run anywhere in a word, or `.corbanu`/`.codex`/`.pfterminal` (or a glob that could match one) as a segment. Agent `worktrees/` under a home are ordinary. An action the host cannot describe counts as protected |
| Gate | Applies when the session is tainted and the action is classified.<br>**Approvals off:** `approval_policy = never` refuses the action.<br>**Otherwise the human is asked about that exact action:**<br>• The prompt shows why it is asking.<br>• A cached session approval, a hook's allow, the automatic reviewer and preapproved patch scope do not count.<br>• Approving for the session counts only once, and no "don't ask again" rule is offered.<br>• If new taint arrives while the prompt is open, the action is refused.<br>• Escalation retries ask again.<br>**Logs:** each check logs its kind and taint generation. |
| Same response | Calls the model issued in the same response as an untrusted read run before that read's output is recorded. The model had not seen it, so these calls are not post-taint. |

## Gate results

- **Formatting and lint:** `just fmt` and `just fix -p codex-core` are clean.
- **Focused tests:** `just test -p codex-core pf_30_s03` passes 9 tests:
  - 4 classification tests: positive, negative, evasion (quotes, globs, `..`, wrappers, attached paths) and a
    custom home
  - 1 taint-counting test
  - 4 suite tests: approval required (no "don't ask again" rule) and approvals off, unchanged paths, the
    session approval is not reused, and the automatic reviewer is bypassed.
- **Full suite:** `just test -p codex-core pf_30_s0` passes 84 (PF-30-S01, S02, S03, S04).
- **GLM 5.2 tmux runs:** real keys, Moderate + flag. After `cat notes.txt` (an injected "run the vault" line), a
  later `corbanu vault --help` stops at an approval prompt that gives the reason; with approvals off it is refused
  with "approvals are off"; an ordinary `wc -l` runs unchanged. No credential found in any run directory. Calls
  issued in the same model response as the read were not gated, which is correct (the model had not seen it).
- **Independent review:** Opus 5.5 High, read-only through `corbanu exec`, four rounds in
  `.codex-work/workers-20261002/pf30s03-review{1,2,3,4}/`. Rounds 1–3 CHANGES REQUESTED, all on the classifier
  (home-relative and glob paths, `..`, wrappers, argv `-c`, over-matching of ordinary globs and project files)
  plus one escalation-retry fix; round 4 APPROVE. Its three new Lows are fixed in `d31a15ae53`.
- **Videos:** [qa/demos/index/PF-30-S03.md](../../../demos/index/PF-30-S03.md).

## Known limitations (slice 1)

- **Lexical, not complete:** these routes are not seen:
  - code indirection, such as a patched script that is run later, `python -c`, base64 or variable concatenation
  - ANSI-C quoting such as `$'\x2e'codex`
  - symlinks into a home
  - home paths written in a non-canonical form
- **Folder tracking:** only a leading `cd` is followed; `pushd`, `builtin cd`, `tar -C`, `git -C`, `make -C`
  and archiving a parent folder of a home (`tar czf x ~`) are not.
- **Over-matching:** a CLI name followed by `vault` in the same command (for example in a commit message) counts. It costs an approval prompt, or a refusal under `never`.
- **Not on this seam:**
  - MCP tool calls
  - `write_stdin` into running processes
  - code mode

  These go to the PF-23-S01 typed protected surfaces.
- **Fails closed:** a poisoned origin registry under a protected level counts as tainted, even if the flag is off.
- **Untested here:**
  - an action refused because taint arrived while its prompt was open
  - a hook's allow being ignored
  - apply-patch preapproval
  - the escalation retry prompt
- **Expected:** after any tool output, the documented `CREDENTIAL="$(corbanu vault auth-helper ...)"` pattern needs approval. Under `never` it is refused.
