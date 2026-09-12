use super::*;
use pretty_assertions::assert_eq;
use rustix::fs::{MemfdFlags, fcntl_add_seals, memfd_create};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

fn image(bytes: &[u8]) -> OwnedFd {
    let mut file = File::from(memfd_create(c"pf27-os-test", MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING).unwrap());
    file.write_all(bytes).unwrap();
    fcntl_add_seals(&file, SealFlags::WRITE | SealFlags::SHRINK | SealFlags::GROW | SealFlags::SEAL).unwrap();
    file.into()
}

fn artifact(key: &str) -> OwnedFd {
    // Explicit ignored OS tests require coordinator-hashed, static synthetic
    // artifacts. Missing prerequisites fail, never silently skip a claimed run.
    image(&std::fs::read(std::env::var_os(key).expect("synthetic artifact path required")).unwrap())
}

fn wait(child: &mut OwnedChild) -> WaitIdStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() { return status; }
        assert!(Instant::now() < deadline, "child did not exit");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn fd_count() -> usize { std::fs::read_dir("/proc/self/fd").unwrap().count() }

#[test]
fn pf27_rejects_unsealed_fd_without_spawning() {
    let image = memfd_create(c"unsealed", MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING).unwrap();
    assert_eq!(launch(image, &[c"test"]).err().unwrap().kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn pf27_rejects_non_cloexec_image() {
    let image = image(b"invalid executable");
    rustix::fs::fcntl_setfd(&image, FdFlags::empty()).unwrap();
    assert_eq!(launch(image, &[c"test"]).err().unwrap().kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn pf27_rejects_unbounded_fd_actions() {
    let image = image(b"invalid executable");
    let high = rustix::io::fcntl_dupfd_cloexec(&image, 1025).unwrap();
    assert_eq!(launch(high, &[c"test"]).err().unwrap().kind(), io::ErrorKind::InvalidInput);
}

#[test]
#[ignore = "explicit source-qualified GNU 2.43 OS proof"]
fn pf27_enoexec_has_no_shell_fallback_or_fd_leak() {
    let before = fd_count();
    for _ in 0..32 {
        assert_eq!(launch(image(b"exit 42\n"), &[c"test"]).err().unwrap().raw_os_error(), Some(libc::ENOEXEC));
    }
    assert_eq!(fd_count(), before);
    assert_eq!(waitid(WaitId::All, WaitIdOptions::EXITED | WaitIdOptions::NOHANG).err(), Some(rustix::io::Errno::CHILD));
}

#[test]
#[ignore = "explicit unsupported-libc host proof"]
fn pf27_unqualified_libc_fails_closed() {
    assert_eq!(launch(image(b"exit 42\n"), &[c"test"]).err().unwrap().kind(), io::ErrorKind::Unsupported);
}

#[test]
#[ignore = "requires hashed static synthetic inspection artifact"]
fn pf27_exact_environment_cwd_stdio_and_fd_allowlist() {
    // Deliberately hold an inheritable extra fd: child actions must close it.
    let inherited = File::open("/dev/null").unwrap();
    rustix::fs::fcntl_setfd(&inherited, FdFlags::empty()).unwrap();
    let mut child = launch(artifact("PF27_SYNTHETIC_INSPECT"), &[c"probe-test"]).unwrap();
    assert_eq!(wait(&mut child).exit_status(), Some(0));
    assert_eq!(child.try_wait().unwrap().unwrap().exit_status(), Some(0));
}

#[test]
#[ignore = "requires hashed static synthetic hold artifact"]
fn pf27_live_pidfd_stop_and_drop_reap() {
    let before = fd_count();
    let mut child = launch(artifact("PF27_SYNTHETIC_HOLD"), &[c"probe-test"]).unwrap();
    assert!(child.try_wait().unwrap().is_none());
    child.terminate().unwrap();
    assert_eq!(wait(&mut child).terminating_signal(), Some(libc::SIGKILL));
    drop(child);
    drop(launch(artifact("PF27_SYNTHETIC_HOLD"), &[c"probe-test"]).unwrap());
    assert_eq!(fd_count(), before);
    assert_eq!(waitid(WaitId::All, WaitIdOptions::EXITED | WaitIdOptions::NOHANG).err(), Some(rustix::io::Errno::CHILD));
}

#[test]
#[ignore = "requires hashed existing static probe"]
fn pf27_all_fixed_recipes_deny_non_root_before_setters() {
    for role in [SyntheticRole::Journal, SyntheticRole::Policy, SyntheticRole::Worker] {
        let mut child = spawn_synthetic_probe(artifact("PF27_SYNTHETIC_PROBE"), role).unwrap();
        assert_eq!(wait(&mut child).exit_status(), Some(78));
    }
}

#[test]
#[ignore = "requires hashed static synthetic inspection artifact"]
fn pf27_held_bytes_survive_original_path_replacement() {
    let source = std::env::var_os("PF27_SYNTHETIC_INSPECT").unwrap();
    let name = format!("pf27-replace-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("image");
    std::fs::copy(source, &path).unwrap();
    let held = image(&std::fs::read(&path).unwrap());
    std::fs::write(&path, b"exit 42\n").unwrap();
    let mut child = launch(held, &[c"probe-test"]).unwrap();
    assert_eq!(wait(&mut child).exit_status(), Some(0));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
