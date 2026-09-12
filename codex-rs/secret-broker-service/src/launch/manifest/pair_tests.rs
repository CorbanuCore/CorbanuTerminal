use super::super::spawn::Completion;
use super::super::spawn::tests::eventually;
use super::super::spawn::tests::static_image;
use super::*;
use pretty_assertions::assert_eq;
use std::io::Read;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::SocketAddr;
use std::os::unix::net::UnixListener;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[derive(Default)]
struct Probe {
    stopped: AtomicUsize,
    exited: AtomicBool,
    denied: AtomicBool,
    polled: AtomicUsize,
}
struct Fake(Arc<Probe>, bool);
impl Child for Fake {
    fn stop(&self) -> io::Result<()> {
        self.0.stopped.fetch_add(1, Ordering::SeqCst);
        if self.0.denied.load(Ordering::SeqCst) {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        self.0.exited.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn exited(&mut self) -> io::Result<bool> {
        self.0.polled.fetch_add(1, Ordering::SeqCst);
        if self.0.denied.load(Ordering::SeqCst) {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(self.0.exited.load(Ordering::SeqCst))
    }
}
impl Process for Fake {
    fn matches(&self, _peer: BorrowedFd<'_>) -> io::Result<bool> {
        Ok(false)
    }
}
impl Backend for Fake {
    type Image = u8;
    type Child = Self;
    fn spawn(self, _image: u8, _control: &Arc<Shared>) -> io::Result<Self> {
        if self.1 {
            Err(io::ErrorKind::NotFound.into())
        } else {
            Ok(self)
        }
    }
}
fn spec() -> Spec {
    Spec {
        generation: NonZeroU64::new(7).unwrap(),
        uid: nix::unistd::getuid().as_raw(),
    }
}
fn images() -> PairImages<u8> {
    PairImages {
        journal: 1,
        policy: 2,
    }
}

#[test]
fn pf_27_s01_pair_failure_and_asymmetric_cleanup_keep_one_generation() {
    for failure in [0, 1, 2] {
        let probes = [Arc::new(Probe::default()), Arc::new(Probe::default())];
        if failure == 2 {
            probes[0].denied.store(true, Ordering::SeqCst);
        }
        let handle = launch_pair_with(
            Reservation::acquire().unwrap(),
            images(),
            [
                Fake(Arc::clone(&probes[0]), failure == 0),
                Fake(Arc::clone(&probes[1]), failure == 1),
            ],
            spec(),
            Instant::now() + Duration::from_secs(3),
            start_worker,
        )
        .unwrap_or_else(|_| panic!("launch failed"));
        if failure == 2 {
            eventually(|| probes[0].polled.load(Ordering::SeqCst) > 2);
            assert!(Reservation::acquire().is_err());
            assert!(probes[1].stopped.load(Ordering::SeqCst) > 0);
            probes[0].denied.store(false, Ordering::SeqCst);
        }
        eventually(|| matches!(handle.status(), Status::Complete(_)));
        assert_eq!(
            handle.status(),
            Status::Complete(if failure == 0 {
                Completion::Rejected
            } else {
                Completion::Exited
            })
        );
        assert!(Reservation::acquire().is_ok());
        if failure == 1 {
            assert!(probes[0].stopped.load(Ordering::SeqCst) > 0);
        }
    }
}

#[test]
fn pf_27_s01_pair_worker_failure_returns_both_images_and_cancel_prevents_spawn() {
    let backends = || {
        [
            Fake(Arc::new(Probe::default()), false),
            Fake(Arc::new(Probe::default()), false),
        ]
    };
    let (_, returned) = launch_pair_with(
        Reservation::acquire().unwrap(),
        images(),
        backends(),
        spec(),
        Instant::now(),
        |_| Err(io::ErrorKind::WouldBlock.into()),
    )
    .err()
    .unwrap();
    assert_eq!((returned.journal, returned.policy), (1, 2));
    let (tx, rx) = mpsc::channel();
    let handle = launch_pair_with(
        Reservation::acquire().unwrap(),
        images(),
        backends(),
        spec(),
        Instant::now() + Duration::from_secs(3),
        |job| {
            start_worker(Box::new(move || {
                rx.recv().unwrap();
                job();
            }))
        },
    )
    .unwrap_or_else(|_| panic!("launch failed"));
    let (stream, _peer) = UnixStream::pair().unwrap();
    let ticket = handle.admit(stream).unwrap();
    handle.cancel();
    assert!(Reservation::acquire().is_err());
    tx.send(()).unwrap();
    eventually(|| handle.status() == Status::Complete(Completion::NotLaunched));
    drop(ticket);
    assert!(Reservation::acquire().is_ok());
}

fn result(ticket: &mut AdmissionTicket) -> io::Result<AdmittedPeer> {
    let mut found = None;
    eventually(|| {
        found = ticket.take_result();
        found.is_some()
    });
    found.unwrap()
}
fn real_images() -> PairImages {
    PairImages {
        journal: static_image("PF27_PAIR_CONNECT", 204),
        policy: static_image("PF27_PAIR_CONNECT", 204),
    }
}
fn listener() -> UnixListener {
    let address =
        SocketAddr::from_abstract_name(format!("corbanu-pf27-{}", std::process::id())).unwrap();
    let listener = UnixListener::bind_addr(&address).unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}
fn connections(listener: &UnixListener) -> Vec<(u8, UnixStream)> {
    let mut peers = Vec::new();
    eventually(|| {
        if let Ok((mut stream, _)) = listener.accept() {
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut role = [0];
            stream.read_exact(&mut role).unwrap();
            peers.push((role[0], stream));
        }
        peers.len() == 4
    });
    peers.sort_by_key(|(role, _)| *role);
    peers
}

#[test]
#[ignore = "requires hashed static connector and qualified GNU2.43 pidfs host"]
fn pf_27_s01_pair_real_admission_roles_duplicates_uid_and_old_generation() {
    let listener = listener();
    let mut old = None;
    for reverse in [false, true] {
        let generation = NonZeroU64::new(if reverse { 8 } else { 7 }).unwrap();
        let handle = Reservation::acquire()
            .unwrap()
            .launch_pair(
                real_images(),
                generation,
                spec().uid,
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap_or_else(|_| panic!("launch failed"));
        let mut peers = connections(&listener);
        if let Some(stream) = old.take() {
            assert!(result(&mut handle.admit(stream).unwrap()).is_err());
        }
        let (parent, _other) = UnixStream::pair().unwrap();
        assert!(result(&mut handle.admit(parent).unwrap()).is_err());
        if reverse {
            peers.reverse();
        }
        let mut admitted = Vec::new();
        for (byte, stream) in peers.drain(..3) {
            let expected = if byte == b'j' {
                SyntheticChildRole::Journal
            } else {
                SyntheticChildRole::Policy
            };
            let outcome = result(&mut handle.admit(stream).unwrap());
            if admitted
                .iter()
                .any(|peer: &AdmittedPeer| peer.role() == expected)
            {
                assert_eq!(outcome.err().unwrap().kind(), io::ErrorKind::AlreadyExists);
            } else {
                let peer = outcome.unwrap();
                assert_eq!((peer.role(), peer.generation()), (expected, generation));
                admitted.push(peer);
            }
        }
        assert_eq!(admitted.len(), 2);
        old = Some(peers.pop().unwrap().1);
        handle.cancel();
        for peer in &admitted {
            assert_eq!(peer.stream().read(&mut [0]).unwrap(), 0);
        }
        eventually(|| handle.status() == Status::Complete(Completion::Exited));
        assert!(Reservation::acquire().is_ok());
    }
    let error = Reservation::acquire()
        .unwrap()
        .launch_pair(
            real_images(),
            spec().generation,
            spec().uid + 1,
            Instant::now(),
        )
        .err()
        .unwrap()
        .0;
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
#[ignore = "requires hashed static connector and qualified GNU2.43 pidfs host"]
fn pf_27_s01_pair_real_peer_drop_and_caller_drop_reap_both() {
    let listener = listener();
    for caller_drop in [false, true] {
        let handle = Reservation::acquire()
            .unwrap()
            .launch_pair(
                real_images(),
                spec().generation,
                spec().uid,
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap_or_else(|_| panic!("launch failed"));
        let mut peers = connections(&listener);
        let peer = result(&mut handle.admit(peers.pop().unwrap().1).unwrap()).unwrap();
        if caller_drop {
            drop(handle);
        } else {
            drop(peer);
            drop(handle);
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
