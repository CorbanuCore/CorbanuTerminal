//! #2548 (D553): a lexical pre-filter before the syntax tree. tree-sitter-bash
//! and the shells (bash 3.2 to 5, dash, zsh, ksh) split some text
//! differently: line continuations, carriage returns, here-documents,
//! pattern and regex tokens, `case` arms, `[ … ]`, expansions and the scans
//! bash 3.2 makes of `$(…)`, comments, subscripts, multibyte locales, zsh's
//! and ksh's own syntax. Rather than chase each difference, a script is read
//! precisely only if it is built from a minimal POSIX set every one of them
//! reads alike (the coordinator's literal list); any other script fails
//! closed: it asks whenever a prompt or forbidden rule exists.
//!
//! The minimal set: words of ASCII letters, digits and `_-./,:@%+=~*?!^]`
//! (and `#` inside a word); the standalone word `{}`; spaces, tabs and
//! newlines; `\` escaping one visible ASCII character; `;` `&&` `|`
//! `||`; `'…'`; `"…"` holding ASCII text and `\` escapes; the special
//! parameters `$?`, `$#`, `$$`, `$!`, `$@`, `$*`, `$-`, `$0`–`$9` (not
//! followed by `(`, `{`, `'`, `[` or a name character); at most one
//! redirection per simple command (`<` `>` `>>` `N>` `N>>` `N>&M` `>&M`) to
//! a plain word or descriptor; comments after whitespace. Nothing else: no
//! `$name`, `${…}`, `$(…)` or backquotes, no here-documents or here-strings,
//! no `&` alone, no `( … )`, `{ … }`, `[ … ]`, `[[ … ]]` or `case`, no
//! `$'…'`, process substitution, `==` or `=~`, no `=` right after a quote
//! (zsh's `""=cmd`), no control characters, and no bytes above ASCII outside
//! single quotes (a multibyte locale can make one swallow a `\`).

/// Whether `script` lies within the minimal set (see the module docs).
pub(super) fn lexically_safe(script: &str) -> bool {
    let mut lexer = Lexer {
        bytes: script.as_bytes(),
        at: 0,
        redirects: 0,
        command_start: true,
    };
    lexer.script()
}

struct Lexer<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Redirections in the current simple command.
    redirects: usize,
    /// The next word is in command position.
    command_start: bool,
}

const WORD_BYTES: &[u8] = b"_-./,:@%+=~*?!^]#";

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

    /// Whether the previous byte is whitespace (or there is none), so a word
    /// starting here stands alone and `#` starts a comment.
    fn after_whitespace(&self) -> bool {
        self.at == 0 || matches!(self.bytes[self.at - 1], b' ' | b'\t' | b'\n')
    }

    /// Whether a word of `len` bytes starting here ends there.
    fn word_ends_after(&self, len: usize) -> bool {
        self.peek(len)
            .is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b';' | b'&' | b'|'))
    }

    fn script(&mut self) -> bool {
        while let Some(byte) = self.current() {
            match byte {
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
                    && self.rest().starts_with(b"{}")
                    && self.word_ends_after(/*len*/ 2) =>
                {
                    self.command_start = false;
                    self.at += 2;
                }
                _ => {
                    if !self.word() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// One word: unquoted bytes, quotes and special parameters. A
    /// digit-only word glued to `<` or `>` is that redirection's descriptor.
    fn word(&mut self) -> bool {
        let start = self.at;
        while let Some(byte) = self.current() {
            match byte {
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' => break,
                // Not after whitespace: shells disagree on whether it starts
                // a comment.
                b'#' if self.at == start => return false,
                b'<' | b'>' => {
                    let descriptor = self.bytes[start..self.at].iter().all(u8::is_ascii_digit);
                    if !descriptor || self.at == start {
                        return false;
                    }
                    break;
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
                // Only before a visible character: the grammar takes `\`
                // with a blank for whitespace (`echo \ #; cmd`), and before a
                // newline for a continuation.
                b'\\' => match self.following() {
                    Some(0x21..=0x7e) => self.at += 2,
                    _ => return false,
                },
                _ if byte.is_ascii_alphanumeric() || WORD_BYTES.contains(&byte) => self.at += 1,
                _ => return false,
            }
        }
        let word = &self.bytes[start..self.at];
        // Test operators the grammar reads as patterns, anywhere; reserved
        // words of constructs outside the set, in command position (as
        // arguments they are ordinary words).
        let command_word = std::mem::replace(
            &mut self.command_start,
            // These keep the next word in command position.
            matches!(
                word,
                b"!" | b"if" | b"then" | b"else" | b"elif" | b"while" | b"until" | b"do" | b"time"
            ) || word
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
            && !(command_word
                && matches!(
                    word,
                    b"case" | b"esac" | b"select" | b"function" | b"coproc"
                ))
    }

    fn single_quoted(&mut self) -> bool {
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
        self.at += 1;
        while let Some(byte) = self.current() {
            match byte {
                b'"' => {
                    self.at += 1;
                    return true;
                }
                // A multibyte locale (Big5, SJIS) can make the byte before a
                // `\` its lead byte.
                b'`' | b'\r' | 0x80.. => return false,
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

    /// A special parameter (`$?`, `$1`) or a lone `$`; nothing else.
    fn dollar(&mut self) -> bool {
        match self.following() {
            // zsh reads `$#a` as a length and `$#a[x]` evaluates `x`; bash
            // 3.2 reads `$$(`, `$${` and `$$'` as if the second `$` began
            // an expansion.
            Some(byte) if byte.is_ascii_digit() || b"?#$!@*-".contains(&byte) => {
                self.at += 2;
                !self.current().is_some_and(|byte| {
                    b"[({'".contains(&byte) || byte.is_ascii_alphanumeric() || byte == b'_'
                })
            }
            Some(b' ' | b'\t' | b'\n' | b'"') | None => {
                self.at += 1;
                true
            }
            _ => false,
        }
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
