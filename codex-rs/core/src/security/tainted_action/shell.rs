//! Shell-like word splitting for the post-taint classifier.
//!
//! This over-approximates the shell on purpose: quotes are removed rather
//! than honoured, so the text inside `sh -c '...'` or `python -c "..."` is
//! split into commands too, and command boundaries are kept so wrappers and
//! pipelines can be followed.

/// Shells whose `-c` argument is its own command line.
pub(super) const SHELLS: &[&str] = &["sh", "bash", "zsh", "dash", "ksh", "fish"];

#[derive(Debug, Default)]
pub(super) struct SimpleCommand {
    /// Words with quotes and backslashes removed, original case.
    pub(super) words: Vec<String>,
    /// Its stdin is the previous command's output (`a | b`).
    pub(super) piped: bool,
    /// It runs inside `$(...)`, `<(...)` or backticks; its output feeds the
    /// command before it.
    pub(super) substituted: bool,
    /// Its text contains a command or process substitution.
    pub(super) takes_substitution: bool,
}

pub(super) fn basename(word: &str) -> &str {
    word.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(word)
}

/// Simple commands of the argv and of any script inside it (`bash -lc "..."`).
pub(super) fn simple_commands(command: &[String]) -> Vec<SimpleCommand> {
    let mut lexer = Lexer {
        done: Vec::new(),
        current: SimpleCommand::default(),
        word: String::new(),
        in_backtick: false,
    };
    let mut previous: Option<&str> = None;
    for arg in command {
        // The script after a shell's `-c`/`-lc` is its own command line.
        let in_shell = lexer
            .current
            .words
            .iter()
            .any(|word| SHELLS.contains(&basename(&word.to_lowercase())));
        if in_shell
            && previous.is_some_and(|flag| {
                flag.starts_with('-') && !flag.starts_with("--") && flag.ends_with('c')
            })
        {
            lexer.start(Start::Plain);
        }
        previous = Some(arg.as_str());
        lexer.arg(arg);
        lexer.flush();
    }
    lexer
        .done
        .into_iter()
        .chain(std::iter::once(lexer.current))
        .filter(|command| !command.words.is_empty() || command.takes_substitution)
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Start {
    Plain,
    Piped,
}

struct Lexer {
    /// Finished commands, in order.
    done: Vec<SimpleCommand>,
    current: SimpleCommand,
    word: String,
    in_backtick: bool,
}

impl Lexer {
    fn flush(&mut self) {
        if !self.word.is_empty() {
            let word = std::mem::take(&mut self.word);
            self.current.words.push(word);
        }
    }

    /// End the current command. An empty one is reused unless it already
    /// carries a substitution of its own.
    fn start(&mut self, start: Start) {
        self.flush();
        if !self.current.words.is_empty() || self.current.takes_substitution {
            self.done.push(std::mem::take(&mut self.current));
        }
        self.current.piped |= start == Start::Piped;
    }

    fn open_substitution(&mut self) {
        self.flush();
        self.current.takes_substitution = true;
        let substitution = SimpleCommand {
            substituted: true,
            ..SimpleCommand::default()
        };
        self.done
            .push(std::mem::replace(&mut self.current, substitution));
    }

    fn arg(&mut self, arg: &str) {
        let chars: Vec<char> = arg.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let ch = chars[index];
            let next = chars.get(index + 1).copied();
            match ch {
                '$' if next == Some('\'') => {
                    index = self.ansi_c(&chars, index + 2);
                    continue;
                }
                '$' | '<' if next == Some('(') => {
                    self.open_substitution();
                    index += 2;
                    continue;
                }
                '`' => {
                    if self.in_backtick {
                        self.start(Start::Plain);
                    } else {
                        self.open_substitution();
                    }
                    self.in_backtick = !self.in_backtick;
                }
                '\'' | '"' | '\\' | '{' | '}' => {}
                '|' => {
                    let pipe = next != Some('|') && (index == 0 || chars[index - 1] != '|');
                    self.start(if pipe { Start::Piped } else { Start::Plain });
                }
                ';' | '&' | '(' | ')' | '\n' => self.start(Start::Plain),
                ch if ch.is_whitespace() || matches!(ch, '<' | '>' | ',') => self.flush(),
                ch => self.word.push(ch),
            }
            index += 1;
        }
    }

    /// Decode a bash `$'...'` string starting at `index` (after the quote)
    /// into the current word; returns the index after the closing quote.
    fn ansi_c(&mut self, chars: &[char], mut index: usize) -> usize {
        let digits = |chars: &[char], start: usize, radix: u32, max: usize| {
            let text: String = chars[start..]
                .iter()
                .take(max)
                .take_while(|ch| ch.is_digit(radix))
                .collect();
            let value = u32::from_str_radix(&text, radix).ok();
            (value.and_then(char::from_u32), text.len())
        };
        while index < chars.len() {
            match chars[index] {
                '\'' => return index + 1,
                '\\' if index + 1 < chars.len() => {
                    let escape = chars[index + 1];
                    index += 2;
                    let simple = match escape {
                        'n' => Some('\n'),
                        't' => Some('\t'),
                        'r' => Some('\r'),
                        'a' => Some('\u{7}'),
                        'b' => Some('\u{8}'),
                        'e' | 'E' => Some('\u{1b}'),
                        'f' => Some('\u{c}'),
                        'v' => Some('\u{b}'),
                        'x' | 'u' | 'U' | '0'..='7' => None,
                        other => Some(other),
                    };
                    if let Some(ch) = simple {
                        self.word.push(ch);
                        continue;
                    }
                    let (decoded, used) = match escape {
                        'x' => digits(chars, index, 16, 2),
                        'u' => digits(chars, index, 16, 4),
                        'U' => digits(chars, index, 16, 8),
                        _ => {
                            // Octal: the escape character is the first digit.
                            index -= 1;
                            digits(chars, index, 8, 3)
                        }
                    };
                    index += used;
                    self.word.extend(decoded);
                }
                ch => {
                    self.word.push(ch);
                    index += 1;
                }
            }
        }
        index
    }
}
