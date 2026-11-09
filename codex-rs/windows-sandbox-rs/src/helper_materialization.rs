//! Resolve sandbox helpers and materialize legacy executables with inherited sandbox ACLs.
//! An explicit registered-runtime request never falls through to copying or PATH lookup;
//! the service and startup handshake independently verify the installed image.

mod copy;
use copy::CopyOutcome;
use copy::copy_from_source_if_needed;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::app_package::registered_core_requested;
use crate::logging::log_note;
use crate::sandbox_bin_dir;
use crate::setup::SetupRuntime;

const DEV_BUILD_VERSION_SENTINEL: &str = "0.0.0";
const COMMAND_RUNNER_EXE: &str = "codex-command-runner.exe";
pub(crate) const BIN_DIRNAME: &str = "bin";
pub(crate) const RESOURCES_DIRNAME: &str = "codex-resources";

pub(crate) fn helper_bin_dir(codex_home: &Path) -> PathBuf {
    sandbox_bin_dir(codex_home)
}

fn legacy_lookup() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(candidate) = bundled_executable_path_for_exe(&exe, COMMAND_RUNNER_EXE)
    {
        return candidate;
    }
    PathBuf::from(COMMAND_RUNNER_EXE)
}

pub(crate) fn resolve_command_runner(codex_home: &Path, log_dir: Option<&Path>) -> Result<PathBuf> {
    if registered_core_requested() {
        let exe = std::env::current_exe().context("resolve registered Core helper source")?;
        let direct_path = exe.with_file_name(COMMAND_RUNNER_EXE);
        log_note(
            &format!(
                "helper launch resolution: using app-contained command-runner path {}",
                direct_path.display()
            ),
            log_dir,
        );
        // Missing packaged helpers must fail rather than search PATH or create a copy.
        return Ok(direct_path);
    }
    Ok(match copy_runner_if_needed(codex_home, log_dir) {
        Ok(path) => {
            keep_helper_bin_dir_medium(codex_home, log_dir);
            log_note(
                &format!(
                    "helper launch resolution: using copied command-runner path {}",
                    path.display()
                ),
                log_dir,
            );
            path
        }
        Err(err) => {
            let fallback = legacy_lookup();
            log_note(
                &format!(
                    "helper copy failed for command-runner: {err:#}; falling back to legacy path {}",
                    fallback.display()
                ),
                log_dir,
            );
            keep_fallback_helper_medium(&fallback, log_dir);
            fallback
        }
    })
}

/// #1819: an executable starts at the lower of its token's and its file's
/// integrity. While a legacy session uses a writable root holding this codex
/// home (`%TEMP%`, or a workspace above it), the root's inheritable Low label
/// reaches the copied helpers, so the runner started at Low and could not
/// open the parent's pipe for writing. An explicit Medium label (what an
/// unlabelled file has anyway) keeps the root's Low label off them, and Low
/// programs out of them. A link is skipped: its label would land on the link
/// itself, not on the folder it points at.
fn keep_helper_bin_dir_medium(codex_home: &Path, log_dir: Option<&Path>) {
    let dir = helper_bin_dir(codex_home);
    if crate::workspace_acl::is_link(&dir) {
        log_note(
            &format!(
                "helper copy: {} is a link (or can't be inspected); not labelling it Medium, so a legacy session's Low label may reach the runner (#1819)",
                dir.display()
            ),
            log_dir,
        );
        return;
    }
    // SAFETY: Win32 security calls on a folder of this codex home.
    if let Err(err) = unsafe { crate::integrity::label_protected_path_medium(&dir) } {
        log_note(
            &format!(
                "helper copy: could not label {} Medium; a legacy session's Low label may reach it: {err:#}",
                dir.display()
            ),
            log_dir,
        );
    }
}

/// #1819, for the runner next to `codex.exe` used when copying failed (say a
/// dev build inside a legacy workspace): if its label is Low, most likely
/// inherited from a folder a legacy session made writable, it would start at
/// Low and time out opening the parent's pipes. Only that file (not its
/// folder) gets an explicit Medium label; a link is left alone.
fn keep_fallback_helper_medium(fallback: &Path, log_dir: Option<&Path>) {
    // A bare name is found on `PATH` at launch; only a file is labelled.
    if !fallback.is_absolute() || !fallback.is_file() {
        return;
    }
    let cause = "the likely cause is a Low label inherited from a folder a legacy session made writable (#1819)";
    if crate::workspace_acl::is_link(fallback) {
        log_note(
            &format!(
                "helper fallback: {} is a link; not checking its label. If the runner times out connecting its pipes, {cause}",
                fallback.display()
            ),
            log_dir,
        );
        return;
    }
    // SAFETY: reads the label of the fallback runner file.
    let note = match unsafe { crate::integrity::effective_label_is_low(fallback) } {
        Ok(false) => return,
        // SAFETY: labels only the fallback runner file, which is no link.
        Ok(true) => match unsafe { crate::integrity::label_protected_path_medium(fallback) } {
            Ok(()) => format!(
                "helper fallback: {} was labelled Low; labelled it Medium so the runner can open its pipes ({cause})",
                fallback.display()
            ),
            Err(err) => format!(
                "helper fallback: {} is labelled Low and could not be labelled Medium ({err:#}); the runner will start at Low and time out connecting its pipes ({cause})",
                fallback.display()
            ),
        },
        Err(err) => format!(
            "helper fallback: could not read the label of {} ({err:#}); if the runner times out connecting its pipes, {cause}",
            fallback.display()
        ),
    };
    log_note(&note, log_dir);
}

pub fn resolve_exe_for_launch(source: &Path, codex_home: &Path) -> PathBuf {
    let runtime = crate::setup::current_setup_runtime();
    resolve_exe_for_runtime(source, codex_home, runtime)
}

fn resolve_exe_for_runtime(source: &Path, codex_home: &Path, runtime: SetupRuntime) -> PathBuf {
    let sandbox_log_dir = crate::sandbox_dir(codex_home);
    if runtime == SetupRuntime::Registered {
        log_note(
            &format!(
                "helper executable resolution: route=direct source={} selected={}",
                source.display(),
                source.display()
            ),
            Some(&sandbox_log_dir),
        );
        return source.to_path_buf();
    }
    let Some(file_name) = source.file_name() else {
        return source.to_path_buf();
    };
    let destination = helper_bin_dir(codex_home).join(file_name);
    match copy_from_source_if_needed(source, &destination) {
        Ok(_) => {
            log_note(
                &format!(
                    "helper executable resolution: route=materialized source={} selected={}",
                    source.display(),
                    destination.display()
                ),
                Some(&sandbox_log_dir),
            );
            destination
        }
        Err(err) => {
            log_note(
                &format!(
                    "helper copy failed for executable: {err:#}; falling back to legacy path {}",
                    source.display()
                ),
                Some(&sandbox_log_dir),
            );
            source.to_path_buf()
        }
    }
}

fn copy_runner_if_needed(codex_home: &Path, log_dir: Option<&Path>) -> Result<PathBuf> {
    let source = sibling_source_path()?;
    let suffix = helper_version_suffix(&source)?;
    let destination = helper_bin_dir(codex_home).join(materialized_file_name(&suffix));
    log_note(
        &format!(
            "helper copy: validating command-runner source={} destination={}",
            source.display(),
            destination.display()
        ),
        log_dir,
    );
    let outcome = copy_from_source_if_needed(&source, &destination)?;
    let action = match outcome {
        CopyOutcome::Reused => "reused",
        CopyOutcome::ReCopied => "recopied",
    };
    log_note(
        &format!(
            "helper copy: {} command-runner source={} destination={}",
            action,
            source.display(),
            destination.display()
        ),
        log_dir,
    );
    Ok(destination)
}

fn sibling_source_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("resolve current executable for helper lookup")?;
    bundled_executable_path_for_exe(&exe, COMMAND_RUNNER_EXE).ok_or_else(|| {
        anyhow!(
            "helper not found next to current executable or under {RESOURCES_DIRNAME}: {}",
            exe.display()
        )
    })
}

pub(crate) fn bundled_executable_path_for_exe(exe: &Path, file_name: &str) -> Option<PathBuf> {
    let find = |exe: &Path| {
        let dir = exe.parent()?;
        let direct_candidate = dir.join(file_name);
        if direct_candidate.is_file() {
            return Some(direct_candidate);
        }

        if dir.file_name() == Some(OsStr::new(BIN_DIRNAME))
            && let Some(package_dir) = dir.parent()
        {
            let package_resource_candidate = package_dir.join(RESOURCES_DIRNAME).join(file_name);
            if package_resource_candidate.is_file() {
                return Some(package_resource_candidate);
            }
        }

        let resource_candidate = dir.join(RESOURCES_DIRNAME).join(file_name);
        resource_candidate.is_file().then_some(resource_candidate)
    };

    // Installer bin directories can be junctions, so retry beside the real executable once.
    find(exe).or_else(|| find(&dunce::canonicalize(exe).ok()?))
}

fn materialized_file_name(suffix: &str) -> String {
    format!("codex-command-runner-{suffix}.exe")
}

fn helper_version_suffix(source: &Path) -> Result<String> {
    let version = env!("CARGO_PKG_VERSION");
    if version == DEV_BUILD_VERSION_SENTINEL {
        dev_build_suffix(source)
    } else {
        Ok(version.to_string())
    }
}

fn dev_build_suffix(source: &Path) -> Result<String> {
    let metadata = fs::metadata(source)
        .with_context(|| format!("read helper source metadata {}", source.display()))?;
    let modified = metadata
        .modified()
        .with_context(|| format!("read helper source mtime {}", source.display()))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .with_context(|| format!("convert helper source mtime {}", source.display()))?;
    Ok(format!("{}-{:x}", metadata.len(), duration.as_secs(),))
}

#[cfg(test)]
mod tests {
    use super::BIN_DIRNAME;
    use super::CopyOutcome;
    use super::DEV_BUILD_VERSION_SENTINEL;
    use super::RESOURCES_DIRNAME;
    use super::bundled_executable_path_for_exe;
    use super::copy_from_source_if_needed;
    use super::dev_build_suffix;
    use super::helper_bin_dir;
    use super::helper_version_suffix;
    use super::keep_fallback_helper_medium;
    use super::keep_helper_bin_dir_medium;
    use super::materialized_file_name;
    use super::resolve_exe_for_runtime;
    use crate::setup::SetupRuntime;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn helper_bin_dir_is_under_sandbox_bin() {
        let codex_home = Path::new(r"C:\Users\example\.codex");

        assert_eq!(
            PathBuf::from(r"C:\Users\example\.codex\.sandbox-bin"),
            helper_bin_dir(codex_home)
        );
    }

    #[test]
    fn registered_request_does_not_materialize_or_replace_a_missing_source() {
        let tmp = TempDir::new().expect("tempdir");
        let executable = tmp.path().join("codex.exe");
        let home = tmp.path().join("home");
        for content in [None, Some(b"fixture".as_slice())] {
            if let Some(content) = content {
                fs::write(&executable, content).expect("write source");
            }
            assert_eq!(
                resolve_exe_for_runtime(&executable, &home, SetupRuntime::Registered),
                executable
            );
            assert!(!helper_bin_dir(&home).exists());
        }
    }

    #[test]
    fn legacy_request_materializes_the_same_source() {
        let tmp = TempDir::new().expect("tempdir");
        let executable = tmp.path().join("codex.exe");
        let home = tmp.path().join("home");
        fs::write(&executable, b"fixture").expect("write source");
        let destination = helper_bin_dir(&home).join("codex.exe");
        assert_eq!(
            resolve_exe_for_runtime(&executable, &home, SetupRuntime::Legacy),
            destination
        );
        assert_eq!(fs::read(destination).expect("read copy"), b"fixture");
    }

    #[test]
    fn copy_runner_into_shared_bin_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let codex_home = tmp.path().join("codex-home");
        let source_dir = tmp.path().join("sibling-source");
        fs::create_dir_all(&source_dir).expect("create source dir");
        let runner_source = source_dir.join("codex-command-runner.exe");
        fs::write(&runner_source, b"runner").expect("runner");
        let runner_suffix = helper_version_suffix(&runner_source).expect("runner suffix");
        let runner_destination =
            helper_bin_dir(&codex_home).join(materialized_file_name(&runner_suffix));

        let runner_outcome =
            copy_from_source_if_needed(&runner_source, &runner_destination).expect("runner copy");

        assert_eq!(CopyOutcome::ReCopied, runner_outcome);
        assert_eq!(
            b"runner".as_slice(),
            fs::read(&runner_destination).expect("read runner")
        );
    }

    #[test]
    fn helper_source_lookup_checks_resource_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let release_dir = tmp.path().join("release");
        let resources_dir = release_dir.join(RESOURCES_DIRNAME);
        fs::create_dir_all(&resources_dir).expect("create resources dir");
        let exe = release_dir.join("codex.exe");
        let helper = resources_dir.join("codex-command-runner.exe");
        fs::write(&exe, b"codex").expect("write exe");
        fs::write(&helper, b"runner").expect("write helper");

        let resolved =
            bundled_executable_path_for_exe(&exe, /*file_name*/ "codex-command-runner.exe")
                .expect("helper path");

        assert_eq!(resolved, helper);
    }

    #[test]
    fn helper_source_lookup_checks_package_resource_dir_for_bin_exe() {
        let tmp = TempDir::new().expect("tempdir");
        let package_dir = tmp.path().join("package");
        let bin_dir = package_dir.join(BIN_DIRNAME);
        let resources_dir = package_dir.join(RESOURCES_DIRNAME);
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::create_dir_all(&resources_dir).expect("create resources dir");
        let exe = bin_dir.join("codex.exe");
        let helper = resources_dir.join("codex-command-runner.exe");
        fs::write(&exe, b"codex").expect("write exe");
        fs::write(&helper, b"runner").expect("write helper");

        let resolved =
            bundled_executable_path_for_exe(&exe, /*file_name*/ "codex-command-runner.exe")
                .expect("helper path");

        assert_eq!(resolved, helper);
    }

    #[test]
    fn helper_source_lookup_resolves_bin_junctions() {
        let tmp = TempDir::new().expect("tempdir");
        let package_dir = tmp.path().join("package");
        let bin_dir = package_dir.join(BIN_DIRNAME);
        let resources_dir = package_dir.join(RESOURCES_DIRNAME);
        let install_dir = tmp.path().join("install");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::create_dir_all(&resources_dir).expect("create resources dir");
        fs::create_dir_all(&install_dir).expect("create install dir");
        fs::write(bin_dir.join("codex.exe"), b"codex").expect("write exe");
        let helper = resources_dir.join("codex-windows-sandbox-setup.exe");
        fs::write(&helper, b"setup").expect("write helper");

        let junction = install_dir.join(BIN_DIRNAME);
        let output = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&junction)
            .arg(&bin_dir)
            .output()
            .expect("create bin junction");
        assert!(output.status.success());

        assert_eq!(
            bundled_executable_path_for_exe(
                &junction.join("codex.exe"),
                /*file_name*/ "codex-windows-sandbox-setup.exe"
            ),
            Some(dunce::canonicalize(&helper).expect("canonical helper"))
        );
    }

    #[test]
    fn helper_source_lookup_prefers_package_resource_dir_over_bin_resource_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let package_dir = tmp.path().join("package");
        let bin_dir = package_dir.join(BIN_DIRNAME);
        let package_resources_dir = package_dir.join(RESOURCES_DIRNAME);
        let bin_resources_dir = bin_dir.join(RESOURCES_DIRNAME);
        fs::create_dir_all(&package_resources_dir).expect("create package resources dir");
        fs::create_dir_all(&bin_resources_dir).expect("create bin resources dir");
        let exe = bin_dir.join("codex.exe");
        let package_helper = package_resources_dir.join("codex-command-runner.exe");
        let bin_helper = bin_resources_dir.join("codex-command-runner.exe");
        fs::write(&exe, b"codex").expect("write exe");
        fs::write(&package_helper, b"package runner").expect("write package helper");
        fs::write(&bin_helper, b"bin runner").expect("write bin helper");

        let resolved =
            bundled_executable_path_for_exe(&exe, /*file_name*/ "codex-command-runner.exe")
                .expect("helper path");

        assert_eq!(resolved, package_helper);
    }

    #[test]
    fn helper_source_lookup_prefers_direct_sibling_over_resource_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let release_dir = tmp.path().join("release");
        let resources_dir = release_dir.join(RESOURCES_DIRNAME);
        fs::create_dir_all(&resources_dir).expect("create resources dir");
        let exe = release_dir.join("codex.exe");
        let sibling_helper = release_dir.join("codex-command-runner.exe");
        let resource_helper = resources_dir.join("codex-command-runner.exe");
        fs::write(&exe, b"codex").expect("write exe");
        fs::write(&sibling_helper, b"sibling runner").expect("write sibling helper");
        fs::write(&resource_helper, b"resource runner").expect("write resource helper");

        let resolved =
            bundled_executable_path_for_exe(&exe, /*file_name*/ "codex-command-runner.exe")
                .expect("helper path");

        assert_eq!(resolved, sibling_helper);
    }

    #[test]
    fn helper_version_suffix_uses_cli_version_or_dev_build_metadata() {
        let tmp = TempDir::new().expect("tempdir");
        let source = tmp.path().join("source.exe");
        fs::write(&source, b"runner-v1").expect("write source");
        let suffix = helper_version_suffix(&source).expect("suffix");

        if env!("CARGO_PKG_VERSION") == DEV_BUILD_VERSION_SENTINEL {
            assert_eq!(suffix, dev_build_suffix(&source).expect("dev build suffix"));
        } else {
            assert_eq!(suffix, env!("CARGO_PKG_VERSION"));
        }
    }

    #[test]
    fn materialized_file_name_adds_suffix_before_extension() {
        let file_name = materialized_file_name("test-suffix");

        assert_eq!(file_name, "codex-command-runner-test-suffix.exe");
    }

    fn read_logs(dir: &Path) -> String {
        fs::read_dir(dir)
            .expect("read log dir")
            .map(|entry| fs::read_to_string(entry.expect("log entry").path()).expect("read log"))
            .collect()
    }

    /// #1819 follow-up: a `.sandbox-bin` that is a junction is not labelled
    /// (Windows would label the link itself, not its target, so the fix
    /// would silently not apply); the log says so.
    #[test]
    fn sec_win_1819_helper_bin_dir_link_is_not_labelled() {
        let tmp = TempDir::new().expect("tempdir");
        let codex_home = tmp.path().join("home");
        let target = tmp.path().join("elsewhere");
        let logs = tmp.path().join("logs");
        fs::create_dir_all(&codex_home).expect("home");
        fs::create_dir_all(&target).expect("target");
        fs::create_dir_all(&logs).expect("logs");
        let link = helper_bin_dir(&codex_home);
        let output = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&link)
            .arg(&target)
            .output()
            .expect("mklink");
        assert!(output.status.success(), "mklink: {output:?}");
        // SAFETY: reads labels of folders this test made.
        let (before, link_before) = unsafe {
            (
                crate::integrity::read_explicit_labels(&target).expect("labels"),
                crate::integrity::read_own_explicit_labels(&link).expect("link labels"),
            )
        };

        keep_helper_bin_dir_medium(&codex_home, Some(&logs));

        let log = read_logs(&logs);
        assert!(
            log.contains("is a link (or can't be inspected); not labelling it Medium"),
            "{log}"
        );
        assert!(!log.contains("could not label"), "{log}");
        assert!(crate::workspace_acl::is_link(&link));
        // SAFETY: reads labels of folders this test made. Before the fix,
        // Windows' walk wrote the Medium label on the junction itself.
        unsafe {
            assert_eq!(
                crate::integrity::read_explicit_labels(&target).expect("labels"),
                before
            );
            assert_eq!(
                crate::integrity::read_own_explicit_labels(&link).expect("link labels"),
                link_before
            );
        }
    }

    /// #1819 follow-up: the runner next to `codex.exe`, used when copying it
    /// failed, gets a Medium label when it inherited a legacy root's Low
    /// label, and the log names that cause; one that isn't Low is left alone.
    #[test]
    fn sec_win_1819_low_fallback_runner_is_labelled_medium() {
        let tmp = TempDir::new().expect("tempdir");
        let root = tmp.path().join("legacy-root");
        let logs = tmp.path().join("logs");
        fs::create_dir_all(&root).expect("root");
        fs::create_dir_all(&logs).expect("logs");
        // SAFETY: labels a folder this test made, as a legacy session does.
        unsafe { crate::integrity::label_writable_root_low(&root) }.expect("label root Low");
        let runner = root.join("codex-command-runner.exe");
        fs::write(&runner, b"runner").expect("write runner");
        let plain = tmp.path().join("plain-runner.exe");
        fs::write(&plain, b"runner").expect("write plain runner");
        // SAFETY: reads labels of files this test made.
        unsafe {
            assert!(crate::integrity::effective_label_is_low(&runner).expect("label"));
            assert!(!crate::integrity::effective_label_is_low(&plain).expect("label"));
        }
        // SAFETY: reads the label of a file this test made.
        let plain_before =
            unsafe { crate::integrity::read_explicit_labels(&plain) }.expect("labels");

        keep_fallback_helper_medium(&plain, Some(&logs));
        keep_fallback_helper_medium(&runner, Some(&logs));

        let log = read_logs(&logs);
        assert!(
            log.contains("was labelled Low; labelled it Medium")
                && log.contains("inherited from a folder a legacy session made writable"),
            "{log}"
        );
        assert!(!log.contains("plain-runner"), "{log}");
        // SAFETY: reads labels of files this test made.
        unsafe {
            assert!(!crate::integrity::effective_label_is_low(&runner).expect("label"));
            assert_eq!(
                crate::integrity::read_explicit_labels(&runner).expect("labels"),
                vec![crate::integrity::LabelAce {
                    flags: 0,
                    mask: 1,
                    sid: "S-1-16-8192".to_string(),
                }]
            );
            assert_eq!(
                crate::integrity::read_explicit_labels(&plain).expect("labels"),
                plain_before
            );
        }
    }
}
