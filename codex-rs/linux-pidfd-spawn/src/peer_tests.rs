use super::*;
use pretty_assertions::assert_eq;
use rustix::process::Pid;
use rustix::process::PidfdFlags;
use rustix::process::getpid;
use rustix::process::pidfd_open;
use std::os::fd::AsFd;
use std::process::Child;
use std::process::Command;

struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn pf_27_s01_pidfs_identity_matches_process_not_descriptor_number() {
    let me = pidfd_open(getpid(), PidfdFlags::empty()).unwrap();
    let other_me = pidfd_open(getpid(), PidfdFlags::empty()).unwrap();
    let mut child = Reap(Command::new("/bin/sleep").arg("30").spawn().unwrap());
    let pid = Pid::from_raw(child.0.id() as i32).unwrap();
    let child_fd = pidfd_open(pid, PidfdFlags::empty()).unwrap();
    let same = same_process(me.as_fd(), other_me.as_fd());
    if !cfg!(target_pointer_width = "64") || fstatfs(me.as_fd()).unwrap().f_type != 0x5049_4446 {
        assert_eq!(same.unwrap_err().kind(), io::ErrorKind::Unsupported);
        return;
    }
    assert_eq!(same.unwrap(), true);
    assert_eq!(same_process(me.as_fd(), child_fd.as_fd()).unwrap(), false);
    let ordinary = std::fs::File::open("/dev/null").unwrap();
    assert_eq!(
        same_process(me.as_fd(), ordinary.as_fd())
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    // Retaining a dead peer still cannot alias an unrelated live process.
    assert_eq!(same_process(me.as_fd(), child_fd.as_fd()).unwrap(), false);
}

#[test]
#[ignore = "requires coordinator-hashed static hold on qualified GNU2.43"]
fn pf_27_s01_pidfs_reaped_owner_rejects_without_reinterpreting_a_peer() {
    use rustix::fs::MemfdFlags;
    use rustix::fs::SealFlags;
    use rustix::fs::fcntl_add_seals;
    use rustix::fs::memfd_create;
    use std::io::Write;
    use std::time::Duration;
    use std::time::Instant;
    let mut file = std::fs::File::from(
        memfd_create("pf27-peer", MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING).unwrap(),
    );
    file.write_all(&std::fs::read(std::env::var_os("PF27_SYNTHETIC_HOLD").unwrap()).unwrap())
        .unwrap();
    fcntl_add_seals(
        &file,
        SealFlags::WRITE | SealFlags::SHRINK | SealFlags::GROW | SealFlags::SEAL,
    )
    .unwrap();
    let mut child =
        crate::spawn_synthetic_probe(file.into(), crate::SyntheticRole::Journal).unwrap();
    let parent = pidfd_open(getpid(), PidfdFlags::empty()).unwrap();
    assert_eq!(child.is_same_process(parent.as_fd()).unwrap(), false);
    child.terminate().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let ordinary = std::fs::File::open("/dev/null").unwrap();
    assert_eq!(child.is_same_process(ordinary.as_fd()).unwrap(), false);
}
