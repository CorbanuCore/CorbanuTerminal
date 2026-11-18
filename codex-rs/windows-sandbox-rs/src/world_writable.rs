//! D607: main's world-writable scan (`audit.rs`), which upstream removed
//! (#44933/#47943). An elevated command's restricted token lists Everyone
//! among its restricting SIDs, so a folder that grants Everyone write access
//! is writable from the sandbox at every level. The launcher finds such
//! folders here, with main's candidates and limits; the elevated setup denies
//! writes to them for the sandbox's capability SIDs
//! (`setup_provisioning::apply_world_writable_denies`), as main's TUI did.

use crate::acl::path_mask_allows;
use crate::logging::debug_log;
use crate::logging::log_note;
use crate::path_normalization::canonical_path_key;
use crate::token::world_sid;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;
use windows_sys::Win32::Storage::FileSystem::FILE_APPEND_DATA;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_DATA;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_EA;

// Main's preflight scan limits.
const MAX_ITEMS_PER_DIR: usize = 1000;
const TIME_LIMIT: Duration = Duration::from_secs(2);
const MAX_CHECKED: usize = 50_000;
// Case-insensitive suffixes (forward slashes) skipped in the one-level child scan.
const SKIP_DIR_SUFFIXES: &[&str] = &[
    "/windows/installer",
    "/windows/registration",
    "/programdata",
];

fn unique_push(set: &mut HashSet<PathBuf>, out: &mut Vec<PathBuf>, path: PathBuf) {
    if let Ok(path) = dunce::canonicalize(path)
        && set.insert(path.clone())
    {
        out.push(path);
    }
}

pub(crate) fn gather_candidates(cwd: &Path, env: &HashMap<String, String>) -> Vec<PathBuf> {
    let mut set = HashSet::new();
    let mut out = Vec::new();
    // 1) The working folder first, so its children are checked early.
    unique_push(&mut set, &mut out, cwd.to_path_buf());
    // 2) TEMP/TMP (usually small).
    for key in ["TEMP", "TMP"] {
        if let Some(value) = env.get(key).cloned().or_else(|| std::env::var(key).ok()) {
            unique_push(&mut set, &mut out, PathBuf::from(value));
        }
    }
    // 3) User roots.
    for key in ["USERPROFILE", "PUBLIC"] {
        if let Some(value) = std::env::var_os(key) {
            unique_push(&mut set, &mut out, PathBuf::from(value));
        }
    }
    // 4) PATH entries.
    if let Some(path) = crate::env::get_ignoring_case(env, "PATH")
        .cloned()
        .or_else(|| std::env::var("PATH").ok())
    {
        for part in std::env::split_paths(OsStr::new(&path)) {
            if !part.as_os_str().is_empty() {
                unique_push(&mut set, &mut out, part);
            }
        }
    }
    // 5) Core system roots last.
    for path in [PathBuf::from("C:/"), PathBuf::from("C:/Windows")] {
        unique_push(&mut set, &mut out, path);
    }
    out
}

fn grants_everyone_write(path: &Path, logs_base_dir: Option<&Path>) -> bool {
    let world = unsafe { world_sid() };
    let result = world.and_then(|mut world| {
        path_mask_allows(
            path,
            &[world.as_mut_ptr() as *mut c_void],
            FILE_WRITE_DATA | FILE_APPEND_DATA | FILE_WRITE_EA | FILE_WRITE_ATTRIBUTES,
            /*require_all_bits*/ false,
        )
    });
    result.unwrap_or_else(|err| {
        debug_log(
            &format!(
                "AUDIT: treating unreadable ACL as not world-writable: {} ({err})",
                path.display()
            ),
            logs_base_dir,
        );
        false
    })
}

/// Folders among main's candidates that grant Everyone write access: the
/// working folder's children, TEMP/TMP, the user and public profiles, `PATH`
/// entries, `C:\` and `C:\Windows`, each with its children one level down.
/// Links are not followed. Best effort, bounded by main's time and count
/// limits.
pub(crate) fn everyone_writable_folders(
    cwd: &Path,
    env: &HashMap<String, String>,
    logs_base_dir: Option<&Path>,
) -> Vec<PathBuf> {
    let start = Instant::now();
    let mut flagged = Vec::new();
    let mut seen = HashSet::new();
    let mut checked = 0usize;
    let out_of_budget = |checked: usize| start.elapsed() > TIME_LIMIT || checked > MAX_CHECKED;
    let mut check = |path: PathBuf, checked: &mut usize, flagged: &mut Vec<PathBuf>| {
        *checked += 1;
        if grants_everyone_write(&path, logs_base_dir) && seen.insert(canonical_path_key(&path)) {
            flagged.push(path);
        }
    };
    let children = |root: &Path| -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(root) else {
            return Vec::new();
        };
        entries
            .flatten()
            .take(MAX_ITEMS_PER_DIR)
            .filter(|entry| {
                entry
                    .file_type()
                    .is_ok_and(|kind| kind.is_dir() && !kind.is_symlink())
            })
            .map(|entry| entry.path())
            .filter(|path| {
                let normalized = path
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .replace('\\', "/");
                !SKIP_DIR_SUFFIXES
                    .iter()
                    .any(|suffix| normalized.ends_with(suffix))
            })
            .collect()
    };
    // The working folder's children first, so workspace issues are caught early.
    for child in children(cwd) {
        if out_of_budget(checked) {
            break;
        }
        check(child, &mut checked, &mut flagged);
    }
    for root in gather_candidates(cwd, env) {
        if out_of_budget(checked) {
            break;
        }
        check(root.clone(), &mut checked, &mut flagged);
        for child in children(&root) {
            if out_of_budget(checked) {
                break;
            }
            check(child, &mut checked, &mut flagged);
        }
    }
    let elapsed_ms = start.elapsed().as_millis();
    if flagged.is_empty() {
        log_note(
            &format!("AUDIT: world-writable scan OK; checked={checked}; duration_ms={elapsed_ms}"),
            logs_base_dir,
        );
    } else {
        let list: String = flagged
            .iter()
            .map(|path| format!("\n - {}", path.display()))
            .collect();
        log_note(
            &format!(
                "AUDIT: world-writable scan flagged; cwd={cwd:?}; checked={checked}; duration_ms={elapsed_ms}; flagged:{list}"
            ),
            logs_base_dir,
        );
    }
    flagged
}

/// [`everyone_writable_folders`], scanned once per working folder and `PATH`
/// in this process, as main scanned once at startup: a setup refresh runs
/// before every elevated command.
pub(crate) fn cached_everyone_writable_folders(
    cwd: &Path,
    env: &HashMap<String, String>,
    logs_base_dir: Option<&Path>,
) -> Vec<PathBuf> {
    static CACHE: OnceLock<Mutex<HashMap<(String, String), Vec<PathBuf>>>> = OnceLock::new();
    let key = (
        canonical_path_key(cwd),
        crate::env::get_ignoring_case(env, "PATH")
            .cloned()
            .unwrap_or_default(),
    );
    let cache = CACHE.get_or_init(Default::default);
    if let Some(flagged) = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&key)
    {
        return flagged.clone();
    }
    let flagged = everyone_writable_folders(cwd, env, logs_base_dir);
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(key, flagged.clone());
    flagged
}

#[cfg(test)]
#[path = "world_writable_tests.rs"]
pub(crate) mod tests;
