use super::super::pair::PairImages;
use super::super::pair::admission::AdmissionSpec;
use super::super::spawn::Reservation;
use super::super::spawn::tests::eventually;
use super::super::spawn::tests::static_image;
use super::*;
use codex_protected_state::NativeAnchorClient;
use codex_protected_state::PolicyRootStore;
use codex_protected_state::SyntheticRootFixture as Fixture;
use codex_security_audit::IntegrityRootStore;
use pretty_assertions::assert_eq;
use std::io::Read;
use std::io::Write;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::SocketAddr;
use std::os::unix::net::UnixListener;
use std::time::Duration;
use std::time::Instant;

fn setup(lifetime: Duration) -> (PairHandle, [[UnixStream; 3]; 2], Fixture) {
    let listeners = ['j', 'p'].map(|role| {
        let name = format!("corbanu-pf27-dispatch-{}{role}", std::process::id());
        let listener =
            UnixListener::bind_addr(&SocketAddr::from_abstract_name(name).unwrap()).unwrap();
        listener.set_nonblocking(true).unwrap();
        listener
    });
    let pair = Reservation::acquire()
        .unwrap()
        .launch_admitting_pair(
            PairImages {
                journal: static_image("PF27_DISPATCH_RELAY", 204),
                policy: static_image("PF27_DISPATCH_RELAY", 204),
            },
            AdmissionSpec {
                generation: NonZeroU64::new(9).unwrap(),
                expected_uid: nix::unistd::getuid().as_raw(),
            },
            Instant::now() + lifetime,
        )
        .unwrap_or_else(|_| panic!("pair failed"));
    let peers = listeners.map(|listener| {
        std::array::from_fn(|_| {
            let mut stream = None;
            eventually(|| match listener.accept() {
                Ok((accepted, _)) => {
                    stream = Some(accepted);
                    true
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => false,
                Err(error) => panic!("accept: {error}"),
            });
            let stream = stream.unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream
        })
    });
    (pair, peers, Fixture::fresh().unwrap())
}
fn receipt(ticket: &mut AdmissionTicket) -> io::Result<AdmittedPeer> {
    let mut result = None;
    eventually(|| {
        result = ticket.take_result();
        result.is_some()
    });
    result.unwrap()
}
fn start(fixture: &Fixture, pair: PairHandle, peers: &[[UnixStream; 3]; 2]) -> RootDispatch {
    let mut run = RootDispatch::open(pair, || Ok(fixture.roots())).unwrap();
    // Deliberately deliver policy before journal; never infer role from order.
    for (index, role) in [
        (1, SyntheticChildRole::Policy),
        (0, SyntheticChildRole::Journal),
    ] {
        let peer = receipt(&mut run.admit(peers[index][0].try_clone().unwrap()).unwrap()).unwrap();
        assert_eq!((peer.role(), peer.generation().get()), (role, 9));
        run.dispatch(peer).unwrap();
    }
    run
}
fn clients(peers: &[[UnixStream; 3]; 2]) -> [NativeAnchorClient; 2] {
    peers
        .each_ref()
        .map(|peer| Fixture::client(peer[1].try_clone().unwrap()).unwrap())
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
#[ignore = "hashed two-role static relay; actual non-root pair and PF20 protocol"]
fn pf27_root_dispatch_actual_namespaces_and_duplicate() {
    let (pair, peers, fixture) = setup(Duration::from_secs(10));
    let mut run = start(&fixture, pair, &peers);
    let [journal, policy] = clients(&peers);
    assert_eq!((journal.load(), policy.load_policy()), (Ok(None), Ok(None)));
    let j = Fixture::journal_checkpoint().unwrap();
    let p = Fixture::policy_checkpoint().unwrap();
    assert_eq!(journal.compare_and_store(None, &j), Ok(()));
    assert_eq!(policy.compare_policy(None, &p), Ok(()));
    assert_eq!(
        (journal.load(), policy.load_policy()),
        (Ok(Some(j)), Ok(Some(p)))
    );
    assert_eq!(fixture.stored(), Ok(true));
    let duplicate = receipt(&mut run.admit(peers[0][2].try_clone().unwrap()).unwrap());
    assert_eq!(
        duplicate.err().unwrap().kind(),
        io::ErrorKind::AlreadyExists
    );
    // No handshake went to the rejected control socket (relay only writes on 'p').
    peers[0][2]
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    assert!(matches!(
        (&peers[0][2]).read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
    assert!(!run.poll_complete());
    run.cancel();
    eventually(|| run.poll_complete());
    reaped();
}

#[test]
#[ignore = "hashed two-role relay; namespace, malformed and replay errors"]
fn pf27_root_dispatch_protocol_rejections() {
    for scenario in 0..4 {
        let (pair, peers, fixture) = setup(Duration::from_secs(10));
        let mut run = start(&fixture, pair, &peers);
        let [journal, policy] = clients(&peers);
        match scenario {
            0 => assert_eq!(journal.load_policy(), Err(RootError::Unavailable)),
            1 => assert_eq!(
                policy.load(),
                Err(codex_security_audit::IntegrityRootError::Unavailable)
            ),
            2 => (&peers[0][1]).write_all(&u32::MAX.to_be_bytes()).unwrap(),
            _ => {
                assert_eq!(journal.load(), Ok(None));
                assert!(Fixture::replay_load(&journal).is_err());
            }
        }
        eventually(|| run.poll_complete());
        // Namespace mismatch closes without an error frame; keep the exact
        // server reason separate from the client's unavailable transport.
        assert_eq!(
            run.errors()[usize::from(scenario == 1)],
            Some(RootError::Invalid)
        );
        assert!(policy.load_policy().is_err());
        assert_eq!(fixture.stored(), Ok(false));
        reaped();
    }
}

#[test]
#[ignore = "hashed two-role relay; committed withheld reply must remain ambiguous"]
fn pf27_root_dispatch_lost_reply() {
    let (pair, mut peers, fixture) = setup(Duration::from_secs(10));
    let mut run = start(&fixture, pair, &peers);
    let [journal, policy] = clients(&peers);
    journal
        .compare_and_store(None, &Fixture::journal_checkpoint().unwrap())
        .unwrap();
    peers[1][2].write_all(b"p").unwrap();
    let mut ack = [0];
    peers[1][2].read_exact(&mut ack).unwrap();
    assert_eq!(ack, *b"k");
    let write = std::thread::spawn(move || {
        let result = policy.compare_policy(None, &Fixture::policy_checkpoint().unwrap());
        (result, policy.load_policy())
    });
    eventually(|| fixture.stored() == Ok(true));
    peers[1][2].write_all(b"x").unwrap();
    assert_eq!(
        write.join().unwrap(),
        (Err(RootError::Ambiguous), Err(RootError::Unavailable))
    );
    eventually(|| run.poll_complete());
    // The relay accepted the kernel write before withholding it. The client,
    // not the server's later EOF, owns the observed ambiguous acknowledgement.
    assert_eq!(fixture.stored(), Ok(true));
    assert!(journal.load().is_err());
    reaped();
}

#[test]
#[ignore = "hashed relay; actual death, cancellation, deadline and caller drop"]
fn pf27_root_dispatch_lifecycle() {
    for scenario in 0..4 {
        let lifetime = if scenario == 2 {
            Duration::from_millis(700)
        } else {
            Duration::from_secs(10)
        };
        let (pair, peers, fixture) = setup(lifetime);
        let mut run = start(&fixture, pair, &peers);
        let [journal, policy] = clients(&peers);
        match scenario {
            0 => (&peers[1][2]).write_all(b"x").unwrap(),
            1 => run.cancel(),
            2 => {}
            _ => {
                drop(run);
                reaped();
                assert!(journal.load().is_err());
                assert!(policy.load_policy().is_err());
                continue;
            }
        }
        eventually(|| run.poll_complete());
        assert!(journal.load().is_err());
        assert!(policy.load_policy().is_err());
        reaped();
    }
}

#[test]
#[ignore = "hashed relay; wrong/stale identities never enter root protocol"]
fn pf27_root_dispatch_wrong_and_stale_receipts() {
    let (pair, peers, fixture) = setup(Duration::from_secs(10));
    let (wrong, mut observer) = UnixStream::pair().unwrap();
    assert!(receipt(&mut pair.admit(wrong).unwrap()).is_err());
    observer
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    assert_eq!(observer.read(&mut [0]).unwrap(), 0);
    let stale = receipt(&mut pair.admit(peers[0][0].try_clone().unwrap()).unwrap()).unwrap();
    pair.cancel();
    reaped();
    drop(pair);
    let (next, _next_peers, _next_fixture) = setup(Duration::from_secs(10));
    let mut run = RootDispatch::open(next, || Ok(fixture.roots())).unwrap();
    assert_eq!(run.dispatch(stale), Err(RootError::Invalid));
    eventually(|| run.poll_complete());
    assert_eq!((&peers[0][1]).read(&mut [0]).unwrap(), 0);
    reaped();
}

#[test]
#[ignore = "hashed relay; actual cleanup around private open/start/panic failure seams"]
fn pf27_root_dispatch_construction_failures() {
    for scenario in 0..4 {
        let (pair, peers, fixture) = setup(Duration::from_secs(10));
        if scenario == 0 {
            assert!(RootDispatch::open_existing(pair).is_err());
        } else if scenario == 1 {
            assert!(matches!(
                RootDispatch::open(pair, || Err(RootError::Ambiguous)),
                Err(RootError::Ambiguous)
            ));
        } else {
            let mut run = RootDispatch::open(pair, || Ok(fixture.roots())).unwrap();
            let peer = receipt(&mut run.admit(peers[0][0].try_clone().unwrap()).unwrap()).unwrap();
            if scenario == 2 {
                assert_eq!(
                    run.start(peer, |_job| Err(io::ErrorKind::Other.into())),
                    Err(RootError::Unavailable)
                );
            } else {
                run.start(peer, |job| {
                    std::thread::Builder::new().spawn(move || {
                        drop(job);
                        panic!("injected root panic")
                    })
                })
                .unwrap();
            }
            eventually(|| run.poll_complete());
            assert_eq!(run.errors()[0], Some(RootError::Unavailable));
        }
        reaped();
        assert_eq!((&peers[0][1]).read(&mut [0]).unwrap(), 0);
    }
}
