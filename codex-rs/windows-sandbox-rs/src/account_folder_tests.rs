//! #2819: the records folder follows the account, not the environment.

use super::*;
use pretty_assertions::assert_eq;
use std::path::Path;

/// Set in the child: where to write the records folder it found.
const CHILD_OUTPUT: &str = "CORBANU_TEST_2819_CHILD_OUTPUT";
const CHILD_TEST: &str = "account_folder::tests::child_reports_records_folder";

/// What the shell's known-folder lookup gives this (unchanged) process.
fn shell_local_app_data() -> PathBuf {
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;
    let mut path: *mut u16 = std::ptr::null_mut();
    // SAFETY: a null token means this user; the string is freed below.
    let status =
        unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, 0, std::mem::zeroed(), &mut path) };
    assert_eq!(status, 0, "SHGetKnownFolderPath");
    // SAFETY: a NUL-terminated string from the call.
    let len = (0..).take_while(|&i| unsafe { *path.add(i) } != 0).count();
    // SAFETY: as above.
    let found = PathBuf::from(String::from_utf16_lossy(unsafe {
        std::slice::from_raw_parts(path, len)
    }));
    // SAFETY: allocated by SHGetKnownFolderPath.
    unsafe { CoTaskMemFree(path.cast()) };
    found
}

/// Runs [`child_reports_records_folder`] with `env` (and without the
/// debug override unless `env` sets it) and returns what it found.
fn records_folder_in_child(dir: &Path, env: &[(&str, &Path)]) -> PathBuf {
    let output = dir.join("records-folder.txt");
    let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
    command
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD_OUTPUT, &output)
        .env_remove(crate::integrity_leases::DEBUG_LOCAL_APP_DATA_ENV_VAR);
    for (name, value) in env {
        command.env(name, value);
    }
    let run = command.output().expect("run child");
    assert!(
        run.status.success(),
        "child failed: {}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    PathBuf::from(std::fs::read_to_string(&output).expect("child output"))
}

#[test]
fn child_reports_records_folder() {
    let Some(output) = std::env::var_os(CHILD_OUTPUT) else {
        return;
    };
    let parent = crate::integrity_leases::registry_parent_dir().expect("records folder");
    std::fs::write(output, parent.to_string_lossy().as_bytes()).expect("write output");
}

/// Normal users: the account lookup gives what the shell gave before.
#[test]
fn account_local_app_data_matches_the_shell_for_an_unchanged_environment() {
    assert_eq!(local_app_data(), Some(shell_local_app_data()));
}

/// A changed `USERPROFILE`/`LOCALAPPDATA` no longer moves the records (the
/// shell's lookup followed `USERPROFILE` to `<fake>\AppData\Local`).
#[test]
fn a_changed_environment_does_not_move_the_records_folder() {
    let dir = tempfile::tempdir().expect("temp");
    let profile = dir.path().join("fake-profile");
    let local = dir.path().join("fake-local");
    std::fs::create_dir_all(profile.join(r"AppData\Local")).expect("fake profile");
    std::fs::create_dir_all(&local).expect("fake LocalAppData");
    let expected = local_app_data()
        .expect("account LocalAppData")
        .join("CorbanuTerminalSandbox");

    let found = records_folder_in_child(
        dir.path(),
        &[("USERPROFILE", &profile), ("LOCALAPPDATA", &local)],
    );

    assert_eq!(found, expected);
    assert!(
        !found.starts_with(dir.path()),
        "the spoofed environment moved the records to {}",
        found.display()
    );
}

/// The debug-build seam integration tests of the real CLI use (#2816).
#[cfg(debug_assertions)]
#[test]
fn debug_builds_take_the_records_folder_from_the_test_variable() {
    let dir = tempfile::tempdir().expect("temp");
    let local = dir.path().join("test-local");
    let found = records_folder_in_child(
        dir.path(),
        &[(
            crate::integrity_leases::DEBUG_LOCAL_APP_DATA_ENV_VAR,
            &local,
        )],
    );
    assert_eq!(found, local.join("CorbanuTerminalSandbox"));
}
