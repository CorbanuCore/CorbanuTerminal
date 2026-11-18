//! #2548 (D553): a lexical pre-filter before the syntax tree. tree-sitter-bash
//! and the shells (bash 3.2 to 5, dash, zsh, ksh) split some text
//! differently: line continuations, carriage returns, here-documents,
//! pattern and regex tokens, `case` arms, `[ … ]`, `${…}` operators,
//! comments inside substitutions, subscripts, zsh's and ksh's own syntax.
//! Rather than chase each difference, a script is read precisely only if it
//! is built from a minimal POSIX set every one of them reads alike; any other
//! script is unread, so it asks whenever a prompt or forbidden rule exists.
//!
//! The minimal set: words of ASCII letters, digits and `_-./,:@%+=~*?!^]`
//! (and `#` inside a word); the standalone word `{}`; spaces, tabs and
//! newlines; `\` escaping one character other than a newline; `;` `&` `&&`
//! `|` `||`; `$name`, `$1`, `$?`-style specials and `${name}`; `$(…)`
//! holding a minimal script without comments; `'…'`; `"…"` holding text,
//! `\` escapes and the same `$` forms; at most one redirection per simple
//! command (`<` `>` `>>` `N>` `N>>` `N>&M` `>&M`) to a plain word or
//! descriptor; comments after whitespace outside substitutions. Nothing
//! else: no here-documents or here-strings, no `( … )`, `{ … }`, `[ … ]`,
//! `[[ … ]]` or `case`, no `$'…'`, backquotes, `$((`, `${x…}` operators,
//! process substitution, `==` or `=~`, no control characters, and no bytes
//! above ASCII outside quotes.

/// Whether `script` lies within the minimal set (see the module docs).
pub(super) fn lexically_safe(script: &str) -> bool {
    let mut lexer = Lexer {
        bytes: script.as_bytes(),
        at: 0,
        redirects: 0,
        command_start: true,
    };
    lexer.script(/*substitution*/ false) && lexer.at == lexer.bytes.len()
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
            .is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b')'))
    }

    /// Reads a script to its end, or a substitution's to its `)`.
    fn script(&mut self, substitution: bool) -> bool {
        while let Some(byte) = self.current() {
            match byte {
                b' ' | b'\t' => self.at += 1,
                b'\n' | b';' | b'&' | b'|' => {
                    // `|&`, `&>` and `;;` are not in the set; `&&` and `||`
                    // are.
                    if (byte == b'|' && self.following() == Some(b'&'))
                        || (byte == b'&' && self.following() == Some(b'>'))
                        || (byte == b';' && self.following() == Some(b';'))
                    {
                        return false;
                    }
                    self.redirects = 0;
                    self.command_start = true;
                    self.at += 1;
                }
                b')' if substitution => return true,
                b')' => return false,
                // bash 3.2 ends a comment inside `$(…)` differently.
                b'#' if self.after_whitespace() && !substitution => {
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
                    self.at += 2;
                }
                // `case` arms, `[[`, `==` and the like never get here: their
                // bytes are not word bytes, or the word is refused below.
                _ => {
                    if !self.word() {
                        return false;
                    }
                }
            }
        }
        !substitution
    }

    /// One word: unquoted bytes, quotes and expansions. A digit-only word
    /// glued to `<` or `>` is that redirection's descriptor.
    fn word(&mut self) -> bool {
        let start = self.at;
        while let Some(byte) = self.current() {
            match byte {
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b')' => break,
                // A `#` starting a word anywhere but after whitespace at the
                // top level: shells disagree on whether it starts a comment.
                b'#' if self.at == start => return false,
                b'<' | b'>' => {
                    let descriptor = self.bytes[start..self.at].iter().all(u8::is_ascii_digit);
                    if !descriptor || self.at == start {
                        return false;
                    }
                    break;
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
                b'\\' => match self.following() {
                    Some(b'\n' | b'\r') | None => return false,
                    Some(_) => self.at += 2,
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
                    b"case" | b"esac" | b"select" | b"function" | b"coproc" | b"[" | b"[["
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
                b'`' | b'\r' => return false,
                b'\\' => match self.following() {
                    Some(b'\n' | b'\r') | None => return false,
                    Some(_) => self.at += 2,
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

    /// `$name`, `$1`, `$?`-style specials, `${name}`, `$(…)`, a lone `$`.
    fn dollar(&mut self) -> bool {
        let name_byte = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
        match self.following() {
            Some(b'(') if self.peek(/*offset*/ 2) == Some(b'(') => false,
            Some(b'(') => {
                self.at += 2;
                let outer = (self.redirects, self.command_start);
                self.redirects = 0;
                self.command_start = true;
                let read = self.script(/*substitution*/ true) && self.current() == Some(b')');
                (self.redirects, self.command_start) = outer;
                self.at += 1;
                read
            }
            Some(b'{') => {
                self.at += 2;
                let start = self.at;
                while self.current().is_some_and(name_byte) {
                    self.at += 1;
                }
                if self.at == start || self.current() != Some(b'}') {
                    return false;
                }
                self.at += 1;
                // zsh reads `${name}[…]` as a subscript.
                self.current() != Some(b'[')
            }
            Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
                self.at += 1;
                while self.current().is_some_and(name_byte) {
                    self.at += 1;
                }
                self.current() != Some(b'[')
            }
            // zsh reads `$#a` as a length and `$#a[x]` evaluates `x`.
            Some(byte) if byte.is_ascii_digit() || b"?#$!@*-".contains(&byte) => {
                self.at += 2;
                !self
                    .current()
                    .is_some_and(|byte| byte == b'[' || name_byte(byte))
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
        } else if self.rest().starts_with(b"<<") || self.rest().starts_with(b"<>") {
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
