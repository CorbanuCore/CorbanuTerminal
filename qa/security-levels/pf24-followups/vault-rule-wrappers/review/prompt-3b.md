You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Second pass on the Aggressive vault-rule matching change. Review commit 635185f815 (`git show 635185f815`); the branch also contains #197 (`strict_rules`, reviewed separately) via merges. Do not edit files; read the code and report.

Your first pass found that the wrapper-unwrapping approach (parsing env/command/exec/nohup options) could be bypassed many ways (9-deep nesting, GNU long-option abbreviations, `env -S` `\_`, `exec -ca`, BSD `-L`, case-insensitive file systems, zsh-fork intercepted execs, complex scripts never reaching rule matching, `corbanu -c k=v vault`), and suggested matching every argv suffix against forbidden rules only.

The rework (`strict_forbidden_matches` in core/src/exec_policy.rs), used only when the session's exec policy was loaded with `strict_rules` (Aggressive):
- every argv suffix is tried, with its first word normalised (`program_key`: shell punctuation trimmed, directories dropped, ASCII-lowercased, `.exe` stripped);
- when the word after the program starts with `-`, candidates that drop up to 16 following words are tried too (`corbanu -c k=v vault`);
- once a shell (`sh/bash/zsh/dash/ksh/mksh/fish`) or `env` word has appeared, any later argument containing whitespace or `\_` is split (shlex, `\_` -> space, an attached `-S`/`--x=` prefix dropped) and its suffixes are tried, followed by the remaining argv;
- each candidate is at most 17 words (prefix rules only need their own length);
- only `forbidden` matches are added; the decision becomes Forbidden;
- the zsh-fork escalation path (`unix_escalation.rs`) applies the same to intercepted execs.
Accepted over-matching: `echo corbanu vault`, `bash -c 'git commit -m "corbanu vault x"'`.

Look for: remaining bypasses of the `corbanu vault` forbidden rule under strict for agent shell commands (state honestly which are out of reach for name matching), ways this could widen an allow (it must not), performance on pathological inputs (huge scripts / argv), false positives that would be painful for normal agent work, and test validity.
