use super::*;
use pretty_assertions::assert_eq;

#[test]
fn pf_27_s02_broker_profile_denies_exec_and_confines_writes() {
    let dir = tempfile_dir();
    let profile = broker_seatbelt_profile(&dir).expect("profile");
    assert!(profile.contains("(deny process-exec*)"));
    assert!(profile.contains("(deny process-fork)"));
    assert!(profile.contains("(deny signal (target others))"));
    assert!(profile.contains(&format!("(require-not (subpath \"{}\"))", dir.display())));
}

#[test]
fn pf_27_s02_broker_profile_rejects_unquotable_paths() {
    assert_eq!(broker_seatbelt_profile(Path::new("/tmp/with\"quote")), None);
    assert_eq!(broker_seatbelt_profile(Path::new("relative/dir")), None);
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("pf27-s02-profile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    dir
}

const CHILD_ENV: &str = "CODEX_PF27_S02_CONTAINMENT_CHILD";
const CHILD_TEST: &str = "broker_containment::tests::pf_27_s02_containment_child_entry";

/// Runs inside a re-executed test binary: contains itself, then probes.
#[test]
fn pf_27_s02_containment_child_entry() {
    let Some(dir) = std::env::var_os(CHILD_ENV) else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let containment = contain_credential_broker(&dir);
    let exec = match std::process::Command::new("/bin/echo")
        .arg("escaped")
        .output()
    {
        Ok(output) if output.status.success() => "allowed",
        _ => "denied",
    };
    let inside = match std::fs::write(dir.join("inside.txt"), "ok") {
        Ok(()) => "ok",
        Err(_) => "denied",
    };
    let outside_dir = dir.parent().expect("parent").join("outside-pf27-s02");
    let outside = match std::fs::write(&outside_dir, "escaped") {
        Ok(()) => "allowed",
        Err(_) => "denied",
    };
    println!(
        "PF27S02 mechanism={} exec={exec} inside={inside} outside={outside}",
        containment.mechanism
    );
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn pf_27_s02_contained_broker_cannot_exec_or_write_outside_its_dir() {
    let root = std::env::temp_dir().join(format!("pf27-s02-contain-{}", std::process::id()));
    let dir = root.join("run");
    std::fs::create_dir_all(&dir).expect("dir");
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, &dir)
        .output()
        .expect("child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find_map(|line| line.find("PF27S02 ").map(|start| &line[start..]))
        .unwrap_or_else(|| panic!("no probe line: {stdout}"));
    if cfg!(target_os = "macos") {
        assert_eq!(
            line,
            "PF27S02 mechanism=seatbelt exec=denied inside=ok outside=denied"
        );
    } else if line.contains("landlock") {
        assert_eq!(
            line,
            "PF27S02 mechanism=landlock+seccomp exec=denied inside=ok outside=denied"
        );
    } else {
        // Kernels without Landlock still get the seccomp layer.
        assert!(
            line.starts_with("PF27S02 mechanism=seccomp exec=denied inside=ok"),
            "{line}"
        );
        return;
    }
    assert!(!root.join("outside-pf27-s02").exists());
}
