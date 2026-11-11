# A real shell grammar for the post-taint classifier (#932)

Owner decisions: D32 (option C, hybrid), D85(c) and D92 (start after #1114
and #1172). This note describes the design and the three phases. Each phase
is its own pull request.

## Problem

Once a session has taken in untrusted content, the post-taint classifier
(`codex-rs/core/src/security/tainted_action*`) judges each command the model
wants to run. It reads shell text with two hand-written readers:

- `shell.rs`, a word lexer that removes quotes on purpose, so quoted text is
  judged as code too;
- `quoted.rs` (#885), a quote-aware reader for nested shells' scripts and sed
  programs.

A cross-check keeps them honest: the shell and sed commands the lexer counts
must also be found by the quoted reader, and anything the readers cannot
follow fails closed (asks). Review rounds of #899, #986, #1114, #1158 and
#1172 found one desync after another (a `case` inside `$(...)`, quotes inside
`${...}`, zsh keywords, `\`+newline, multibyte locales). Every one was fixed
by failing closed, so each fix added prompts. D88, D90 and D104 accepted
those prompts until a real grammar lands.

## Design (option C)

`tainted_action/grammar.rs` reads shell text with tree-sitter-bash, through
`codex-shell-command`, which already depends on it for safe-command
detection. Core adds no new crate, and the C code already builds on the
Windows and Bazel lanes.

The reading gives the simple commands (those in substitutions, process
substitutions and here-document bodies included) and their words. Each word
is marked:

- **data**: every part of it is text that bash, zsh and dash all pass on
  literally. That means single-quoted text, double-quoted text without `$`, a
  backquote or a backslash, or unquoted letters, digits and `_ . / , : + @ %
  -` (and `=` after the first character). Expansions, globs, braces, `~`, a
  leading `=` (zsh), backslashes and `$'...'` are never data. Old dash reads
  `$'a'` as `$a`.
- **dynamic**: it holds an expansion or a substitution.

### Fail closed

`read` refuses the text, rather than guess at it, when:

| Refusal | Why |
| --- | --- |
| `TooLarge` (> 4 KiB) | Some valid text (`x | x | ...`, nested arrays) takes tree-sitter time quadratic in its length. Neither of its work counters bounds this, so the input is capped. |
| `Characters` | A control character other than newline or tab, or ASCII punctuation right after a non-ASCII character. In GBK, Big5, Shift-JIS or JOHAB a shell may read that byte as part of the character (same rule as the D85 gate, widened per the JOHAB note). |
| `LineContinuation` | A backslash-newline. The shells join the lines before splitting words; the grammar ends a word there. |
| `HereDocuments` | More than 8 here-documents, or a delimiter longer than 64 bytes: the grammar's scanner keeps pending here-documents in at most 1024 bytes of state and silently forgets them all past that. Also any here-document the scanner may end elsewhere than the shells: a delimiter other than plain characters or one whole quoted word, a body not on the line after its operator (one per line), an end that is not exactly the delimiter on its own line (tabs only for `<<-`), a body line equal to the delimiter before that end, expansions in a quoted body, and a backquote, backslash or unparsed expansion in an unquoted body. |
| `ParseBudget` / `Syntax` | The parse stops at the first error that every parse in progress has met (error recovery can be quadratic), or after `bytes/2 + 64` progress checks. Any `ERROR` or `MISSING` node also refuses. So do constructs the grammar accepts and bash rejects: `;;` outside `case`, a redirection whose word is on the next line, `f(x)`, and a function body that is not a compound command. |
| `Words` | The grammar's words may not be the shells': a reserved word as a command's name (`done x`, `coproc cmd`, `} x`), an "assignment" with an invalid name (`--x=1 cmd`), `[v]` read as a test, blanks or operators inside a word token, a comment starting inside a word, a word glued to a redirection's word, a redirection the grammar gives more than one word (`rm >/dev/null -rf x` runs `rm -rf x`; `>&-` takes none), words after a here-document's delimiter, `{fd}>f`, a non-blank character that is in no token (`\ ` before a word), a token holding blanks or operators the shells split on (`echo == x; rm y` makes `x; rm y` one pattern token; in a test, blanks and `|` count outside parentheses and quotes, as bash reads `=~`), `{echo,hi}` read as a block, `[ a > b ]` (a redirection to bash), or a token that holds an expansion the grammar did not parse (`${x#$(cmd)}` makes `$(cmd)` one pattern token), single-quoted text included where the shells treat quotes as ordinary characters (inside double quotes, a here-document body, arithmetic or `${...}`). Also any quoted text inside double quotes, a body or arithmetic (zsh and dash end `"${x:-'}"` at the quoted `}` and `$(( '))' ))` at the quoted `))`), and a backquote substitution holding a quote, `#`, backslash or backquote (the shells end it at the first backquote). |
| `Reparse` | A command that may change how the shells read the lines after it: `alias`, `shopt`, `setopt`, `emulate`, ... named anywhere in a command (so `builtin alias` counts), `set` with any option but `-e -u -v`, `-o errexit/nounset/verbose/pipefail` and `--` (`set -k` makes later `name=value` arguments assignments, `set -o posix` turns on aliases, `set -x` runs `PS4`), `SHELLOPTS`, `BASHOPTS`, `POSIXLY_CORRECT`, `BASH_ALIASES`, `BASH_CMDS`, `BASH_ENV` or `PS4` anywhere in the text, a variable named through an expansion by `export`, `declare`, `read`, `printf -v`, ... , or a command name spelled through an expansion or braces (behind `command`/`builtin` and their options too). |
| `Unsupported` | Extended glob patterns, which bash rejects without `shopt -s extglob`. |
| `TooDeep` | Syntax nodes nested deeper than 512. |

Parts with nothing between them are one word, as they are to the shells
(`~/$a$b/x`, `$"t"`).

`zsh_unsure` flags text zsh may read differently from bash. It covers
zsh-only reserved words and wrappers (`repeat`, `foreach`, `noglob`, `-`,
`builtin`, `command`), expansion flags (`${(e)x}`, `${=x}`), a subscript or
modifier after an expansion (`$x[1]`, `$x:h`), an argument `}`, and a
command name that is not data.
When the command may run under zsh, a flagged reading has no authority.

`bash_only` flags text dash (`sh` on Debian and Ubuntu) reads differently:
`[[` and `((` (dash runs `[[ a || cmd ]]`'s `cmd`; `(( id ))` is two
subshells), process substitution, here-strings, `$'...'`, arrays, `&>`,
`|&`, `function`, `select`. Under dash, a flagged reading has no
authority.

The walk is iterative, with every loop stepping the `work.rs` counter. The
linearity test covers it.

### What the reading promises

`read()` is best-effort. Five review rounds on #1183 each found more ways
bash, zsh and dash read the same text differently from each other or from
the grammar. Each is now a refusal, but the tail is open, just like the
hand-written reader's. So when `read()` returns Ok, its commands and data
marks are evidence, not proof. Grammar authority may only be granted for
narrow syntactic forms where an independent check also holds: the plain
words parse D85 uses (no substitutions, tests, arithmetic, compound
commands or redirections) and agreement with the quote-aware reader. The
phase 3 slice below is such a form.

### The cross-check stays

The current lexer and reader keep running. A command the lexer counts that
the grammar does not find, or the reverse, fails closed (asks). The grammar
only ever **removes** a prompt where it proves a word is data and both
readers agree on the commands. A refusal leaves today's behaviour unchanged.

## Phases

1. **Design and differential harness (this PR).** `grammar.rs` is compiled
   for tests only. `grammar_harness_tests.rs` reads every string literal in
   the classifier's sources and tests, the cases in `grammar_corpus.txt`
   (lines from the issue threads), and 3000 generated texts (fixed seed).
   It reads each one with `quoted.rs` and with the grammar and compares the
   commands found. `CORBANU_GRAMMAR_SHELLS=1` also checks each text with
   `bash`, `zsh` and `dash -n`, which reads the text and runs nothing.
   `CORBANU_GRAMMAR_REPORT=<file>` writes every disagreement. The harness
   reports and does not judge. CI runs it without real shells, as a
   no-panic and bounded-work check.
2. **Grammar behind the cross-check, log only.** Each command line
   `quoted_pass` reads (the action's own and nested shells' scripts, not
   the quoted words it re-reads) is read by the grammar too. A disagreement
   is logged at debug level, by kind only, and decides nothing. The reader
   stays authoritative. The work is bounded by the 4 KiB cap per line and a
   per-classification budget of 64 KiB, with each line costing 256 bytes
   more than its length (at most 256 lines). A skipped line is logged once
   per classification.
3. **Authority where the grammar proves data.** The rules behind the
   D88/D90/D104 prompts, plus quoted text judged as code, use the grammar's
   reading where it proves a word is data. That holds only when the grammar
   read the line, `zsh_unsure` (and `bash_only` under dash) is clear or that
   shell cannot run it, D85's plain-words parse holds and the commands
   agree. Anything else falls back to today's rule. This phase
   stops at a pull request with the harness results, and the coordinator
   decides on the merge.

## Phase 1 results (harness, macOS, bash 3.2, zsh 5.9, dash)

These are from 5482 distinct texts (2629 test literals, 65 corpus lines and
2788 generated), after the fail-closed rules above.

- **Agree:** 1790 of the test literals and 29 of the corpus lines.
- **Commands differ, reader finds more:** 185 test literals. These are
  mostly arithmetic and subscript text that `quoted.rs` reads as commands on
  purpose (#937), and `[[ ... =~ (re) ]]` patterns.
- **Grammar finds commands the reader misses:** 2 test literals.
  `[[ $v = (#b)(*) ]]; echo ...` hides the `echo` from the reader, which
  takes `#b)...` as a comment. zsh runs it. A bare `time` is the other.
- **Grammar reads, a shell rejects:** almost all of these are bash 3.2 gaps
  (`case` in `$(...)`, `[[ -v ]]`, `|&`) or bash-isms dash rejects. The lane
  only sees syntax one side rejects: it cannot see text both accept with
  different commands (dash runs `[[ a || cmd ]]`'s `cmd`; see `bash_only`).
  Only the refusal tests in `grammar_tests.rs` cover those.
- **Grammar refuses, the shells accept:** zsh-only syntax (expansion flags,
  `$~x`, `${ cmd; }`), `<>` redirections, and `\`+newline continuations.
  Each costs nothing beyond today's behaviour.

## Phase 3, first slice (held for the coordinator)

The zsh-only checks (#1028 glob qualifiers, #1084 glob substitution) no
longer ask for a top-level line under a system zsh (or bash, dash, sh) when
`named_shell::grammar_proves_data` holds:

- the host runs the argv as named, a system shell with `-c`/`-lc`;
- the grammar reads the line, with no refusal, nothing `zsh_unsure` and
  nothing `bash_only`;
- `shadow::compare` agrees with the quote-aware reader;
- every word of every command is grammar-proven data (no expansion,
  substitution, unquoted glob, brace, `~`, leading `=` or parenthesis);
- the line is D85's plain commands of programs that never run their words
  (`echo`, `ls`, `cat`, `grep`, data-only `git` subcommands, ...). `cd` is
  excluded (zsh `chpwd` hooks), and so is `''` (zsh `RC_QUOTES`).

With no unquoted glob character and no expansion left, zsh has nothing to
apply a qualifier or glob substitution to. This is D85 extended from bash
to zsh users (the macOS default), per D92. Everything else keeps today's
checks: nested lines, other argv shapes, shells named without a system
path.

Harness: 78 corpus texts qualify (75 test literals, 2 corpus lines, 1
generated). All are plain `echo`, `cat`, `ls`, `grep` and `git`
status/log/commit lines. None of the review-history bypass lines qualifies.

## Limits phase 3 must respect

- "Data" means the shell does not expand the text. Builtins may still
  evaluate it: `declare 'a[$(id)]=1'`, `unset`, `printf -v`, `read` and
  `test -v` run a substitution in an array subscript, and a test command
  keeps only its keyword (`[[ -v 'a[$(id)]' ]]`). Data words of such
  commands keep today's rules.
- `time cmd` is read as a command named `time` (bash's keyword). Treat it as
  a wrapper, as the classifier does today.
- Strings that run later (`eval '...'`, `trap '...' DEBUG`, `source`/`.`
  files) can change how later lines are read (`eval 'shopt -s
  expand_aliases'`). Phase 3 must refuse a line whenever a nested reading
  of such a string refuses.
- Some commands change which program a data name runs without changing how
  text is read: `hash -p`, `BASH_CMDS`, `PATH=`, functions. Arithmetic
  operands (`let`, `[[ ... -eq ... ]]`) are evaluated too. Today's rules
  for these stay.
- Some checks are not counted by `work.rs`: substring scans of a node's
  text at each nesting level, the scanner's look-ahead, and the token sort.
  All are bounded by the 4 KiB cap (a few million character steps at
  most).
- A nested shell's reading depends on its environment as well as its text
  (`ENV`, `BASH_ENV`, `ZDOTDIR`, startup files); the classifier's existing
  rules for those stay.
- The harness compares command names only; arguments and data marks are
  covered by `grammar_tests.rs`.

## Decisions for the coordinator

1. **The 4 KiB cap (phase 2/3).** Commands over 4 KiB keep today's prompts.
   Recommendation: keep it. A wall-clock deadline in the progress callback
   would allow a larger cap, but the same text would then be judged
   differently on a loaded machine.
2. **Backslash-newline.** Multi-line commands with `\` continuations keep
   today's prompts. Recommendation: keep for phase 3. Joining continuations
   ourselves means writing quote-aware text handling again.
