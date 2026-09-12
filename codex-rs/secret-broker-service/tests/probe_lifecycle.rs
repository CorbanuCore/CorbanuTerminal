#![cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
#![forbid(unsafe_code)]
// Fixture setup failures intentionally stop the test immediately.
#![allow(clippy::unwrap_used)]

use pretty_assertions::assert_eq;
use std::process::Command;

fn probe() -> std::path::PathBuf {
    codex_utils_cargo_bin::cargo_bin("codex-protected-root-probe").unwrap()
}

#[test]
fn pf_27_s01_probe_hardens_only_its_own_actual_process() {
    let before = (
        nix::sys::prctl::get_dumpable().unwrap(),
        nix::sys::prctl::get_no_new_privs().unwrap(),
        nix::unistd::getgroups().unwrap(),
    );
    // The build harness deliberately holds fd9. A clean-start fixture closes
    // inherited descriptors explicitly; a separate case passes one through.
    let output = Command::new("python3")
        .arg("-c")
        .arg("import subprocess,sys\nr=subprocess.run([sys.argv[1],'--inspect-post-exec'],close_fds=True,capture_output=True)\nsys.stdout.buffer.write(r.stdout);sys.stderr.buffer.write(r.stderr);sys.exit(r.returncode)")
        .arg(probe())
        .output()
        .expect("probe fixture requires python3 on PATH");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.len() < 512);
    for field in [
        "no_new_privileges",
        "nondumpable",
        "keepcaps_disabled",
        "capabilities_empty",
        "ambient_empty",
        "descriptor_allowlist",
    ] {
        assert!(text.contains(&format!("\"{field}\":true")), "{text}");
    }
    assert!(text.contains(&format!("\"supplementary_group_count\":{}", before.2.len())));
    assert!(text.contains("\"native_eligible\":false"));
    assert_eq!(
        before,
        (
            nix::sys::prctl::get_dumpable().unwrap(),
            nix::sys::prctl::get_no_new_privs().unwrap(),
            nix::unistd::getgroups().unwrap()
        )
    );
}

#[test]
fn pf_27_s01_probe_native_and_invalid_modes_remain_unavailable() {
    for args in [
        vec![],
        vec!["--open-existing"],
        vec!["--journal-child"],
        vec!["--policy-child"],
        vec!["--unknown"],
        vec!["--inspect-post-exec", "extra"],
        vec!["--inspect-post-exec", "--journal-child"],
    ] {
        let output = Command::new(probe()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(78));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("native launch is not implemented")
        );
    }
}

#[test]
fn pf_27_s01_probe_reports_inherited_synthetic_fd_without_disclosing_it() {
    // Python owns the fixture descriptor and passes just that descriptor into
    // exec. The child never reads or prints its contents/target pathname.
    let output = Command::new("python3").arg("-c").arg(
        "import os,subprocess,sys,tempfile\nwith tempfile.TemporaryFile() as f:\n f.write(b'PF27_SYNTHETIC_PRIVATE_BYTES'); f.flush()\n r=subprocess.run([sys.argv[1],'--inspect-post-exec'],pass_fds=(f.fileno(),),capture_output=True)\n sys.stdout.buffer.write(r.stdout); sys.stderr.buffer.write(r.stderr); sys.exit(r.returncode)"
    ).arg(probe()).output().expect("probe fixture requires python3 on PATH");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("\"descriptor_allowlist\":false"));
    assert!(text.contains("\"native_eligible\":false"));
    assert!(!text.contains("PF27_SYNTHETIC_PRIVATE_BYTES"));
    assert!(!text.contains("/tmp/"));
}

#[test]
fn pf_27_s01_preparation_denies_nonroot_and_invalid_invocations() {
    assert!(
        !nix::unistd::geteuid().is_root(),
        "no root-positive test authorized"
    );
    let before = (
        nix::unistd::getresuid().unwrap(),
        nix::unistd::getresgid().unwrap(),
        nix::unistd::getgroups().unwrap(),
    );
    for args in [
        vec!["--prepare-synthetic-child", "journal", "101", "201", "204"],
        vec!["--prepare-synthetic-child", "worker", "103", "203", "none"],
        vec!["--prepare-synthetic-child", "worker", "103", "203", "204"],
        vec!["--prepare-synthetic-child", "journal", "0", "201", "204"],
        vec![
            "--prepare-synthetic-child",
            "journal",
            "101",
            "201",
            "204",
            "extra",
        ],
        vec!["--prepare-synthetic-child", "--inspect-post-exec"],
    ] {
        let output = Command::new(probe()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(78));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "synthetic probe unavailable: post-exec inspection failed\n"
        );
    }
    assert_eq!(
        before,
        (
            nix::unistd::getresuid().unwrap(),
            nix::unistd::getresgid().unwrap(),
            nix::unistd::getgroups().unwrap()
        )
    );
}
