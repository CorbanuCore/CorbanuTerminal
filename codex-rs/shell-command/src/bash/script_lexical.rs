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
//! after an expansion or inside a word, `X(` for a glob character X, braces
//! inside words, control characters, bytes above ASCII outside quotes.

/// Whether `script` lies within the safe set (see the module docs).
pub(super) fn lexically_safe(script: &str) -> bool {
    let mut lexer = Lexer {
        bytes: script.as_bytes(),
        at: 0,
        heredocs: Vec::new(),
    };
    lexer.script(/*closing*/ None) && lexer.at == lexer.bytes.len() && lexer.heredocs.is_empty()
}

struct Heredoc {
    delimiter: Vec<u8>,
    quoted: bool,
    strip_tabs: bool,
}

struct Lexer<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Here-documents whose bodies start after the current line.
    heredocs: Vec<Heredoc>,
}

const WORD_BYTES: &[u8] = b"_-./,:@%+=~*?!^]#";
const GLOB_BYTES: &[u8] = b"*?+@!";

impl Lexer<'_> {
    fn peek(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.at + offset).copied()
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

    /// Reads until the end, or until `closing` (`)`) at depth 0.
    fn script(&mut self, closing: Option<u8>) -> bool {
        while let Some(byte) = self.peek(0) {
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
                b'(' if self.peek(1) == Some(b'(') => return false,
                b'(' => {
                    self.at += 1;
                    if !self.script(Some(b')')) || self.peek(0) != Some(b')') {
                        return false;
                    }
                    self.at += 1;
                }
                b'#' if self.at_comment_start() => {
                    while self.peek(0).is_some_and(|byte| byte != b'\n') {
                        if self.peek(0) == Some(b'\r') {
                            return false;
                        }
                        self.at += 1;
                    }
                }
                b';' | b'&' | b'|' => self.at += 1,
                b'<' | b'>' => {
                    if !self.redirect() {
                        return false;
                    }
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
        if (self.rest().starts_with(b"==") || self.rest().starts_with(b"=~"))
            && self.at_word_start()
        {
            return false;
        }
        while let Some(byte) = self.peek(0) {
            match byte {
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b')' | b'<' | b'>' => return true,
                b'(' if self.at > 0 && GLOB_BYTES.contains(&self.bytes[self.at - 1]) => {
                    return false;
                }
                b'(' => return true,
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
                b'\\' => match self.peek(1) {
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
        while let Some(byte) = self.peek(0) {
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
        while let Some(byte) = self.peek(0) {
            match byte {
                b'"' => {
                    self.at += 1;
                    return true;
                }
                b'`' | b'\r' => return false,
                b'\\' => match self.peek(1) {
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
        match self.peek(1) {
            Some(b'(') if self.peek(2) == Some(b'(') => false,
            Some(b'(') => {
                self.at += 2;
                if !self.script(Some(b')')) || self.peek(0) != Some(b')') {
                    return false;
                }
                self.at += 1;
                true
            }
            Some(b'\'') if !quoted => {
                self.at += 1;
                self.ansi_c_quoted()
            }
            Some(b'{') => self.braced_expansion(quoted),
            Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
                self.at += 1;
                while self
                    .peek(0)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                {
                    self.at += 1;
                }
                // zsh reads `$name[…]` as a subscript.
                self.peek(0) != Some(b'[')
            }
            Some(byte) if byte.is_ascii_digit() || b"?#$!@*-".contains(&byte) => {
                self.at += 2;
                self.peek(0) != Some(b'[')
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
        while let Some(byte) = self.peek(0) {
            self.at += 1;
            match byte {
                b'\'' => return true,
                b'\\' if self.peek(0).is_some() => self.at += 1,
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
            .peek(0)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.at += 1;
        }
        if self.at == name_start {
            match self.peek(0) {
                Some(byte) if b"?#$!@*-".contains(&byte) => self.at += 1,
                _ => return false,
            }
        }
        if self.peek(0) == Some(b':') {
            self.at += 1;
            if !self.peek(0).is_some_and(|byte| b"-=+?".contains(&byte)) {
                return false;
            }
        }
        match self.peek(0) {
            Some(b'}') => {
                self.at += 1;
                return self.peek(0) != Some(b'[');
            }
            Some(byte) if b"-=+?".contains(&byte) => self.at += 1,
            _ => return false,
        }
        while let Some(byte) = self.peek(0) {
            match byte {
                b'}' => {
                    self.at += 1;
                    return self.peek(0) != Some(b'[');
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
                if digits == 0 && self.peek(0) == Some(b'-') {
                    self.at += 1;
                }
                return digits > 0 || self.bytes[self.at - 1] == b'-';
            }
            _ => {}
        }
        // Process substitutions `<(…)` and `>(…)`.
        if self.peek(0) == Some(b'(') && matches!(operator, b">" | b"<") {
            self.at += 1;
            if !self.script(Some(b')')) || self.peek(0) != Some(b')') {
                return false;
            }
            self.at += 1;
            return true;
        }
        while matches!(self.peek(0), Some(b' ' | b'\t')) {
            self.at += 1;
        }
        let start = self.at;
        self.word() && self.at > start
    }

    /// `<<WORD`, `<<'WORD'`, `<<"WORD"`: the body follows the line.
    fn heredoc_start(&mut self, strip_tabs: bool) -> bool {
        while matches!(self.peek(0), Some(b' ' | b'\t')) {
            self.at += 1;
        }
        let quote = match self.peek(0) {
            Some(quote @ (b'\'' | b'"')) => {
                self.at += 1;
                Some(quote)
            }
            _ => None,
        };
        let start = self.at;
        while self
            .peek(0)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.at += 1;
        }
        if self.at == start {
            return false;
        }
        let delimiter = self.bytes[start..self.at].to_vec();
        if let Some(quote) = quote {
            if self.peek(0) != Some(quote) {
                return false;
            }
            self.at += 1;
        }
        self.heredocs.push(Heredoc {
            delimiter,
            quoted: quote.is_some(),
            strip_tabs,
        });
        // The delimiter must end the word.
        self.word_ends_after(0)
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
                        self.at = self.bytes.len();
                        break;
                    }
                    return false;
                };
                let line = &self.bytes[self.at..self.at + line_len];
                self.at += line_len + 1;
                let trimmed = if heredoc.strip_tabs {
                    trim_tabs(line)
                } else {
                    line
                };
                if trimmed == heredoc.delimiter.as_slice() {
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
                Some(b'(' | b'[') => return false,
                _ => {}
            },
            _ => {}
        }
    }
    true
}
