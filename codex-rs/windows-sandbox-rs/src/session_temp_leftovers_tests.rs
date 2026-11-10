use super::*;
use pretty_assertions::assert_eq;
use std::fs;
use tempfile::TempDir;

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("create folder");
    fs::write(path, text).expect("write");
}

/// Whether `dir` still holds `name`, and what it holds renamed aside as a
/// code cache.
fn where_is(dir: &Path, name: &str) -> (bool, Vec<PathBuf>) {
    let aside = fs::read_dir(dir)
        .expect("list")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().is_some_and(|found| {
                found
                    .to_string_lossy()
                    .starts_with(&format!("{name}.corbanu-sandbox-cache-"))
            })
        })
        .collect();
    (dir.join(name).exists(), aside)
}

/// `dir`'s `name`, renamed aside (once).
fn moved(dir: &Path, name: &str) -> PathBuf {
    let (still_there, aside) = where_is(dir, name);
    assert!(!still_there, "{name} was not renamed aside");
    assert_eq!(aside.len(), 1, "{name}: {aside:?}");
    aside[0].clone()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// D456: at the D377 upgrade step, the code caches other programs keep in
/// the temp folder and run code from are renamed in place (moved, not
/// deleted), one inside another first, so those programs make new ones; a
/// link with such a name is renamed itself, its target left alone. Other
/// folders stay.
#[test]
fn sec_win_d456_code_caches_in_temp_are_renamed_aside() {
    let temp = TempDir::new().expect("temp");
    let base = dunce::canonicalize(temp.path()).expect("canonical");
    let elsewhere = TempDir::new().expect("elsewhere");
    let target = dunce::canonicalize(elsewhere.path()).expect("canonical");
    // Code caches.
    write(
        &base.join("node-compile-cache\\v22.12.0-x64-1a2b-0\\c.cache"),
        "node",
    );
    write(&base.join("v8-compile-cache-0\\12.4\\x.BLOB"), "v8");
    write(&base.join("tsx-user\\t.js"), "tsx");
    write(&base.join("node-jiti\\a.js"), "jiti");
    write(&base.join("node-jiti\\x\\__pycache__\\n.pyc"), "nested");
    write(&base.join("proj\\__pycache__\\m.cpython-312.pyc"), "pyc");
    write(&base.join("proj\\m.py"), "print(1)\n");
    write(&base.join("other\\__pycache__\\notes.txt"), "pyc folder");
    // A link with a cache's name, to a folder the sandbox may write.
    write(&target.join("planted.BLOB"), "planted");
    let link = base.join("v8-compile-cache");
    let made = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .output()
        .expect("mklink");
    assert!(made.status.success(), "mklink /J: {made:?}");
    // Not: a cache's name below the top, and a file with a cache's name.
    write(&base.join("sub\\node-compile-cache\\c.cache"), "keep");
    write(&base.join("jiti"), "a file");

    // An account that owns nothing here: only the code caches change.
    let found =
        take_sandbox_leftovers(&base, &["S-1-5-21-1-2-3-4".to_string()]).expect("take leftovers");
    println!("d456: {found:?}");
    assert_eq!(
        (
            found.code_caches,
            found.code_caches_set_aside,
            found.code_caches_capped
        ),
        (8, 8, false)
    );

    assert_eq!(
        read(&moved(&base, "node-compile-cache").join("v22.12.0-x64-1a2b-0\\c.cache")),
        "node"
    );
    assert_eq!(
        read(&moved(&base, "v8-compile-cache-0").join("12.4\\x.BLOB")),
        "v8"
    );
    assert_eq!(read(&moved(&base, "tsx-user").join("t.js")), "tsx");
    let jiti = moved(&base, "node-jiti");
    assert_eq!(read(&jiti.join("a.js")), "jiti");
    assert_eq!(
        read(&moved(&jiti.join("x"), "__pycache__").join("n.pyc")),
        "nested"
    );
    assert_eq!(
        read(&moved(&base.join("proj"), "__pycache__").join("m.cpython-312.pyc")),
        "pyc"
    );
    assert_eq!(
        read(&moved(&base.join("other"), "__pycache__").join("notes.txt")),
        "pyc folder"
    );
    // The link itself, still leading where it did; its target unchanged.
    let link_aside = moved(&base, "v8-compile-cache");
    assert!(
        fs::symlink_metadata(&link_aside)
            .expect("link")
            .file_type()
            .is_symlink(),
        "{} is not the link",
        link_aside.display()
    );
    assert_eq!(read(&target.join("planted.BLOB")), "planted");
    assert_eq!(
        fs::read_dir(&target).expect("list").count(),
        1,
        "the link's target changed"
    );
    for (path, text) in [
        ("proj\\m.py", "print(1)\n"),
        ("sub\\node-compile-cache\\c.cache", "keep"),
        ("jiti", "a file"),
    ] {
        assert_eq!(read(&base.join(path)), text, "{path}");
    }
}
