use super::*;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn system_bwrap_warning_reports_missing_system_bwrap() {
    assert_eq!(
        system_bwrap_warning_for_path(/*system_bwrap_path*/ None, || true, || true),
        Some(MISSING_BWRAP_WARNING.to_string())
    );
}

/// Issue #563: without a bundled copy either, the warning does not promise
/// one: it says the sandbox cannot run commands and what to install.
#[test]
fn system_bwrap_warning_without_a_bundled_copy_names_what_to_install() {
    let warning = system_bwrap_warning_for_path(/*system_bwrap_path*/ None, || false, || true);
    assert_eq!(warning, Some(NO_BWRAP_WARNING.to_string()));
    assert!(NO_BWRAP_WARNING.contains("cannot run commands in its Linux sandbox"));
    assert!(NO_BWRAP_WARNING.contains("sudo apt install bubblewrap"));
    assert!(!NO_BWRAP_WARNING.contains("bundled bubblewrap in the meantime"));
}

/// #581: where the bundled copy cannot create user namespaces (AppArmor
/// restricts them and no `rootlesskit` profile allows them), the warning
/// says the sandbox cannot start and names the package, instead of
/// promising the bundled copy.
#[test]
fn system_bwrap_warning_says_when_the_bundled_copy_cannot_start() {
    let warning = system_bwrap_warning_for_path(/*system_bwrap_path*/ None, || true, || false);
    assert_eq!(warning, Some(BUNDLED_BWRAP_BLOCKED_WARNING.to_string()));
    assert!(BUNDLED_BWRAP_BLOCKED_WARNING.contains("cannot start"));
    assert!(BUNDLED_BWRAP_BLOCKED_WARNING.contains("sudo apt install bubblewrap"));
    assert!(BUNDLED_BWRAP_BLOCKED_WARNING.contains("rootlesskit"));
    assert!(!BUNDLED_BWRAP_BLOCKED_WARNING.contains("in the meantime"));
}

/// Issue #563: the refusal printed when no bubblewrap is found keeps the
/// marker the tool orchestrator treats as a sandbox startup failure, and
/// counts as a sandbox denial, as the panic it replaces did; and it names
/// what to install.
#[test]
fn bwrap_unavailable_message_keeps_its_classification_and_names_what_to_install() {
    use codex_protocol::exec_output::ExecToolCallOutput;
    use codex_protocol::exec_output::StreamOutput;

    assert!(BWRAP_UNAVAILABLE_MESSAGE.starts_with("bubblewrap is unavailable"));
    assert!(BWRAP_UNAVAILABLE_MESSAGE.contains("sudo apt install bubblewrap"));
    assert!(BWRAP_UNAVAILABLE_MESSAGE.contains("did not run"));
    let output = ExecToolCallOutput {
        exit_code: 1,
        stderr: StreamOutput::new(BWRAP_UNAVAILABLE_MESSAGE.to_string()),
        aggregated_output: StreamOutput::new(BWRAP_UNAVAILABLE_MESSAGE.to_string()),
        ..Default::default()
    };
    assert!(crate::is_likely_sandbox_denied(
        crate::SandboxType::LinuxSeccomp,
        &output
    ));
}

#[test]
fn finds_package_layout_bwrap_from_install_context() {
    use codex_install_context::CodexPackageLayout;
    use codex_install_context::InstallMethod;

    let temp_dir = tempdir().expect("temp dir");
    let package_dir = temp_dir.path();
    let bin_dir = package_dir.join("bin");
    let resources_dir = package_dir.join("codex-resources");
    let expected_bwrap = resources_dir.join("bwrap");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    write_executable(&expected_bwrap);

    let context = InstallContext {
        method: InstallMethod::Other,
        package_layout: Some(CodexPackageLayout {
            package_dir: AbsolutePathBuf::from_absolute_path(package_dir).expect("absolute"),
            bin_dir: AbsolutePathBuf::from_absolute_path(&bin_dir).expect("absolute"),
            resources_dir: Some(
                AbsolutePathBuf::from_absolute_path(&resources_dir).expect("absolute"),
            ),
            path_dir: None,
        }),
    };

    assert_eq!(
        find_bundled_bwrap_for_install_context(&context),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn finds_legacy_standalone_bundled_bwrap_next_to_exe_resources() {
    let temp_dir = tempdir().expect("temp dir");
    let exe = temp_dir.path().join("codex");
    let expected_bwrap = temp_dir.path().join("codex-resources").join("bwrap");
    write_executable(&exe);
    write_executable(&expected_bwrap);

    assert_eq!(
        find_legacy_bundled_bwrap_for_exe(&exe),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn finds_npm_bundled_bwrap_next_to_target_vendor_dir() {
    let temp_dir = tempdir().expect("temp dir");
    let target_dir = temp_dir.path().join("vendor/x86_64-unknown-linux-musl");
    let exe = target_dir.join("codex").join("codex");
    let expected_bwrap = target_dir.join("codex-resources").join("bwrap");
    write_executable(&exe);
    write_executable(&expected_bwrap);

    assert_eq!(
        find_legacy_bundled_bwrap_for_exe(&exe),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn finds_adjacent_dev_bwrap() {
    let temp_dir = tempdir().expect("temp dir");
    let exe = temp_dir.path().join("codex");
    let expected_bwrap = temp_dir.path().join("bwrap");
    write_executable(&exe);
    write_executable(&expected_bwrap);

    assert_eq!(
        find_legacy_bundled_bwrap_for_exe(&exe),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

/// Issue #563: an executable with nothing next to it has no bundled copy.
#[test]
fn finds_no_bundled_bwrap_next_to_a_lone_exe() {
    let temp_dir = tempdir().expect("temp dir");
    let exe = temp_dir.path().join("bin").join("codex");
    write_executable(&exe);

    assert_eq!(find_legacy_bundled_bwrap_for_exe(&exe), None);
}

fn write_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, b"").expect("write executable");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("set executable permissions");
}

#[test]
fn system_bwrap_warning_reports_user_namespace_failures() {
    for failure in USER_NAMESPACE_FAILURES {
        let fake_bwrap = write_fake_bwrap(&format!(
            r#"#!/bin/sh
echo '{failure}' >&2
exit 1
"#
        ));
        let fake_bwrap_path: &Path = fake_bwrap.as_ref();

        assert_eq!(
            system_bwrap_warning_for_path(Some(fake_bwrap_path), || true, || true),
            Some(USER_NAMESPACE_WARNING.to_string()),
            "{failure}",
        );
    }
}

#[test]
fn system_bwrap_warning_skips_unrelated_bwrap_failures() {
    let fake_bwrap = write_fake_bwrap(
        r#"#!/bin/sh
echo 'bwrap: Unknown option --argv0' >&2
exit 1
"#,
    );
    let fake_bwrap_path: &Path = fake_bwrap.as_ref();

    assert_eq!(
        system_bwrap_warning_for_path(Some(fake_bwrap_path), || true, || true),
        None
    );
}

#[test]
fn system_bwrap_probe_times_out_without_reporting_a_warning() {
    let fake_bwrap = write_fake_bwrap(
        r#"#!/bin/sh
sleep 1
exit 0
"#,
    );
    let fake_bwrap_path: &Path = fake_bwrap.as_ref();
    let started_at = Instant::now();

    assert!(system_bwrap_has_user_namespace_access(
        fake_bwrap_path,
        Duration::from_millis(10),
    ));
    assert!(started_at.elapsed() < Duration::from_millis(500));
}

#[test]
fn system_bwrap_probe_does_not_wait_for_descendants_holding_stderr_open() {
    let fake_bwrap = write_fake_bwrap(
        r#"#!/bin/sh
echo 'No permissions to create a new namespace' >&2
sleep 1 &
exit 1
"#,
    );
    let fake_bwrap_path: &Path = fake_bwrap.as_ref();
    let started_at = Instant::now();

    assert!(!system_bwrap_has_user_namespace_access(
        fake_bwrap_path,
        Duration::from_millis(100),
    ));
    assert!(started_at.elapsed() < Duration::from_millis(500));
}

#[test]
fn detects_wsl1_proc_version_formats() {
    assert!(proc_version_indicates_wsl1(
        "Linux version 4.4.0-22621-Microsoft"
    ));
    assert!(proc_version_indicates_wsl1(
        "Linux version 5.15.0-microsoft-standard-WSL1"
    ));
    assert!(proc_version_indicates_wsl1(
        "Linux version 5.15.0-wsl-microsoft-standard-WSL1"
    ));
}

#[test]
fn does_not_treat_wsl2_or_native_linux_as_wsl1() {
    assert!(!proc_version_indicates_wsl1(
        "Linux version 6.6.87.2-microsoft-standard-WSL2"
    ));
    assert!(!proc_version_indicates_wsl1(
        "Linux version 6.6.87.2-wsl-microsoft-standard-WSL2"
    ));
    assert!(!proc_version_indicates_wsl1(
        "Linux version 4.19.104-microsoft-standard"
    ));
    assert!(!proc_version_indicates_wsl1(
        "Linux version 6.6.87.2-microsoft-standard-WSL3"
    ));
    assert!(!proc_version_indicates_wsl1("Linux version 6.8.0"));
}

#[test]
fn finds_first_executable_bwrap_in_joined_search_path() {
    let temp_dir = tempdir().expect("temp dir");
    let cwd = temp_dir.path().join("cwd:with-colon");
    let first_dir = temp_dir.path().join("first");
    let second_dir = temp_dir.path().join("second");
    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&first_dir).expect("create first dir");
    std::fs::create_dir_all(&second_dir).expect("create second dir");
    std::fs::write(first_dir.join("bwrap"), "not executable").expect("write non-executable bwrap");
    let expected_bwrap = write_named_fake_bwrap_in(&second_dir);
    for search_paths in [
        vec![first_dir, second_dir],
        vec![PathBuf::from("../first"), PathBuf::from("../second")],
    ] {
        let search_path = std::env::join_paths(search_paths).expect("join search path");
        assert_eq!(
            find_executable_in_search_paths(
                "bwrap",
                std::env::split_paths(&search_path),
                &cwd,
                &PermissionProfile::read_only().file_system_sandbox_policy(),
                &cwd,
            ),
            Some(expected_bwrap.clone())
        );
    }
}

#[test]
fn skips_workspace_local_bwrap_in_joined_search_path() {
    let temp_dir = tempdir().expect("temp dir");
    let cwd = temp_dir.path().join("cwd");
    let trusted_dir = temp_dir.path().join("trusted");
    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&trusted_dir).expect("create trusted dir");
    let _workspace_bwrap = write_named_fake_bwrap_in(&cwd);
    let expected_bwrap = write_named_fake_bwrap_in(&trusted_dir);
    let search_path = std::env::join_paths([cwd.clone(), trusted_dir]).expect("join search path");

    assert_eq!(
        find_executable_in_search_paths(
            "bwrap",
            std::env::split_paths(&search_path),
            &cwd,
            &PermissionProfile::read_only().file_system_sandbox_policy(),
            &cwd,
        ),
        Some(expected_bwrap)
    );
}

#[test]
fn root_cwd_does_not_hide_system_bwrap_candidates() {
    let temp_dir = tempdir().expect("temp dir");
    let bin_dir = temp_dir.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    let expected_bwrap = write_named_fake_bwrap_in(&bin_dir);
    let search_path = std::env::join_paths([bin_dir]).expect("join search path");

    assert_eq!(
        find_executable_in_search_paths(
            "bwrap",
            std::env::split_paths(&search_path),
            Path::new("/"),
            &PermissionProfile::read_only().file_system_sandbox_policy(),
            Path::new("/"),
        ),
        Some(expected_bwrap)
    );
    assert_eq!(
        find_executable_in_search_paths(
            "bwrap",
            std::env::split_paths(&search_path),
            Path::new("/"),
            &PermissionProfile::Disabled.file_system_sandbox_policy(),
            Path::new("/"),
        ),
        None,
    );
}

#[test]
fn skips_bwrap_in_writable_roots_outside_command_cwd() {
    let temp_dir = tempdir().expect("temp dir");
    let workspace = temp_dir.path().join("workspace");
    let cwd = workspace.join("service");
    let workspace_bin = workspace.join("bin");
    let extra_root = temp_dir.path().join("extra");
    let trusted_dir = temp_dir.path().join("trusted");
    for dir in [&cwd, &workspace_bin, &extra_root, &trusted_dir] {
        std::fs::create_dir_all(dir).expect("create directory");
    }
    write_named_fake_bwrap_in(&workspace_bin);
    write_named_fake_bwrap_in(&extra_root);
    let expected_bwrap = write_named_fake_bwrap_in(&trusted_dir);
    let writable_roots = vec![workspace, extra_root.clone()];
    let policy = PermissionProfile::read_only()
        .file_system_sandbox_policy()
        .with_additional_writable_roots(
            &cwd,
            &writable_roots
                .into_iter()
                .map(|path| AbsolutePathBuf::try_from(path).unwrap())
                .collect::<Vec<_>>(),
        );
    let search_paths = vec![workspace_bin, extra_root];

    assert_eq!(
        find_executable_in_search_paths("bwrap", search_paths.clone(), &cwd, &policy, &cwd),
        None,
    );
    assert_eq!(
        find_executable_in_search_paths(
            "bwrap",
            search_paths.into_iter().chain([trusted_dir]),
            &cwd,
            &policy,
            &cwd,
        ),
        Some(expected_bwrap),
    );
}

#[test]
fn skips_bwrap_in_symlinked_writable_root() {
    let temp_dir = tempdir().expect("temp dir");
    let workspace = temp_dir.path().join("workspace");
    let cwd = workspace.join("service");
    let workspace_bin = workspace.join("bin");
    let alias = temp_dir.path().join("alias");
    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&workspace_bin).expect("create bin");
    std::os::unix::fs::symlink(&workspace, &alias).expect("create workspace alias");
    write_named_fake_bwrap_in(&workspace_bin);
    let policy = PermissionProfile::read_only()
        .file_system_sandbox_policy()
        .with_additional_writable_roots(&cwd, &[AbsolutePathBuf::try_from(alias).unwrap()]);

    assert_eq!(
        find_executable_in_search_paths("bwrap", [workspace_bin], &cwd, &policy, &cwd),
        None,
    );
}

#[test]
fn skips_writable_bwrap_symlink_to_read_only_target() {
    let temp = tempdir().unwrap();
    let bin = temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let system_binary = std::fs::canonicalize("/bin/true").unwrap();
    std::os::unix::fs::symlink(&system_binary, bin.join("bwrap")).unwrap();
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
        AbsolutePathBuf::try_from(bin.as_path()).unwrap().into(),
        FileSystemAccessMode::Write,
    )]);

    assert_eq!(
        find_executable_in_search_paths("bwrap", [bin], Path::new("/"), &policy, Path::new("/")),
        None,
    );
}

#[test]
fn skips_bwrap_through_symlinked_path_directory_into_writable_root() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let bin = workspace.join("bin");
    let alias = temp.path().join("alias");
    std::fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(&workspace, &alias).unwrap();
    let system_binary = std::fs::canonicalize("/bin/true").unwrap();
    std::os::unix::fs::symlink(&system_binary, bin.join("bwrap")).unwrap();
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
        AbsolutePathBuf::try_from(workspace.as_path())
            .unwrap()
            .into(),
        FileSystemAccessMode::Write,
    )]);

    assert_eq!(
        find_executable_in_search_paths(
            "bwrap",
            [alias.join("bin")],
            Path::new("/"),
            &policy,
            Path::new("/"),
        ),
        None,
    );
}

#[test]
fn full_disk_write_rejects_owned_bwrap_even_when_chmod_read_only() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let bin = temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let bwrap = write_named_fake_bwrap_in(&bin);
    std::fs::set_permissions(&bwrap, std::fs::Permissions::from_mode(0o555)).unwrap();
    let policy = PermissionProfile::Disabled.file_system_sandbox_policy();

    assert_eq!(
        find_executable_in_search_paths("bwrap", [bin], Path::new("/"), &policy, Path::new("/")),
        None,
    );
}

#[test]
fn read_only_bwrap_carveout_does_not_trust_a_replaceable_parent() {
    let temp = tempdir().unwrap();
    let bin = temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let bwrap = write_named_fake_bwrap_in(&bin);
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            AbsolutePathBuf::try_from(temp.path()).unwrap().into(),
            FileSystemAccessMode::Write,
        ),
        FileSystemSandboxEntry::new(
            AbsolutePathBuf::try_from(bwrap).unwrap().into(),
            FileSystemAccessMode::Read,
        ),
    ]);

    assert_eq!(
        find_executable_in_search_paths("bwrap", [bin], Path::new("/"), &policy, Path::new("/")),
        None,
    );
}

#[test]
fn root_write_preserves_a_read_only_system_installation() {
    use codex_protocol::permissions::FileSystemPath;
    use codex_protocol::permissions::FileSystemSpecialPath;

    // Use a system executable as a discovery-only candidate. Never execute it.
    let system_binary = std::fs::canonicalize("/bin/true").unwrap();
    let system_program = system_binary.file_name().unwrap().to_str().unwrap();
    let system_search_path = system_binary.parent().unwrap().to_path_buf();
    let protected_root = system_binary
        .ancestors()
        .find(|path| path.parent() == Some(Path::new("/")))
        .unwrap();
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        ),
        FileSystemSandboxEntry::new(
            AbsolutePathBuf::try_from(protected_root).unwrap().into(),
            FileSystemAccessMode::Read,
        ),
    ]);

    for policy in [
        policy,
        PermissionProfile::Disabled.file_system_sandbox_policy(),
    ] {
        // Root can modify the system installation unless the policy protects it.
        // SAFETY: geteuid has no preconditions.
        let can_modify_system =
            policy.has_full_disk_write_access() && unsafe { libc::geteuid() } == 0;
        assert_eq!(
            find_executable_in_search_paths(
                system_program,
                [system_search_path.clone()],
                Path::new("/"),
                &policy,
                Path::new("/"),
            ),
            (!can_modify_system).then(|| system_binary.clone()),
        );
    }
}

fn write_fake_bwrap(contents: &str) -> tempfile::TempPath {
    write_fake_bwrap_in(
        &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        contents,
    )
}

fn write_fake_bwrap_in(dir: &Path, contents: &str) -> tempfile::TempPath {
    use tempfile::NamedTempFile;

    // Bazel can mount the OS temp directory `noexec`, so prefer the current
    // working directory for fake executables and fall back to the default temp
    // dir outside that environment.
    let temp_file = NamedTempFile::new_in(dir)
        .ok()
        .unwrap_or_else(|| NamedTempFile::new().expect("temp file"));
    let path = temp_file.into_temp_path();
    // A sibling spawn may retain the temporary file's writable descriptor.
    // Remove that inode before the helper creates the executable in its child.
    std::fs::remove_file(&path).expect("remove empty temporary file");
    codex_utils_cargo_bin::write_executable(&path, contents).expect("write fake bwrap");
    path
}

fn write_named_fake_bwrap_in(dir: &Path) -> PathBuf {
    use std::fs;

    let path = dir.join("bwrap");
    codex_utils_cargo_bin::write_executable(&path, "#!/bin/sh\n").expect("write fake bwrap");
    fs::canonicalize(path).expect("canonicalize fake bwrap")
}

/// #635: the probe runs a file that another process still holds open for
/// writing once the writer closes it (here this test holds it; in a test
/// binary, a child forked by another thread does), instead of reading the
/// `ETXTBSY` spawn error as "has access".
#[test]
fn system_bwrap_probe_waits_while_the_file_is_open_for_writing() {
    let fake_bwrap = write_fake_bwrap(
        r#"#!/bin/sh
echo 'No permissions to create a new namespace' >&2
exit 1
"#,
    );
    let fake_bwrap_path: &Path = fake_bwrap.as_ref();
    let writer = std::fs::OpenOptions::new()
        .append(true)
        .open(fake_bwrap_path)
        .expect("open fake bwrap for writing");
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        drop(writer);
    });

    assert!(!system_bwrap_has_user_namespace_access(
        fake_bwrap_path,
        SYSTEM_BWRAP_PROBE_TIMEOUT,
    ));
    release.join().expect("writer thread");
}
