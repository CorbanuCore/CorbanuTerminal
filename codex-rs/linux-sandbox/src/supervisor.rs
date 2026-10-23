//! #935 (Travis, 2026-10-19, D35 option A): a bubblewrap sandbox cannot
//! outlive the helper that started it.
//!
//! bubblewrap binds the sandbox's PID 1 to its parent with
//! `PR_SET_PDEATHSIG` only after setting the sandbox up, and never checks
//! that the parent is still alive afterwards. When the outer bubblewrap dies
//! before then (the helper killed during setup, a credential stop, a crash),
//! PID 1 keeps running with PPID 1 in its own session, and so does the
//! command it starts.
//!
//! So the helper runs bubblewrap under a supervisor: a fork of the helper in
//! its own process group (out of reach of a kill aimed at the helper's; its
//! own session for a command run without bubblewrap), set as a child
//! subreaper. Every orphaned sandbox PID 1 becomes its child. It
//! stops the sandbox when bubblewrap exits or when its lifeline from the
//! helper closes (the helper exited or was killed, saw its own parent exit,
//! or stopped the command): it kills bubblewrap and every remaining child
//! with `SIGKILL`, which for a PID 1 takes its whole PID namespace with it
//! (double-forked daemons included), and reaps them all before exiting with
//! bubblewrap's status.
//!
//! #1006 (Travis, 2026-10-20, D47 option B): a command run without
//! bubblewrap (full disk write access without managed-proxy routing, legacy
//! Landlock) runs under the same supervisor, in place of bubblewrap. There is
//! no PID namespace, but every process the command leaves behind is the
//! supervisor's descendant, and an orphaned one becomes its child: killing
//! its children, listing them again and repeating stops the whole tree. The
//! command keeps the helper's process group (see [`Relay`]).

use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::thread;

/// The helper's end of the lifeline: the sandbox runs while it is open.
pub(crate) struct Lifeline {
    write: Mutex<Option<OwnedFd>>,
}

impl Lifeline {
    /// Stops the sandbox. Idempotent; the helper's exit does the same.
    pub(crate) fn cut(&self) {
        drop(
            self.write
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take(),
        );
    }
}

/// A lifeline pipe (both ends close-on-exec, so bubblewrap and the command
/// never hold it): the supervisor's read end and the helper's [`Lifeline`].
pub(crate) fn lifeline() -> (OwnedFd, Lifeline) {
    let mut fds = [-1; 2];
    // SAFETY: `fds` is a valid out array of two descriptors.
    if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to create the sandbox lifeline: {err}");
    }
    // SAFETY: both descriptors were just created and are owned here.
    let (read, write) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
    (
        read,
        Lifeline {
            write: Mutex::new(Some(write)),
        },
    )
}

/// The helper's parent, opened while it is known to be alive.
pub(crate) enum ParentWatch {
    /// A pidfd: readable once the parent process has exited.
    Pidfd(OwnedFd),
    /// The parent exited before it could be opened.
    Gone,
    /// No pidfd (kernels before 5.3): `PR_SET_PDEATHSIG` delivers the
    /// death signal instead (`SIGTERM`, which the helper forwards to
    /// bubblewrap, or `SIGKILL`, which cuts the lifeline).
    DeathSignal,
}

impl ParentWatch {
    /// Opens the helper's parent. Checks `getppid()` after opening (or
    /// after arming the death signal), so a parent that exits in between is
    /// never missed. `death_signal` is used without a pidfd.
    pub(crate) fn open(death_signal: libc::c_int) -> Self {
        // SAFETY: plain syscalls.
        let parent = unsafe { libc::getppid() };
        let pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, parent, 0) };
        let watch = if pidfd >= 0 {
            // SAFETY: the descriptor was just returned and is owned here.
            Self::Pidfd(unsafe { OwnedFd::from_raw_fd(pidfd as RawFd) })
        } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            Self::Gone
        } else {
            // SAFETY: plain syscall on this process.
            if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, death_signal) } < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to watch the sandbox helper's parent: {err}");
            }
            Self::DeathSignal
        };
        // SAFETY: plain syscall.
        if unsafe { libc::getppid() } != parent {
            return Self::Gone;
        }
        watch
    }

    /// Cuts `lifeline` once the parent has exited. Call after forking the
    /// supervisor: this may start a thread.
    pub(crate) fn cut_when_gone(self, lifeline: Arc<Lifeline>) {
        match self {
            Self::Gone => lifeline.cut(),
            Self::DeathSignal => {}
            Self::Pidfd(pidfd) => {
                thread::spawn(move || {
                    let mut poll_fd = libc::pollfd {
                        fd: pidfd.as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    // SAFETY: one valid pollfd.
                    while unsafe { libc::poll(&mut poll_fd, 1, -1) } < 0
                        && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                    {
                    }
                    lifeline.cut();
                });
            }
        }
    }
}

/// #1006: how signals reach a command run without bubblewrap. The command
/// starts in the helper's process group (`group`, where the session's
/// signals and the terminal's go) and the supervisor leaves for a session of
/// its own, so a signal sent to the group reaches the command directly and
/// the helper ignores it. The helper writes each forwarded signal
/// (`signals`) it receives to `requests` (as an `i32`), and the supervisor
/// relays it only once the command has left `group` (an interactive shell
/// moves to a group of its own): the command then gets every signal sent to
/// the group once (twice at most if it arrives shortly before or while the
/// command leaves the group). A terminal hangup's `SIGHUP` goes to the
/// session leader alone, which the command was before #1006; the helper asks
/// for that one to be relayed in any case. Forwarded signals sent before the
/// command existed (`pending`) are sent to it before it starts (except in a
/// window of a few instructions just before the helper forks).
pub(crate) struct Relay {
    requests: OwnedFd,
    group: libc::pid_t,
    signals: &'static [libc::c_int],
    pending: Vec<libc::c_int>,
}

impl Relay {
    /// The supervisor's read end and the helper's write end (close-on-exec,
    /// non-blocking).
    pub(crate) fn pipe(group: libc::pid_t, signals: &'static [libc::c_int]) -> (Self, OwnedFd) {
        let mut fds = [-1; 2];
        // SAFETY: `fds` is a valid out array of two descriptors.
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) } < 0 {
            let err = std::io::Error::last_os_error();
            panic!("failed to create the sandbox signal relay: {err}");
        }
        // SAFETY: both descriptors were just created and are owned here.
        let (read, write) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
        (
            Self {
                requests: read,
                group,
                signals,
                pending: Vec::new(),
            },
            write,
        )
    }

    /// Notes `signal`, received before the command existed, to be sent to
    /// it as it starts.
    pub(crate) fn add_pending(&mut self, signal: libc::c_int) {
        if self.signals.contains(&signal) && !self.pending.contains(&signal) {
            self.pending.push(signal);
        }
    }

    /// Notes the forwarded signals pending in this process (the supervisor,
    /// which shared the helper's group until now and has them blocked).
    fn note_pending(&mut self) {
        // SAFETY: valid out set; plain syscall on this process.
        unsafe {
            let mut pending: libc::sigset_t = std::mem::zeroed();
            if libc::sigpending(&mut pending) < 0 {
                return;
            }
            for signal in self.signals {
                if libc::sigismember(&pending, *signal) == 1 {
                    self.add_pending(*signal);
                }
            }
        }
    }

    /// Asks the supervisor to relay `signal`, also to a command still in the
    /// helper's group with `always` (best effort: dropped if the supervisor
    /// is gone or the pipe is full).
    pub(crate) fn request(write: &OwnedFd, signal: libc::c_int, always: bool) {
        let bytes = if always { -signal } else { signal }.to_ne_bytes();
        // SAFETY: writes `bytes` (fewer than PIPE_BUF bytes: atomic).
        unsafe {
            libc::write(write.as_raw_fd(), bytes.as_ptr().cast(), bytes.len());
        }
    }

    /// Relays the requested signals to `command` (unreaped) if it has left
    /// the helper's group; whether the helper's end is still open.
    fn relay_requested(&self, command: libc::pid_t) -> bool {
        let mut bytes = [0u8; 4];
        loop {
            // SAFETY: reads at most `bytes.len()` bytes into `bytes`.
            let read = unsafe {
                libc::read(
                    self.requests.as_raw_fd(),
                    bytes.as_mut_ptr().cast(),
                    bytes.len(),
                )
            };
            if read == 0 {
                return false;
            }
            if read < 0 {
                return std::io::Error::last_os_error().raw_os_error() != Some(libc::EBADF);
            }
            if read as usize != bytes.len() {
                continue;
            }
            let request = libc::c_int::from_ne_bytes(bytes);
            let signal = request.abs();
            // SAFETY: `command` is this process's unreaped child, so its pid
            // (and, while it leads it, its group's id) is not reused.
            unsafe {
                let current = libc::getpgid(command);
                if current < 0 || (current == self.group && request > 0) {
                    continue;
                }
                if current == command {
                    libc::kill(-command, signal);
                } else {
                    libc::kill(command, signal);
                }
            }
        }
    }
}

/// Runs in the helper's fork, single-threaded: the supervisor. Never
/// returns. `start_child` runs in the supervisor's child, gets the
/// supervisor's pid and must exec bubblewrap, or the command itself with
/// `relay` (the child exits 127 if it returns; a panic ends it as a panic
/// ends the helper). `close_after_fork` are descriptors the supervisor itself
/// must not keep. Signals the helper forwards (`forwarded_signals`) stay
/// blocked here: they reach bubblewrap through the process group. `SIGCHLD`
/// is blocked too, so `start_child` must set its signal mask. With `relay`,
/// the supervisor is still in the helper's process group (with every signal
/// blocked): the child is born there and waits until the supervisor has
/// left for a session of its own.
pub(crate) fn run(
    lifeline: OwnedFd,
    forwarded_signals: &[libc::c_int],
    close_after_fork: &[RawFd],
    relay: Option<Relay>,
    start_child: impl FnOnce(libc::pid_t),
) -> ! {
    // SAFETY: plain syscalls on this process.
    unsafe {
        if libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) < 0 {
            let err = std::io::Error::last_os_error();
            panic!("failed to make the sandbox supervisor a subreaper: {err}");
        }
        libc::signal(libc::SIGCHLD, libc::SIG_DFL);
    }
    let child_signals = SignalFd::sigchld();
    // SAFETY: plain syscall.
    let supervisor = unsafe { libc::getpid() };
    let mut relay = relay;
    let start = relay.as_ref().map(|_| StartGate::new());
    // SAFETY: the supervisor is single-threaded.
    let bubblewrap = unsafe { libc::fork() };
    if bubblewrap < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to fork for bubblewrap: {err}");
    }
    if bubblewrap == 0 {
        if let Some(start) = start {
            start.wait();
        }
        start_child(supervisor);
        // SAFETY: ends the child, which failed to exec.
        unsafe { libc::_exit(127) }
    }
    if let (Some(relay), Some(start)) = (&mut relay, start) {
        // Out of reach of a kill aimed at the helper's group, which the
        // command was born in (so a signal sent to the group never misses
        // it), and outside the helper's session, so the command's group stays
        // orphaned as it was when the helper was the command (the terminal's
        // stop signals do not stop it).
        // SAFETY: plain syscall on this process, which leads no group.
        if unsafe { libc::setsid() } < 0 {
            let err = std::io::Error::last_os_error();
            panic!("failed to place the sandbox supervisor in its own session: {err}");
        }
        // Sent to the group while this process was in it, maybe before the
        // command was born. One the command got too merges with this copy:
        // it has every signal blocked until it starts.
        relay.note_pending();
        for signal in &relay.pending {
            // SAFETY: `bubblewrap` (the command) is this process's unreaped
            // child; it has every signal blocked until it starts.
            unsafe {
                libc::kill(bubblewrap, *signal);
            }
        }
        start.release();
    }
    // What the child inherits (its arguments' descriptors) is not kept here.
    drop(start_child);
    relay_pending(bubblewrap, forwarded_signals);
    for fd in close_after_fork.iter().copied().filter(|fd| *fd >= 0) {
        // SAFETY: closes this process's copy of a descriptor it does not use.
        unsafe {
            libc::close(fd);
        }
    }

    let mut status = None;
    let mut relay_open = relay.is_some();
    loop {
        let mut fds = [
            libc::pollfd {
                fd: lifeline.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: child_signals.0.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                // A negative descriptor is skipped.
                fd: match &relay {
                    Some(relay) if relay_open => relay.requests.as_raw_fd(),
                    _ => -1,
                },
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        // SAFETY: three valid pollfds.
        let ready = unsafe { libc::poll(fds.as_mut_ptr(), 3, -1) };
        if ready < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            break;
        }
        if fds[1].revents != 0 {
            child_signals.drain();
            reap_exited(bubblewrap, &mut status);
            if status.is_some() {
                break;
            }
        }
        if fds[0].revents != 0 {
            // The helper is gone or stopped the command.
            break;
        }
        if fds[2].revents != 0
            && let Some(relay) = &relay
        {
            relay_open = relay.relay_requested(bubblewrap);
        }
    }
    if status.is_none() {
        // SAFETY: bubblewrap is this process's unreaped child.
        unsafe {
            libc::kill(bubblewrap, libc::SIGKILL);
        }
    }
    kill_and_reap_children(bubblewrap, &mut status, &child_signals);
    exit_like(status.unwrap_or(libc::SIGKILL))
}

/// Reaps every exited child and records bubblewrap's status; whether any
/// child is left.
fn reap_exited(bubblewrap: libc::pid_t, status: &mut Option<libc::c_int>) -> bool {
    reap_exited_into(bubblewrap, status, &mut Vec::new())
}

/// [`reap_exited`], adding the reaped pids to `reaped`.
fn reap_exited_into(
    bubblewrap: libc::pid_t,
    status: &mut Option<libc::c_int>,
    reaped: &mut Vec<libc::pid_t>,
) -> bool {
    loop {
        let mut wait_status = 0;
        // SAFETY: valid out pointer.
        let pid = unsafe { libc::waitpid(-1, &mut wait_status, libc::WNOHANG | libc::__WALL) };
        if pid > 0 {
            reaped.push(pid);
        }
        if pid == bubblewrap {
            *status = Some(wait_status);
        } else if pid == 0 {
            return true;
        } else if pid < 0 {
            match std::io::Error::last_os_error().raw_os_error() {
                Some(libc::EINTR) => {}
                // ECHILD: none left.
                _ => return false,
            }
        }
    }
}

/// Kills and reaps every remaining child: bubblewrap if still running, and
/// any sandbox PID 1 it left behind. A child's pid cannot be reused before
/// it is reaped here, so the kill never reaches another process. Lists the
/// children again at least every 100 ms, so one that became a child after
/// a listing is not waited for without being killed.
fn kill_and_reap_children(
    bubblewrap: libc::pid_t,
    status: &mut Option<libc::c_int>,
    child_signals: &SignalFd,
) {
    // Children this process may not signal: another user's (a setuid program
    // run where `no_new_privs` is not set). They are left to init.
    let mut foreign = Vec::new();
    loop {
        let Some(children) = children() else {
            // Cannot list them (no /proc): reap what has exited rather than
            // wait for a child that may never exit.
            reap_exited(bubblewrap, status);
            return;
        };
        let mut killed = false;
        for child in children {
            if foreign.contains(&child) {
                continue;
            }
            // SAFETY: an unreaped child of this process.
            if unsafe { libc::kill(child, libc::SIGKILL) } < 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
            {
                foreign.push(child);
            } else {
                killed = true;
            }
        }
        let mut reaped = Vec::new();
        if !reap_exited_into(bubblewrap, status, &mut reaped) {
            return;
        }
        foreign.retain(|pid| !reaped.contains(pid));
        // An exited child's children are this process's by the time it is
        // reaped, so with none killed or reaped, only foreign ones are left.
        // (Processes started under a foreign one are not its children.)
        if !killed && reaped.is_empty() {
            return;
        }
        let mut poll_fd = libc::pollfd {
            fd: child_signals.0.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one valid pollfd.
        unsafe {
            libc::poll(&mut poll_fd, 1, 100);
        }
        child_signals.drain();
    }
}

/// This single-threaded process's children, from
/// `/proc/<pid>/task/<pid>/children`, or by scanning `/proc` where the
/// kernel lacks that file.
fn children() -> Option<Vec<libc::pid_t>> {
    // SAFETY: plain syscall.
    let me = unsafe { libc::getpid() };
    if let Ok(list) = std::fs::read_to_string(format!("/proc/{me}/task/{me}/children")) {
        return Some(
            list.split_ascii_whitespace()
                .filter_map(|pid| pid.parse().ok())
                .collect(),
        );
    }
    let mut children = Vec::new();
    for entry in std::fs::read_dir("/proc").ok()?.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<libc::pid_t>().ok())
        else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        // `pid (comm) state ppid …`; `comm` may contain anything but the
        // last `)`.
        let parent = stat
            .rsplit_once(')')
            .and_then(|(_, rest)| rest.split_ascii_whitespace().nth(1))
            .and_then(|ppid| ppid.parse::<libc::pid_t>().ok());
        if parent == Some(me) {
            children.push(pid);
        }
    }
    Some(children)
}

/// Sends bubblewrap the forwarded signals that reached the process group
/// before it existed (pending here). One that arrived since reaches it
/// twice, which changes nothing for these signals.
fn relay_pending(bubblewrap: libc::pid_t, forwarded_signals: &[libc::c_int]) {
    // SAFETY: valid out set; plain syscalls on this process's child.
    unsafe {
        let mut pending: libc::sigset_t = std::mem::zeroed();
        if libc::sigpending(&mut pending) < 0 {
            return;
        }
        for signal in forwarded_signals {
            if libc::sigismember(&pending, *signal) == 1 {
                libc::kill(bubblewrap, *signal);
            }
        }
    }
}

/// Exits the way `status` says bubblewrap (or the supervisor) did, with
/// other signals left blocked.
pub(crate) fn exit_like(status: libc::c_int) -> ! {
    if libc::WIFEXITED(status) {
        // SAFETY: ends this forked process without running its parent's
        // exit handlers.
        unsafe { libc::_exit(libc::WEXITSTATUS(status)) }
    }
    let signal = if libc::WIFSIGNALED(status) {
        libc::WTERMSIG(status)
    } else {
        libc::SIGKILL
    };
    // SAFETY: plain syscalls on this process; the signal is delivered on
    // unblocking it. The helper re-raises it too; one core dump at most.
    unsafe {
        let no_core = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &no_core);
        libc::signal(signal, libc::SIG_DFL);
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, signal);
        libc::kill(libc::getpid(), signal);
        libc::sigprocmask(libc::SIG_UNBLOCK, &set, std::ptr::null_mut());
        libc::_exit(128 + signal)
    }
}

/// #1006: holds a command born in the helper's process group until the
/// supervisor has left the group (so a kill aimed at the group cannot catch
/// the supervisor once the command runs) and sent it the signals it missed.
struct StartGate {
    read: OwnedFd,
    write: OwnedFd,
}

impl StartGate {
    fn new() -> Self {
        let mut fds = [-1; 2];
        // SAFETY: `fds` is a valid out array of two descriptors.
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
            let err = std::io::Error::last_os_error();
            panic!("failed to create the sandbox start gate: {err}");
        }
        // SAFETY: both descriptors were just created and are owned here.
        let (read, write) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
        Self { read, write }
    }

    /// In the child: returns once the supervisor released it or is gone.
    fn wait(self) {
        drop(self.write);
        let mut byte = 0u8;
        // SAFETY: reads at most one byte into `byte`.
        while unsafe { libc::read(self.read.as_raw_fd(), (&mut byte as *mut u8).cast(), 1) } < 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
        {}
    }

    /// In the supervisor.
    fn release(self) {
        drop(self);
    }
}

/// Blocks every signal (`SIGKILL` and `SIGSTOP` cannot be); the previous
/// mask.
pub(crate) fn block_all_signals() -> libc::sigset_t {
    // SAFETY: valid sets; plain syscall on this thread.
    unsafe {
        let mut all: libc::sigset_t = std::mem::zeroed();
        let mut previous: libc::sigset_t = std::mem::zeroed();
        libc::sigfillset(&mut all);
        if libc::sigprocmask(libc::SIG_BLOCK, &all, &mut previous) < 0 {
            let err = std::io::Error::last_os_error();
            panic!("failed to block signals in the sandbox helper: {err}");
        }
        previous
    }
}

/// Signals, blocked and read through a signalfd.
pub(crate) struct SignalFd(OwnedFd);

impl SignalFd {
    fn sigchld() -> Self {
        Self::new(&[libc::SIGCHLD])
    }

    pub(crate) fn new(signals: &[libc::c_int]) -> Self {
        // SAFETY: plain syscalls with valid sets.
        unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut set);
            for signal in signals {
                libc::sigaddset(&mut set, *signal);
            }
            if libc::sigprocmask(libc::SIG_BLOCK, &set, std::ptr::null_mut()) < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to block signals in the sandbox supervisor: {err}");
            }
            let fd = libc::signalfd(-1, &set, libc::SFD_CLOEXEC | libc::SFD_NONBLOCK);
            if fd < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to watch signals in the sandbox supervisor: {err}");
            }
            Self(OwnedFd::from_raw_fd(fd))
        }
    }

    /// The next pending signal, if any: its number and `si_code`.
    pub(crate) fn try_next(&self) -> Option<(libc::c_int, libc::c_int)> {
        // SAFETY: plain data, valid when zeroed.
        let mut info: libc::signalfd_siginfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<libc::signalfd_siginfo>();
        // SAFETY: reads at most `size` bytes into `info`.
        let read = unsafe {
            libc::read(
                self.0.as_raw_fd(),
                (&mut info as *mut libc::signalfd_siginfo).cast(),
                size,
            )
        };
        (read == size as isize).then_some((info.ssi_signo as libc::c_int, info.ssi_code))
    }

    /// Waits for the next signal: its number and `si_code` (a failed `poll`
    /// is retried).
    pub(crate) fn next(&self) -> (libc::c_int, libc::c_int) {
        loop {
            if let Some(next) = self.try_next() {
                return next;
            }
            let mut poll_fd = libc::pollfd {
                fd: self.0.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: one valid pollfd.
            unsafe {
                libc::poll(&mut poll_fd, 1, -1);
            }
        }
    }

    fn drain(&self) {
        let mut info: libc::signalfd_siginfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<libc::signalfd_siginfo>();
        // SAFETY: reads at most `size` bytes into `info`.
        while unsafe {
            libc::read(
                self.0.as_raw_fd(),
                (&mut info as *mut libc::signalfd_siginfo).cast(),
                size,
            )
        } > 0
        {}
    }
}
