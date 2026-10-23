//! #158, option (b) (security sign-off, Travis 2026-10-13): a writable root
//! carries the Low label only while a legacy session uses it.
//!
//! Legacy commands run at Low integrity ([`crate::integrity`]), so each
//! writable root needs an inheritable Low label. A label left in place lets
//! later Low-integrity programs delete there, such as a read-only session or
//! the PF-27-S08 credential broker in `%TEMP%`. So every session takes a
//! lease on its roots: the first lease on a root records the root's own
//! (explicit) label and labels it Low; when the last lease on it ends, the
//! recorded label is put back and propagates to the folder's children again.
//!
//! The registry is per Windows user, since sessions of different
//! `CODEX_HOME`s share roots such as `%TEMP%`:
//! `%LOCALAPPDATA%\CorbanuTerminalSandbox\integrity-labels`, labelled Medium
//! with no-write-up and no-read-up, so no Low-integrity program can write,
//! read or hold open anything in it. It holds:
//! - `labels.json`: each labelled root and the label to put back;
//! - one `session-*.json` per live session, listing its roots, held open
//!   without delete sharing for the session's lifetime. A file that can be
//!   deleted belongs to a session that ended without releasing its lease (a
//!   crash): its roots no longer count;
//! - `registry.lock`, held exclusively while the registry changes.
//!
//! A lease ends when the command's last process exits, so processes it
//! leaves running (a server, say) keep writing until they exit too.
//!
//! Every acquire, every release and the startup sweep
//! ([`repair_stale_integrity_labels`]) put back the label of each recorded
//! root that no live session holds.

use crate::integrity::LabelAce;
use crate::integrity::effective_label_is_low;
use crate::integrity::label_private_dir_medium;
use crate::integrity::label_writable_root_low;
use crate::integrity::read_explicit_labels;
use crate::integrity::set_explicit_labels;
use crate::logging::log_note;
use anyhow::Context;
use anyhow::Result;
use codex_utils_pty::JobObject;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::os::windows::fs::MetadataExt as _;
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

const REGISTRY_PARENT: &str = "CorbanuTerminalSandbox";
const REGISTRY_DIR: &str = "integrity-labels";
const LOCK_FILE: &str = "registry.lock";
const LABELS_FILE: &str = "labels.json";
const SESSION_PREFIX: &str = "session-";
const SESSION_SUFFIX: &str = ".json";
/// Labelling a large tree under the lock can take a while.
const LOCK_WAIT: Duration = Duration::from_secs(600);
const ERROR_SHARING_VIOLATION: i32 = 32;
const FILE_SHARE_READ: u32 = 0x1;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

/// Each labelled root and its own label before the first lease.
#[derive(Default, Serialize, Deserialize)]
struct Labels {
    roots: BTreeMap<PathBuf, Vec<LabelAce>>,
}

#[derive(Serialize, Deserialize)]
struct SessionRecord {
    roots: Vec<PathBuf>,
}

/// A live session's hold on its roots' Low label; released on drop.
pub(crate) struct IntegrityLease {
    registry: PathBuf,
    session: Option<(File, PathBuf)>,
    log_dir: Option<PathBuf>,
}

/// `%LOCALAPPDATA%\CorbanuTerminalSandbox`, which a session whose writable
/// root holds it protects like `.git`.
pub(crate) fn registry_parent_dir() -> Option<PathBuf> {
    local_app_data().map(|dir| dir.join(REGISTRY_PARENT))
}

fn registry_dir() -> Option<PathBuf> {
    registry_parent_dir().map(|dir| dir.join(REGISTRY_DIR))
}

fn local_app_data() -> Option<PathBuf> {
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::FOLDERID_LocalAppData;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;
    let mut path: *mut u16 = std::ptr::null_mut();
    // SAFETY: `path` receives a string the call allocates, freed below.
    let status = unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, 0, 0, &mut path) };
    let result = (status == 0 && !path.is_null()).then(|| {
        // SAFETY: a NUL-terminated string from the call.
        let len = (0..).take_while(|&i| unsafe { *path.add(i) } != 0).count();
        // SAFETY: as above.
        PathBuf::from(String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts(path, len)
        }))
    });
    // SAFETY: allocated by SHGetKnownFolderPath (null is allowed).
    unsafe { CoTaskMemFree(path.cast()) };
    result
}

/// Labels `roots` (canonical paths) Low for a session, until the lease is
/// dropped. A root another live session already holds keeps its label.
pub(crate) fn acquire(roots: &[PathBuf], log_dir: Option<&Path>) -> Result<IntegrityLease> {
    let registry = ensure_registry()?;
    let lock = lock(&registry)?;
    let mut lease = IntegrityLease {
        registry,
        session: None,
        log_dir: log_dir.map(Path::to_path_buf),
    };
    let wanted: BTreeSet<PathBuf> = roots.iter().cloned().collect();
    let result = acquire_locked(&mut lease, &wanted);
    drop(lock);
    // On failure the lease drops here and puts back what it labelled.
    result.map(|()| lease)
}

fn acquire_locked(lease: &mut IntegrityLease, wanted: &BTreeSet<PathBuf>) -> Result<()> {
    // Registered first: whatever happens below, a sweep can undo it.
    lease.session = Some(create_session(&lease.registry, wanted)?);
    let mut labels = load_labels(&lease.registry)?;
    sweep(&lease.registry, &mut labels, lease.log_dir.as_deref())?;
    for root in wanted {
        // A root that can't be labelled stays as it is: commands can't write
        // there, which is safe, and the others still work.
        let leased = lease_root(&lease.registry, &mut labels, root);
        // #734: core tells the person about a root left read-only.
        crate::unlabelled_roots::record(root, leased.is_ok());
        if let Err(err) = leased {
            log_note(
                &format!(
                    "integrity labels: writable root {} can't be labelled Low, so sandboxed commands can't write there (labelling needs permission to change the folder's owner settings, WRITE_OWNER, and a file system that holds integrity labels): {err:#}",
                    root.display()
                ),
                lease.log_dir.as_deref(),
            );
        }
    }
    Ok(())
}

fn lease_root(registry: &Path, labels: &mut Labels, root: &Path) -> Result<()> {
    let new = !labels.roots.contains_key(root);
    if new {
        // SAFETY: reads the label of an existing canonical root.
        let original = unsafe { read_explicit_labels(root) }
            .with_context(|| format!("read the integrity label of {}", root.display()))?;
        labels.roots.insert(root.to_path_buf(), original);
        // Recorded before it changes, so a crash cannot lose it.
        save_labels(registry, labels)?;
    }
    // SAFETY: labels an existing canonical root.
    if let Err(err) = unsafe { label_writable_root_low(root) } {
        // Unchanged: nothing to put back later.
        // SAFETY: as above.
        if new && unsafe { read_explicit_labels(root) }.is_ok_and(|now| labels.roots[root] == now) {
            labels.roots.remove(root);
            save_labels(registry, labels)?;
        }
        return Err(err);
    }
    Ok(())
}

impl IntegrityLease {
    /// Ends the lease once no process of the command's `job` runs: right
    /// away when none does, so a caller that reports the exit afterwards
    /// finds the labels back; otherwise, when `descendants_preserved` (the
    /// command exited and left processes running on purpose), on a thread of
    /// its own when the last of them exits. Without it (the job timed out, or
    /// still kills its processes when its handle closes), what is left of the
    /// job is ended now, so nothing Low keeps running in the roots while
    /// their label is put back, and the release keeps no reference to the job.
    pub(crate) fn release_after_job(self, job: Arc<JobObject>, descendants_preserved: bool) {
        if !descendants_preserved {
            let _ = job.terminate();
        }
        if wait_for_empty_job(&job, Duration::from_secs(/*secs*/ 2)) || !descendants_preserved {
            drop(self);
            return;
        }
        std::thread::spawn(move || {
            while !wait_for_empty_job(&job, Duration::from_secs(/*secs*/ 1)) {}
            drop(self);
        });
    }

    fn release(&mut self) -> Result<()> {
        let Some((file, path)) = self.session.take() else {
            return Ok(());
        };
        // If the lock can't be had, the closed session file is stale and
        // the next sweep puts the labels back.
        let lock = lock(&self.registry)?;
        drop(file);
        // Closed now: a sweep that finds it left behind removes it.
        let _ = std::fs::remove_file(&path);
        let mut labels = load_labels(&self.registry)?;
        sweep(&self.registry, &mut labels, self.log_dir.as_deref())?;
        drop(lock);
        Ok(())
    }
}

impl Drop for IntegrityLease {
    fn drop(&mut self) {
        if let Err(err) = self.release() {
            log_note(
                &format!(
                    "integrity labels: could not end this session's lease; the next Windows sandbox command or start puts the labels back: {err:#}"
                ),
                self.log_dir.as_deref(),
            );
        }
    }
}

/// Whether `job` has no process left within `timeout` (or can't be asked).
fn wait_for_empty_job(job: &JobObject, timeout: Duration) -> bool {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::JOBOBJECT_BASIC_ACCOUNTING_INFORMATION;
    use windows_sys::Win32::System::JobObjects::JobObjectBasicAccountingInformation;
    use windows_sys::Win32::System::JobObjects::QueryInformationJobObject;
    let deadline = Instant::now() + timeout;
    loop {
        // SAFETY: a zeroed plain-data struct the call fills in.
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: `job` holds a valid job handle; the buffer and its size match.
        let ok = unsafe {
            QueryInformationJobObject(
                job.as_raw_handle() as HANDLE,
                JobObjectBasicAccountingInformation,
                &mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION as *mut std::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 || info.ActiveProcesses == 0 {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Puts back the label of every root that only sessions which ended without
/// releasing their lease (a crash) had labelled Low. Returns how many roots
/// it put back. Run at startup.
pub fn repair_stale_integrity_labels() -> Result<usize> {
    let Some(registry) = registry_dir() else {
        return Ok(0);
    };
    if std::fs::symlink_metadata(&registry).is_err() {
        return Ok(0);
    }
    let registry = ensure_registry()?;
    let lock = lock(&registry)?;
    let mut labels = load_labels(&registry)?;
    let restored = sweep(&registry, &mut labels, /*log_dir*/ None)?;
    drop(lock);
    Ok(restored)
}

/// Puts back the label of each recorded root no live session holds, and
/// stores the result. Returns how many it put back.
fn sweep(registry: &Path, labels: &mut Labels, log_dir: Option<&Path>) -> Result<usize> {
    let mut live = BTreeSet::new();
    for entry in
        std::fs::read_dir(registry).with_context(|| format!("list {}", registry.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !(name.starts_with(SESSION_PREFIX) && name.ends_with(SESSION_SUFFIX)) {
            continue;
        }
        let path = entry.path();
        let record = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SessionRecord>(&bytes).ok());
        match std::fs::remove_file(&path) {
            // Its session ended without releasing the lease.
            Ok(()) => {}
            Err(err) => match record {
                Some(record) if err.raw_os_error() == Some(ERROR_SHARING_VIOLATION) => {
                    live.extend(record.roots);
                }
                // A session file that can't be read or probed may belong to
                // a live session: keep every label this time.
                _ => {
                    log_note(
                        &format!(
                            "integrity labels: cannot tell whether {} is live ({err}); no label is put back",
                            path.display()
                        ),
                        log_dir,
                    );
                    return Ok(0);
                }
            },
        }
    }
    let stale: Vec<PathBuf> = labels
        .roots
        .keys()
        .filter(|root| !live.contains(*root))
        .cloned()
        .collect();
    let mut restored = 0;
    for root in stale {
        let original = labels.roots[&root].clone();
        match restore(&root, &original) {
            Ok(()) => {
                labels.roots.remove(&root);
                restored += 1;
            }
            // Kept recorded; a later sweep tries again.
            Err(err) => log_note(
                &format!(
                    "integrity labels: could not put back the label of {}: {err:#}",
                    root.display()
                ),
                log_dir,
            ),
        }
    }
    save_labels(registry, labels)?;
    Ok(restored)
}

fn restore(root: &Path, original: &[LabelAce]) -> Result<()> {
    match std::fs::symlink_metadata(root) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("inspect {}", root.display())),
        // A link now stands where the root was: never relabel what it points at.
        Ok(meta) if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 => Ok(()),
        // SAFETY: relabels an existing folder or file that is not a link.
        Ok(_) => unsafe { set_explicit_labels(root, original) },
    }
}

/// Creates the registry folders, or checks them: plain folders, not at Low
/// integrity (a Low-integrity program would have made them), with their
/// private Medium label.
fn ensure_registry() -> Result<PathBuf> {
    let parent = registry_parent_dir().context("no LocalAppData folder")?;
    let registry = parent.join(REGISTRY_DIR);
    for dir in [&parent, &registry] {
        match std::fs::symlink_metadata(dir) {
            Ok(meta) => {
                if !meta.is_dir() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    anyhow::bail!(
                        "{} is not a plain folder, so the Windows sandbox can't keep its integrity label records there; remove it",
                        dir.display()
                    );
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(dir).with_context(|| format!("create {}", dir.display()))?;
            }
            Err(err) => return Err(err).with_context(|| format!("inspect {}", dir.display())),
        }
        // SAFETY: reads and sets the label of a plain folder of the host's.
        unsafe {
            if effective_label_is_low(dir)? {
                anyhow::bail!(
                    "{} has a Low integrity label, so a sandboxed command may have made it; remove it",
                    dir.display()
                );
            }
            label_private_dir_medium(dir)
                .with_context(|| format!("label {} Medium", dir.display()))?;
        }
    }
    Ok(registry)
}

fn lock(registry: &Path) -> Result<File> {
    let path = registry.join(LOCK_FILE);
    let deadline = Instant::now() + LOCK_WAIT;
    loop {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)
        {
            Ok(file) => return Ok(file),
            Err(err)
                if err.raw_os_error() == Some(ERROR_SHARING_VIOLATION)
                    && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(err) => return Err(err).with_context(|| format!("lock {}", path.display())),
        }
    }
}

fn create_session(registry: &Path, roots: &BTreeSet<PathBuf>) -> Result<(File, PathBuf)> {
    let mut rng = SmallRng::from_entropy();
    let path = registry.join(format!(
        "{SESSION_PREFIX}{}-{:016x}{SESSION_SUFFIX}",
        std::process::id(),
        rng.r#gen::<u64>()
    ));
    // No delete sharing: while this handle is open, the file can't be
    // deleted, which is how sweeps tell the session is live.
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&path)
        .with_context(|| format!("create {}", path.display()))?;
    let record = SessionRecord {
        roots: roots.iter().cloned().collect(),
    };
    file.write_all(&serde_json::to_vec(&record)?)?;
    file.flush()?;
    Ok((file, path))
}

fn load_labels(registry: &Path) -> Result<Labels> {
    let path = registry.join(LABELS_FILE);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).with_context(|| {
            format!(
                "{} is damaged, so the Windows sandbox doesn't know which folders' integrity labels to put back; remove it, and give each writable root you used (%TEMP% included, and any nested root) a Medium label again (icacls <folder> /setintegritylevel (OI)(CI)M)",
                path.display()
            )
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Labels::default()),
        Err(err) => Err(err).with_context(|| format!("read {}", path.display())),
    }
}

fn save_labels(registry: &Path, labels: &Labels) -> Result<()> {
    let path = registry.join(LABELS_FILE);
    let tmp = registry.join(format!("{LABELS_FILE}.tmp"));
    // Flushed before it replaces the old records, so a power loss cannot
    // leave damaged records behind.
    let mut file = File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
    file.write_all(&serde_json::to_vec_pretty(labels)?)
        .and_then(|()| file.sync_all())
        .with_context(|| format!("write {}", tmp.display()))?;
    drop(file);
    std::fs::rename(&tmp, &path).with_context(|| format!("replace {}", path.display()))
}

/// Tests only: the recorded roots.
#[cfg(test)]
pub(crate) fn recorded_roots() -> Result<Vec<PathBuf>> {
    let registry = ensure_registry()?;
    let lock = lock(&registry)?;
    let labels = load_labels(&registry)?;
    drop(lock);
    Ok(labels.roots.into_keys().collect())
}
