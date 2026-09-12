use super::super::super::spawn::tests::eventually;
use super::super::super::spawn::tests::static_image;
use super::*;
use pretty_assertions::assert_eq;
use std::io::Read;
use std::io::Write;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::SocketAddr;
use std::os::unix::net::UnixListener;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn spec() -> AdmissionSpec {
    AdmissionSpec {
        generation: NonZeroU64::new(7).unwrap(),
        expected_uid: nix::unistd::getuid().as_raw(),
    }
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
struct Gate {
    release: mpsc::Sender<()>,
    done: mpsc::Receiver<()>,
    permit: Arc<AtomicBool>,
}
fn control(deadline: Instant) -> (Arc<Shared>, LaunchHandle, Gate) {
    let permit = Arc::new(AtomicBool::new(false));
    let owner = Reservation::acquire_from(Arc::clone(&permit)).unwrap();
    let (tx, rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let handle = owner
        .launch_with((), Empty, deadline, |job| {
            start_worker(Box::new(move || {
                let _ = rx.recv();
                job();
                let _ = done_tx.send(());
            }))
        })
        .unwrap();
    (
        handle.control(),
        handle,
        Gate {
            release: tx,
            done: done_rx,
            permit,
        },
    )
}
struct Empty;
impl Child for Empty {
    fn stop(&self) -> io::Result<()> {
        Ok(())
    }
    fn exited(&mut self) -> io::Result<bool> {
        Ok(false)
    }
}
impl Process for Empty {
    fn matches(&self, _peer: BorrowedFd<'_>) -> io::Result<bool> {
        Ok(false)
    }
}
impl Backend for Empty {
    type Image = ();
    type Child = Self;
    fn spawn(self, _: (), _: &Arc<Shared>) -> io::Result<Self> {
        Err(io::ErrorKind::NotFound.into())
    }
}
struct Probe(Box<dyn Fn() -> io::Result<bool> + Send>);
impl Child for Probe {
    fn stop(&self) -> io::Result<()> {
        Ok(())
    }
    fn exited(&mut self) -> io::Result<bool> {
        Ok(false)
    }
}
impl Process for Probe {
    fn matches(&self, _: BorrowedFd<'_>) -> io::Result<bool> {
        (self.0)()
    }
}
fn admission() -> (mpsc::SyncSender<Request>, Admission) {
    let (tx, rx) = mpsc::sync_channel(2);
    (
        tx,
        Admission {
            spec: spec(),
            requests: rx,
            channels: [None, None],
        },
    )
}
fn outcome(ticket: &mut AdmissionTicket) -> io::Result<AdmittedPeer> {
    let mut result = None;
    eventually(|| {
        result = ticket.take_result();
        result.is_some()
    });
    result.unwrap()
}

#[test]
fn pf_27_s01_admission_bounded_queue_cancel_deadline_and_lost_ticket() {
    for scenario in 0..5 {
        let (control, owner, gate) = control(if scenario == 3 {
            Instant::now()
        } else {
            if scenario == 4 {
                Instant::now() + Duration::from_millis(100)
            } else {
                deadline()
            }
        });
        let (requests, mut admission) = admission();
        let handle = PairHandle { owner, requests };
        let (stream, _peer) = UnixStream::pair().unwrap();
        if scenario == 3 {
            assert!(handle.admit(stream).is_err());
            continue;
        }
        let first = handle.admit(stream).unwrap();
        let (stream, _peer) = UnixStream::pair().unwrap();
        let mut second = handle.admit(stream).unwrap();
        let (stream, _peer) = UnixStream::pair().unwrap();
        let fd = stream.as_raw_fd();
        let (error, returned) = handle.admit(stream).err().unwrap();
        assert_eq!(
            (error.kind(), returned.as_raw_fd()),
            (io::ErrorKind::WouldBlock, fd)
        );
        match scenario {
            0 => drop(first),
            1 => handle.cancel(),
            4 => eventually(|| control.cancelled()),
            _ => drop(handle),
        }
        admission.tick(&mut [Some(Empty), Some(Empty)], &control);
        assert!(control.cancelled());
        drop(admission);
        assert!(outcome(&mut second).is_err());
        assert!(Reservation::acquire_from(Arc::clone(&gate.permit)).is_err());
        drop(gate.release);
        gate.done.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(Reservation::acquire_from(gate.permit).is_ok());
    }
}

#[test]
fn pf_27_s01_admission_late_success_and_lost_receiver_close_channels() {
    for cancel_before_read in [false, true] {
        let (control, _owner, _release) = control(deadline());
        let (_requests, admission) = admission();
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let guard = stream.try_clone().unwrap();
        let result = AdmittedPeer {
            role: SyntheticChildRole::Journal,
            generation: spec().generation,
            stream,
            control: Arc::clone(&control),
        };
        let (tx, rx) = mpsc::sync_channel(1);
        let mut ticket = AdmissionTicket {
            reply: rx,
            control: Some(Arc::clone(&control)),
        };
        assert!(tx.send(Ok(result)).is_ok());
        if cancel_before_read {
            control.cancel();
            assert!(outcome(&mut ticket).is_err());
        } else {
            drop(ticket);
        }
        assert!(control.cancelled());
        let mut admission = admission;
        admission.channels[0] = Some(guard);
        admission.tick(&mut [Some(Empty), Some(Empty)], &control);
        peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }
}

fn listener() -> UnixListener {
    let address =
        SocketAddr::from_abstract_name(format!("corbanu-pf27-{}", std::process::id())).unwrap();
    let listener = UnixListener::bind_addr(&address).unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}
fn launch(generation: u64, expires: Instant) -> PairHandle {
    let owner = Reservation::acquire().unwrap();
    let images = PairImages {
        journal: static_image("PF27_PAIR_CONNECT", 204),
        policy: static_image("PF27_PAIR_CONNECT", 204),
    };
    owner
        .launch_admitting_pair(
            images,
            AdmissionSpec {
                generation: NonZeroU64::new(generation).unwrap(),
                ..spec()
            },
            expires,
        )
        .unwrap_or_else(|_| panic!("launch failed"))
}
fn connections(listener: &UnixListener, count: usize) -> Vec<(u8, UnixStream)> {
    let mut peers = Vec::new();
    eventually(|| {
        if let Ok((stream, _)) = listener.accept() {
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut role = [0];
            assert_eq!(
                recv(stream.as_raw_fd(), &mut role, MsgFlags::MSG_PEEK).unwrap(),
                1
            );
            peers.push((role[0], stream));
        }
        peers.len() == count
    });
    peers.sort_by_key(|(role, _)| *role);
    peers
}
fn reaped() {
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

#[test]
#[ignore = "requires hashed static connector and qualified GNU2.43 pidfs host"]
fn pf_27_s01_admission_real_roles_reverse_duplicate_parent_old_and_dead() {
    let listener = listener();
    let mut old = None;
    for generation in [7, 8] {
        let handle = launch(generation, deadline());
        let mut peers = connections(&listener, 4);
        if let Some(stream) = old.take() {
            assert!(live_peer(getsockopt(&stream, sockopt::PeerPidfd).unwrap().as_fd()).is_err());
            assert!(outcome(&mut handle.admit(stream).unwrap()).is_err());
        }
        let (stream, _peer) = UnixStream::pair().unwrap();
        assert!(outcome(&mut handle.admit(stream).unwrap()).is_err());
        if generation == 8 {
            peers.reverse();
        }
        let mut accepted: Vec<AdmittedPeer> = Vec::new();
        for (byte, stream) in peers.drain(..3) {
            let role = if byte == b'j' {
                SyntheticChildRole::Journal
            } else {
                SyntheticChildRole::Policy
            };
            let result = outcome(&mut handle.admit(stream).unwrap());
            if accepted.iter().any(|peer| peer.role() == role) {
                assert_eq!(result.err().unwrap().kind(), io::ErrorKind::AlreadyExists);
            } else {
                let peer = result.unwrap();
                assert_eq!((peer.role(), peer.generation().get()), (role, generation));
                accepted.push(peer);
            }
        }
        assert_eq!(accepted.len(), 2);
        old = Some(peers.pop().unwrap().1);
        handle.cancel();
        reaped();
        assert!(handle.admit(UnixStream::pair().unwrap().0).is_err());
    }
}

#[test]
#[ignore = "nextest-isolated process; actual kernel errors and temporary own fd limit"]
fn pf_27_s01_admission_kernel_faults_and_pre_delivery_cancellation() {
    use rustix::process::Resource;
    use rustix::process::Rlimit;
    use rustix::process::getrlimit;
    use rustix::process::setrlimit;
    let old = getrlimit(Resource::Nofile);
    let (control, _owner, _release) = control(deadline());
    let (_tx, mut verifier) = admission();
    let fd: std::os::fd::OwnedFd = std::fs::File::open("/dev/null").unwrap().into();
    let stream = UnixStream::from(fd);
    assert_eq!(
        verifier
            .admit(stream, &mut [Some(Empty), Some(Empty)], &control)
            .err()
            .unwrap()
            .raw_os_error(),
        Some(nix::libc::ENOTSOCK)
    );
    let (stream, _peer) = UnixStream::pair().unwrap();
    setrlimit(
        Resource::Nofile,
        Rlimit {
            current: Some(0),
            ..old
        },
    )
    .unwrap();
    let poll_error = live_peer(stream.as_fd());
    setrlimit(Resource::Nofile, old).unwrap();
    assert_eq!(
        poll_error.unwrap_err().raw_os_error(),
        Some(nix::libc::EINVAL)
    );
    for scenario in 0..4 {
        let (control, _owner, _release) = self::control(if scenario == 3 {
            Instant::now() + Duration::from_millis(100)
        } else {
            deadline()
        });
        let observer = Arc::clone(&control);
        let mut children = [
            Some(Probe(Box::new(move || {
                match scenario {
                    0 => return Err(io::ErrorKind::InvalidData.into()),
                    1 => setrlimit(
                        Resource::Nofile,
                        Rlimit {
                            current: Some(0),
                            ..old
                        },
                    )?,
                    2 => observer.cancel(),
                    _ => eventually(|| observer.cancelled()),
                }
                Ok(true)
            }))),
            Some(Probe(Box::new(|| Ok(false)))),
        ];
        let (stream, _peer) = UnixStream::pair().unwrap();
        let result = verifier.admit(stream, &mut children, &control);
        setrlimit(Resource::Nofile, old).unwrap();
        let error = result.err().unwrap();
        if scenario == 1 {
            // F_DUPFD_CLOEXEC's minimum fd is outside the zero soft limit.
            assert_eq!(error.raw_os_error(), Some(nix::libc::EINVAL));
        } else {
            assert_eq!(
                error.kind(),
                if scenario == 0 {
                    io::ErrorKind::InvalidData
                } else {
                    io::ErrorKind::ConnectionAborted
                }
            );
        }
        assert!(verifier.channels.iter().all(Option::is_none));
    }
}

#[test]
#[ignore = "requires real SO_PEERPIDFD; deterministic receiver loss at successful send"]
fn pf_27_s01_admission_worker_lost_success_receiver() {
    let (control, _owner, _release) = control(deadline());
    let (requests, mut admission) = admission();
    let (stream, mut peer) = UnixStream::pair().unwrap();
    let (reply, receiver) = mpsc::sync_channel(1);
    drop(receiver);
    requests.try_send(Request { stream, reply }).ok().unwrap();
    let mut children = [
        Some(Probe(Box::new(|| Ok(true)))),
        Some(Probe(Box::new(|| Ok(false)))),
    ];
    admission.tick(&mut children, &control);
    assert!(admission.channels[0].is_some());
    assert!(control.cancelled());
    peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    assert_eq!(peer.read(&mut [0]).unwrap(), 0);
}

#[test]
#[ignore = "requires hashed static connector and qualified GNU2.43 pidfs host"]
fn pf_27_s01_admission_real_uid_mismatch_and_unrelated_process() {
    let listener = listener();
    let handle = launch(7, deadline());
    let peers = connections(&listener, 4);
    let (control, _owner, _release) = control(deadline());
    let (_tx, mut admission) = admission();
    admission.spec.expected_uid += 1;
    for (_, stream) in peers {
        assert_eq!(
            admission
                .admit(stream, &mut [Some(Empty), Some(Empty)], &control)
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
    }
    assert!(!control.cancelled());
    let unrelated = static_image("PF27_PAIR_CONNECT", 204)
        .launch_owned(SyntheticChildRole::Worker)
        .unwrap();
    for (_, stream) in connections(&listener, 2) {
        assert!(outcome(&mut handle.admit(stream).unwrap()).is_err());
    }
    drop(unrelated);
    handle.cancel();
    reaped();
}

#[test]
#[ignore = "requires hashed static connector and qualified GNU2.43 pidfs host"]
fn pf_27_s01_admission_real_peer_eof_death_receipt_drop_and_caller_drop() {
    let listener = listener();
    for scenario in 0..10 {
        let handle = launch(7, deadline());
        let control = handle.owner.control();
        let mut peers = connections(&listener, 4);
        let policy = outcome(&mut handle.admit(peers.remove(2).1).unwrap()).unwrap();
        let journal = outcome(&mut handle.admit(peers.remove(0).1).unwrap()).unwrap();
        let (peer, other) = if scenario < 5 {
            (journal, policy)
        } else {
            (policy, journal)
        };
        // Two worker round trips with buffered data must not fence either role.
        for (_, stream) in peers {
            assert_eq!(
                outcome(&mut handle.admit(stream).unwrap())
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::AlreadyExists
            );
        }
        other.stream().read_exact(&mut [0]).unwrap();
        let reader = other.stream().try_clone().unwrap();
        reader
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let (tx, rx) = mpsc::channel();
        let read = std::thread::spawn(move || {
            tx.send((&reader).read(&mut [0])).unwrap();
        });
        match scenario % 5 {
            0 => {
                peer.stream().read_exact(&mut [0]).unwrap();
                peer.stream().write_all(b"e").unwrap();
            }
            1 => peer.stream().write_all(b"x").unwrap(),
            2 => peer.stream().write_all(b"e").unwrap(),
            3 => drop(peer),
            _ => drop(handle),
        }
        let result = rx.recv_timeout(Duration::from_secs(3));
        control.cancel();
        read.join().unwrap();
        reaped();
        assert_eq!(result.unwrap().unwrap(), 0);
    }
}

#[test]
#[ignore = "requires hashed static connector; actual pair ticket/deadline cleanup"]
fn pf_27_s01_admission_real_pending_ticket_and_deadline_reap_both() {
    let listener = listener();
    for expire in [false, true] {
        let handle = launch(7, Instant::now() + Duration::from_secs(1));
        let mut peers = connections(&listener, 4);
        assert!(Reservation::acquire().is_err());
        let first = handle.admit(peers.remove(2).1).unwrap();
        let mut second = handle.admit(peers.remove(0).1).unwrap();
        if expire {
            eventually(|| handle.owner.control().cancelled());
        } else {
            // Pending means not consumed: the worker may already have formed its reply.
            drop(first);
        }
        reaped();
        assert!(outcome(&mut second).is_err());
        assert!(handle.admit(UnixStream::pair().unwrap().0).is_err());
    }
}
