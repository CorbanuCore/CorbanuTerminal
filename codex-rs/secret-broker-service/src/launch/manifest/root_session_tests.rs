use super::super::super::spawn::tests::eventually;
use super::super::tests::fixture_streams;
use super::super::tests::listening;
use super::super::tests::reaped;
use super::*;
use codex_protected_state::NativeAnchorClient;
use codex_protected_state::PolicyRootStore;
use codex_protected_state::SyntheticRootFixture as Fixture;
use codex_security_audit::IntegrityRootStore;
use nix::poll::PollFd;
use nix::poll::PollFlags;
use nix::poll::PollTimeout;
use nix::poll::poll;
use pretty_assertions::assert_eq;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsFd;
use std::time::Duration;

fn pending(lifetime: Duration) -> (RootSession, Fixture) {
    let (pair, mut listeners) = listening(lifetime);
    // Only observe readiness. The pump itself accepts both first/server sockets.
    eventually(|| {
        let mut fds = listeners
            .each_ref()
            .map(|l| PollFd::new(l.as_fd(), PollFlags::POLLIN));
        poll(&mut fds, PollTimeout::ZERO).unwrap() == 2
    });
    listeners.swap(0, 1); // Policy arrives first; index must not determine root.
    let fixture = Fixture::fresh().unwrap();
    let run = RootDispatch::open(pair, || Ok(fixture.roots())).unwrap();
    let mut session = RootSession::new(run, listeners).unwrap();
    assert!(!session.poll());
    assert_eq!(session.pending.iter().filter(|p| p.is_some()).count(), 2);
    assert_eq!(session.run.used, [false, false]);
    (session, fixture)
}
fn ready(lifetime: Duration) -> (RootSession, Fixture, [[UnixStream; 2]; 2]) {
    let (mut session, fixture) = pending(lifetime);
    // Between pump polls consume ONLY the fixture's separate client/control
    // sockets. The server sockets above are already owned by admission tickets.
    let listeners = session.listeners.as_ref().unwrap();
    let peers = [
        fixture_streams(&listeners[1]),
        fixture_streams(&listeners[0]),
    ];
    eventually(|| {
        assert!(!session.poll());
        session.run.used == [true, true]
    });
    (session, fixture, peers)
}
fn clients(peers: &[[UnixStream; 2]; 2]) -> [NativeAnchorClient; 2] {
    peers
        .each_ref()
        .map(|p| Fixture::client(p[0].try_clone().unwrap()).unwrap())
}
fn bounds(session: &RootSession) {
    assert!(session.pending.iter().filter(|p| p.is_some()).count() <= 2);
    assert!(session.run.jobs.iter().filter(|j| j.is_some()).count() <= 2);
}

#[test]
#[ignore = "hashed relay; actual pump admission and PF20 namespaces"]
fn pf27_root_session_routing_and_storage() {
    let (mut session, fixture, peers) = ready(Duration::from_secs(10));
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
    assert_eq!(
        (session.failure(), session.root_errors()),
        (None, [None, None])
    );
    bounds(&session);
    session.cancel();
    eventually(|| session.poll());
    reaped();
}

#[test]
#[ignore = "actual wrong peers, asymmetric flood and listener fairness"]
fn pf27_root_session_flood_fairness() {
    let (mut session, _fixture, peers) = ready(Duration::from_secs(10));
    let [journal, policy] = clients(&peers);
    let listeners = session.listeners.as_ref().unwrap();
    // Flood listener zero, but listener one must also be serviced immediately.
    let flood: Vec<_> = (0..12)
        .map(|_| {
            let stream = UnixStream::connect_addr(&listeners[0].local_addr().unwrap()).unwrap();
            stream.set_nonblocking(true).unwrap();
            stream
        })
        .collect();
    let mut other = UnixStream::connect_addr(&listeners[1].local_addr().unwrap()).unwrap();
    other.set_nonblocking(true).unwrap();
    assert!(!session.poll());
    assert_eq!(session.pending.iter().filter(|p| p.is_some()).count(), 2);
    eventually(|| match other.read(&mut [0]) {
        Ok(0) => true, // Wrong peer closed without any root handshake bytes.
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => false,
        result => panic!("wrong peer received data: {result:?}"),
    });
    // Global two-pending cap leaves later flood connections in the OS backlog.
    assert_eq!(
        (&flood[2]).read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::WouldBlock
    );
    let mut closed = [false; 12];
    eventually(|| {
        assert!(!session.poll());
        bounds(&session);
        for (index, mut socket) in flood.iter().enumerate() {
            match socket.read(&mut [0]) {
                Ok(0) => closed[index] = true,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                result => panic!("wrong peer received data: {result:?}"),
            }
        }
        closed.iter().all(|closed| *closed)
    });
    assert_eq!((journal.load(), policy.load_policy()), (Ok(None), Ok(None)));
    session.cancel();
    eventually(|| session.poll());
    reaped();
}

#[test]
#[ignore = "actual pair cleanup; deterministic accept error/interruption seam"]
fn pf27_root_session_accept_faults_are_bounded() {
    let (mut session, _fixture, peers) = ready(Duration::from_secs(10));
    let [journal, policy] = clients(&peers);
    for kind in [io::ErrorKind::Interrupted, io::ErrorKind::WouldBlock] {
        let mut attempts = 0;
        assert!(!session.poll_with(|_| {
            attempts += 1;
            Err(kind.into())
        }));
        assert_eq!(attempts, 4);
        assert_eq!(session.failure(), None);
    }
    let mut attempts = 0;
    session.poll_with(|_| {
        attempts += 1;
        Err(io::ErrorKind::Other.into())
    });
    assert_eq!(attempts, 1);
    assert_eq!(session.failure(), Some(RootError::Unavailable));
    assert!(session.listeners.is_none());
    eventually(|| session.poll());
    assert!(journal.load().is_err());
    assert!(policy.load_policy().is_err());
    reaped();
}

#[test]
#[ignore = "hashed relay; actual death/cancel/deadline/drop fencing"]
fn pf27_root_session_lifecycle() {
    for scenario in 0..4 {
        let lifetime = if scenario == 2 {
            Duration::from_millis(700)
        } else {
            Duration::from_secs(10)
        };
        let (mut session, _fixture, peers) = ready(lifetime);
        let [journal, policy] = clients(&peers);
        match scenario {
            0 => (&peers[1][1]).write_all(b"x").unwrap(),
            1 => session.cancel(),
            2 => {}
            _ => {
                drop(session);
                reaped();
                assert!(journal.load().is_err());
                assert!(policy.load_policy().is_err());
                continue;
            }
        }
        eventually(|| session.poll());
        assert!(session.listeners.is_none());
        assert!(session.pending.iter().all(Option::is_none));
        assert!(journal.load().is_err());
        assert!(policy.load_policy().is_err());
        reaped();
    }
}

#[test]
#[ignore = "actual pending receipt cancellation/drop; no root dispatch or detached cleanup"]
fn pf27_root_session_pending_cancellation() {
    for drop_caller in [false, true] {
        let (mut session, fixture) = pending(Duration::from_secs(10));
        let listeners = session.listeners.as_ref().unwrap();
        let peers: [[UnixStream; 2]; 2] = [
            fixture_streams(&listeners[1]),
            fixture_streams(&listeners[0]),
        ];
        if drop_caller {
            drop(session);
        } else {
            session.cancel();
            eventually(|| session.poll());
            assert_eq!(session.run.used, [false, false]);
        }
        reaped();
        for peer in peers {
            assert_eq!((&peer[0]).read(&mut [0]).unwrap(), 0);
        }
        assert_eq!(fixture.stored(), Ok(false));
    }
}

#[test]
#[ignore = "actual duplicate child sockets cannot create extra root jobs"]
fn pf27_root_session_duplicate_receipts() {
    let (mut session, fixture) = pending(Duration::from_secs(10));
    let listeners = session.listeners.as_ref().unwrap();
    // Take only client sockets. Leave the relay's distinct control sockets for
    // the pump: they have the same kernel child identity and must be rejected.
    let _clients: [[UnixStream; 1]; 2] = [
        fixture_streams(&listeners[1]),
        fixture_streams(&listeners[0]),
    ];
    eventually(|| {
        let complete = session.poll();
        bounds(&session);
        complete
    });
    assert_eq!(session.run.used, [true, true]);
    assert_eq!(fixture.stored(), Ok(false));
    reaped();
}

#[test]
#[ignore = "actual PF20 rejection and committed withheld reply preserve typed errors"]
fn pf27_root_session_protocol_errors() {
    let (mut session, _fixture, peers) = ready(Duration::from_secs(10));
    let [journal, policy] = clients(&peers);
    assert_eq!(journal.load_policy(), Err(RootError::Unavailable));
    eventually(|| session.poll());
    assert_eq!(session.root_errors()[0], Some(RootError::Invalid));
    assert!(policy.load_policy().is_err());
    reaped();
    drop(session);
    let (mut session, fixture, mut peers) = ready(Duration::from_secs(10));
    let [journal, policy] = clients(&peers);
    journal
        .compare_and_store(None, &Fixture::journal_checkpoint().unwrap())
        .unwrap();
    peers[1][1].write_all(b"p").unwrap();
    let mut ack = [0];
    peers[1][1].read_exact(&mut ack).unwrap();
    assert_eq!(ack, *b"k");
    let write = std::thread::spawn(move || {
        policy.compare_policy(None, &Fixture::policy_checkpoint().unwrap())
    });
    eventually(|| fixture.stored() == Ok(true));
    peers[1][1].write_all(b"x").unwrap();
    assert_eq!(write.join().unwrap(), Err(RootError::Ambiguous));
    eventually(|| session.poll());
    assert_eq!(fixture.stored(), Ok(true));
    assert!(journal.load().is_err());
    reaped();
}
