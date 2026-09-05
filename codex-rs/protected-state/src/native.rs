//! Native transport is an authenticated storage dependency, not worker
//! containment. PF-27 owns launch restrictions and the system socket listener.
use crate::ControllerRoot;
use crate::PolicyCheckpoint;
use crate::RootError;
use crate::checkpoint::Checkpoint;
use crate::linux::MAX_BYTES;
use codex_security_audit::IntegrityCheckpoint;
use codex_security_audit::IntegrityRootError;
use codex_security_audit::IntegrityRootStore;
use hmac::Hmac;
use hmac::Mac;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::process::Child;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use zeroize::Zeroizing;

const SOCKET: &str = "/run/corbanu-protected-state.sock";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Request {
    LoadJournal,
    LoadPolicy,
    Compare {
        expected: Option<Checkpoint>,
        next: Box<Checkpoint>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Reply {
    Loaded(Option<Checkpoint>),
    Stored,
    Rejected(RootError),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    sequence: u64,
    payload: Vec<u8>,
    tag: Vec<u8>,
}

fn peer(stream: &UnixStream) -> Result<libc::ucred, RootError> {
    let mut credentials = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut size = size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: valid writable credential buffer and length, live socket fd.
    if unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            credentials.as_mut_ptr().cast(),
            &mut size,
        )
    } != 0
        || size as usize != size_of::<libc::ucred>()
    {
        return Err(RootError::Unavailable);
    }
    // SAFETY: successful getsockopt filled exactly the expected credential size.
    Ok(unsafe { credentials.assume_init() })
}

fn configure(stream: &UnixStream) -> Result<(), RootError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| RootError::Unavailable)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| RootError::Unavailable)
}

fn read<T: DeserializeOwned>(stream: &mut UnixStream) -> Result<T, RootError> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut prefix = [0; 4];
    read_before(stream, &mut prefix, deadline)?;
    let length = u32::from_be_bytes(prefix) as usize;
    if length > MAX_BYTES {
        return Err(RootError::Invalid);
    }
    let mut bytes = Zeroizing::new(vec![0; length]);
    read_before(stream, &mut bytes, deadline)?;
    serde_json::from_slice(&bytes).map_err(|_| RootError::Invalid)
}

fn write<T: Serialize + ?Sized>(stream: &mut UnixStream, value: &T) -> Result<(), RootError> {
    let bytes = Zeroizing::new(serde_json::to_vec(value).map_err(|_| RootError::Invalid)?);
    if bytes.len() > MAX_BYTES {
        return Err(RootError::Invalid);
    }
    let mut frame = Zeroizing::new(Vec::with_capacity(bytes.len() + 4));
    frame.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    frame.extend_from_slice(&bytes);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut remaining = frame.as_slice();
    while !remaining.is_empty() {
        stream.set_write_timeout(Some(deadline.checked_duration_since(Instant::now()).ok_or(RootError::Unavailable)?)).map_err(|_| RootError::Unavailable)?;
        match stream.write(remaining) {
            Ok(0) => return Err(RootError::Unavailable),
            Ok(count) => remaining = &remaining[count..],
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(RootError::Unavailable),
        }
    }
    Ok(())
}

fn read_before(stream: &mut UnixStream, mut output: &mut [u8], deadline: Instant) -> Result<(), RootError> {
    while !output.is_empty() {
        stream.set_read_timeout(Some(deadline.checked_duration_since(Instant::now()).ok_or(RootError::Unavailable)?)).map_err(|_| RootError::Unavailable)?;
        match stream.read(output) {
            Ok(0) => return Err(RootError::Unavailable),
            Ok(count) => output = &mut output[count..],
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(RootError::Unavailable),
        }
    }
    Ok(())
}

struct Channel {
    stream: UnixStream,
    key: Zeroizing<[u8; 32]>,
    sequence: u64,
}

impl Channel {
    fn send<T: Serialize>(&mut self, value: &T, direction: &[u8]) -> Result<(), RootError> {
        let payload = serde_json::to_vec(value).map_err(|_| RootError::Invalid)?;
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.key.as_ref()).map_err(|_| RootError::Invalid)?;
        mac.update(direction);
        mac.update(&self.sequence.to_be_bytes());
        mac.update(&payload);
        write(
            &mut self.stream,
            &Packet {
                sequence: self.sequence,
                payload,
                tag: mac.finalize().into_bytes().to_vec(),
            },
        )
    }

    fn receive<T: DeserializeOwned>(&mut self, direction: &[u8]) -> Result<T, RootError> {
        let packet: Packet = read(&mut self.stream)?;
        if packet.sequence != self.sequence {
            return Err(RootError::Invalid);
        }
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.key.as_ref()).map_err(|_| RootError::Invalid)?;
        mac.update(direction);
        mac.update(&packet.sequence.to_be_bytes());
        mac.update(&packet.payload);
        mac.verify_slice(&packet.tag)
            .map_err(|_| RootError::Invalid)?;
        serde_json::from_slice(&packet.payload).map_err(|_| RootError::Invalid)
    }
}

fn alive(pidfd: &File) -> Result<(), RootError> {
    let mut poll = libc::pollfd {
        fd: pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: one live, writable pollfd; zero timeout is nonblocking.
    if unsafe { libc::poll(&mut poll, 1, 0) } == 0 {
        Ok(())
    } else {
        Err(RootError::Unavailable)
    }
}

impl ControllerRoot {
    /// Trusted launcher supplies its actual Child handle, never a worker PID
    /// string. The post-exec connection must belong to that still-live process.
    /// The root's fixed system construction is the authority boundary; this
    /// method does not establish separate-principal containment or activation.
    pub fn serve_child(&self, mut stream: UnixStream, child: &mut Child) -> Result<(), RootError> {
        if child
            .try_wait()
            .map_err(|_| RootError::Unavailable)?
            .is_some()
        {
            return Err(RootError::Unavailable);
        }
        let pid = i32::try_from(child.id()).map_err(|_| RootError::Invalid)?;
        // SAFETY: pidfd_open receives a numeric PID and zero flags.
        let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if raw < 0 {
            return Err(RootError::Unsupported);
        }
        // SAFETY: pidfd_open returned a newly owned descriptor.
        let pidfd = unsafe { File::from_raw_fd(raw as i32) };
        if child
            .try_wait()
            .map_err(|_| RootError::Unavailable)?
            .is_some()
            || peer(&stream)?.pid != pid
        {
            return Err(RootError::Invalid);
        }
        alive(&pidfd)?;
        configure(&stream)?;
        let key = Zeroizing::new(rand::random::<[u8; 32]>());
        // This one-time key crosses only the kernel-authenticated live-child
        // channel. It is never stored in argv, environment, logs or worker data.
        write(&mut stream, key.as_ref())?;
        let mut channel = Channel {
            stream,
            key,
            sequence: 1,
        };
        loop {
            let request: Request = channel.receive(b"corbanu-anchor-request/v1")?;
            alive(&pidfd)?;
            let reply = match request {
                Request::LoadJournal => {
                    self.require_journal()?;
                    Reply::Loaded(self.load()?)
                }
                Request::LoadPolicy => {
                    self.require_policy()?;
                    Reply::Loaded(self.load()?)
                }
                Request::Compare { expected, next } => {
                    match self.compare(expected.as_ref(), &next) {
                        Ok(()) => Reply::Stored,
                        Err(error) => {
                            let _ =
                                channel.send(&Reply::Rejected(error), b"corbanu-anchor-reply/v1");
                            return Err(error);
                        }
                    }
                }
            };
            alive(&pidfd).map_err(|error| if matches!(reply, Reply::Stored) { RootError::Ambiguous } else { error })?;
            channel
                .send(&reply, b"corbanu-anchor-reply/v1")
                .map_err(|_| RootError::Ambiguous)?;
            channel.sequence = channel.sequence.checked_add(1).ok_or(RootError::Invalid)?;
        }
    }
}

/// One native controller channel; any error consumes its local capability.
/// An implementation of IntegrityRootStore is not evidence of OS isolation.
pub struct NativeAnchorClient {
    channel: Mutex<Option<Channel>>,
}

impl std::fmt::Debug for NativeAnchorClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeAnchorClient").finish_non_exhaustive()
    }
}

impl NativeAnchorClient {
    /// Fixed trusted system endpoint, never a caller-supplied path or root UID.
    /// PF-27 must install its root-owned listener and authorize the live child.
    pub fn connect_system() -> Result<Self, RootError> {
        let metadata = std::fs::symlink_metadata(SOCKET).map_err(|_| RootError::Unavailable)?;
        if metadata.uid() != 0 || metadata.file_type().is_symlink() {
            return Err(RootError::Invalid);
        }
        let stream = UnixStream::connect(SOCKET).map_err(|_| RootError::Unavailable)?;
        if peer(&stream)?.uid != 0 {
            return Err(RootError::Invalid);
        }
        Self::from_authenticated_stream(stream)
    }

    fn from_authenticated_stream(mut stream: UnixStream) -> Result<Self, RootError> {
        configure(&stream)?;
        let key: [u8; 32] = read(&mut stream)?;
        Ok(Self {
            channel: Mutex::new(Some(Channel {
                stream,
                key: Zeroizing::new(key),
                sequence: 1,
            })),
        })
    }

    fn exchange(&self, request: &Request) -> Result<Reply, RootError> {
        let mut slot = self.channel.lock().map_err(|_| RootError::Unavailable)?;
        let channel = slot.as_mut().ok_or(RootError::Unavailable)?;
        let result = channel
            .send(request, b"corbanu-anchor-request/v1")
            .and_then(|()| channel.receive(b"corbanu-anchor-reply/v1"))
            .and_then(|reply| {
                channel.sequence = channel.sequence.checked_add(1).ok_or(RootError::Invalid)?;
                Ok(reply)
            });
        if result.is_err() {
            *slot = None;
        }
        // Any failed compare transport may have committed. Never retry it or
        // collapse uncertain publication to an ordinary safe rejection.
        match result.map_err(|error| {
            if matches!(request, Request::Compare { .. }) {
                RootError::Ambiguous
            } else {
                error
            }
        })? {
            Reply::Rejected(error) => {
                *slot = None;
                Err(error)
            }
            reply => Ok(reply),
        }
    }
}

impl crate::PolicyRootStore for NativeAnchorClient {
    fn load_policy(&self) -> Result<Option<PolicyCheckpoint>, RootError> {
        match self.exchange(&Request::LoadPolicy)? {
            Reply::Loaded(None) => Ok(None),
            Reply::Loaded(Some(Checkpoint::Policy(value))) => Ok(Some(value)),
            _ => Err(RootError::Invalid),
        }
    }

    fn compare_policy(
        &self,
        expected: Option<&PolicyCheckpoint>,
        next: &PolicyCheckpoint,
    ) -> Result<(), RootError> {
        match self.exchange(&Request::Compare {
            expected: expected.cloned().map(Checkpoint::Policy),
            next: Box::new(Checkpoint::Policy(next.clone())),
        })? {
            Reply::Stored => Ok(()),
            _ => Err(RootError::Invalid),
        }
    }
}

impl IntegrityRootStore for NativeAnchorClient {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        match self.exchange(&Request::LoadJournal)? {
            Reply::Loaded(None) => Ok(None),
            Reply::Loaded(Some(Checkpoint::Journal(value))) => Ok(Some(value)),
            _ => Err(IntegrityRootError::Invalid),
        }
    }

    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        match self.exchange(&Request::Compare {
            expected: expected.cloned().map(Checkpoint::Journal),
            next: Box::new(Checkpoint::Journal(next.clone())),
        })? {
            Reply::Stored => Ok(()),
            _ => Err(IntegrityRootError::Invalid),
        }
    }
}

#[cfg(test)]
#[path = "native_tests.rs"]
mod tests;
