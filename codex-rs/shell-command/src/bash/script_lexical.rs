//! #2548 (D553): a lexical pre-filter before the syntax tree. tree-sitter-bash
//! and the shells split some text differently (line continuations, carriage
//! returns, pattern and regex tokens, braces in `${…}`, zsh subscripts and
//! glob qualifiers, here-document ends). Rather than chase each difference,
//! a script is read precisely only if every byte belongs to a small set of
//! constructs both read alike; any other script is unread, so it asks when
//! a prompt or forbidden rule exists.
//!
//! The safe set, outside quotes: words of ASCII letters, digits and
//! `_-./,:@%+=~*?!^]` (and `#` inside a word); spaces, tabs and newlines; `\` escaping one character
//! other than a newline; `;` `;;` `&` `&&` `|` `||` `|&` `(` `)`; the
//! standalone words `{` `}` `{}` `[` `]` `[[` `]]`; `$name`, `$1`, `$?`-style
//! specials, `${name}` and `${name:-word}` (`-` `=` `+` `?`, with or
//! without `:`); `$(…)`, `<(…)`, `>(…)` holding a safe script; redirections
//! `<` `>` `>>` `>|` `&>` `&>>` `N>&M` `N<&M` `<<<`; here-documents with a
//! plain delimiter (quoted, or unquoted with a body holding only text and
//! `$name`/`${name}`); comments starting a word. Inside `'…'` and `$'…'`
//! anything but a carriage return; inside `"…"` text, `\` escapes and the
//! same `$` forms. Never: backquotes, `$((`, `((`, `$[`, `==`, `=~`, `[`
//! after an expansion or inside a word, braces inside words, `(` after a
//! word other than `NAME=(`, `NAME+=(` and `name()` (zsh glob qualifiers,
//! `=(…)`), control characters, bytes above ASCII outside quotes.
//! Here-documents must end where the syntax tree ends them, and one started
//! before a `$(…)` is read after the line holding it, as bash does.

/// Where each here-document's delimiter line starts (after `<<-` tabs),
/// sorted, if `script` lies within the safe set (see the module docs). The
/// caller checks these against the syntax tree's.
pub(super) fn lexically_safe(script: &str, dialect: Dialect) -> Option<Vec<usize>> {
    let mut lexer = Lexer {
        bytes: script.as_bytes(),
        at: 0,
        heredocs: Vec::new(),
        heredoc_ends: Vec::new(),
        posix: dialect == Dialect::Posix,
    };
    (lexer.script(/*closing*/ None) && lexer.at == lexer.bytes.len() && lexer.heredocs.is_empty())
        .then(|| {
            lexer.heredoc_ends.sort_unstable();
            lexer.heredoc_ends
        })
}

use super::script_read::Dialect;

struct Heredoc {
    delimiter: Vec<u8>,
    quoted: bool,
    strip_tabs: bool,
}

struct Lexer<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Here-documents whose bodies start after the current line (of the
    /// current substitution: bash reads an outer one only after the line
    /// holding the whole substitution).
    heredocs: Vec<Heredoc>,
    heredoc_ends: Vec<usize>,
    /// `sh` may be dash, which lacks `$'…'`, `[[`, `&>`, `|&`, `<<<` and
    /// `<(…)`: there they mean something else.
    posix: bool,
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

    /// Whether the previous byte ends a word (so `#` starts a comment and a
    /// word stands alone).
    fn at_word_start(&self) -> bool {
        self.at == 0
            || matches!(
                self.bytes[self.at - 1],
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b'(' | b')'
            )
    }

    /// Whether `#` here starts a comment for bash: at the start of a word
    /// after whitespace or `;`, `&`, `|` (not after `)`, as in `x=(a)#`).
    fn at_comment_start(&self) -> bool {
        self.at == 0
            || matches!(
                self.bytes[self.at - 1],
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|'
            )
    }

    /// Whether the word starting here ends at `len` bytes.
    fn word_ends_after(&self, len: usize) -> bool {
        self.peek(len)
            .is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b')'))
    }

    /// `$(…)`, `<(…)` or `>(…)` from just after its `(`: its own
    /// here-documents end inside it.
    fn substitution(&mut self) -> bool {
        let outer = std::mem::take(&mut self.heredocs);
        let read =
            self.script(Some(b')')) && self.current() == Some(b')') && self.heredocs.is_empty();
        self.heredocs = outer;
        if read {
            self.at += 1;
        }
        read
    }

    /// Reads until the end, or until `closing` (`)`) at depth 0.
    fn script(&mut self, closing: Option<u8>) -> bool {
        while let Some(byte) = self.current() {
            match byte {
                b'\n' => {
                    self.at += 1;
                    if !self.heredoc_bodies() {
                        return false;
                    }
                }
                b' ' | b'\t' => self.at += 1,
                b')' if closing == Some(b')') => return true,
                b')' => self.at += 1,
                b'(' if self.following() == Some(b'(') => return false,
                b'(' => {
                    self.at += 1;
                    if !self.script(Some(b')')) || self.current() != Some(b')') {
                        return false;
                    }
                    self.at += 1;
                }
                b'#' if self.at_comment_start() => {
                    while self.current().is_some_and(|byte| byte != b'\n') {
                        if self.current() == Some(b'\r') {
                            return false;
                        }
                        self.at += 1;
                    }
                }
                b'&' | b'|'
                    if self.posix
                        && (self.rest().starts_with(b"|&") || self.rest().starts_with(b"&>")) =>
                {
                    return false;
                }
                b';' | b'&' | b'|' => self.at += 1,
                b'<' | b'>' => {
                    if !self.redirect() {
                        return false;
                    }
                }
                // A `case` arm's `)` would end the substitution or subshell
                // here as bash does not.
                b'c' if closing.is_some()
                    && self.at_word_start()
                    && self.rest().starts_with(b"case")
                    && self.word_ends_after(/*len*/ 4) =>
                {
                    return false;
                }
                b'[' | b']'
                    if self.posix
                        && (self.rest().starts_with(b"[[") || self.rest().starts_with(b"]]")) =>
                {
                    return false;
                }
                b'{' | b'}' | b'[' | b']' if self.at_word_start() => {
                    let standalone = [&b"{"[..], b"}", b"{}", b"[", b"]", b"[[", b"]]"]
                        .into_iter()
                        .find(|word| {
                            self.rest().starts_with(word) && self.word_ends_after(word.len())
                        });
                    let Some(word) = standalone else {
                        return false;
                    };
                    self.at += word.len();
                }
                _ => {
                    if !self.word() {
                        return false;
                    }
                }
            }
        }
        closing.is_none()
    }

    /// One word: unquoted bytes, quotes and expansions.
    fn word(&mut self) -> bool {
        let start = self.at;
        if (self.rest().starts_with(b"==") || self.rest().starts_with(b"=~"))
            && self.at_word_start()
        {
            return false;
        }
        while let Some(byte) = self.current() {
            match byte {
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b')' | b'<' | b'>' => return true,
                // Only `NAME=(…)`, `NAME+=(…)` and `name()` open a paren
                // after a word: elsewhere zsh reads glob qualifiers or
                // `=(…)` process substitution.
                b'(' => {
                    return self.at == start
                        || paren_after_word(&self.bytes[start..self.at], self.following());
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
                    if !self.dollar(/*quoted*/ false) {
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
        true
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
                    if !self.dollar(/*quoted*/ true) {
                        return false;
                    }
                }
                _ if byte < 0x20 && byte != b'\t' && byte != b'\n' => return false,
                _ => self.at += 1,
            }
        }
        false
    }

    /// `$name`, `$?`, `${name}`, `${name:-word}`, `$(…)`, `$'…'`.
    fn dollar(&mut self, quoted: bool) -> bool {
        match self.following() {
            Some(b'(') if self.peek(/*offset*/ 2) == Some(b'(') => false,
            Some(b'(') => {
                self.at += 2;
                self.substitution()
            }
            Some(b'\'') if !quoted && !self.posix => {
                self.at += 1;
                self.ansi_c_quoted()
            }
            Some(b'{') => self.braced_expansion(quoted),
            Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
                self.at += 1;
                while self
                    .current()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                {
                    self.at += 1;
                }
                // zsh reads `$name[…]` as a subscript.
                self.current() != Some(b'[')
            }
            // zsh reads `$#a` as a length and `$#a[x]` evaluates `x`.
            Some(byte) if byte.is_ascii_digit() || b"?#$!@*-".contains(&byte) => {
                self.at += 2;
                !self.current().is_some_and(|byte| {
                    byte == b'[' || byte.is_ascii_alphanumeric() || byte == b'_'
                })
            }
            // A `$` alone is a literal dollar.
            Some(b' ' | b'\t' | b'\n' | b'"') | None => {
                self.at += 1;
                true
            }
            _ => false,
        }
    }

    fn ansi_c_quoted(&mut self) -> bool {
        self.at += 1;
        while let Some(byte) = self.current() {
            self.at += 1;
            match byte {
                b'\'' => return true,
                b'\\' if self.current().is_some() => self.at += 1,
                b'\r' => return false,
                _ => {}
            }
        }
        false
    }

    /// `${name}` or `${name OP word}` for OP in `-` `=` `+` `?` (with or
    /// without `:`), where the word is plain text or a simple `$` form.
    fn braced_expansion(&mut self, quoted: bool) -> bool {
        self.at += 2;
        let name_start = self.at;
        while self
            .current()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.at += 1;
        }
        if self.at == name_start {
            match self.current() {
                Some(byte) if b"?#$!@*-".contains(&byte) => self.at += 1,
                _ => return false,
            }
        }
        if self.current() == Some(b':') {
            self.at += 1;
            if !self.current().is_some_and(|byte| b"-=+?".contains(&byte)) {
                return false;
            }
        }
        match self.current() {
            Some(b'}') => {
                self.at += 1;
                return self.current() != Some(b'[');
            }
            Some(byte) if b"-=+?".contains(&byte) => self.at += 1,
            _ => return false,
        }
        while let Some(byte) = self.current() {
            match byte {
                b'}' => {
                    self.at += 1;
                    return self.current() != Some(b'[');
                }
                b'{' | b'`' | b'\\' | b'\r' | b'\n' => return false,
                b'"' if !quoted => {
                    if !self.double_quoted() {
                        return false;
                    }
                }
                b'\'' if !quoted => {
                    if !self.single_quoted() {
                        return false;
                    }
                }
                b'$' => {
                    if !self.dollar(quoted) {
                        return false;
                    }
                }
                _ if byte.is_ascii_alphanumeric() || WORD_BYTES.contains(&byte) || byte == b' ' => {
                    self.at += 1;
                }
                _ => return false,
            }
        }
        false
    }

    /// A redirection operator and, for files, its target word.
    fn redirect(&mut self) -> bool {
        let operators: [&[u8]; 11] = [
            b"<<<", b"<<-", b"<<", b"&>>", b">>", b">|", b">&", b"<&", b"&>", b">", b"<",
        ];
        // A descriptor number glued before it belongs to the redirection.
        let Some(operator) = operators.into_iter().find(|op| self.rest().starts_with(op)) else {
            return false;
        };
        if self.posix && matches!(operator, b"<<<" | b"&>" | b"&>>") {
            return false;
        }
        self.at += operator.len();
        match operator {
            b"<<" | b"<<-" => return self.heredoc_start(operator == b"<<-"),
            b">&" | b"<&" => {
                // `2>&1`, `>&2`, `<&-`.
                let digits = self
                    .rest()
                    .iter()
                    .take_while(|byte| byte.is_ascii_digit())
                    .count();
                self.at += digits;
                if digits == 0 && self.current() == Some(b'-') {
                    self.at += 1;
                }
                return digits > 0 || self.bytes[self.at - 1] == b'-';
            }
            _ => {}
        }
        // Process substitutions `<(…)` and `>(…)`.
        if self.current() == Some(b'(') && matches!(operator, b">" | b"<") && !self.posix {
            self.at += 1;
            return self.substitution();
        }
        while matches!(self.current(), Some(b' ' | b'\t')) {
            self.at += 1;
        }
        let start = self.at;
        self.word() && self.at > start
    }

    /// `<<WORD`, `<<'WORD'`, `<<"WORD"`: the body follows the line.
    fn heredoc_start(&mut self, strip_tabs: bool) -> bool {
        while matches!(self.current(), Some(b' ' | b'\t')) {
            self.at += 1;
        }
        let quote = match self.current() {
            Some(quote @ (b'\'' | b'"')) => {
                self.at += 1;
                Some(quote)
            }
            _ => None,
        };
        let start = self.at;
        while self
            .current()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.at += 1;
        }
        if self.at == start {
            return false;
        }
        let delimiter = self.bytes[start..self.at].to_vec();
        if let Some(quote) = quote {
            if self.current() != Some(quote) {
                return false;
            }
            self.at += 1;
        }
        // Shells pair bodies with same-line here-documents first in, first
        // out; the grammar does not always: allow one per line.
        if !self.heredocs.is_empty() {
            return false;
        }
        self.heredocs.push(Heredoc {
            delimiter,
            quoted: quote.is_some(),
            strip_tabs,
        });
        // The delimiter must end the word.
        self.word_ends_after(/*len*/ 0)
    }

    /// After a newline: the bodies of the here-documents started on the line
    /// before, each up to a line that is exactly its delimiter.
    fn heredoc_bodies(&mut self) -> bool {
        for heredoc in std::mem::take(&mut self.heredocs) {
            loop {
                let Some(line_len) = self.rest().iter().position(|byte| *byte == b'\n') else {
                    // The last line may end the script without a newline.
                    let line = self.rest();
                    let line = if heredoc.strip_tabs {
                        trim_tabs(line)
                    } else {
                        line
                    };
                    if line == heredoc.delimiter.as_slice() {
                        self.heredoc_ends.push(self.bytes.len() - line.len());
                        self.at = self.bytes.len();
                        break;
                    }
                    return false;
                };
                let line_start = self.at;
                let line = &self.bytes[line_start..line_start + line_len];
                self.at += line_len + 1;
                let trimmed = if heredoc.strip_tabs {
                    trim_tabs(line)
                } else {
                    line
                };
                if trimmed == heredoc.delimiter.as_slice() {
                    self.heredoc_ends
                        .push(line_start + line_len - trimmed.len());
                    break;
                }
                if line.contains(&b'\r') || (!heredoc.quoted && !plain_heredoc_line(line)) {
                    return false;
                }
            }
        }
        true
    }
}

/// `word` (the text before a `(`) is `NAME=` or `NAME+=` (an array), or a
/// function name before `()`.
fn paren_after_word(word: &[u8], next: Option<u8>) -> bool {
    let name = |bytes: &[u8]| {
        bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    };
    if let Some(assigned) = word.strip_suffix(b"=") {
        return name(assigned.strip_suffix(b"+").unwrap_or(assigned));
    }
    next == Some(b')')
        && word
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.:-".contains(byte))
}

fn trim_tabs(line: &[u8]) -> &[u8] {
    let tabs = line.iter().take_while(|byte| **byte == b'\t').count();
    &line[tabs..]
}

/// A line of an unquoted here-document holding only text and `$name` or
/// `${name}` (no backquotes, backslashes, substitutions or subscripts).
fn plain_heredoc_line(line: &[u8]) -> bool {
    let mut at = 0;
    while let Some(&byte) = line.get(at) {
        at += 1;
        match byte {
            b'`' | b'\\' => return false,
            b'$' => match line.get(at) {
                Some(b'{') => {
                    let name = line[at + 1..]
                        .iter()
                        .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_')
                        .count();
                    if name == 0 || line.get(at + 1 + name) != Some(&b'}') {
                        return false;
                    }
                    at += name + 2;
                    if line.get(at) == Some(&b'[') {
                        return false;
                    }
                }
                Some(next) if next.is_ascii_alphabetic() || *next == b'_' => {
                    while line
                        .get(at)
                        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    {
                        at += 1;
                    }
                    if line.get(at) == Some(&b'[') {
                        return false;
                    }
                }
                // Only `$name`, `${name}` or a lone `$`: zsh reads `$#a[x]`,
                // `$=a`, `$+a` and the like.
                Some(b' ' | b'\t') | None => {}
                _ => return false,
            },
            _ => {}
        }
    }
    true
}
