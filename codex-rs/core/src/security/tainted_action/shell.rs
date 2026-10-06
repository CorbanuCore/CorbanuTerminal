//! Shell-like word splitting for the post-taint classifier.
//!
//! This over-approximates the shell on purpose: quotes are removed rather
//! than honoured, so the text inside `sh -c '...'` or `python -c "..."` is
//! split into commands too. Command boundaries, pipes and substitutions are
//! kept so wrappers, pipelines and code fed to an interpreter can be followed.

/// Shells whose `-c` argument is its own command line.
pub(super) const SHELLS: &[&str] = &["sh", "bash", "zsh", "dash", "ksh", "fish"];
/// Words one brace expression may expand to before it is left as written.
const MAX_BRACE_EXPANSION: usize = 64;
/// Placeholder for the output of a substitution the classifier cannot see.
pub(super) const UNSEEN_OUTPUT: &str = "$__substitution__";

#[derive(Debug, Default)]
pub(super) struct SimpleCommand {
    /// Words with quotes and backslashes removed, original case.
    pub(super) words: Vec<String>,
    /// The command whose output is this command's stdin (`a | b`).
    pub(super) pipe_from: Option<usize>,
    /// It runs inside `$(...)`, `<(...)` or backticks.
    pub(super) substituted: bool,
    /// Commands inside this command's substitutions; their output is part of
    /// its words.
    pub(super) fed_by: Vec<usize>,
}

pub(super) fn basename(word: &str) -> &str {
    word.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(word)
}

/// Simple commands of the argv and of any script inside it (`bash -lc "..."`),
/// in the order they finish: a substitution comes before the command using it.
pub(super) fn simple_commands(command: &[String]) -> Vec<SimpleCommand> {
    let mut lexer = Lexer::default();
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
        for expanded in expand_braces(&replace_ifs(arg)) {
            lexer.arg(&expanded);
            lexer.flush();
        }
    }
    lexer.finish()
}

/// `$IFS` and `${IFS}` separate words.
fn replace_ifs(arg: &str) -> String {
    let arg = arg.replace("${IFS}", " ");
    super::paths::replace_variable(&arg, "IFS", " ")
}

/// Bounded brace expansion: `~/.{x,a}ws` names `~/.xws` and `~/.aws`. Each
/// alternative becomes its own argument; too many are left as written.
fn expand_braces(arg: &str) -> Vec<String> {
    let mut words = vec![arg.to_string()];
    loop {
        let mut next = Vec::new();
        let mut expanded = false;
        for word in &words {
            match brace_alternatives(word) {
                Some(alternatives) => {
                    expanded = true;
                    next.extend(alternatives);
                }
                None => next.push(word.clone()),
            }
            if next.len() > MAX_BRACE_EXPANSION {
                return vec![arg.to_string()];
            }
        }
        words = next;
        if !expanded {
            return words;
        }
    }
}

/// The alternatives of the first innermost `{a,b}` group, if any.
fn brace_alternatives(word: &str) -> Option<Vec<String>> {
    let close = word.find('}')?;
    let open = word[..close].rfind('{')?;
    let inner = &word[open + 1..close];
    if !inner.contains(',') || inner.contains(char::is_whitespace) {
        return None;
    }
    let (prefix, suffix) = (&word[..open], &word[close + 1..]);
    Some(
        inner
            .split(',')
            .map(|alternative| format!("{prefix}{alternative}{suffix}"))
            .collect(),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Start {
    Plain,
    Piped,
}

/// An open substitution: the command it interrupts and where its own
/// commands start.
struct Open {
    outer: SimpleCommand,
    outer_word: String,
    first: usize,
    backtick: bool,
}

#[derive(Default)]
struct Lexer {
    /// Finished commands, in order.
    done: Vec<SimpleCommand>,
    current: SimpleCommand,
    word: String,
    open: Vec<Open>,
}

impl Lexer {
    fn flush(&mut self) {
        if !self.word.is_empty() {
            let word = std::mem::take(&mut self.word);
            self.current.words.push(word);
        }
    }

    fn finish_current(&mut self) -> Option<usize> {
        self.flush();
        if self.current.words.is_empty() && self.current.fed_by.is_empty() {
            let substituted = self.current.substituted;
            self.current = SimpleCommand {
                substituted,
                ..SimpleCommand::default()
            };
            return None;
        }
        let substituted = self.current.substituted;
        let finished = std::mem::replace(
            &mut self.current,
            SimpleCommand {
                substituted,
                ..SimpleCommand::default()
            },
        );
        self.done.push(finished);
        Some(self.done.len() - 1)
    }

    /// End the current command; a piped one reads the previous one's output.
    fn start(&mut self, start: Start) {
        let finished = self.finish_current();
        if start == Start::Piped {
            self.current.pipe_from = finished.or(self.done.len().checked_sub(1));
        }
    }

    fn open_substitution(&mut self, backtick: bool) {
        let outer = std::mem::replace(
            &mut self.current,
            SimpleCommand {
                substituted: true,
                ..SimpleCommand::default()
            },
        );
        let outer_word = std::mem::take(&mut self.word);
        self.open.push(Open {
            outer,
            outer_word,
            first: self.done.len(),
            backtick,
        });
    }

    /// Close the innermost substitution and continue the command it
    /// interrupted. `echo`/`printf` output is spliced into the word; any
    /// other output is a placeholder the classifier cannot see through.
    fn close_substitution(&mut self) {
        self.finish_current();
        let Some(open) = self.open.pop() else {
            return;
        };
        let inner: Vec<usize> = (open.first..self.done.len()).collect();
        let spliced = match inner.as_slice() {
            [only] => literal_output(&self.done[*only].words),
            _ => None,
        };
        self.current = open.outer;
        self.current.fed_by.extend(inner);
        self.word = open.outer_word;
        self.word
            .push_str(spliced.as_deref().unwrap_or(UNSEEN_OUTPUT));
    }

    fn finish(mut self) -> Vec<SimpleCommand> {
        while !self.open.is_empty() {
            self.close_substitution();
        }
        self.finish_current();
        self.done
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
                    self.open_substitution(/*backtick*/ false);
                    index += 2;
                    continue;
                }
                // A line continuation joins the two lines.
                '\\' if next == Some('\n') => index += 1,
                '`' => {
                    if self.open.last().is_some_and(|open| open.backtick) {
                        self.close_substitution();
                    } else {
                        self.open_substitution(/*backtick*/ true);
                    }
                }
                ')' if self.open.last().is_some_and(|open| !open.backtick) => {
                    self.close_substitution();
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

/// What `echo`/`printf` with these words prints, if that is all they are.
pub(super) fn literal_output(words: &[String]) -> Option<String> {
    let (first, rest) = words.split_first()?;
    if !matches!(basename(&first.to_lowercase()), "echo" | "printf") {
        return None;
    }
    let printed: Vec<&str> = rest
        .iter()
        .map(String::as_str)
        .skip_while(|word| word.starts_with('-'))
        .collect();
    Some(printed.join(" "))
}
