//! PF-23-S01: commands that move value, or send local content (a file, the
//! output of another command, a value the classifier cannot see) to another
//! machine. Data written literally in the command is visible in it and is not
//! counted; neither is a request to this machine (`localhost`, `127.0.0.1`,
//! `::1`) when every destination is named and none can be redirected.

use super::ProtectedActionKind;
use super::shell::UNSEEN_OUTPUT;
use super::shell::basename;

/// Commands that transfer value, with the words that make them do it.
const VALUE_TRANSFER: &[(&str, &[&str])] = &[
    (
        "solana",
        &[
            "transfer",
            "pay",
            "withdraw-stake",
            "withdraw-from-vote-account",
            "withdraw-from-nonce-account",
        ],
    ),
    (
        "spl-token",
        &["transfer", "unwrap", "withdraw-withheld-tokens"],
    ),
    ("cast", &["send", "publish"]),
    (
        "bitcoin-cli",
        &[
            "send",
            "sendall",
            "sendmany",
            "sendtoaddress",
            "sendrawtransaction",
        ],
    ),
    ("electrum", &["payto", "paytomany", "broadcast"]),
    (
        "sui",
        &["transfer", "transfer-sui", "pay", "pay-sui", "pay-all-sui"],
    ),
    ("aptos", &["transfer"]),
    ("near", &["send", "send-near", "transfer"]),
];
/// curl options whose value is the request body or an upload.
const CURL_DATA: &[&str] = &[
    "-d",
    "--data",
    "--data-ascii",
    "--data-binary",
    "--data-urlencode",
    "--json",
    "-F",
    "--form",
];
const CURL_UPLOAD: &[&str] = &["-T", "--upload-file"];
/// Other curl options that take a value (so it is not a destination).
const CURL_VALUE: &[&str] = &[
    "--variable",
    "-o",
    "--output",
    "-H",
    "--header",
    "-X",
    "--request",
    "-u",
    "--user",
    "-A",
    "--user-agent",
    "-e",
    "--referer",
    "-b",
    "--cookie",
    "-c",
    "--cookie-jar",
    "-m",
    "--max-time",
    "--connect-timeout",
    "-w",
    "--write-out",
    "-E",
    "--cert",
    "--key",
    "--cacert",
    "-r",
    "--range",
    "--retry",
    "-K",
    "--config",
];
/// curl options whose value stays on this machine (output, TLS files, limits).
const CURL_LOCAL_VALUE: &[&str] = &[
    "-o",
    "--output",
    "-c",
    "--cookie-jar",
    "-w",
    "--write-out",
    "-E",
    "--cert",
    "--key",
    "--cacert",
    "-r",
    "--range",
    "--retry",
    "-m",
    "--max-time",
    "--connect-timeout",
    "-X",
    "--request",
];
/// curl options that can send a named local destination somewhere else.
const CURL_REDIRECT: &[&str] = &[
    "-x",
    "--proxy",
    "--preproxy",
    "--resolve",
    "--connect-to",
    "--socks4",
    "--socks4a",
    "--socks5",
    "--socks5-hostname",
    "-K",
    "--config",
];
const WGET_UPLOAD: &[&str] = &["--post-file", "--body-file"];
const HTTPIE: &[&str] = &["http", "https", "xh", "xhs"];
const RAW_SOCKETS: &[&str] = &["nc", "ncat", "netcat", "socat", "telnet"];
const COPY_TO_HOST: &[&str] = &["scp", "rsync"];
const MAIL: &[&str] = &["mail", "mailx", "sendmail", "mutt", "msmtp"];

/// Wrappers that run a command given later in their words, with or without
/// options of their own (`proxychains -q curl`, `uv run solana`, `npx x`).
const EXEC_WRAPPERS: &[&str] = &[
    "proxychains",
    "proxychains4",
    "torsocks",
    "torify",
    "strace",
    "ltrace",
    "npx",
    "bunx",
    "pnpx",
    "uvx",
    "uv",
    "poetry",
    "pipenv",
    "pdm",
    "hatch",
    "op",
    "parallel",
    "dotenv",
    "doppler",
    "infisical",
    "aws-vault",
    "direnv",
    "chpst",
    "with-contenv",
    "bundle",
    "npm",
    "pnpm",
    "yarn",
];
/// Words after which the next word is run as a command (`find -exec curl`).
const EXEC_MARKERS: &[&str] = &["-exec", "-execdir", "-ok", "-okdir", "--"];

/// The protected kind of one simple command. `command` is the index of its
/// command word (after assignments and plain wrappers); the command itself,
/// anything an exec-style wrapper runs, and any word after an exec marker
/// are judged. Names and subcommands match in any case, options as written.
/// `stdin_fed`: its input comes from a pipe or a redirected file.
pub(super) fn classify(
    words: &[String],
    command: usize,
    stdin_fed: bool,
) -> Option<ProtectedActionKind> {
    // `ALL_PROXY=... curl http://localhost` leaves the machine.
    let proxied = words[..command].iter().any(|word| {
        word.split_once('=')
            .is_some_and(|(name, _)| name.to_lowercase().ends_with("_proxy"))
    });
    let wrapped = words
        .get(command)
        .is_some_and(|word| EXEC_WRAPPERS.contains(&basename(&word.to_lowercase())));
    let mut found = None;
    for index in command..words.len() {
        let judged =
            index == command || wrapped || EXEC_MARKERS.contains(&words[index - 1].as_str());
        if !judged {
            continue;
        }
        let kind = classify_at(&words[index..], stdin_fed, proxied);
        if kind == Some(ProtectedActionKind::ValueTransfer) {
            return kind;
        }
        found = found.or(kind);
    }
    found
}

fn classify_at(words: &[String], stdin_fed: bool, proxied: bool) -> Option<ProtectedActionKind> {
    let (name, args) = words.split_first()?;
    let name = basename(name).to_lowercase();
    let lower: Vec<String> = args.iter().map(|word| word.to_lowercase()).collect();
    let transfer = VALUE_TRANSFER.iter().any(|(command, verbs)| {
        name == *command && lower.iter().any(|arg| verbs.contains(&arg.as_str()))
    });
    if transfer {
        return Some(ProtectedActionKind::ValueTransfer);
    }
    sends_local_content(&name, args, &lower, stdin_fed, proxied)
        .then_some(ProtectedActionKind::Disclosure)
}

fn sends_local_content(
    name: &str,
    args: &[String],
    lower: &[String],
    stdin_fed: bool,
    proxied: bool,
) -> bool {
    let leaves = |args: &[String]| {
        proxied || lower.iter().any(|arg| arg.starts_with("--proxy")) || !only_this_machine(args)
    };
    match name {
        "curl" => curl_sends_local_content(args, proxied),
        "wget" => {
            let uploads = args.iter().enumerate().any(|(index, arg)| {
                WGET_UPLOAD.iter().any(|flag| {
                    option_value(arg, flag).is_some() || (arg == flag && index + 1 < args.len())
                })
            }) || args.iter().any(|arg| arg.contains(UNSEEN_OUTPUT));
            uploads && leaves(args)
        }
        _ if HTTPIE.contains(&name) => {
            (stdin_fed
                || args
                    .iter()
                    .any(|arg| !arg.starts_with('-') && (arg.contains('@') || unseen(arg))))
                && leaves(args)
        }
        _ if RAW_SOCKETS.contains(&name) => {
            (stdin_fed || (name == "socat" && lower.iter().any(|arg| arg.contains("file:"))))
                && leaves(args)
        }
        _ if COPY_TO_HOST.contains(&name) => copies_to_a_host(args),
        "gh" => lower.windows(2).any(|pair| pair == ["gist", "create"]),
        _ if MAIL.contains(&name) => true,
        _ => false,
    }
}

/// Text the classifier cannot see: a substitution's output or a variable.
fn unseen(value: &str) -> bool {
    value.contains(UNSEEN_OUTPUT) || value.contains('$')
}

/// A curl data value that reads a file, stdin or unseen text.
fn names_local_content(value: &str) -> bool {
    value.starts_with('@')
        || value.contains("=@")
        || value.contains("=<")
        || value.contains("@-")
        || unseen(value)
}

/// Whether curl uploads local content to a destination other than this
/// machine. Destinations are its operands (and `--url` values).
fn curl_sends_local_content(args: &[String], proxied: bool) -> bool {
    // A substitution or variable in what is sent (URL, header, body, user)
    // carries text the classifier cannot see out of the machine.
    let mut uploads = false;
    let mut redirects = proxied;
    let mut destinations: Vec<&str> = Vec::new();
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        index += 1;
        let (flag, attached) = split_option(arg);
        let Some(flag) = flag else {
            uploads |= unseen(arg);
            destinations.push(arg);
            continue;
        };
        redirects |= CURL_REDIRECT.contains(&flag);
        let takes_value = CURL_DATA.contains(&flag)
            || CURL_UPLOAD.contains(&flag)
            || CURL_VALUE.contains(&flag)
            || CURL_REDIRECT.contains(&flag)
            || flag == "--url";
        if !takes_value {
            continue;
        }
        let value = match attached {
            Some(value) => value,
            None => {
                index += 1;
                args.get(index - 1).map_or("", String::as_str)
            }
        };
        let reads_file = match flag {
            "-T" | "--upload-file" | "-K" | "--config" => true,
            "--data-urlencode" | "--variable" => value.contains('@'),
            "-H" | "--header" => value.starts_with('@'),
            _ => CURL_DATA.contains(&flag) && names_local_content(value),
        };
        uploads |= reads_file || (unseen(value) && !CURL_LOCAL_VALUE.contains(&flag));
        if flag == "--url" {
            destinations.push(value);
        }
    }
    uploads && (redirects || !all_loopback(&destinations))
}

/// The option an argument sets and its attached value: `--data=v`, `-dv`,
/// or the last letter of a short group (`-sd`, `-sd@f`). `None` for operands.
fn split_option(arg: &str) -> (Option<&str>, Option<&str>) {
    if let Some(long) = arg.strip_prefix("--") {
        if long.is_empty() {
            return (None, None);
        }
        return match arg.split_once('=') {
            Some((flag, value)) => (Some(flag), Some(value)),
            None => (Some(arg), None),
        };
    }
    let Some(group) = arg.strip_prefix('-').filter(|group| !group.is_empty()) else {
        return (None, None);
    };
    // The first letter that takes a value ends the group; the rest is its value.
    for (at, letter) in group.char_indices() {
        let candidate = format!("-{letter}");
        let known = CURL_DATA
            .iter()
            .chain(CURL_UPLOAD)
            .chain(CURL_VALUE)
            .chain(CURL_REDIRECT)
            .find(|known| **known == candidate);
        if let Some(known) = known {
            let rest = &group[at + letter.len_utf8()..];
            return (Some(known), (!rest.is_empty()).then_some(rest));
        }
    }
    (Some(arg), None)
}

/// The value of `flag` attached to `arg`: `--flag=value`.
fn option_value<'a>(arg: &'a str, flag: &str) -> Option<&'a str> {
    arg.strip_prefix(flag)?.strip_prefix('=')
}

/// `scp`/`rsync` with a remote (`host:path`) operand after a local one.
fn copies_to_a_host(args: &[String]) -> bool {
    let operands: Vec<&String> = args.iter().filter(|arg| !arg.starts_with('-')).collect();
    let remote = |arg: &str| {
        if arg.contains("://") {
            return !all_loopback(&[arg]);
        }
        arg.split_once(':').is_some_and(|(host, _)| {
            !host.is_empty()
                && !host.contains('/')
                && !host.contains('=')
                && !is_loopback(host_of(host))
        })
    };
    operands
        .iter()
        .position(|arg| !remote(arg))
        .is_some_and(|first_local| operands[first_local + 1..].iter().any(|arg| remote(arg)))
}

/// Operand words of a generic network command: each one that names a host
/// (a URL, `localhost`, or anything with a dot or colon) is this machine.
fn only_this_machine(args: &[String]) -> bool {
    let destinations: Vec<&str> = args
        .iter()
        .filter(|arg| !arg.starts_with('-') && !arg.starts_with('@'))
        .map(String::as_str)
        .filter(|arg| {
            arg.contains("://")
                || arg.contains('.')
                || arg.contains(':')
                || is_loopback(host_of(arg))
        })
        .collect();
    all_loopback(&destinations)
}

/// At least one destination, and every one is this machine.
fn all_loopback(destinations: &[&str]) -> bool {
    !destinations.is_empty()
        && destinations.iter().all(|destination| {
            let rest = destination
                .split_once("://")
                .map_or(*destination, |(_, rest)| rest);
            !unseen(destination) && is_loopback(host_of(rest))
        })
}

fn host_of(rest: &str) -> &str {
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    if let Some(bracketed) = authority.strip_prefix('[') {
        return bracketed.split(']').next().unwrap_or(bracketed);
    }
    authority.split(':').next().unwrap_or(authority)
}

fn is_loopback(host: &str) -> bool {
    let host = host.to_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "::1"
        || host.strip_prefix("127.").is_some_and(|rest| {
            rest.split('.').count() == 3
                && rest
                    .split('.')
                    .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
        })
}
