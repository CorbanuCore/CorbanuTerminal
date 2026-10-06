//! Where a shell or interpreter invocation gets the code it runs. Options are
//! read per family and only up to the first operand, as the program itself
//! reads them: `bash -e p.sh` runs `p.sh`, `node -r ./p.js x.js` preloads
//! `p.js`, `perl -pe '...' f` runs inline code.

use super::shell::SHELLS;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Code<'a> {
    /// Inline code: the word after the flag. `None` when the code is not
    /// among these words (a shell's `-c` script given as its own argument is
    /// lexed as the next command; `xargs sh -c` gets it from its input).
    Inline(Option<&'a str>),
    /// A script file.
    File(&'a str),
    /// A Python module (`python -m name`): read when it is a local file.
    Module(&'a str),
    /// Code from stdin.
    Stdin,
    /// A subcommand that runs project tasks or tools (`bun install`,
    /// `deno task x`); not code the classifier can read.
    Tool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Invocation<'a> {
    pub(super) code: Code<'a>,
    /// Files loaded before the code (`node -r ./p.js`, `ruby -r ./p.rb`).
    pub(super) preloads: Vec<&'a str>,
}

/// Parse an invocation of `name` (a lowercase basename) with `args`.
/// `None` when `name` is not a shell or interpreter.
pub(super) fn parse<'a>(name: &str, args: &'a [String]) -> Option<Invocation<'a>> {
    if matches!(name, "source" | ".") {
        let code = args.first().map_or(Code::Stdin, |file| Code::File(file));
        return Some(Invocation {
            code,
            preloads: Vec::new(),
        });
    }
    let family = Family::of(name)?;
    let mut preloads = Vec::new();
    let mut index = 0;
    let mut subcommand_seen = false;
    while let Some(arg) = args.get(index) {
        let next = args.get(index + 1).map(String::as_str);
        index += 1;
        if arg == "--" {
            if let Some(file) = next {
                return Some(Invocation {
                    code: Code::File(file),
                    preloads,
                });
            }
            break;
        }
        if arg == "-" {
            return Some(Invocation {
                code: Code::Stdin,
                preloads,
            });
        }
        let is_option = arg.starts_with('-') || (family == Family::Shell && arg.starts_with('+'));
        if !is_option {
            // Subcommands that pick what runs (`deno run x.ts`, `bun run x`).
            match (family, subcommand_seen, arg.as_str()) {
                (Family::Deno, false, "run") | (Family::Tsx, false, "watch") => {
                    subcommand_seen = true;
                    continue;
                }
                (Family::Deno, false, "eval") => {
                    return Some(Invocation {
                        code: Code::Inline(next),
                        preloads,
                    });
                }
                (Family::Deno, false, _) => {
                    return Some(Invocation {
                        code: Code::Tool,
                        preloads,
                    });
                }
                (Family::Bun, false, "run") => {
                    subcommand_seen = true;
                    continue;
                }
                (Family::Bun, _, word) if !word.contains(['.', '/']) => {
                    // A package.json script or a bun tool.
                    return Some(Invocation {
                        code: Code::Tool,
                        preloads,
                    });
                }
                _ => {
                    return Some(Invocation {
                        code: Code::File(arg),
                        preloads,
                    });
                }
            }
        }
        match family.option(arg) {
            Opt::Inline => {
                return Some(Invocation {
                    code: Code::Inline(next),
                    preloads,
                });
            }
            Opt::InlineAttached(code) => {
                return Some(Invocation {
                    code: Code::Inline(Some(code)),
                    preloads,
                });
            }
            Opt::File => {
                return Some(Invocation {
                    code: next.map_or(Code::Stdin, Code::File),
                    preloads,
                });
            }
            Opt::Module => {
                return Some(Invocation {
                    code: next.map_or(Code::Stdin, Code::Module),
                    preloads,
                });
            }
            Opt::Preload => {
                preloads.extend(next);
                index += 1;
            }
            Opt::PreloadAttached(file) => preloads.push(file),
            Opt::Value => index += 1,
            Opt::Stdin => {
                return Some(Invocation {
                    code: Code::Stdin,
                    preloads,
                });
            }
            Opt::Flag => {}
        }
    }
    Some(Invocation {
        code: Code::Stdin,
        preloads,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Shell,
    Python,
    Node,
    Deno,
    Bun,
    Tsx,
    PerlRuby,
    Php,
    Lua,
    Osascript,
    Pwsh,
}

enum Opt<'a> {
    Inline,
    InlineAttached(&'a str),
    File,
    Module,
    Preload,
    PreloadAttached(&'a str),
    Value,
    Stdin,
    Flag,
}

impl Family {
    fn of(name: &str) -> Option<Self> {
        if SHELLS.contains(&name) {
            return Some(Self::Shell);
        }
        Some(match name {
            "python" | "python2" | "python3" => Self::Python,
            "node" | "nodejs" | "ts-node" => Self::Node,
            "deno" => Self::Deno,
            "bun" => Self::Bun,
            "tsx" => Self::Tsx,
            "perl" | "ruby" => Self::PerlRuby,
            "php" => Self::Php,
            "lua" => Self::Lua,
            "osascript" => Self::Osascript,
            "pwsh" => Self::Pwsh,
            _ => return None,
        })
    }

    /// How this family reads one option word.
    fn option<'a>(self, arg: &'a str) -> Opt<'a> {
        let long = arg.starts_with("--");
        let group = arg.trim_start_matches(['-', '+']);
        let (key, attached) = match arg.split_once('=') {
            Some((key, value)) => (key, Some(value)),
            None => (arg, None),
        };
        match self {
            Self::Shell => match key {
                "--command" => Opt::Inline,
                "--rcfile" | "--init-file" => Opt::Value,
                _ if long => Opt::Flag,
                _ if group.contains('c') => Opt::Inline,
                _ if group.contains('s') => Opt::Stdin,
                _ if group.ends_with(['o', 'O']) => Opt::Value,
                _ => Opt::Flag,
            },
            Self::Python => match key {
                "--check-hash-based-pycs" => Opt::Value,
                _ if long => Opt::Flag,
                _ if group.ends_with('c') => Opt::Inline,
                _ if group.ends_with('m') => Opt::Module,
                _ if group.ends_with(['W', 'X']) => Opt::Value,
                _ => Opt::Flag,
            },
            Self::Node | Self::Deno | Self::Bun | Self::Tsx => match key {
                "-e" | "--eval" | "-p" | "--print" => match attached {
                    Some(code) => Opt::InlineAttached(code),
                    None => Opt::Inline,
                },
                "-r"
                | "--require"
                | "--import"
                | "--loader"
                | "--experimental-loader"
                | "--preload" => match attached {
                    Some(file) => Opt::PreloadAttached(file),
                    None => Opt::Preload,
                },
                "--env-file" | "--inspect-port" | "--title" | "--cwd" => {
                    if attached.is_some() {
                        Opt::Flag
                    } else {
                        Opt::Value
                    }
                }
                _ => Opt::Flag,
            },
            Self::PerlRuby => match arg {
                _ if long => Opt::Flag,
                "-r" => Opt::Preload,
                "-I" | "-C" | "-x" => Opt::Value,
                // Attached values: `-Ilib`, `-MJSON`, `-rjson`.
                _ if group.starts_with(['I', 'M', 'm']) && group.len() > 1 => Opt::Flag,
                _ if group.starts_with('r') && group.len() > 1 => Opt::PreloadAttached(&group[1..]),
                // `-e`, `-pe`, `-lne`: the next word is code.
                _ if group.ends_with(['e', 'E']) => Opt::Inline,
                _ => Opt::Flag,
            },
            Self::Php => match arg {
                "-r" => Opt::Inline,
                "-f" => Opt::File,
                "-d" | "-c" | "-z" => Opt::Value,
                _ => Opt::Flag,
            },
            Self::Lua => match arg {
                "-e" => Opt::Inline,
                "-l" => Opt::Value,
                _ => Opt::Flag,
            },
            Self::Osascript => match arg {
                "-e" => Opt::Inline,
                "-l" | "-s" => Opt::Value,
                _ => Opt::Flag,
            },
            Self::Pwsh => match arg.to_lowercase().as_str() {
                "-c" | "-command" => Opt::Inline,
                "-f" | "-file" => Opt::File,
                _ => Opt::Flag,
            },
        }
    }
}
