use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsString;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::fd::RawFd;
use std::os::raw::c_char;
use std::os::unix::fs::FileExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::OnceLock;

use crate::bazel_bwrap;
use crate::exec_util::argv_to_cstrings;
use crate::exec_util::make_files_inheritable;
use codex_utils_absolute_path::AbsolutePathBuf;
use sha2::Digest as _;
use sha2::Sha256;

const SHA256_HEX_LEN: usize = 64;
const NULL_SHA256_DIGEST: [u8; 32] = [0; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BundledBwrapLauncher {
    program: AbsolutePathBuf,
    /// #581: where AppArmor restricts user namespaces (Ubuntu 24.04), the
    /// copy runs through `aa-exec -p rootlesskit`, from its verified
    /// descriptor.
    apparmor: Option<AppArmorLauncher>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppArmorLauncher {
    aa_exec: AbsolutePathBuf,
    /// The digest-verified copy's descriptor (`VERIFIED_BWRAP`), open
    /// (close-on-exec) for the helper's life.
    fd: RawFd,
}

/// The verified copy the AppArmor launcher runs (see [`AppArmorLauncher`]).
static VERIFIED_BWRAP: OnceLock<File> = OnceLock::new();

pub(crate) fn launcher() -> Option<BundledBwrapLauncher> {
<<<<<<< HEAD
    codex_sandboxing::find_bundled_bwrap()
        .or_else(bazel_bwrap_candidate)
        .map(|program| BundledBwrapLauncher {
            apparmor: apparmor_launcher(&program),
            program,
        })
}

/// #581: probes the bundled copy as the system one is probed
/// (`find_apparmor_bwrap_launcher`), but only once its digest is verified,
/// and through that verified descriptor: `aa-exec` runs
/// `/proc/self/fd/<fd>`, never the path, so no other file can take its
/// place. `None` when the copy creates user namespaces by itself (or cannot
/// be verified: the run then fails as before).
fn apparmor_launcher(program: &AbsolutePathBuf) -> Option<AppArmorLauncher> {
    apparmor_launcher_with(
        program,
        expected_sha256(),
        codex_sandboxing::find_apparmor_bwrap_launcher_for_fd,
    )
}

fn apparmor_launcher_with(
    program: &AbsolutePathBuf,
    expected: Option<[u8; 32]>,
    probe: impl FnOnce(RawFd) -> Option<std::path::PathBuf>,
) -> Option<AppArmorLauncher> {
    let file = File::open(program.as_path()).ok()?;
    // Above the standard streams: with one of them closed when the helper
    // started, the copy could otherwise become the command's stdin, stdout
    // or stderr.
    if file.as_raw_fd() <= libc::STDERR_FILENO {
        return None;
    }
    verify_digest(&file, expected, program.as_path()).ok()?;
    let aa_exec = probe(file.as_raw_fd())?;
    let aa_exec = AbsolutePathBuf::from_absolute_path(aa_exec).ok()?;
    let fd = file.as_raw_fd();
    VERIFIED_BWRAP.set(file).ok()?;
    Some(AppArmorLauncher { aa_exec, fd })
}

/// The bubblewrap a Bazel test run provides (debug builds only).
fn bazel_bwrap_candidate() -> Option<AbsolutePathBuf> {
    let path = bazel_bwrap::candidate()?;
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .then(|| AbsolutePathBuf::from_absolute_path(&path).ok())
        .flatten()
||||||| 413492cd6c
    let current_exe = std::env::current_exe().ok()?;
    find_for_install_context(InstallContext::current())
        .or_else(|| find_legacy_for_exe(&current_exe))
        .map(|program| BundledBwrapLauncher { program })
=======
    let current_exe = std::env::current_exe().ok()?;
    find_program(
        InstallContext::current(),
        &current_exe,
        Path::new("."),
        |key| std::env::var_os(key),
    )
    .map(|program| BundledBwrapLauncher { program })
}

/// Discover the bundled fallback for an explicit launcher and its environment.
pub fn find_bundled_bwrap_for_exe(
    exe: &Path,
    command_cwd: &Path,
    env: impl Fn(&str) -> Option<OsString>,
) -> Option<AbsolutePathBuf> {
    let exe = std::fs::canonicalize(exe).ok()?;
    let context = InstallContext::from_exe(
        /*is_macos*/ false,
        Some(&exe),
        /*method_override*/ None,
    );
    find_program(&context, &exe, command_cwd, env)
}

fn find_program(
    context: &InstallContext,
    exe: &Path,
    cwd: &Path,
    env: impl Fn(&str) -> Option<OsString>,
) -> Option<AbsolutePathBuf> {
    find_for_install_context(context)
        .or_else(|| find_legacy_for_exe(exe, bazel_bwrap::candidate(cwd, env)))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

impl BundledBwrapLauncher {
    /// #581: the descriptor of the verified copy that `aa-exec` must pass on
    /// to bubblewrap, which then passes it on to the command. The sandbox's
    /// inner stage closes it before running the command
    /// (`--close-inherited-fd`), so the command cannot reopen the copy
    /// through `/proc/self/fd`.
    pub(crate) fn inherited_fd(&self) -> Option<RawFd> {
        self.apparmor.as_ref().map(|apparmor| apparmor.fd)
    }

    pub(crate) fn exec(&self, argv: Vec<String>, preserved_files: Vec<File>) -> ! {
        if let Some(apparmor) = &self.apparmor {
            exec_through_apparmor(apparmor, self.program.as_path(), argv, preserved_files);
        }
        let bwrap_file = File::open(self.program.as_path()).unwrap_or_else(|err| {
            panic!(
                "failed to open bundled bubblewrap {}: {err}",
                self.program.as_path().display()
            )
        });
        if let Err(err) = verify_digest(&bwrap_file, expected_sha256(), self.program.as_path()) {
            eprintln!("{err}");
            std::process::exit(crate::BUNDLED_BWRAP_DIGEST_VERIFICATION_FAILURE_EXIT_CODE);
        }

        make_files_inheritable(&preserved_files);

        let fd_path = format!("/proc/self/fd/{}", bwrap_file.as_raw_fd());
        let program_cstring = CString::new(fd_path.as_str())
            .unwrap_or_else(|err| panic!("invalid bundled bubblewrap fd path: {err}"));
        let cstrings = argv_to_cstrings(&argv);
        let mut argv_ptrs: Vec<*const c_char> = cstrings
            .iter()
            .map(CString::as_c_str)
            .map(CStr::as_ptr)
            .collect();
        argv_ptrs.push(std::ptr::null());

        // SAFETY: `program_cstring` and every entry in `argv_ptrs` are valid C
        // strings for the duration of the call. On success `execv` does not return.
        unsafe {
            libc::execv(program_cstring.as_ptr(), argv_ptrs.as_ptr());
        }
        let err = std::io::Error::last_os_error();
        panic!(
            "failed to exec bundled bubblewrap {} via {fd_path}: {err}",
            self.program.as_path().display()
        );
    }
}

<<<<<<< HEAD
/// `aa-exec -p rootlesskit -- /proc/self/fd/<fd> <bwrap args>`: the
/// verified copy, made inheritable for this exec only.
fn exec_through_apparmor(
    apparmor: &AppArmorLauncher,
    program: &Path,
    argv: Vec<String>,
    preserved_files: Vec<File>,
) -> ! {
    // Verified again right before it runs (the probes, the `/proc` check and
    // the setup ran since): what `aa-exec` starts is this file as hashed now.
    let file = VERIFIED_BWRAP
        .get()
        .unwrap_or_else(|| panic!("the verified bundled bubblewrap is not open"));
    verify_digest(file, expected_sha256(), program).unwrap_or_else(|err| panic!("{err}"));
    make_files_inheritable(&preserved_files);
    // SAFETY: clears close-on-exec on a descriptor this process keeps open
    // (`VERIFIED_BWRAP`).
    if unsafe { libc::fcntl(apparmor.fd, libc::F_SETFD, 0) } < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to pass the bundled bubblewrap to aa-exec: {err}");
||||||| 413492cd6c
fn find_for_install_context(context: &InstallContext) -> Option<AbsolutePathBuf> {
    context
        .bundled_resource("bwrap")
        .filter(|path| is_executable_file(path))
}

fn find_legacy_for_exe(exe: &Path) -> Option<AbsolutePathBuf> {
    legacy_candidates_for_exe(exe)
        .into_iter()
        .find(|candidate| is_executable_file(candidate))
        .map(|path| {
            AbsolutePathBuf::from_absolute_path(&path).unwrap_or_else(|err| {
                panic!(
                    "failed to normalize bundled bubblewrap path {}: {err}",
                    path.display()
                )
            })
        })
}

fn legacy_candidates_for_exe(exe: &Path) -> Vec<PathBuf> {
    let Some(exe_dir) = exe.parent() else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    candidates.push(exe_dir.join("codex-resources").join("bwrap"));
    if let Some(package_target_dir) = exe_dir.parent() {
        candidates.push(package_target_dir.join("codex-resources").join("bwrap"));
=======
fn find_for_install_context(context: &InstallContext) -> Option<AbsolutePathBuf> {
    context
        .bundled_resource("bwrap")
        .filter(|path| is_executable_file(path))
}

fn find_legacy_for_exe(exe: &Path, bazel_candidate: Option<PathBuf>) -> Option<AbsolutePathBuf> {
    legacy_candidates_for_exe(exe, bazel_candidate)
        .into_iter()
        .find(|candidate| is_executable_file(candidate))
        .map(|path| {
            AbsolutePathBuf::from_absolute_path(&path).unwrap_or_else(|err| {
                panic!(
                    "failed to normalize bundled bubblewrap path {}: {err}",
                    path.display()
                )
            })
        })
}

fn legacy_candidates_for_exe(exe: &Path, bazel_candidate: Option<PathBuf>) -> Vec<PathBuf> {
    let Some(exe_dir) = exe.parent() else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    candidates.push(exe_dir.join("codex-resources").join("bwrap"));
    if let Some(package_target_dir) = exe_dir.parent() {
        candidates.push(package_target_dir.join("codex-resources").join("bwrap"));
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
<<<<<<< HEAD
    let aa_exec = CString::new(apparmor.aa_exec.as_path().as_os_str().as_encoded_bytes())
        .unwrap_or_else(|err| panic!("invalid aa-exec path: {err}"));
    let mut wrapped = vec![
        "aa-exec".to_string(),
        "-p".to_string(),
        codex_sandboxing::APPARMOR_BWRAP_USERNS_PROFILE.to_string(),
        "--".to_string(),
        format!("/proc/self/fd/{}", apparmor.fd),
    ];
    wrapped.extend(argv.into_iter().skip(1));
    let cstrings = argv_to_cstrings(&wrapped);
    let mut argv_ptrs: Vec<*const c_char> = cstrings
        .iter()
        .map(CString::as_c_str)
        .map(CStr::as_ptr)
        .collect();
    argv_ptrs.push(std::ptr::null());
    // SAFETY: `aa_exec` and every entry in `argv_ptrs` are valid C strings
    // for the duration of the call. On success `execv` does not return.
    unsafe {
        libc::execv(aa_exec.as_ptr(), argv_ptrs.as_ptr());
||||||| 413492cd6c
    candidates.push(exe_dir.join("bwrap"));
    if let Some(path) = bazel_bwrap::candidate() {
        candidates.push(path);
=======
    candidates.push(exe_dir.join("bwrap"));
    if let Some(path) = bazel_candidate {
        candidates.push(path);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
    let err = std::io::Error::last_os_error();
    panic!(
        "failed to exec AppArmor launcher {} for bundled bubblewrap: {err}",
        apparmor.aa_exec.as_path().display()
    );
}

fn expected_sha256() -> Option<[u8; 32]> {
    static EXPECTED: OnceLock<Option<[u8; 32]>> = OnceLock::new();
    *EXPECTED.get_or_init(|| {
        let raw_digest = option_env!("CODEX_BWRAP_SHA256")?;
        let digest = parse_sha256_hex(raw_digest)
            .unwrap_or_else(|err| panic!("invalid CODEX_BWRAP_SHA256 value: {err}"));
        (digest != NULL_SHA256_DIGEST).then_some(digest)
    })
}

fn verify_digest(file: &File, expected: Option<[u8; 32]>, path: &Path) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };

    // Read by offset: the descriptor may be verified again (#581) and its
    // shared file offset is left as it is.
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    let mut offset = 0_u64;
    loop {
        let read = file.read_at(&mut buffer, offset).map_err(|err| {
            format!(
                "failed to read bundled bubblewrap {} for digest verification: {err}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        offset += read as u64;
    }

    let actual: [u8; 32] = hasher.finalize().into();
    if actual == expected {
        return Ok(());
    }

    Err(format!(
        "bundled bubblewrap digest mismatch for {}: expected sha256:{}, got sha256:{}",
        path.display(),
        bytes_to_hex(&expected),
        bytes_to_hex(&actual),
    ))
}

fn parse_sha256_hex(raw: &str) -> Result<[u8; 32], String> {
    if raw.len() != SHA256_HEX_LEN {
        return Err(format!(
            "expected {SHA256_HEX_LEN} hex characters, got {}",
            raw.len()
        ));
    }

    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&raw[start..start + 2], 16)
            .map_err(|err| format!("invalid hex byte at offset {start}: {err}"))?;
    }
    Ok(digest)
}

fn bytes_to_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(SHA256_HEX_LEN);
    for byte in bytes {
        hex.push(HEX[(byte >> 4) as usize] as char);
        hex.push(HEX[(byte & 0x0f) as usize] as char);
    }
    hex
}

#[cfg(test)]
<<<<<<< HEAD
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn digest_verification_skips_missing_expected_digest() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");

        verify_digest(file.as_file(), /*expected*/ None, file.path())
            .expect("missing digest should skip verification");
    }

    #[test]
    fn digest_verification_accepts_matching_digest() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");
        let expected: [u8; 32] = Sha256::digest(b"contents").into();

        verify_digest(file.as_file(), Some(expected), file.path())
            .expect("matching digest should verify");
    }

    #[test]
    fn digest_verification_rejects_mismatched_digest() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");

        let err = verify_digest(file.as_file(), Some([0xab; 32]), file.path())
            .expect_err("mismatched digest should fail");
        assert!(err.contains("bundled bubblewrap digest mismatch"));
    }

    /// #581: a copy whose digest does not match is never probed (so never
    /// run); one that matches is probed through its descriptor.
    #[test]
    fn a_copy_that_fails_verification_is_never_probed() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");
        let program = AbsolutePathBuf::from_absolute_path(file.path()).expect("absolute");
        let mut probed = false;
        let launcher = apparmor_launcher_with(&program, Some([0xab; 32]), |_| {
            probed = true;
            None
        });
        assert_eq!(launcher, None);
        assert!(!probed);
        let expected: [u8; 32] = Sha256::digest(b"contents").into();
        let mut probed_fd = None;
        apparmor_launcher_with(&program, Some(expected), |fd| {
            probed_fd = Some(fd);
            None
        });
        assert!(probed_fd.is_some_and(|fd| fd > libc::STDERR_FILENO));
    }

    /// The digest is read by offset, so verifying a descriptor twice gives
    /// the same answer (#581 verifies again right before `aa-exec`).
    #[test]
    fn digest_verification_can_repeat_on_one_descriptor() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");
        let expected: [u8; 32] = Sha256::digest(b"contents").into();
        for _ in 0..2 {
            verify_digest(file.as_file(), Some(expected), file.path()).expect("verifies");
        }
    }

    #[test]
    fn parses_sha256_hex_digest() {
        assert_eq!(parse_sha256_hex(&"ab".repeat(32)), Ok([0xab; 32]));
        assert_eq!(parse_sha256_hex(&"00".repeat(32)), Ok(NULL_SHA256_DIGEST));
        assert!(parse_sha256_hex("ab").is_err());
        assert!(parse_sha256_hex(&format!("{}xx", "00".repeat(31))).is_err());
    }
}
||||||| 413492cd6c
mod tests {
    use super::*;
    use codex_install_context::CodexPackageLayout;
    use codex_install_context::InstallContext;
    use codex_install_context::InstallMethod;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::NamedTempFile;
    use tempfile::tempdir;

    #[test]
    fn finds_package_layout_bwrap_from_install_context() {
        let temp_dir = tempdir().expect("temp dir");
        let package_dir = temp_dir.path();
        let bin_dir = package_dir.join("bin");
        let resources_dir = package_dir.join("codex-resources");
        let expected_bwrap = resources_dir.join("bwrap");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
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
            find_for_install_context(&context),
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
            find_legacy_for_exe(&exe),
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
            find_legacy_for_exe(&exe),
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
            find_legacy_for_exe(&exe),
            Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
        );
    }

    #[test]
    fn digest_verification_skips_missing_expected_digest() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");

        verify_digest(file.as_file(), /*expected*/ None, file.path())
            .expect("missing digest should skip verification");
    }

    #[test]
    fn digest_verification_accepts_matching_digest() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");
        let expected: [u8; 32] = Sha256::digest(b"contents").into();

        verify_digest(file.as_file(), Some(expected), file.path())
            .expect("matching digest should verify");
    }

    #[test]
    fn digest_verification_rejects_mismatched_digest() {
        let file = NamedTempFile::new().expect("temp file");
        fs::write(file.path(), b"contents").expect("write file");

        let err = verify_digest(file.as_file(), Some([0xab; 32]), file.path())
            .expect_err("mismatched digest should fail");
        assert!(err.contains("bundled bubblewrap digest mismatch"));
    }

    #[test]
    fn parses_sha256_hex_digest() {
        assert_eq!(parse_sha256_hex(&"ab".repeat(32)), Ok([0xab; 32]));
        assert_eq!(parse_sha256_hex(&"00".repeat(32)), Ok(NULL_SHA256_DIGEST));
        assert!(parse_sha256_hex("ab").is_err());
        assert!(parse_sha256_hex(&format!("{}xx", "00".repeat(31))).is_err());
    }

    fn write_executable(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(path, b"").expect("write executable");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .expect("set executable permissions");
    }
}
=======
#[path = "bundled_bwrap_tests.rs"]
mod tests;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
