//! Runs only inside the dedicated single-threaded probe, never in the parent.
use nix::sys::prctl;
use nix::unistd;
use rustix::thread::CapabilitySet;
use rustix::thread::CapabilitySets;
use std::fs;
use std::io;
use std::io::Read;

const MAX_STATUS: u64 = 8192;
const MAX_DESCRIPTORS: usize = 256;

fn unavailable() -> io::Error {
    io::Error::other("post-exec inspection unavailable")
}

fn valid_ids(uid: [u32; 3], gid: [u32; 3]) -> bool {
    uid[0] != 0
        && uid.iter().all(|id| *id == uid[0])
        && gid[0] != 0
        && gid.iter().all(|id| *id == gid[0])
}

fn ambient_is_empty(status: &str) -> io::Result<bool> {
    if status.len() as u64 > MAX_STATUS {
        return Err(unavailable());
    }
    let mut values = status
        .lines()
        .filter_map(|line| line.strip_prefix("CapAmb:"));
    let value = values.next().ok_or_else(unavailable)?.trim();
    if values.next().is_some() || value.len() != 16 || !value.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(unavailable());
    }
    Ok(u64::from_str_radix(value, 16).map_err(|_| unavailable())? == 0)
}

fn descriptor_allowlist() -> io::Result<bool> {
    let mut descriptors = Vec::new();
    // Drop this iterator (and its own descriptor) before probing the observed
    // numbers. No new descriptors/threads are opened during the second phase.
    for entry in fs::read_dir("/proc/self/fd")? {
        if descriptors.len() == MAX_DESCRIPTORS {
            return Err(unavailable());
        }
        let name = entry?.file_name();
        let fd: u32 = name
            .to_str()
            .ok_or_else(unavailable)?
            .parse()
            .map_err(|_| unavailable())?;
        descriptors.push(fd);
    }
    let mut standard = [false; 3];
    let mut allowed = true;
    for fd in descriptors {
        match fs::metadata(format!("/proc/self/fd/{fd}")) {
            Ok(_) => {
                if let Some(slot) = standard.get_mut(fd as usize) {
                    *slot = true;
                } else {
                    allowed = false;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err(unavailable()),
        }
    }
    Ok(allowed && standard.into_iter().all(|present| present))
}

/// Bounded synthetic observations, not an authority or containment report.
pub(super) struct Report {
    no_new_privileges: bool,
    nondumpable: bool,
    keepcaps_disabled: bool,
    capabilities_empty: bool,
    ambient_empty: bool,
    groups: usize,
    descriptors_allowed: bool,
}

impl Report {
    fn verified(&self) -> bool {
        self.no_new_privileges
            && self.nondumpable
            && self.keepcaps_disabled
            && self.capabilities_empty
            && self.ambient_empty
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // No paths, identities, environment values or descriptor contents.
        write!(
            f,
            "{{\"no_new_privileges\":{},\"nondumpable\":{},\"keepcaps_disabled\":{},\"capabilities_empty\":{},\"ambient_empty\":{},\"supplementary_group_count\":{},\"descriptor_allowlist\":{},\"native_eligible\":false}}",
            self.no_new_privileges,
            self.nondumpable,
            self.keepcaps_disabled,
            self.capabilities_empty,
            self.ambient_empty,
            self.groups,
            self.descriptors_allowed
        )
    }
}

pub(super) fn inspect() -> io::Result<Report> {
    // These setters affect the caller. Never invoke this function from tests
    // in-process or from any future multi-threaded/root supervisor.
    let uid = unistd::getresuid()?;
    let gid = unistd::getresgid()?;
    if !valid_ids(
        [
            uid.real.as_raw(),
            uid.effective.as_raw(),
            uid.saved.as_raw(),
        ],
        [
            gid.real.as_raw(),
            gid.effective.as_raw(),
            gid.saved.as_raw(),
        ],
    ) || fs::read_dir("/proc/self/task")?
        .take(2)
        .collect::<io::Result<Vec<_>>>()?
        .len()
        != 1
    {
        return Err(unavailable());
    }
    prctl::set_no_new_privs()?;
    prctl::set_keepcaps(false)?;
    rustix::thread::clear_ambient_capability_set()?;
    let empty = CapabilitySets {
        effective: CapabilitySet::empty(),
        permitted: CapabilitySet::empty(),
        inheritable: CapabilitySet::empty(),
    };
    rustix::thread::set_capabilities(/*pid*/ None, empty)?;
    prctl::set_dumpable(false)?;
    let mut status = String::new();
    fs::File::open("/proc/self/status")?
        .take(MAX_STATUS + 1)
        .read_to_string(&mut status)?;
    if status.len() as u64 > MAX_STATUS {
        return Err(unavailable());
    }
    let report = Report {
        no_new_privileges: prctl::get_no_new_privs()?,
        nondumpable: !prctl::get_dumpable()?,
        keepcaps_disabled: !prctl::get_keepcaps()?,
        capabilities_empty: rustix::thread::capabilities(/*pid*/ None)? == empty,
        ambient_empty: ambient_is_empty(&status)?,
        groups: unistd::getgroups()?.len(),
        descriptors_allowed: descriptor_allowlist()?,
    };
    if !report.verified() {
        return Err(unavailable());
    }
    Ok(report)
}

#[cfg(test)]
#[path = "hardening_tests.rs"]
mod tests;
