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
                (Family::Deno, false, word) if !word.contains(['.', '/']) => {
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
            Opt::FileAttached(file) => {
                return Some(Invocation {
                    code: Code::File(file),
                    preloads,
                });
            }
            Opt::Module => {
                return Some(Invocation {
                    code: next.map_or(Code::Stdin, Code::Module),
                    preloads,
                });
            }
            Opt::ModuleAttached(module) => {
                return Some(Invocation {
                    code: Code::Module(module),
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
    /// Tcl, expect, swift, R: `-c`/`-e` code, `-f` file, else a file or stdin.
    Generic,
}

enum Opt<'a> {
    Inline,
    InlineAttached(&'a str),
    File,
    FileAttached(&'a str),
    Module,
    ModuleAttached(&'a str),
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
        // `python3.12`, `perl5.34`, `node18`: the version does not matter.
        let base = name.trim_end_matches(|ch: char| ch.is_ascii_digit() || ch == '.');
        Some(match base {
            "python" | "pypy" => Self::Python,
            "node" | "nodejs" | "ts-node" => Self::Node,
            "deno" => Self::Deno,
            "bun" => Self::Bun,
            "tsx" => Self::Tsx,
            "perl" | "ruby" => Self::PerlRuby,
            "php" => Self::Php,
            "lua" | "luajit" => Self::Lua,
            "osascript" => Self::Osascript,
            "pwsh" | "powershell" => Self::Pwsh,
            "tclsh" | "wish" | "expect" | "swift" | "rscript" => Self::Generic,
            _ => return None,
        })
    }

    /// How this family reads one option word: long options by name, short
    /// groups one letter at a time, with attached values (`-c$X`, `-Ilib`).
    fn option<'a>(self, arg: &'a str) -> Opt<'a> {
        if self == Self::Pwsh {
            return match arg.to_lowercase().as_str() {
                "-c" | "-command" => Opt::Inline,
                "-f" | "-file" => Opt::File,
                _ => Opt::Flag,
            };
        }
        let scripted = matches!(self, Self::Node | Self::Deno | Self::Bun | Self::Tsx);
        if let Some(long) = arg.strip_prefix("--") {
            let (key, attached) = match long.split_once('=') {
                Some((key, value)) => (key, Some(value)),
                None => (long, None),
            };
            return match key {
                "command" if self == Self::Shell => Opt::Inline,
                "rcfile" | "init-file" if self == Self::Shell => {
                    attached.map_or(Opt::Value, |_| Opt::Flag)
                }
                "check-hash-based-pycs" if self == Self::Python => {
                    attached.map_or(Opt::Value, |_| Opt::Flag)
                }
                "eval" | "print" if scripted => attached.map_or(Opt::Inline, Opt::InlineAttached),
                "require" | "import" | "loader" | "experimental-loader" | "preload" if scripted => {
                    attached.map_or(Opt::Preload, Opt::PreloadAttached)
                }
                "env-file" | "inspect-port" | "title" | "cwd" if scripted => {
                    attached.map_or(Opt::Value, |_| Opt::Flag)
                }
                _ => Opt::Flag,
            };
        }
        let group = arg.trim_start_matches(['-', '+']);
        for (at, letter) in group.char_indices() {
            let rest = &group[at + letter.len_utf8()..];
            let attached = (!rest.is_empty()).then_some(rest);
            let inline = attached.map_or(Opt::Inline, Opt::InlineAttached);
            let value = attached.map_or(Opt::Value, |_| Opt::Flag);
            let preload = attached.map_or(Opt::Preload, Opt::PreloadAttached);
            let file = attached.map_or(Opt::File, Opt::FileAttached);
            return match (self, letter) {
                (Self::Shell, 'c') => Opt::Inline,
                (Self::Shell, 's') => Opt::Stdin,
                (Self::Shell, 'o' | 'O') => value,
                (Self::Python, 'c') => inline,
                (Self::Python, 'm') => attached.map_or(Opt::Module, Opt::ModuleAttached),
                (Self::Python, 'W' | 'X') => value,
                (_, 'e' | 'p') if scripted => inline,
                (_, 'r') if scripted => preload,
                (Self::PerlRuby, 'e' | 'E') => inline,
                (Self::PerlRuby, 'r') => preload,
                (Self::PerlRuby, 'I') => value,
                // Letters whose value can only be attached.
                (Self::PerlRuby, 'M' | 'm' | 'x' | 'C' | 'l' | 'i' | 'F' | '0' | 'd' | 'D') => {
                    Opt::Flag
                }
                (Self::Php, 'r') => inline,
                (Self::Php, 'f') => file,
                (Self::Php, 'd' | 'c' | 'z') => value,
                (Self::Lua, 'e') => inline,
                (Self::Lua, 'l') => value,
                (Self::Osascript, 'e') => inline,
                (Self::Osascript, 'l' | 's') => value,
                (Self::Generic, 'c' | 'e') => inline,
                (Self::Generic, 'f') => file,
                _ => continue,
            };
        }
        Opt::Flag
    }
}
