use super::super::spawn::Completion;
use super::super::spawn::Status;
use super::super::spawn::tests::eventually;
use super::super::spawn::tests::static_image;
use super::*;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Default)]
struct Probe {
    launches: AtomicUsize,
    stops: AtomicUsize,
    polls: AtomicUsize,
    drops: AtomicUsize,
    exited: AtomicBool,
    stop_error: AtomicBool,
    wait_error: AtomicBool,
    panic_poll: AtomicBool,
}
struct Token(Arc<Probe>);
impl Child for Token {
    fn stop(&self) -> io::Result<()> {
        self.0.stops.fetch_add(1, Ordering::SeqCst);
        if self.0.stop_error.load(Ordering::SeqCst) {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        self.0.exited.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn exited(&mut self) -> io::Result<bool> {
        self.0.polls.fetch_add(1, Ordering::SeqCst);
        assert!(
            !self.0.panic_poll.load(Ordering::SeqCst),
            "injected pair panic"
        );
        if self.0.wait_error.load(Ordering::SeqCst) {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(self.0.exited.load(Ordering::SeqCst))
    }
}
impl Drop for Token {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct Fake {
    probe: Arc<Probe>,
    reject: bool,
    cancel_after: bool,
    gate: Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>,
}
impl Backend for Fake {
    type Image = u8;
    type Child = Token;
    fn spawn(self, _image: u8, control: &Arc<Shared>) -> io::Result<Token> {
        self.probe.launches.fetch_add(1, Ordering::SeqCst);
        if self.reject {
            return Err(io::ErrorKind::NotFound.into());
        }
        let child = Token(self.probe);
        if let Some((entered, release)) = self.gate {
            entered.send(()).unwrap();
            release.recv().unwrap();
        }
        if self.cancel_after {
            control.cancel();
        }
        Ok(child)
    }
}
fn probes() -> [Arc<Probe>; 2] {
    [Arc::new(Probe::default()), Arc::new(Probe::default())]
}
fn backends(probes: &[Arc<Probe>; 2]) -> [Fake; 2] {
    probes.each_ref().map(|probe| Fake {
        probe: Arc::clone(probe),
        reject: false,
        cancel_after: false,
        gate: None,
    })
}
fn reservation() -> (Arc<AtomicBool>, Reservation) {
    let permit = Arc::new(AtomicBool::new(false));
    let reservation = Reservation::acquire_from(Arc::clone(&permit)).unwrap();
    (permit, reservation)
}
fn images() -> PairImages<u8> {
    PairImages {
        journal: 1,
        policy: 2,
    }
}
fn launch(reservation: Reservation, backends: [Fake; 2], deadline: Instant) -> LaunchHandle {
    reservation
        .launch_with(images(), PairBackend(backends), deadline, start_worker)
        .unwrap_or_else(|_| panic!("worker creation failed"))
}
fn released(permit: &AtomicBool) {
    eventually(|| !permit.load(Ordering::Acquire));
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

#[test]
fn pf_27_s01_pair_reservation_worker_failure_and_prelaunch_cancel() {
    let (permit, owner) = reservation();
    assert!(Reservation::acquire_from(Arc::clone(&permit)).is_err());
    let probes = probes();
    let (_, returned) = owner
        .launch_with(images(), PairBackend(backends(&probes)), deadline(), |_| {
            Err(io::ErrorKind::WouldBlock.into())
        })
        .err()
        .unwrap();
    assert_eq!((returned.journal, returned.policy), (1, 2));
    released(&permit);
    for expired in [false, true] {
        let (permit, owner) = reservation();
        let (tx, rx) = mpsc::channel();
        let handle = owner
            .launch_with(
                images(),
                PairBackend(backends(&probes)),
                if expired { Instant::now() } else { deadline() },
                |job| {
                    start_worker(Box::new(move || {
                        rx.recv().unwrap();
                        job();
                    }))
                },
            )
            .unwrap_or_else(|_| panic!("worker creation failed"));
        if !expired {
            handle.cancel();
        }
        assert!(permit.load(Ordering::Acquire));
        tx.send(()).unwrap();
        released(&permit);
        assert_eq!(handle.status(), Status::Complete(Completion::NotLaunched));
    }
    assert_eq!(
        probes.each_ref().map(|p| p.launches.load(Ordering::SeqCst)),
        [0, 0]
    );
}

#[test]
fn pf_27_s01_pair_first_second_failure_and_between_spawn_cancel() {
    for scenario in 0..3 {
        let (permit, owner) = reservation();
        let probes = probes();
        let mut backends = backends(&probes);
        if scenario < 2 {
            backends[scenario].reject = true;
        } else {
            backends[0].cancel_after = true;
        }
        let handle = launch(owner, backends, deadline());
        released(&permit);
        assert_eq!(
            probes.each_ref().map(|p| p.launches.load(Ordering::SeqCst)),
            if scenario == 1 { [1, 1] } else { [1, 0] }
        );
        assert_eq!(
            probes.each_ref().map(|p| p.drops.load(Ordering::SeqCst)),
            if scenario == 0 { [0, 0] } else { [1, 0] }
        );
        assert_eq!(
            handle.status(),
            Status::Complete(if scenario == 0 {
                Completion::Rejected
            } else {
                Completion::Exited
            })
        );
    }
}

#[test]
fn pf_27_s01_pair_late_second_spawn_cancel_drop_and_deadline_keep_capacity() {
    for scenario in 0..3 {
        let (permit, owner) = reservation();
        let probes = probes();
        let mut backends = backends(&probes);
        let (entered, reached) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        backends[1].gate = Some((entered, rx));
        let deadline = Instant::now() + Duration::from_millis(200);
        let handle = launch(owner, backends, deadline);
        reached.recv_timeout(Duration::from_secs(2)).unwrap();
        let start = Instant::now();
        match scenario {
            0 => handle.cancel(),
            1 => {
                drop(handle);
                assert!(start.elapsed() < Duration::from_millis(100));
            }
            _ => {
                eventually(|| Instant::now() >= deadline);
                assert_eq!(handle.status(), Status::CleanupPending);
            }
        }
        assert!(permit.load(Ordering::Acquire));
        assert!(Reservation::acquire_from(Arc::clone(&permit)).is_err());
        assert_eq!(
            probes.each_ref().map(|p| p.drops.load(Ordering::SeqCst)),
            [0, 0]
        );
        tx.send(()).unwrap();
        released(&permit);
        assert_eq!(
            probes.each_ref().map(|p| p.drops.load(Ordering::SeqCst)),
            [1, 1]
        );
        assert!(probes.iter().all(|p| p.stops.load(Ordering::SeqCst) > 0));
    }
}

#[test]
fn pf_27_s01_pair_either_death_and_asymmetric_errors_poll_and_stop_both() {
    for first in 0..2 {
        let (permit, owner) = reservation();
        let probes = probes();
        probes[first].exited.store(true, Ordering::SeqCst);
        probes[1 - first].stop_error.store(true, Ordering::SeqCst);
        probes[1 - first].wait_error.store(true, Ordering::SeqCst);
        let handle = launch(owner, backends(&probes), deadline());
        eventually(|| probes.iter().all(|p| p.stops.load(Ordering::SeqCst) > 1));
        assert!(permit.load(Ordering::Acquire));
        assert_eq!(
            probes.each_ref().map(|p| p.drops.load(Ordering::SeqCst)),
            [0, 0]
        );
        assert!(probes.iter().all(|p| p.polls.load(Ordering::SeqCst) > 1));
        probes[1 - first].stop_error.store(false, Ordering::SeqCst);
        probes[1 - first].wait_error.store(false, Ordering::SeqCst);
        released(&permit);
        assert_eq!(handle.status(), Status::Complete(Completion::Exited));
        assert_eq!(
            probes.each_ref().map(|p| p.drops.load(Ordering::SeqCst)),
            [1, 1]
        );
    }
}

#[test]
fn pf_27_s01_pair_panic_signals_both_without_releasing_quarantine() {
    let (permit, owner) = reservation();
    let probes = probes();
    probes[0].panic_poll.store(true, Ordering::SeqCst);
    let handle = launch(owner, backends(&probes), deadline());
    eventually(|| handle.status() == Status::Quarantined);
    assert_eq!(
        probes.each_ref().map(|p| p.drops.load(Ordering::SeqCst)),
        [1, 1]
    );
    assert!(probes.iter().all(|p| p.stops.load(Ordering::SeqCst) > 0));
    drop(handle);
    assert!(Reservation::acquire_from(permit).is_err());
}

#[test]
#[ignore = "requires coordinator-hashed static hold on qualified GNU2.43"]
fn pf_27_s01_pair_real_two_holds_cancel_drop_and_reap() {
    for drop_caller in [false, true] {
        let owner = Reservation::acquire().unwrap();
        assert!(Reservation::acquire().is_err());
        let images = PairImages {
            journal: static_image("PF27_SYNTHETIC_HOLD", 204),
            policy: static_image("PF27_SYNTHETIC_HOLD", 204),
        };
        let handle = owner
            .launch_pair(images, deadline())
            .unwrap_or_else(|_| panic!("worker creation failed"));
        eventually(|| handle.status() == Status::Running);
        let children: std::collections::HashSet<_> = std::fs::read_dir("/proc/self/task")
            .unwrap()
            .flat_map(|task| {
                std::fs::read_to_string(task.unwrap().path().join("children"))
                    .unwrap()
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(children.len(), 2);
        if drop_caller {
            let start = Instant::now();
            drop(handle);
            assert!(start.elapsed() < Duration::from_millis(100));
        } else {
            handle.cancel();
            eventually(|| handle.status() == Status::Complete(Completion::Exited));
        }
        eventually(|| Reservation::acquire().is_ok());
        assert_eq!(
            rustix::process::waitid(
                rustix::process::WaitId::All,
                rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOHANG
            )
            .err(),
            Some(rustix::io::Errno::CHILD)
        );
    }
}
