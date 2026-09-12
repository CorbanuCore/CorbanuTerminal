use super::*;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;

#[derive(Default)]
struct Probe {
    calls: AtomicUsize,
    stops: AtomicUsize,
    polls: AtomicUsize,
    drops: AtomicUsize,
    image_drops: AtomicUsize,
    exited: AtomicBool,
    stop_error: AtomicBool,
    wait_error: AtomicBool,
    panic_wait: AtomicBool,
}
struct Image(Arc<Probe>);
impl Drop for Image {
    fn drop(&mut self) {
        self.0.image_drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct FakeChild(Arc<Probe>);
impl Drop for FakeChild {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl Child for FakeChild {
    fn stop(&self) -> io::Result<()> {
        self.0.stops.fetch_add(1, Ordering::SeqCst);
        if self.0.stop_error.load(Ordering::SeqCst) {
            return Err(io::Error::other("stop denied"));
        }
        self.0.exited.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn exited(&mut self) -> io::Result<bool> {
        self.0.polls.fetch_add(1, Ordering::SeqCst);
        assert!(
            !self.0.panic_wait.load(Ordering::SeqCst),
            "injected worker panic"
        );
        if self.0.wait_error.load(Ordering::SeqCst) {
            return Err(io::Error::other("wait denied"));
        }
        Ok(self.0.exited.load(Ordering::SeqCst))
    }
}
struct Fake {
    probe: Arc<Probe>,
    gate: Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>,
    reject: bool,
}
impl Backend for Fake {
    type Image = Image;
    type Child = FakeChild;
    fn spawn(self, image: Image) -> io::Result<FakeChild> {
        self.probe.calls.fetch_add(1, Ordering::SeqCst);
        if let Some((entered, release)) = self.gate {
            entered.send(()).unwrap();
            release.recv().unwrap();
        }
        drop(image);
        if self.reject {
            Err(io::Error::other("exec rejected"))
        } else {
            Ok(FakeChild(self.probe))
        }
    }
}
fn eventually(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "owner did not reach expected checkpoint"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn fixture() -> (Arc<AtomicBool>, Arc<Probe>, Reservation, Fake) {
    let permit = Arc::new(AtomicBool::new(false));
    let probe = Arc::new(Probe::default());
    let reservation = Reservation::acquire_from(Arc::clone(&permit)).unwrap();
    let backend = Fake {
        probe: Arc::clone(&probe),
        gate: None,
        reject: false,
    };
    (permit, probe, reservation, backend)
}
fn launch(reservation: Reservation, probe: &Arc<Probe>, backend: Fake) -> LaunchHandle {
    reservation
        .launch_with(
            Image(Arc::clone(probe)),
            backend,
            Instant::now() + Duration::from_secs(30),
            start_worker,
        )
        .unwrap_or_else(|_| panic!("worker creation failed"))
}
fn busy(permit: &Arc<AtomicBool>) -> bool {
    permit.load(Ordering::Acquire)
}

#[test]
fn pf_27_s01_owner_reservation_precedes_image_and_rejects_second_launch() {
    let (permit, probe, reservation, _) = fixture();
    assert!(Reservation::acquire_from(Arc::clone(&permit)).is_err());
    assert_eq!(probe.image_drops.load(Ordering::SeqCst), 0);
    drop(reservation);
    assert!(!busy(&permit));
    assert!(Reservation::acquire_from(permit).is_ok());
}

#[test]
fn pf_27_s01_owner_worker_failure_returns_image_without_spawn() {
    let (permit, probe, reservation, backend) = fixture();
    let (_, returned) = reservation
        .launch_with(Image(Arc::clone(&probe)), backend, Instant::now(), |_| {
            Err(io::Error::other("thread creation denied"))
        })
        .err()
        .expect("worker creation should fail");
    assert_eq!(
        (
            probe.calls.load(Ordering::SeqCst),
            probe.image_drops.load(Ordering::SeqCst)
        ),
        (0, 0)
    );
    assert!(!busy(&permit));
    drop(returned);
    assert_eq!(probe.image_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s01_owner_cancel_before_worker_starts_never_launches() {
    let (permit, probe, reservation, backend) = fixture();
    let (release, wait) = mpsc::channel();
    let handle = reservation
        .launch_with(
            Image(Arc::clone(&probe)),
            backend,
            Instant::now() + Duration::from_secs(30),
            move |job| {
                start_worker(Box::new(move || {
                    wait.recv().unwrap();
                    job();
                }))
            },
        )
        .unwrap_or_else(|_| panic!("thread failed"));
    handle.cancel();
    assert_eq!(handle.status(), Status::CleanupPending);
    assert!(busy(&permit));
    release.send(()).unwrap();
    eventually(|| !busy(&permit));
    assert_eq!(
        (
            handle.status(),
            probe.calls.load(Ordering::SeqCst),
            probe.image_drops.load(Ordering::SeqCst)
        ),
        (Status::Complete(Completion::NotLaunched), 0, 1)
    );
}

#[test]
fn pf_27_s01_owner_late_spawn_drop_keeps_permit_until_reaping() {
    let (permit, probe, reservation, mut backend) = fixture();
    let (entered, reached) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    backend.gate = Some((entered, wait));
    let handle = launch(reservation, &probe, backend);
    reached.recv_timeout(Duration::from_secs(2)).unwrap();
    let start = Instant::now();
    drop(handle);
    assert!(start.elapsed() < Duration::from_millis(100));
    assert!(Reservation::acquire_from(Arc::clone(&permit)).is_err());
    assert_eq!(probe.drops.load(Ordering::SeqCst), 0);
    release.send(()).unwrap();
    eventually(|| !busy(&permit));
    assert_eq!(
        (
            probe.calls.load(Ordering::SeqCst),
            probe.stops.load(Ordering::SeqCst),
            probe.drops.load(Ordering::SeqCst)
        ),
        (1, 1, 1)
    );
}

#[test]
fn pf_27_s01_owner_deadline_while_spawn_blocked_is_not_cleanup() {
    let (permit, probe, reservation, mut backend) = fixture();
    let (entered, reached) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    backend.gate = Some((entered, wait));
    let deadline = Instant::now() + Duration::from_millis(100);
    let handle = reservation
        .launch_with(Image(Arc::clone(&probe)), backend, deadline, start_worker)
        .unwrap_or_else(|_| panic!("thread failed"));
    reached.recv_timeout(Duration::from_secs(2)).unwrap();
    eventually(|| Instant::now() >= deadline);
    assert_eq!(handle.status(), Status::CleanupPending);
    assert!(busy(&permit));
    release.send(()).unwrap();
    eventually(|| !busy(&permit));
    assert_eq!(handle.status(), Status::Complete(Completion::Exited));
}

#[test]
fn pf_27_s01_owner_signal_wait_failure_quarantines_until_same_owner_recovers() {
    let (permit, probe, reservation, backend) = fixture();
    probe.stop_error.store(true, Ordering::SeqCst);
    probe.wait_error.store(true, Ordering::SeqCst);
    let handle = launch(reservation, &probe, backend);
    eventually(|| probe.stops.load(Ordering::SeqCst) >= 2);
    assert_eq!(
        (
            handle.status(),
            busy(&permit),
            probe.drops.load(Ordering::SeqCst)
        ),
        (Status::CleanupPending, true, 0)
    );
    probe.stop_error.store(false, Ordering::SeqCst);
    eventually(|| probe.exited.load(Ordering::SeqCst));
    assert!(Reservation::acquire_from(Arc::clone(&permit)).is_err());
    assert_eq!(probe.drops.load(Ordering::SeqCst), 0);
    probe.wait_error.store(false, Ordering::SeqCst);
    eventually(|| !busy(&permit));
    assert_eq!(
        (handle.status(), probe.drops.load(Ordering::SeqCst)),
        (Status::Complete(Completion::Exited), 1)
    );
}

#[test]
fn pf_27_s01_owner_early_exit_and_rejected_spawn_release_only_after_cleanup() {
    for reject in [false, true] {
        let (permit, probe, reservation, mut backend) = fixture();
        probe.exited.store(true, Ordering::SeqCst);
        backend.reject = reject;
        let handle = launch(reservation, &probe, backend);
        eventually(|| !busy(&permit));
        let result = if reject {
            Completion::Rejected
        } else {
            Completion::Exited
        };
        assert_eq!(
            (
                handle.status(),
                probe.image_drops.load(Ordering::SeqCst),
                probe.stops.load(Ordering::SeqCst)
            ),
            (Status::Complete(result), 1, 0)
        );
    }
}

#[test]
fn pf_27_s01_owner_panic_is_not_cleanup_receipt_or_relaunch_permission() {
    let (permit, probe, reservation, backend) = fixture();
    probe.panic_wait.store(true, Ordering::SeqCst);
    let handle = launch(reservation, &probe, backend);
    eventually(|| handle.status() == Status::Quarantined);
    drop(handle);
    assert_eq!(
        (probe.drops.load(Ordering::SeqCst), busy(&permit)),
        (1, true)
    );
    assert!(Reservation::acquire_from(permit).is_err());
}

fn static_image(key: &str, anchor: u32) -> SyntheticProfileInspectedImage {
    use super::super::IMAGE_LIMIT;
    use super::super::SyntheticLaunchRecipe;
    use super::super::SyntheticManifestInspection;
    use super::super::files;
    let path = std::env::var_os(key).expect("hashed synthetic fixture required");
    let mut file = std::fs::File::open(path).unwrap();
    let digest = files::image_digest(&mut file, IMAGE_LIMIT).unwrap();
    let manifest = SyntheticManifestInspection {
        _stamp: files::stamp(&file).unwrap(),
        _image: file,
        _recipe: SyntheticLaunchRecipe::new(
            "/unused".into(),
            [(101, 201), (102, 202), (103, 203)],
            anchor,
        )
        .unwrap(),
        _digest: digest,
    };
    manifest.seal().unwrap().inspect_static_profile().unwrap()
}

#[test]
#[ignore = "requires coordinator-hashed static hold and probe on qualified GNU2.43"]
fn pf_27_s01_owner_real_static_hold_cancel_drop_and_early_exit() {
    for drop_early in [false, true] {
        let reservation = Reservation::acquire().unwrap();
        let handle = reservation
            .launch(
                static_image("PF27_SYNTHETIC_HOLD", 204),
                SyntheticChildRole::Journal,
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap_or_else(|_| panic!("worker creation failed"));
        eventually(|| handle.status() == Status::Running);
        if drop_early {
            drop(handle);
        } else {
            handle.cancel();
            eventually(|| handle.status() == Status::Complete(Completion::Exited));
        }
        eventually(|| !busy(&PERMIT));
        assert_eq!(
            rustix::process::waitid(
                rustix::process::WaitId::All,
                rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOHANG
            )
            .err(),
            Some(rustix::io::Errno::CHILD)
        );
    }
    let reservation = Reservation::acquire().unwrap();
    let handle = reservation
        .launch(
            static_image("PF27_SYNTHETIC_PROBE", 204),
            SyntheticChildRole::Journal,
            Instant::now() + Duration::from_secs(5),
        )
        .unwrap_or_else(|_| panic!("worker creation failed"));
    eventually(|| !busy(&PERMIT));
    assert_eq!(handle.status(), Status::Complete(Completion::Exited));
}

#[test]
#[ignore = "requires coordinator-hashed static hold on qualified GNU2.43"]
fn pf_27_s01_owner_rejects_identity_substitution_before_spawn() {
    let reservation = Reservation::acquire().unwrap();
    let handle = reservation
        .launch(
            static_image("PF27_SYNTHETIC_HOLD", 205),
            SyntheticChildRole::Journal,
            Instant::now() + Duration::from_secs(5),
        )
        .unwrap_or_else(|_| panic!("worker creation failed"));
    eventually(|| !busy(&PERMIT));
    assert_eq!(handle.status(), Status::Complete(Completion::Rejected));
}

#[test]
#[ignore = "requires coordinator-hashed static hold on qualified GNU2.43"]
fn pf_27_s01_owner_real_child_returned_late_after_caller_drop_is_reaped() {
    struct DelayedKernel(mpsc::Sender<()>, mpsc::Receiver<()>);
    impl Backend for DelayedKernel {
        type Image = SyntheticProfileInspectedImage;
        type Child = OwnedChild;
        fn spawn(self, image: Self::Image) -> io::Result<OwnedChild> {
            let child = image.launch_owned(SyntheticChildRole::Journal)?;
            self.0.send(()).unwrap();
            self.1.recv().unwrap();
            Ok(child)
        }
    }
    let reservation = Reservation::acquire().unwrap();
    let (entered, reached) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let handle = reservation
        .launch_with(
            static_image("PF27_SYNTHETIC_HOLD", 204),
            DelayedKernel(entered, wait),
            Instant::now() + Duration::from_secs(5),
            start_worker,
        )
        .unwrap_or_else(|_| panic!("worker creation failed"));
    reached.recv_timeout(Duration::from_secs(2)).unwrap();
    drop(handle);
    assert!(Reservation::acquire().is_err());
    release.send(()).unwrap();
    eventually(|| !busy(&PERMIT));
    assert_eq!(
        rustix::process::waitid(
            rustix::process::WaitId::All,
            rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOHANG
        )
        .err(),
        Some(rustix::io::Errno::CHILD)
    );
}
