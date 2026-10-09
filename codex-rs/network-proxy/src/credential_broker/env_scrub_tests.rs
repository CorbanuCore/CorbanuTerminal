use super::*;
use pretty_assertions::assert_eq;

#[test]
fn pf_27_s05_take_env_var_returns_and_removes_the_value() {
    let name = format!("PF27_S05_SCRUB_{}", std::process::id());
    set_env_var_for_test(&name, "synthetic-scrub-value");
    let value = take_env_var(&name).expect("value");
    assert_eq!(value.as_slice(), b"synthetic-scrub-value");
    assert_eq!(std::env::var_os(&name), None);
    assert!(take_env_var(&name).is_none());
}

#[test]
fn pf_27_s05_take_env_var_ignores_unset_empty_and_invalid_names() {
    let name = format!("PF27_S05_EMPTY_{}", std::process::id());
    set_env_var_for_test(&name, "");
    assert!(take_env_var(&name).is_none());
    assert_eq!(
        std::env::var_os(&name).as_deref(),
        Some(std::ffi::OsStr::new(""))
    );
    assert!(take_env_var("PF27_S05_NEVER_SET_XYZ").is_none());
    assert!(take_env_var("A=B").is_none());
    assert!(take_env_var("").is_none());
}

const RACE_CHILD_ENV: &str = "CODEX_PF27_S05_ENV_RACE_CHILD";
const RACE_CHILD_TEST: &str = "credential_broker::env_scrub::tests::pf_27_s05_env_race_child_entry";

/// Hammers `take_env_var` with concurrent `setenv`s. Runs only in the child
/// started below, so a crash cannot take down the other tests' process.
#[test]
fn pf_27_s05_env_race_child_entry() {
    if std::env::var_os(RACE_CHILD_ENV).is_none() {
        return;
    }
    let workers: Vec<_> = (0..4)
        .map(|worker| {
            std::thread::spawn(move || {
                // Fewer rounds on Windows, where each take also sweeps the
                // process heaps and the parameters' allocation (PF-27-S09).
                for round in 0..if cfg!(windows) { 300 } else { 2_000 } {
                    // A fresh name each round grows (and reallocates) `environ`.
                    let name = format!("PF27_S05_RACE_{worker}_{round}");
                    // Distinct values, formatted again only after the take:
                    // on Windows a take overwrites every other copy of its
                    // value in the process (PF-27-S09).
                    let value = || format!("synthetic-race-value-{worker}-{round}");
                    set_env_var_for_test(&name, &value());
                    let taken = take_env_var(&name).expect("value");
                    assert_eq!(taken.as_slice(), value().as_bytes());
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker");
    }
}

/// Regression for the credential-canary crash (issue #222): `setenv` on one
/// thread reallocated `environ` while another walked it in `take_env_var`,
/// and the whole test binary died with SIGSEGV. Without `env_write_lock` in
/// `take_env_var` the child crashes within milliseconds.
#[test]
fn pf_27_s05_take_env_var_is_serialized_with_concurrent_env_writes() {
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([RACE_CHILD_TEST, "--exact", "--test-threads=1"])
        .env(RACE_CHILD_ENV, "1")
        .output()
        .expect("run race child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "race child: {:?}\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
const CRT_CHILD_ENV: &str = "CODEX_PF27_S09_CRT_CHILD";
#[cfg(windows)]
const CRT_KEY_ENV: &str = "PF27_S09_CRT_KEY";
#[cfg(windows)]
const CRT_CHILD_TEST: &str = "credential_broker::env_scrub::tests::pf_27_s09_crt_copy_child_entry";

/// Runs in a child whose launch environment holds the key, so the C runtime
/// made its own copy at startup, as it does in Core.
#[cfg(windows)]
#[test]
fn pf_27_s09_crt_copy_child_entry() {
    if std::env::var_os(CRT_CHILD_ENV).is_none() {
        return;
    }
    unsafe extern "C" {
        fn getenv(name: *const std::ffi::c_char) -> *const std::ffi::c_char;
    }
    let crt_value = || {
        let name = std::ffi::CString::new(CRT_KEY_ENV).expect("name");
        // SAFETY: a NUL-terminated name; the result is copied at once.
        let value = unsafe { getenv(name.as_ptr()) };
        (!value.is_null()).then(|| {
            // SAFETY: the C runtime returns a NUL-terminated string.
            unsafe { std::ffi::CStr::from_ptr(value) }
                .to_string_lossy()
                .into_owned()
        })
    };
    assert_eq!(crt_value().as_deref(), Some("synthetic-crt-value"));
    let value = take_env_var(CRT_KEY_ENV).expect("value");
    assert_eq!(value.as_slice(), b"synthetic-crt-value");
    assert_eq!(std::env::var_os(CRT_KEY_ENV), None);
    // The C runtime's copy is overwritten, then dropped.
    assert_eq!(crt_value(), None);
}

/// PF-27-S09: on Windows the value is also wiped from (and the variable
/// dropped from) the C runtime's copy of the environment, which
/// `SetEnvironmentVariableW` does not update.
#[cfg(windows)]
#[test]
fn pf_27_s09_take_env_var_wipes_the_c_runtime_copy() {
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([CRT_CHILD_TEST, "--exact", "--test-threads=1"])
        .env(CRT_CHILD_ENV, "1")
        .env(CRT_KEY_ENV, "synthetic-crt-value")
        .output()
        .expect("run CRT child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "CRT child: {:?}\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Writes `masked ^ 0x5a` into `live`'s spare capacity at `offset`, as a
/// stale copy in reused memory would sit there.
#[cfg(windows)]
fn plant(live: &mut Vec<u8>, offset: usize, masked: &[u8]) {
    assert!(offset + masked.len() <= live.capacity());
    for (index, byte) in masked.iter().enumerate() {
        // SAFETY: within the vector's allocation.
        unsafe { live.as_mut_ptr().add(offset + index).write(byte ^ 0x5a) };
    }
}

/// The bytes at `offset` in `live`'s allocation.
#[cfg(windows)]
fn planted(live: &[u8], offset: usize, len: usize) -> Vec<u8> {
    // SAFETY: written by `plant`, within the allocation.
    unsafe { std::slice::from_raw_parts(live.as_ptr().add(offset), len) }.to_vec()
}

/// PF-27-S09: a stale copy of a handed-over key in a block in use (no
/// `NAME=` before it) is overwritten with `0` characters, never NUL; a short
/// placeholder value is left alone. Values are kept masked here so the test
/// holds no plain copy of its own.
#[cfg(windows)]
#[test]
fn pf_27_s09_take_env_var_overwrites_stale_copies_in_live_blocks() {
    let unmask = |masked: &[u8]| {
        masked
            .iter()
            .map(|byte| char::from(byte ^ 0x5a))
            .collect::<String>()
    };
    let mask = |plain: &str| plain.bytes().map(|byte| byte ^ 0x5a).collect::<Vec<u8>>();
    let key = mask(&format!("sk-pf27s09-stale-{:016x}", rand::random::<u64>()));
    let placeholder = mask("dummy-key-9");
    let mut live: Vec<u8> = Vec::with_capacity(4096);
    plant(&mut live, 1000, &key);
    plant(&mut live, 2000, &placeholder);

    let key_name = format!("PF27_S09_STALE_KEY_{}", std::process::id());
    let placeholder_name = format!("PF27_S09_STALE_PLACEHOLDER_{}", std::process::id());
    set_env_var_for_test(&key_name, &unmask(&key));
    set_env_var_for_test(&placeholder_name, &unmask(&placeholder));
    let taken = take_env_var(&key_name).expect("key");
    assert_eq!(mask(std::str::from_utf8(&taken).expect("utf-8")), key);
    let taken = take_env_var(&placeholder_name).expect("placeholder");
    assert_eq!(
        mask(std::str::from_utf8(&taken).expect("utf-8")),
        placeholder
    );

    assert_eq!(planted(&live, 1000, key.len()), vec![b'0'; key.len()]);
    assert_eq!(
        planted(&live, 2000, placeholder.len()),
        unmask(&placeholder).into_bytes()
    );
}

/// PF-27-S09: a `NAME=value` entry outside the heaps, in an allocation the
/// sweep is pointed at (in production: the process parameters' allocation,
/// which keeps the environment block Windows replaced when the environment
/// grew), gets `0` characters for its value.
#[cfg(windows)]
#[test]
fn pf_27_s09_entries_outside_the_heaps_are_overwritten() {
    use windows_sys::Win32::System::Memory::MEM_COMMIT;
    use windows_sys::Win32::System::Memory::MEM_RELEASE;
    use windows_sys::Win32::System::Memory::MEM_RESERVE;
    use windows_sys::Win32::System::Memory::PAGE_READWRITE;
    use windows_sys::Win32::System::Memory::VirtualAlloc;
    use windows_sys::Win32::System::Memory::VirtualFree;
    let mask = |plain: &str| plain.bytes().map(|byte| byte ^ 0x5a).collect::<Vec<u8>>();
    let key = mask(&format!("sk-pf27s09-block-{:016x}", rand::random::<u64>()));
    let name = format!("PF27_S09_BLOCK_{}", std::process::id());
    // SAFETY: a fresh private page, released below.
    let page = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            4096,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    }
    .cast::<u16>();
    assert!(!page.is_null(), "page");
    let entry: Vec<u16> = format!("{name}=")
        .encode_utf16()
        .chain(key.iter().map(|byte| u16::from(byte ^ 0x5a)))
        .chain(std::iter::once(0))
        .collect();
    let offset = 100;
    // SAFETY: within the committed page.
    unsafe { std::ptr::copy_nonoverlapping(entry.as_ptr(), page.add(offset), entry.len()) };
    drop(entry);

    let plain: String = key.iter().map(|byte| char::from(byte ^ 0x5a)).collect();
    set_env_var_for_test(&name, &plain);
    drop(plain);
    let taken = take_env_var(&name).expect("value");
    assert_eq!(mask(std::str::from_utf8(&taken).expect("utf-8")), key);
    let wide: Vec<u8> = std::str::from_utf8(&taken)
        .expect("utf-8")
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    super::windows_env::wipe_entries_in_allocation(page as usize, &name, &[&wide]);
    drop((taken, wide));
    assert!(super::windows_env::process_parameters_allocation().is_some());

    let start = offset + name.len() + 1;
    // SAFETY: within the committed page.
    let left = unsafe { std::slice::from_raw_parts(page.add(start), key.len()) }.to_vec();
    // SAFETY: allocated above.
    unsafe { VirtualFree(page.cast(), 0, MEM_RELEASE) };
    assert_eq!(left, vec![u16::from(b'0'); key.len()]);
}
