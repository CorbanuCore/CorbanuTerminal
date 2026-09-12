//! Dedicated post-exec bootstrap only: never call KernelOps in a test runner.
use super::hardening;
use codex_secret_broker_service::SyntheticChildIdentity;
use nix::sys::prctl;
use nix::unistd::Gid;
use nix::unistd::Uid;
use nix::unistd::{self};
use std::io;

fn unavailable() -> io::Error {
    io::Error::other("synthetic identity preparation unavailable")
}

/// Private injection boundary for construction tests; no alternate public authority.
trait IdentityOps {
    fn root_and_single_thread(&mut self) -> io::Result<()>;
    fn restrict_exec(&mut self) -> io::Result<()>;
    fn groups(&mut self, groups: &[u32]) -> io::Result<()>;
    fn gid(&mut self, gid: u32) -> io::Result<()>;
    fn uid(&mut self, uid: u32) -> io::Result<()>;
    fn verify(&mut self, identity: &SyntheticChildIdentity) -> io::Result<()>;
}

fn apply(ops: &mut impl IdentityOps, identity: &SyntheticChildIdentity) -> io::Result<()> {
    ops.root_and_single_thread()?;
    ops.restrict_exec()?;
    ops.groups(identity.groups())?;
    ops.gid(identity.gid())?;
    ops.uid(identity.uid())?;
    ops.verify(identity)
}

fn matches_target(
    identity: &SyntheticChildIdentity,
    uid: [u32; 3],
    gid: [u32; 3],
    groups: &[u32],
) -> bool {
    uid == [identity.uid(); 3] && gid == [identity.gid(); 3] && groups == identity.groups()
}

fn exact_identity(identity: &SyntheticChildIdentity) -> io::Result<()> {
    let actual_uid = unistd::getresuid()?;
    let actual_gid = unistd::getresgid()?;
    if !matches_target(
        identity,
        [
            actual_uid.real.as_raw(),
            actual_uid.effective.as_raw(),
            actual_uid.saved.as_raw(),
        ],
        [
            actual_gid.real.as_raw(),
            actual_gid.effective.as_raw(),
            actual_gid.saved.as_raw(),
        ],
        &unistd::getgroups()?
            .iter()
            .map(|g| g.as_raw())
            .collect::<Vec<_>>(),
    ) {
        return Err(unavailable());
    }
    Ok(())
}

struct KernelOps;

impl IdentityOps for KernelOps {
    fn root_and_single_thread(&mut self) -> io::Result<()> {
        let uid = unistd::getresuid()?;
        let gid = unistd::getresgid()?;
        if [
            uid.real.as_raw(),
            uid.effective.as_raw(),
            uid.saved.as_raw(),
        ] != [0; 3]
            || [
                gid.real.as_raw(),
                gid.effective.as_raw(),
                gid.saved.as_raw(),
            ] != [0; 3]
            || !hardening::single_threaded()?
            || !hardening::descriptor_allowlist()?
        {
            return Err(unavailable());
        }
        Ok(())
    }
    fn restrict_exec(&mut self) -> io::Result<()> {
        prctl::set_no_new_privs()?;
        prctl::set_keepcaps(false)?;
        Ok(())
    }
    fn groups(&mut self, groups: &[u32]) -> io::Result<()> {
        unistd::setgroups(
            &groups
                .iter()
                .copied()
                .map(Gid::from_raw)
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }
    fn gid(&mut self, gid: u32) -> io::Result<()> {
        let gid = Gid::from_raw(gid);
        unistd::setresgid(gid, gid, gid)?;
        Ok(())
    }
    fn uid(&mut self, uid: u32) -> io::Result<()> {
        let uid = Uid::from_raw(uid);
        unistd::setresuid(uid, uid, uid)?;
        Ok(())
    }
    fn verify(&mut self, identity: &SyntheticChildIdentity) -> io::Result<()> {
        exact_identity(identity)
    }
}

pub(super) fn prepare(identity: &SyntheticChildIdentity) -> io::Result<hardening::Report> {
    apply(&mut KernelOps, identity)?;
    // Capabilities and nondumpability must be applied/rechecked AFTER dropping
    // identity. No channel is opened and no result is printed on partial failure.
    let report = hardening::inspect()?;
    exact_identity(identity)?;
    if !hardening::descriptor_allowlist()? {
        return Err(unavailable());
    }
    Ok(report)
}

#[cfg(test)]
#[path = "prepare_tests.rs"]
mod tests;
