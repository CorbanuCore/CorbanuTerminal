//! #623: the agent settings folder names (`.codex`, `.agents`) as a command
//! may spell them: any letter case (macOS and Windows disks ignore it), a
//! glob (`.agen*`, `.[[:alpha:]]gents`), a brace (`.agent{s,}`) or zsh group
//! (`.agent(s|)`) that could make one, quotes or a backslash inside the name
//! (`.age''nts`). Defense in depth for ordinary link commands, bounded; a
//! name built at run time (`$'\x2eagents'`, `$(printf …)`) is not seen.

/// The agent settings folder names.
const AGENT_SETTINGS: [&str; 2] = [".codex", ".agents"];
/// Spellings of one word looked at (brace and group alternatives); past it
/// the word counts as naming one.
const MAX_VARIANTS: usize = 64;
/// Characters that end a name in command text: whitespace, path separators
/// and shell syntax (`$(…)`, quotes, `=`, `:`, `,`, redirections).
const NAME_DELIMITERS: [char; 22] = [
    '/', '\\', '$', '(', ')', '{', '}', '"', '\'', '`', ',', ':', ';', '=', '<', '>', '|', '&',
    ' ', '\t', '\n', '\r',
];

/// Whether `name` is an agent settings folder name, or a glob that could
/// match one.
fn is_settings_name(name: &str) -> bool {
    // `${X:-.agents}`: the default after `-`.
    let name = name.trim_start_matches('-');
    AGENT_SETTINGS.iter().any(|settings| {
        name.eq_ignore_ascii_case(settings)
            || (name.contains(['*', '?', '[']) && glob_matches(name, settings))
    })
}

/// The first agent settings name in `text` (a command's words), read with
/// every shell delimiter splitting names (`"$(realpath .agents)"`,
/// `A=.agents`, `$'.agents'`), and again with quotes and backslashes taken
/// out of each word; brace and zsh group alternatives are expanded. A word
/// with more alternatives than are looked at is named whole.
pub(super) fn settings_name_in(text: &str) -> Option<String> {
    for word in text.split_whitespace() {
        let unquoted: String = word
            .chars()
            .filter(|c| !matches!(c, '\'' | '"' | '\\'))
            .collect();
        // zsh: `.agent(s|)` is `.agent{s,}`.
        let grouped: String = unquoted
            .chars()
            .map(|c| match c {
                '(' => '{',
                '|' => ',',
                ')' => '}',
                c => c,
            })
            .collect();
        // zsh: `.(agents)` is `.agents`.
        let ungrouped: String = unquoted
            .chars()
            .filter(|c| !matches!(c, '(' | ')'))
            .collect();
        for spelling in [word, &unquoted, &grouped, &ungrouped] {
            let Some(variants) = brace_variants(spelling) else {
                return Some(word.to_string());
            };
            for variant in variants {
                // Split on every delimiter, and on path separators only (a
                // glob class such as `[[:alpha:]]` holds `:`).
                let found = variant
                    .split(NAME_DELIMITERS)
                    .chain(variant.split(['/', '\\']))
                    .find(|name| is_settings_name(name));
                if let Some(name) = found {
                    return Some(name.trim_start_matches('-').to_string());
                }
            }
        }
    }
    None
}

/// `word` with its `{a,b}` groups expanded (the shell's brace expansion); a
/// word without one is itself. `None` past [`MAX_VARIANTS`], and for a word
/// with a nested group or a sequence (`{a..z}`): such a word is named whole,
/// so `ln -s src{1..3} d` is refused too (refusing more is intended).
fn brace_variants(word: &str) -> Option<Vec<String>> {
    // Nested groups (`{{s,},x}`) and sequences (`{a..z}`) are not expanded
    // here: such a word is named whole.
    let nested_or_sequence = word.match_indices('{').any(|(open, _)| {
        word[open + 1..].find('}').is_some_and(|close| {
            let inside = &word[open + 1..open + 1 + close];
            inside.contains('{') || inside.contains("..")
        })
    });
    if nested_or_sequence {
        return None;
    }
    let mut variants = vec![word.to_string()];
    let mut done = Vec::new();
    while let Some(variant) = variants.pop() {
        // The first `{…}` holding a comma (`${N}.agent{s,}`: not `{N}`).
        let group = variant.match_indices('{').find_map(|(open, _)| {
            variant[open..]
                .find('}')
                .map(|close| (open, open + close))
                .filter(|(open, close)| variant[*open..*close].contains(','))
        });
        let Some((open, close)) = group else {
            done.push(variant);
            continue;
        };
        let alternatives: Vec<&str> = variant[open + 1..close].split(',').collect();
        if done.len() + variants.len() + alternatives.len() > MAX_VARIANTS {
            return None;
        }
        for alternative in alternatives {
            variants.push(format!(
                "{}{alternative}{}",
                &variant[..open],
                &variant[close + 1..]
            ));
        }
    }
    Some(done)
}

/// Whether the shell glob `pattern` (`*`, `?`, `[…]`, `[!…]`) matches
/// `name`, any letter case; as in a shell, a leading `.` must be written.
/// Linear in the pattern's length (a table over the short name).
pub(super) fn glob_matches(pattern: &str, name: &str) -> bool {
    enum Token {
        Star,
        Any,
        Set {
            negated: bool,
            ranges: Vec<(char, char)>,
        },
        Char(char),
    }
    let pattern: Vec<char> = pattern.to_ascii_lowercase().chars().collect();
    let name: Vec<char> = name.to_ascii_lowercase().chars().collect();
    if name.first() == Some(&'.') && pattern.first() != Some(&'.') {
        return false;
    }
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < pattern.len() {
        match pattern[index] {
            '*' => {
                if !matches!(tokens.last(), Some(Token::Star)) {
                    tokens.push(Token::Star);
                }
                index += 1;
            }
            '?' => {
                tokens.push(Token::Any);
                index += 1;
            }
            // `[[:alpha:]]`-style classes: any character (matching more is
            // safe here).
            '[' if pattern[index + 1..].starts_with(&['[', ':'])
                || pattern[index + 1..].starts_with(&['!', '[', ':'])
                || pattern[index + 1..].starts_with(&['^', '[', ':']) =>
            {
                let rest: String = pattern[index..].iter().collect();
                match rest.find(":]]") {
                    Some(end) => {
                        tokens.push(Token::Any);
                        index += rest[..end + 3].chars().count();
                    }
                    None => {
                        tokens.push(Token::Char('['));
                        index += 1;
                    }
                }
            }
            '[' => match pattern[index + 1..]
                .iter()
                .skip(1)
                .position(|c| *c == ']')
                .map(|close| index + 2 + close)
            {
                Some(close) => {
                    let mut set = &pattern[index + 1..close];
                    let negated = matches!(set.first(), Some('!' | '^'));
                    if negated {
                        set = &set[1..];
                    }
                    let mut ranges = Vec::new();
                    let mut at = 0;
                    while at < set.len() {
                        if at + 2 < set.len() && set[at + 1] == '-' {
                            ranges.push((set[at], set[at + 2]));
                            at += 3;
                        } else {
                            ranges.push((set[at], set[at]));
                            at += 1;
                        }
                    }
                    tokens.push(Token::Set { negated, ranges });
                    index = close + 1;
                }
                None => {
                    tokens.push(Token::Char('['));
                    index += 1;
                }
            },
            c => {
                tokens.push(Token::Char(c));
                index += 1;
            }
        }
    }
    // `row[j]`: the tokens so far match the first `j` characters of `name`.
    let mut row = vec![false; name.len() + 1];
    row[0] = true;
    for token in &tokens {
        let mut next = vec![false; name.len() + 1];
        for j in 0..=name.len() {
            next[j] = match token {
                Token::Star => row[j] || (j > 0 && next[j - 1]),
                _ if j == 0 => false,
                Token::Any => row[j - 1],
                Token::Set { negated, ranges } => {
                    row[j - 1]
                        && (ranges
                            .iter()
                            .any(|(low, high)| (*low..=*high).contains(&name[j - 1]))
                            != *negated)
                }
                Token::Char(c) => row[j - 1] && name[j - 1] == *c,
            };
        }
        row = next;
    }
    row[name.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn issue_623_globs_braces_and_shell_syntax_name_agent_settings() {
        for target in [
            ".agents",
            "sub/.agents",
            ".AGENTS",
            ".agen*",
            ".a?ents",
            ".[a]gents",
            ".[!x]odex",
            ".[a-c]odex",
            "..\\.agents",
            "../.agent{s,}",
            ".{agents,x}",
            "../${N}.agent{s,}",
            "${N}.age''nts",
            "${N}.agent\\s",
            ".(agents)",
            ".agent(s|)",
            ".[[:alpha:]]gents",
        ] {
            assert!(settings_name_in(target).is_some(), "{target}");
        }
        for target in [".agents-old", "agents", "*", ".a", ".[!a]gents", "{a,b}"] {
            assert_eq!(settings_name_in(target), None, "{target}");
        }
        for text in [
            "ln -s \"$(realpath .agents)\" ag",
            "ln -s \"${X:-.agents}\" ag",
            "ln -s $.agents ag",
            "A=.agents; ln -s \"$A\" ag",
            "echo $(ln -s .agents ag)",
        ] {
            assert_eq!(settings_name_in(text).as_deref(), Some(".agents"), "{text}");
        }
        assert_eq!(settings_name_in("ln -s src/notes.py n"), None);
        assert_eq!(settings_name_in("ln -s .agents-old x"), None);
    }

    #[test]
    fn issue_623_nested_braces_and_sequences_are_named_whole() {
        assert!(settings_name_in("ln -s .agent{{s,},x} ag").is_some());
        assert!(settings_name_in("ln -s .agent{s..s} ag").is_some());
        assert_eq!(settings_name_in("ln -s notes{1,2} ag"), None);
    }

    #[test]
    fn issue_623_bounds_hold() {
        // A huge range is two characters, not a million.
        assert!(glob_matches(".[#-\u{10FFFF}]gents", ".agents"));
        let ranges = format!(".{}", "[#-\u{10FFFF}]".repeat(1000));
        assert!(!glob_matches(&ranges, ".agents"));
        // Too many alternatives name the word whole (refused).
        let commas = format!(".agent{{{}}}", ",".repeat(25_000));
        assert_eq!(settings_name_in(&commas), Some(commas.clone()));
    }

    #[test]
    fn issue_623_glob_matching_is_linear() {
        let stars = format!(".{}", "*".repeat(100_000));
        let started = std::time::Instant::now();
        assert!(glob_matches(&stars, ".agents"));
        let many = format!(".{}x", "*a".repeat(50_000));
        assert!(!glob_matches(&many, ".agents"));
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }
}
