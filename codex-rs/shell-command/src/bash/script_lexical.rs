//! #2548 (D553): a lexical pre-filter before the syntax tree. tree-sitter-bash
//! and the shells (bash 3.2 to 5, dash, zsh, ksh) split some text
//! differently: line continuations, carriage returns, here-documents,
//! pattern and regex tokens, `case` arms, `[ … ]`, expansions and the scans
//! bash 3.2 makes of `$(…)`, comments, subscripts, multibyte locales, zsh's
//! and ksh's own syntax. Rather than chase each difference, a script is read
//! precisely only if it is built from a minimal POSIX set every one of them
//! reads alike (the coordinator's literal list), extended under D713 with
//! forms whose meaning is known exactly from the text; any other script
//! fails closed: it asks whenever a prompt or forbidden rule exists.
//!
//! The minimal set: words of ASCII letters, digits and `_-./,:@%+=~*?!^]`
//! (and `#` inside a word); the standalone word `{}`; spaces, tabs and
//! newlines; `\` escaping one visible ASCII character; `;` `&&` `|`
//! `||`; `'…'`; `"…"` holding ASCII text and `\` escapes; the special
//! parameters `$?`, `$#`, `$$`, `$!`, `$@`, `$*`, `$-`, `$0`–`$9` (not
//! followed by `(`, `{`, `'`, `[` or a name character); at most one
//! redirection per simple command (`<` `>` `>>` `N>` `N>>` `N>&M` `>&M`) to
//! a plain word or descriptor; comments after whitespace. Not `&` alone,
//! here-documents or here-strings, `( … )`, `{ … }`, `case`, `$'…'`,
//! process substitution, `=~`, `=` right after a quote (zsh's `""=cmd`),
//! control characters, or bytes above ASCII outside single quotes (a
//! multibyte locale can make one swallow a `\`).
//!
//! D713 adds, never in a command's name (an assignment's value is fine):
//! - `$NAME`, and `${NAME}`, `${#NAME}` and `${NAME OP WORD}` for the
//!   operators `:-` `-` `:+` `+` `:?` `?` `:=` `=` (not assigning a variable
//!   a shell runs or evaluates) and the trims `#` `##` `%` `%%`. WORD holds
//!   word bytes only, and in the non-pattern forms `$NAME` or a special
//!   parameter. None may be followed by `[`, `(`, `{` or `'` (zsh's
//!   subscripts). Expanding a variable runs no code, but arithmetic on its
//!   value can (`x='a[$(cmd)]'`), so none is allowed where it is evaluated.
//! - `$(( … ))` holding only digits, blanks, `+ - * / %` and `$?` `$#` `$$`
//!   `$!`.
//! - `$( … )` and backquotes holding a script of this same set, which the
//!   reader then reads command by command. Inside them nothing bash 3.2's
//!   and 4.x's scans for the closing `)` read differently: no `#` or `\`
//!   outside single quotes, no `{}`, `case` or `esac`, and in backquotes no
//!   quotes or `$(` at all. (bash 3.2 scans a quoted string in `$(…)` as
//!   one, `'…'` to its end, but takes `#` after a blank, even an escaped
//!   one, for a comment.)
//! - `[ … ]` and `[[ … ]]` on one line, with no `;` `&` `|` `<` `>` `(` `)`
//!   inside (dash, which has no `[[`, would run text after `||` as a
//!   command). Operands are words and quoted strings; `[[` may also expand
//!   parameters (it does not split them). `[` splits unquoted ones into
//!   words that may become operators: there `$?` `$#` `$$` `$!` anywhere,
//!   and a parameter in double quotes only where the operators around it
//!   are fixed: `[ "$x" ]`, `[ -n "$x" ]` (unary operators that test text
//!   or a file), `[ "$x" = "$y" ]` (`=` `!=` `-nt` `-ot` `-ef`), each
//!   optionally after `!`. A quoted operand may not read as an operator
//!   (`"-v"`), and `[` takes no `==` and no wildcards (it would read a
//!   file named `-v` as an operator). The reader checks the operators
//!   themselves (`-v`, arithmetic).
//!
//! Where zsh (or a shell not known) runs the script, a parameter or command
//! substitution outside double quotes fails closed: options a user's
//! startup files may set (`glob_subst`, `glob_assign`) make its value a
//! pattern whose qualifiers run code (`*(e:cmd:)`), and `sh_word_split`
//! splits it. bash, sh and dash read it (`ScriptShell`).

use super::script_read::runs_code_from_variable;

/// Substitutions nest at most this deep.
const MAX_DEPTH: usize = 8;

/// Whether `script` lies within the minimal set (see the module docs).
pub(super) fn lexically_safe(script: &str) -> bool {
    lexical_read(script).0
}

/// Whether `script` lies within the minimal set, and whether a parameter
/// or command substitution in it lies outside double quotes, where zsh
/// options a user may set (`glob_subst`, `glob_assign`, `sh_word_split`)
/// make its value a pattern whose qualifiers run code (`*(e:cmd:)`) or
/// split it into several words.
pub(super) fn lexical_read(script: &str) -> (bool, bool) {
    let mut lexer = Lexer {
        bytes: script.as_bytes(),
        at: 0,
        redirects: 0,
        command_start: true,
        depth: 0,
        closing: Close::End,
        test: None,
        dquote: false,
        expanded: false,
        unquoted: false,
    };
    let safe = lexer.script() && lexer.at == lexer.bytes.len();
    (safe, lexer.unquoted)
}

/// What ends the script being lexed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Close {
    End,
    /// `$( … )`.
    Paren,
    /// `` ` … ` ``.
    Backquote,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Test {
    /// `[ … ]`.
    Single,
    /// `[[ … ]]`.
    Double,
}

struct Lexer<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Redirections in the current simple command.
    redirects: usize,
    /// The next word is in command position.
    command_start: bool,
    /// How many substitutions enclose the current position.
    depth: usize,
    closing: Close,
    /// Inside the operands of a test.
    test: Option<Test>,
    /// Inside double quotes.
    dquote: bool,
    /// The current word holds an expansion added under D713.
    expanded: bool,
    /// Some parameter or command substitution lies outside double quotes,
    /// where zsh's `glob_subst` would take its value for a pattern.
    unquoted: bool,
}

const WORD_BYTES: &[u8] = b"_-./,:@%+=~*?!^]#";
/// `[`'s unary operators that only test their operand's text or the file
/// it names (not `-v`, `-R` or `-t`, which evaluate it).
const SAFE_UNARY_TESTS: &[&[u8]] = &[
    b"-n", b"-z", b"-e", b"-f", b"-d", b"-r", b"-w", b"-x", b"-s", b"-L", b"-h", b"-b", b"-c",
    b"-p", b"-S", b"-g", b"-u", b"-k", b"-O", b"-G", b"-N",
];
/// `[`'s binary operators that compare text or files.
const SAFE_BINARY_TESTS: &[&[u8]] = &[b"=", b"!=", b"-nt", b"-ot", b"-ef"];
/// Bytes of the WORD in `${NAME OP WORD}`.
const BRACE_WORD_BYTES: &[u8] = b"_-./,:@%+=~*?";
/// Operators of `${NAME OP WORD}`, longest first.
const BRACE_OPERATORS: &[&[u8]] = &[
    b":-", b":+", b":?", b":=", b"##", b"%%", b"-", b"+", b"?", b"=", b"#", b"%",
];

impl Lexer<'_> {
    fn peek(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.at + offset).copied()
    }

    /// The byte at the current position.
    fn current(&self) -> Option<u8> {
        self.peek(/*offset*/ 0)
    }

    /// The byte after the current one.
    fn following(&self) -> Option<u8> {
        self.peek(/*offset*/ 1)
    }

    fn rest(&self) -> &[u8] {
        &self.bytes[self.at..]
    }

    /// Inside `$( … )` or backquotes.
    fn substituted(&self) -> bool {
        self.depth > 0
    }

    /// Whether the previous byte is whitespace (or there is none), so a word
    /// starting here stands alone and `#` starts a comment.
    fn after_whitespace(&self) -> bool {
        self.at == 0 || matches!(self.bytes[self.at - 1], b' ' | b'\t' | b'\n')
    }

    /// Whether a word of `len` bytes starting here ends there.
    fn word_ends_after(&self, len: usize) -> bool {
        self.peek(len).is_none_or(|byte| {
            matches!(byte, b' ' | b'\t' | b'\n' | b';' | b'&' | b'|')
                || (byte == b')' && self.closing == Close::Paren)
                || (byte == b'`' && self.closing == Close::Backquote)
        })
    }

    /// A script, up to the end of the text or of the enclosing
    /// substitution, which the caller consumes.
    fn script(&mut self) -> bool {
        while let Some(byte) = self.current() {
            match byte {
                b')' if self.closing == Close::Paren => return true,
                b'`' if self.closing == Close::Backquote => return true,
                b' ' | b'\t' => self.at += 1,
                b'\n' | b';' | b'|' => {
                    // `|&` and `;;` are not in the set; `||` is.
                    if (byte == b'|' && self.following() == Some(b'&'))
                        || (byte == b';' && self.following() == Some(b';'))
                    {
                        return false;
                    }
                    self.redirects = 0;
                    self.command_start = true;
                    self.at += if byte == b'|' && self.following() == Some(b'|') {
                        2
                    } else {
                        1
                    };
                }
                b'&' if self.following() == Some(b'&') => {
                    self.redirects = 0;
                    self.command_start = true;
                    self.at += 2;
                }
                // Background jobs (and bash's `&>`) are not in the set.
                b'&' => return false,
                // bash 3.2 decides where a comment starts inside `$(…)` by
                // the raw byte before it.
                b'#' if self.after_whitespace() && self.substituted() => return false,
                b'#' if self.after_whitespace() => {
                    while self.current().is_some_and(|byte| byte != b'\n') {
                        if self.current() == Some(b'\r') {
                            return false;
                        }
                        self.at += 1;
                    }
                }
                b'<' | b'>' => {
                    if !self.redirect() {
                        return false;
                    }
                }
                b'{' if self.after_whitespace()
                    && !self.substituted()
                    && self.rest().starts_with(b"{}")
                    && self.word_ends_after(/*len*/ 2) =>
                {
                    self.command_start = false;
                    self.at += 2;
                }
                b'[' if self.command_start => {
                    if !self.test() {
                        return false;
                    }
                }
                _ => {
                    if !self.word() {
                        return false;
                    }
                }
            }
        }
        self.closing == Close::End
    }

    /// `[ … ]` or `[[ … ]]` in command position, on one line, with words
    /// for operands; then the end of the command.
    fn test(&mut self) -> bool {
        let (kind, close): (Test, &[u8]) = if self.rest().starts_with(b"[[") {
            (Test::Double, b"]]")
        } else {
            (Test::Single, b"]")
        };
        let open = close.len();
        if !matches!(self.peek(open), Some(b' ' | b'\t')) {
            return false;
        }
        self.at += open;
        self.test = Some(kind);
        // Each operand, and whether it expands a parameter.
        let mut operands = Vec::new();
        loop {
            while matches!(self.current(), Some(b' ' | b'\t')) {
                self.at += 1;
            }
            if self.rest().starts_with(close) && self.word_ends_after(close.len()) {
                self.at += close.len();
                break;
            }
            // dash has no `[[`: it would run what follows `||`, `;` or a
            // newline as a command, and `<`, `>` and parentheses are
            // operators or redirections depending on the shell.
            match self.current() {
                None | Some(b'\n' | b';' | b'&' | b'|' | b'<' | b'>' | b'(' | b')' | b'#') => {
                    return false;
                }
                _ => {}
            }
            let start = self.at;
            if !self.word() {
                return false;
            }
            operands.push((&self.bytes[start..self.at], self.expanded));
        }
        // `[` reads a quoted parameter's value as an operand wherever the
        // operators around it are fixed: one operand, a unary operator and
        // one, or a string or file comparison (`-v` and arithmetic would
        // evaluate the value; with more operands it may become one).
        if kind == Test::Single && operands.iter().any(|(_, expanded)| *expanded) {
            let word = |index: usize| operands.get(index).map(|(word, _)| *word);
            let unary =
                |index: usize| word(index).is_some_and(|word| SAFE_UNARY_TESTS.contains(&word));
            let fixed = match operands.len() {
                1 => true,
                2 => word(0) == Some(b"!") || unary(0),
                3 => {
                    word(1).is_some_and(|word| SAFE_BINARY_TESTS.contains(&word))
                        || (word(0) == Some(b"!") && unary(1))
                }
                _ => false,
            };
            if !fixed {
                return false;
            }
        }
        self.test = None;
        self.command_start = false;
        // Then the end of the command, or its redirection.
        let mut at = self.at;
        while matches!(self.bytes.get(at), Some(b' ' | b'\t')) {
            at += 1;
        }
        let digits = self.bytes[at..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        match self.bytes.get(at + digits) {
            None => digits == 0,
            Some(b'<' | b'>') => true,
            Some(_) if digits > 0 => false,
            Some(b'\n' | b';' | b'&' | b'|') => true,
            Some(b'#') => at > self.at,
            Some(b')') => self.closing == Close::Paren,
            Some(b'`') => self.closing == Close::Backquote,
            Some(_) => false,
        }
    }

    /// One word: unquoted bytes, quotes and expansions. A digit-only word
    /// glued to `<` or `>` is that redirection's descriptor.
    fn word(&mut self) -> bool {
        let start = self.at;
        let command = self.command_start && self.test.is_none();
        self.expanded = false;
        while let Some(byte) = self.current() {
            match byte {
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' => break,
                b')' if self.closing == Close::Paren => break,
                b'`' if self.closing == Close::Backquote => break,
                // Not after whitespace: shells disagree on whether it starts
                // a comment; inside a substitution, bash 3.2 may take it for
                // one anywhere.
                b'#' if self.at == start || self.substituted() => return false,
                // `[` operands are expanded as file names, which `[` may read
                // as operators (a file named `-v`); `^` and `#` are wildcards
                // under zsh's `extended_glob`.
                b'*' | b'?' | b'^' | b'#' if self.test == Some(Test::Single) => return false,
                b'<' | b'>' => {
                    let descriptor = self.bytes[start..self.at].iter().all(u8::is_ascii_digit);
                    if !descriptor || self.at == start || self.test.is_some() {
                        return false;
                    }
                    // A redirection's descriptor: the next word may still be
                    // the command's name (`2>/dev/null $cmd`).
                    return true;
                }
                // zsh expands `""=git` to git's path.
                b'=' if self.at > start && matches!(self.bytes[self.at - 1], b'\'' | b'"') => {
                    return false;
                }
                b'\'' => {
                    if !self.single_quoted() {
                        return false;
                    }
                }
                b'"' => {
                    if !self.double_quoted() {
                        return false;
                    }
                }
                b'$' => {
                    if !self.dollar() {
                        return false;
                    }
                }
                b'`' => {
                    if !self.backquoted() {
                        return false;
                    }
                }
                // Only before a visible character: the grammar takes `\`
                // with a blank for whitespace (`echo \ #; cmd`), and before a
                // newline for a continuation. Backquotes and bash 3.2's
                // scan of `$(…)` read escapes their own way.
                b'\\' if self.substituted() => return false,
                // tree-sitter-bash joins a word that starts with `\` to the
                // command on the line before (`true⏎\git push` reads as
                // `true` with arguments); bash runs it as a new command.
                b'\\' if self.at == start && self.at > 0 && self.bytes[self.at - 1] == b'\n' => {
                    return false;
                }
                b'\\' => match self.following() {
                    Some(0x21..=0x7e) => self.at += 2,
                    _ => return false,
                },
                _ if byte.is_ascii_alphanumeric() || WORD_BYTES.contains(&byte) => self.at += 1,
                _ => return false,
            }
        }
        // Nothing read (a test's operand at a closing backquote).
        if self.at == start {
            return false;
        }
        let word = &self.bytes[start..self.at];
        // An expansion in a command's name: what runs is known only then.
        if command && self.expanded && assignment_name(word).is_none() {
            return false;
        }
        if let Some(test) = self.test {
            // Operators read as patterns; a quoted operand that `[` would
            // take for an operator.
            return !(word == b"=~"
                || (word == b"==" && test == Test::Single)
                || quoted_operator(word));
        }
        // Test operators the grammar reads as patterns, anywhere; reserved
        // words of constructs outside the set, in command position (as
        // arguments they are ordinary words), and `case` anywhere in a
        // substitution (bash 4's scan for its end reads `case` arms).
        let command_word = std::mem::replace(
            &mut self.command_start,
            // These keep the next word in command position.
            matches!(
                word,
                b"!" | b"if" | b"then" | b"else" | b"elif" | b"while" | b"until" | b"do" | b"time"
            ) || assignment_name(word).is_some()
                || word
                    .iter()
                    .position(|byte| *byte == b'=')
                    .is_some_and(|at| {
                        at > 0
                            && word[..at]
                                .iter()
                                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    }),
        );
        !matches!(word, b"==" | b"=~")
            && !(self.substituted() && matches!(word, b"case" | b"esac"))
            && !(command_word
                && matches!(
                    word,
                    b"case" | b"esac" | b"select" | b"function" | b"coproc"
                ))
    }

    fn single_quoted(&mut self) -> bool {
        // Backquotes nest quotes by escaping.
        if self.closing == Close::Backquote {
            return false;
        }
        self.at += 1;
        while let Some(byte) = self.current() {
            self.at += 1;
            match byte {
                b'\'' => return true,
                b'\r' => return false,
                _ => {}
            }
        }
        false
    }

    fn double_quoted(&mut self) -> bool {
        if self.closing == Close::Backquote {
            return false;
        }
        let outer = std::mem::replace(&mut self.dquote, true);
        let read = self.double_quoted_text();
        self.dquote = outer;
        read
    }

    /// The text of a double-quoted string after its opening quote.
    fn double_quoted_text(&mut self) -> bool {
        self.at += 1;
        while let Some(byte) = self.current() {
            match byte {
                b'"' => {
                    self.at += 1;
                    return true;
                }
                // A multibyte locale (Big5, SJIS) can make the byte before a
                // `\` its lead byte.
                b'\r' | 0x80.. => return false,
                b'`' => {
                    if !self.backquoted() {
                        return false;
                    }
                }
                // bash 3.2's scan of `$(…)` passes the byte after `\` its
                // own way.
                b'\\' if self.substituted() => return false,
                // Only before a visible character: the grammar takes `\`
                // with a blank for whitespace (`echo \ #; cmd`), and before a
                // newline for a continuation.
                b'\\' => match self.following() {
                    Some(0x21..=0x7e) => self.at += 2,
                    _ => return false,
                },
                b'$' => {
                    if !self.dollar() {
                        return false;
                    }
                }
                _ if byte < 0x20 && byte != b'\t' && byte != b'\n' => return false,
                _ => self.at += 1,
            }
        }
        false
    }

    /// A special parameter (`$?`, `$1`), a lone `$`, or a D713 expansion.
    fn dollar(&mut self) -> bool {
        let test = self.test;
        match self.following() {
            // zsh reads `$#a` as a length and `$#a[x]` evaluates `x`; bash
            // 3.2 reads `$$(`, `$${` and `$$'` as if the second `$` began
            // an expansion. `[` splits a parameter into words, which may
            // become operators: only those that are always a number.
            Some(byte) if byte.is_ascii_digit() || b"?#$!@*-".contains(&byte) => {
                if !b"?#$!".contains(&byte) {
                    // In `[`, only quoted, and never `$@`'s several words.
                    if test == Some(Test::Single) && (!self.dquote || byte == b'@') {
                        return false;
                    }
                    self.expanded |= test == Some(Test::Single);
                    self.unquoted |= !self.dquote;
                }
                self.at += 2;
                self.expansion_ends()
            }
            Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
                if test == Some(Test::Single) && !self.dquote {
                    return false;
                }
                self.at += 1;
                self.name();
                self.expanded = true;
                self.unquoted |= !self.dquote;
                self.expansion_ends()
            }
            Some(b'{') if test != Some(Test::Single) || self.dquote => self.braced(),
            Some(b'(') if test.is_none() && self.closing != Close::Backquote => {
                if self.peek(/*offset*/ 2) == Some(b'(') {
                    self.arithmetic()
                } else {
                    self.substitution()
                }
            }
            Some(b' ' | b'\t' | b'\n' | b'"') | None => {
                self.at += 1;
                true
            }
            _ => false,
        }
    }

    /// Consumes a name's letters, digits and underscores.
    fn name(&mut self) -> usize {
        let start = self.at;
        while self
            .current()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.at += 1;
        }
        self.at - start
    }

    /// Nothing a shell reads as part of the expansion follows it: a
    /// subscript, a name character, or what bash 3.2 reads after `$`.
    fn expansion_ends(&self) -> bool {
        !self.current().is_some_and(|byte| {
            b"[({'".contains(&byte) || byte.is_ascii_alphanumeric() || byte == b'_'
        })
    }

    /// `${NAME}`, `${#NAME}`, `${NAME OP WORD}`.
    fn braced(&mut self) -> bool {
        self.at += 2;
        let length = self.current() == Some(b'#');
        if length {
            self.at += 1;
        }
        let name_start = self.at;
        let name_len = match self.current() {
            Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => self.name(),
            Some(byte) if byte.is_ascii_digit() => {
                while self.current().is_some_and(|byte| byte.is_ascii_digit()) {
                    self.at += 1;
                }
                self.at - name_start
            }
            _ => return false,
        };
        let name = &self.bytes[name_start..name_start + name_len];
        if !length && self.current() != Some(b'}') {
            let Some(operator) = BRACE_OPERATORS
                .iter()
                .find(|operator| self.rest().starts_with(operator))
            else {
                return false;
            };
            // `${OPTIND:=…}` assigns, which evaluates the value.
            if operator.contains(&b'=')
                && runs_code_from_variable(std::str::from_utf8(name).unwrap_or_default())
            {
                return false;
            }
            let pattern = matches!(operator.first(), Some(b'#' | b'%'));
            self.at += operator.len();
            loop {
                match self.current() {
                    Some(b'}') => break,
                    Some(byte)
                        if byte.is_ascii_alphanumeric() || BRACE_WORD_BYTES.contains(&byte) =>
                    {
                        self.at += 1;
                    }
                    // A parameter in a value, not in a pattern.
                    Some(b'$') if !pattern => match self.following() {
                        Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
                            self.at += 1;
                            self.name();
                        }
                        Some(byte) if byte.is_ascii_digit() || b"?#$!".contains(&byte) => {
                            self.at += 2;
                            if !self.current().is_some_and(|byte| {
                                byte == b'}' || BRACE_WORD_BYTES.contains(&byte)
                            }) {
                                return false;
                            }
                        }
                        _ => return false,
                    },
                    _ => return false,
                }
            }
        }
        if self.current() != Some(b'}') {
            return false;
        }
        self.at += 1;
        self.expanded = true;
        self.unquoted |= !self.dquote;
        // zsh subscripts `${x}[…]` too.
        !self.current().is_some_and(|byte| b"[({'".contains(&byte))
    }

    /// `$(( … ))` of digits, blanks, `+ - * / %` and `$?` `$#` `$$` `$!`.
    /// A name, or another parameter, has its value evaluated as an
    /// expression, which can run code.
    fn arithmetic(&mut self) -> bool {
        self.at += 3;
        loop {
            match self.current() {
                Some(b')') if self.following() == Some(b')') => {
                    self.at += 2;
                    self.expanded = true;
                    return true;
                }
                Some(byte) if byte.is_ascii_digit() || b" +-*/%".contains(&byte) => self.at += 1,
                Some(b'$') if self.following().is_some_and(|byte| b"?#$!".contains(&byte)) => {
                    self.at += 2;
                    if !self.expansion_ends() {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    }

    /// `$( … )`: a script of the same set.
    fn substitution(&mut self) -> bool {
        self.at += 2;
        if self.current() == Some(b')') {
            return false;
        }
        self.nested(Close::Paren, /*end*/ b')')
    }

    /// `` ` … ` ``: a script of the same set without quotes, escapes or
    /// `$(…)`; not inside another, nor in a test.
    fn backquoted(&mut self) -> bool {
        if self.closing == Close::Backquote || self.test.is_some() {
            return false;
        }
        self.at += 1;
        if self.current() == Some(b'`') {
            return false;
        }
        self.nested(Close::Backquote, /*end*/ b'`')
    }

    /// Lexes the script of a substitution up to `end`, which it consumes.
    fn nested(&mut self, closing: Close, end: u8) -> bool {
        if self.depth >= MAX_DEPTH {
            return false;
        }
        let saved = (
            self.redirects,
            self.command_start,
            self.closing,
            self.test,
            self.dquote,
        );
        self.depth += 1;
        self.redirects = 0;
        self.command_start = true;
        self.closing = closing;
        self.test = None;
        self.dquote = false;
        let read = self.script() && self.current() == Some(end);
        self.depth -= 1;
        (
            self.redirects,
            self.command_start,
            self.closing,
            self.test,
            self.dquote,
        ) = saved;
        if !read {
            return false;
        }
        self.at += 1;
        self.expanded = true;
        self.unquoted |= !self.dquote;
        true
    }

    /// One redirection per simple command: `<`, `>`, `>>` to a plain word,
    /// or `>&N`/`<&N` to a descriptor.
    fn redirect(&mut self) -> bool {
        self.redirects += 1;
        if self.redirects > 1 {
            return false;
        }
        let operator: &[u8] = if self.rest().starts_with(b">>") {
            b">>"
        } else if self.rest().starts_with(b">&") {
            b">&"
        } else if self.rest().starts_with(b"<&") {
            b"<&"
        } else if self.rest().starts_with(b"<<")
            || self.rest().starts_with(b"<>")
            || self.rest().starts_with(b">|")
        {
            return false;
        } else if self.current() == Some(b'<') {
            b"<"
        } else {
            b">"
        };
        self.at += operator.len();
        if matches!(operator, b">&" | b"<&") {
            let digits = self
                .rest()
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count();
            self.at += digits;
            return digits > 0 && self.word_ends_after(/*len*/ 0);
        }
        // `>(` and `<(` are process substitution.
        while matches!(self.current(), Some(b' ' | b'\t')) {
            self.at += 1;
        }
        let start = self.at;
        while self
            .current()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || b"_-./,:@%+=~".contains(&byte))
        {
            self.at += 1;
        }
        self.at > start && self.word_ends_after(/*len*/ 0)
    }
}

/// The length of the name `word` assigns (`NAME=…`, `NAME+=…`).
fn assignment_name(word: &[u8]) -> Option<usize> {
    let first = *word.first()?;
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    let len = word
        .iter()
        .position(|byte| !(byte.is_ascii_alphanumeric() || *byte == b'_'))?;
    (word[len..].starts_with(b"=") || word[len..].starts_with(b"+=")).then_some(len)
}

/// Whether a test operand `word` holding quotes or escapes starts, once
/// they are removed, with a byte `[` reads as (part of) an operator.
fn quoted_operator(word: &[u8]) -> bool {
    if !word.iter().any(|byte| matches!(byte, b'\'' | b'"' | b'\\')) {
        return false;
    }
    let mut at = 0;
    let first = loop {
        match (word.get(at), word.get(at + 1)) {
            (Some(b'\'' | b'"'), Some(next)) if *next == word[at] => at += 2,
            (Some(b'\\' | b'\'' | b'"'), next) => break next,
            (byte, _) => break byte,
        }
    };
    first.is_some_and(|byte| matches!(byte, b'-' | b'!' | b'(' | b')'))
}
