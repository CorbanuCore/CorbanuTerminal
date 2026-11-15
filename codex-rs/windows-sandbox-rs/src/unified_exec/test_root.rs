use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use tempfile::TempDir;

/// A probe workspace outside the checkout, with checked, bounded cleanup.
/// Windows can briefly retain file handles after a sandboxed process exits.
/// `TempDir`'s best-effort drop alone silently leaves those workspaces behind.
pub(super) struct TestRoot(TempDir);

impl TestRoot {
    pub(super) fn new() -> Self {
        Self(tempfile::tempdir().expect("create user-owned test root in system temp"))
    }

    pub(super) fn path(&self) -> &Path {
        self.0.path()
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match fs::remove_dir_all(self.path()) {
                Ok(()) => return,
                Err(err) if err.kind() == io::ErrorKind::NotFound => return,
                Err(_) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(err) => {
                    let path = self.path().display();
                    if std::thread::panicking() {
                        eprintln!("failed to clean probe workspace {path}: {err}");
                    } else {
                        panic!("failed to clean probe workspace {path}: {err}");
                    }
                    return;
                }
            }
        }
    }
}

#[test]
fn pf_27_s07_test_root_is_outside_checkout_and_removed() {
    let root = super::user_owned_test_root();
    let path = root.path().canonicalize().expect("canonical test root");
    let checkout = super::sandbox_cwd()
        .canonicalize()
        .expect("canonical checkout");
    assert!(
        !path.starts_with(checkout),
        "probe root is in the checkout: {path:?}"
    );
    assert!(path.starts_with(std::env::temp_dir().canonicalize().expect("canonical temp")));
    fs::write(path.join("fixture"), "probe").expect("write fixture");
    drop(root);
    assert!(!path.exists(), "probe root survived cleanup: {path:?}");
}

#[test]
fn pf_27_s07_test_root_retries_a_held_windows_handle() {
    use std::os::windows::fs::OpenOptionsExt;

    let root = super::user_owned_test_root();
    let path = root.path().to_path_buf();
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(0)
        .open(path.join("held"))
        .expect("open fixture without delete sharing");
    // Establish that this is a real Windows deletion failure, not just a
    // delayed thread whose handle never prevented cleanup.
    assert!(fs::remove_dir_all(&path).is_err());
    std::thread::scope(|scope| {
        scope.spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            drop(file);
        });
        drop(root);
    });
    assert!(
        !path.exists(),
        "probe root survived handle release: {path:?}"
    );
}
