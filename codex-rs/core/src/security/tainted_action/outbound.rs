//! PF-23-S01: commands that move value, or send local content (a file or the
//! output of another command) to another machine. Data written literally in
//! the command is visible in it and is not counted; neither is a request to
//! this machine (`localhost`, `127.0.0.1`, `::1`).

use super::ProtectedActionKind;
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
const WGET_UPLOAD: &[&str] = &["--post-file", "--body-file"];
const HTTPIE: &[&str] = &["http", "https", "xh", "xhs"];
const RAW_SOCKETS: &[&str] = &["nc", "ncat", "netcat", "socat", "telnet"];
const COPY_TO_HOST: &[&str] = &["scp", "rsync"];
const MAIL: &[&str] = &["mail", "mailx", "sendmail", "mutt", "msmtp"];

/// The protected kind of one simple command's words. Command names and
/// subcommands match in any case, options as written. `stdin_fed`: its input
/// comes from a pipe or a redirected file.
pub(super) fn classify(words: &[String], stdin_fed: bool) -> Option<ProtectedActionKind> {
    let lower: Vec<String> = words.iter().map(|word| word.to_lowercase()).collect();
    let mut found = None;
    for (index, word) in lower.iter().enumerate() {
        let name = basename(word);
        let args = &words[index + 1..];
        let lower_args = &lower[index + 1..];
        let transfer = VALUE_TRANSFER.iter().any(|(command, verbs)| {
            name == *command && lower_args.iter().any(|arg| verbs.contains(&arg.as_str()))
        });
        if transfer {
            return Some(ProtectedActionKind::ValueTransfer);
        }
        if sends_local_content(name, args, lower_args, stdin_fed) {
            found = Some(ProtectedActionKind::Disclosure);
        }
    }
    found
}

fn sends_local_content(
    name: &str,
    args: &[String],
    lower_args: &[String],
    stdin_fed: bool,
) -> bool {
    let uploads = match name {
        "curl" => curl_uploads(args),
        "wget" => args.iter().any(|arg| {
            WGET_UPLOAD
                .iter()
                .any(|flag| option_value(arg, flag).is_some())
        }),
        _ if HTTPIE.contains(&name) => {
            stdin_fed
                || args
                    .iter()
                    .any(|arg| !arg.starts_with('-') && arg.contains('@'))
        }
        _ if RAW_SOCKETS.contains(&name) => {
            stdin_fed || (name == "socat" && lower_args.iter().any(|arg| arg.contains("file:")))
        }
        _ if COPY_TO_HOST.contains(&name) => return copies_to_a_host(args),
        "gh" => return lower_args.windows(2).any(|pair| pair == ["gist", "create"]),
        _ if MAIL.contains(&name) => return true,
        _ => false,
    };
    uploads && !only_this_machine(args)
}

/// `-d @file`, `-d@file`, `--data=@file`, `-F f=@file`, `-F f=<file`, `-T file`.
fn curl_uploads(args: &[String]) -> bool {
    let mut previous: Option<&str> = None;
    for arg in args {
        if let Some(flag) = previous.take() {
            if CURL_UPLOAD.contains(&flag) || names_local_content(arg) {
                return true;
            }
            continue;
        }
        // A flag taking the next word, alone or last in a group (`-sd`).
        let flag = CURL_DATA.iter().chain(CURL_UPLOAD).find(|flag| {
            arg == *flag
                || (flag.len() == 2
                    && !arg.starts_with("--")
                    && arg.len() > 2
                    && arg.starts_with('-')
                    && arg[1..].chars().all(|ch| ch.is_ascii_alphabetic())
                    && arg.ends_with(&flag[1..]))
        });
        if let Some(flag) = flag {
            previous = Some(flag);
            continue;
        }
        for flag in CURL_DATA.iter().chain(CURL_UPLOAD) {
            if let Some(value) = option_value(arg, flag)
                && (CURL_UPLOAD.contains(flag) || names_local_content(value))
            {
                return true;
            }
        }
    }
    false
}

/// A curl data value that reads a file or stdin rather than carrying text.
fn names_local_content(value: &str) -> bool {
    value.starts_with('@') || value.contains("=@") || value.contains("=<") || value.contains("@-")
}

/// The value of `flag` attached to `arg`: `--flag=value`, or `-fvalue` for a
/// short flag.
fn option_value<'a>(arg: &'a str, flag: &str) -> Option<&'a str> {
    if flag.starts_with("--") {
        arg.strip_prefix(flag)?.strip_prefix('=')
    } else {
        arg.strip_prefix(flag).filter(|value| !value.is_empty())
    }
}

/// `scp`/`rsync` with a remote (`host:path`) operand after a local one.
fn copies_to_a_host(args: &[String]) -> bool {
    let operands: Vec<&String> = args.iter().filter(|arg| !arg.starts_with('-')).collect();
    let remote = |arg: &str| {
        arg.split_once(':').is_some_and(|(host, _)| {
            !host.is_empty() && !host.contains('/') && !host.contains('=') && !is_loopback(host)
        }) && !arg.contains("://")
            || arg.starts_with("rsync://") && !only_this_machine(&[arg.to_string()])
    };
    operands
        .iter()
        .position(|arg| !remote(arg))
        .is_some_and(|first_local| operands[first_local + 1..].iter().any(|arg| remote(arg)))
}

/// Every destination named in `args` is this machine (and at least one is named).
fn only_this_machine(args: &[String]) -> bool {
    let mut named = false;
    for arg in args.iter().filter(|arg| !arg.starts_with('-')) {
        let rest = arg.split_once("://").map_or(arg.as_str(), |(_, rest)| rest);
        let host = host_of(rest);
        if arg.contains("://") {
            if !is_loopback(host) {
                return false;
            }
            named = true;
        } else if is_loopback(host) {
            named = true;
        }
    }
    named
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
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "::1"
        || host == "0.0.0.0"
        || host.strip_prefix("127.").is_some_and(|rest| {
            rest.split('.').count() == 3
                && rest
                    .split('.')
                    .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
        })
}
