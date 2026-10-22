//! #1261: the files the first judgment opens on Windows (script and sed
//! program files). A lookup never reaches a network share or a mapped drive.
//! A file is opened only through a plain path on a drive proven to be a
//! local fixed disk, after every folder on the way and the file itself were
//! checked not to be a link, junction, mount point or remote (cloud or
//! offline) file; the file is then opened without following a link. Anything
//! else fails closed, and the classifier asks.
//!
//! A plain path is a drive-letter path with ordinary names only: no network
//! share or device path (`\\server\share`, `\\?\`, `\\.\`, anything starting
//! with two separators), no `..`, stream (`:`), wildcard or other reserved
//! character, no name ending in a dot or space (Win32 drops them, a `\\?\`
//! path keeps them: two spellings of different files) and no device name
//! (`CON`, `NUL`, `COM1`, ...).

use super::work;
#[cfg(test)]
use std::cell::RefCell;
use std::path::PathBuf;

#[cfg(test)]
thread_local! {
    /// Every filesystem call made here, in order, for the tests.
    static TOUCHED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// The filesystem calls made on this thread since the last call.
#[cfg(test)]
pub(super) fn take_touched() -> Vec<String> {
    TOUCHED.with(|touched| std::mem::take(&mut *touched.borrow_mut()))
}

/// What a lookup of a plain path found.
#[derive(Debug)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(super) enum Located {
    /// A regular local file (not opened yet).
    File(PathBuf),
    /// The file, or a folder on the way, is not there.
    Missing,
    /// Not looked up, or something on the way is not plainly local.
    Refused,
}

fn separator(ch: char) -> bool {
    ch == '/' || ch == '\\'
}

/// Whether `text` starts with two separators: a network share (`\\h\s`,
/// `//h/s`) or a device or verbatim path (`\\.\`, `\\?\`).
pub(super) fn starts_at_a_share(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(separator) && chars.next().is_some_and(separator)
}

/// The upper-case drive letter of `text` starting `X:`, and the rest.
fn drive(text: &str) -> Option<(char, &str)> {
    let mut chars = text.chars();
    let letter = chars.next().filter(char::is_ascii_alphabetic)?;
    (chars.next() == Some(':')).then(|| (letter.to_ascii_uppercase(), &text[2..]))
}

/// A working folder's drive and the rest: `C:\work` as the action gives it,
/// or `/C:/work` as the classifier's Windows path rules spell a folder it
/// followed (`cd`). Nothing else (a share, a drive-relative `C:`) is plain.
fn folder(cwd: &str) -> Option<(char, &str)> {
    if let Some((letter, rest)) = drive(cwd) {
        return rest.starts_with(separator).then_some((letter, rest));
    }
    let (letter, rest) = drive(cwd.strip_prefix('/')?)?;
    (rest.is_empty() || rest.starts_with('/')).then_some((letter, rest))
}

/// Whether Windows could open `name` as an ordinary file or folder name.
fn plain_name(name: &str) -> bool {
    name != ".."
        && !name.ends_with(['.', ' '])
        && !name
            .chars()
            .any(|ch| ch < ' ' || matches!(ch, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        && !device_name(name)
}

/// A DOS device name, with any extension (`nul.txt` is `NUL`).
fn device_name(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" | "CLOCK$"
    ) {
        return true;
    }
    let mut chars = stem.chars();
    let prefix: String = chars.by_ref().take(3).collect();
    let number: Vec<char> = chars.collect();
    matches!(prefix.as_str(), "COM" | "LPT")
        && matches!(number.as_slice(), [digit] if digit.is_ascii_digit() || matches!(digit, '¹' | '²' | '³'))
}

/// `word` (home spellings already expanded) made absolute against `cwd` as
/// Windows does, as its drive letter and the names below the drive's root,
/// when both are plain paths (see the module comment). A drive-relative
/// word (`C:x`) is plain only on the working folder's drive.
pub(super) fn plain_path(word: &str, cwd: &str) -> Option<(char, Vec<String>)> {
    if starts_at_a_share(word) || starts_at_a_share(cwd) {
        return None;
    }
    let (cwd_drive, cwd_rest) = folder(cwd)?;
    let (letter, path) = match drive(word) {
        Some((letter, rest)) if rest.starts_with(separator) => (letter, rest.to_string()),
        Some((letter, rest)) if letter == cwd_drive => (letter, format!("{cwd_rest}\\{rest}")),
        Some(_) => return None,
        None if word.starts_with(separator) => (cwd_drive, word.to_string()),
        None => (cwd_drive, format!("{cwd_rest}\\{word}")),
    };
    let mut names = Vec::new();
    for name in path.split(separator) {
        work::step();
        match name {
            "" | "." => {}
            name if plain_name(name) => names.push(name.to_string()),
            _ => return None,
        }
    }
    Some((letter, names))
}

/// Record a filesystem call (tests only).
#[cfg(windows)]
fn touch(path: &str) {
    #[cfg(test)]
    TOUCHED.with(|touched| touched.borrow_mut().push(path.to_string()));
    #[cfg(not(test))]
    let _ = path;
}

/// Look up `drive:\names...` without following a link: the drive must be a
/// local fixed disk and every folder on the way, and the file, must be no
/// link, junction, mount point or remote file. `lookup` is asked before
/// each filesystem call and is false once the lookup budget is spent.
#[cfg(windows)]
pub(super) fn locate(drive: char, names: &[String], lookup: &dyn Fn() -> bool) -> Located {
    // PROBE (#1261 fail-before): a naive lookup, no guard.
    if names.is_empty() || !lookup() {
        return Located::Refused;
    }
    touch(&format!("{drive}:"));
    let mut path = PathBuf::from(format!("{drive}:\\"));
    for name in names {
        work::step();
        path.push(name);
        touch(&path.to_string_lossy());
    }
    match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Located::File(path),
        Ok(_) => Located::Refused,
        Err(_) => Located::Missing,
    }
}

/// Elsewhere no drive is proven local: nothing is looked up.
#[cfg(not(windows))]
pub(super) fn locate(_drive: char, _names: &[String], _lookup: &dyn Fn() -> bool) -> Located {
    Located::Refused
}

/// Open a file [`locate`] found, without following a link, if it is still a
/// regular local file.
#[cfg(windows)]
pub(super) fn open(path: &std::path::Path) -> Option<std::fs::File> {
    touch(&path.to_string_lossy());
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .ok()?;
    let metadata = handle.metadata().ok()?;
    metadata.is_file().then_some(handle)
}

#[cfg(not(windows))]
pub(super) fn open(_path: &std::path::Path) -> Option<std::fs::File> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn plain(word: &str, cwd: &str) -> Option<String> {
        plain_path(word, cwd).map(|(drive, names)| format!("{drive}:\\{}", names.join("\\")))
    }

    /// #1261: plain paths are made absolute as Windows does.
    #[test]
    fn issue_1261_plain_paths() {
        for (word, cwd, expected) in [
            ("run.sh", r"C:\work", r"C:\work\run.sh"),
            (r".\tools\run.sh", r"c:\work", r"C:\work\tools\run.sh"),
            ("./run.sh", "/C:/work/sub", r"C:\work\sub\run.sh"),
            (r"D:\x\run.sh", r"C:\work", r"D:\x\run.sh"),
            ("d:/x/run.sh", r"C:\work", r"D:\x\run.sh"),
            (r"\x\run.sh", r"C:\work", r"C:\x\run.sh"),
            (
                "/home/fixture/run.sh",
                r"C:\work",
                r"C:\home\fixture\run.sh",
            ),
            ("C:run.sh", r"C:\work", r"C:\work\run.sh"),
            ("run.sh", r"C:\", r"C:\run.sh"),
            ("run.sh", "/C:", r"C:\run.sh"),
            (r"PROGRA~1\run.sh", r"C:\", r"C:\PROGRA~1\run.sh"),
            ("comx/run.sh", r"C:\work", r"C:\work\comx\run.sh"),
        ] {
            assert_eq!(
                plain(word, cwd).as_deref(),
                Some(expected),
                "{word} in {cwd}"
            );
        }
    }

    /// #1261: a share, device or verbatim path, a doubtful drive and any
    /// name Windows reads differently are never looked up.
    #[test]
    fn issue_1261_network_and_doubtful_paths_are_not_plain() {
        for (word, cwd) in [
            // Network shares, administrative shares and device paths.
            (r"\\server\share\run.sh", r"C:\work"),
            ("//server/share/run.sh", r"C:\work"),
            (r"\/server\share\run.sh", r"C:\work"),
            (r"\\server\c$\run.sh", r"C:\work"),
            (r"\\localhost\c$\work\run.sh", r"C:\work"),
            (r"\\?\UNC\server\share\run.sh", r"C:\work"),
            (r"\\?\C:\work\run.sh", r"C:\work"),
            (r"\\.\C:\work\run.sh", r"C:\work"),
            (r"\\.\pipe\run", r"C:\work"),
            (r"\\?\GLOBALROOT\Device\Mup\server\share\run.sh", r"C:\work"),
            // A working folder on a share.
            ("run.sh", r"\\server\share\work"),
            ("run.sh", r"\\?\UNC\server\share\work"),
            ("run.sh", "/unc:/server/share/work"),
            ("run.sh", "//server/share"),
            // Working folders that are not a drive's.
            ("run.sh", "C:"),
            ("run.sh", "work"),
            ("run.sh", ""),
            ("run.sh", "/work"),
            // Another drive's working folder is not known.
            ("D:run.sh", r"C:\work"),
            // Names Windows reads differently.
            (r"..\run.sh", r"C:\work"),
            ("run.sh:hidden", r"C:\work"),
            ("run.sh::$DATA", r"C:\work"),
            ("run.sh.", r"C:\work"),
            ("run.sh ", r"C:\work"),
            ("dir./run.sh", r"C:\work"),
            ("r*n.sh", r"C:\work"),
            ("r?n.sh", r"C:\work"),
            ("a|b", r"C:\work"),
            ("/C:/work/run.sh", r"C:\work"),
            // Devices.
            ("CON", r"C:\work"),
            ("nul.sh", r"C:\work"),
            ("Aux .txt", r"C:\work"),
            ("com1", r"C:\work"),
            ("LPT9.sh", r"C:\work"),
            ("COM¹", r"C:\work"),
            ("conin$", r"C:\work"),
            ("tools/NUL/run.sh", r"C:\work"),
            ("con.sh.d/run.sh", r"C:\work"),
        ] {
            assert_eq!(plain(word, cwd), None, "{word} in {cwd}");
        }
    }

    /// #1261: where no drive is proven local (every platform but Windows)
    /// nothing is looked up.
    #[cfg(not(windows))]
    #[test]
    fn issue_1261_no_lookups_off_windows() {
        let names = vec!["work".to_string(), "run.sh".to_string()];
        assert!(matches!(locate('C', &names, &|| true), Located::Refused));
        assert_eq!(take_touched(), Vec::<String>::new());
    }
}
