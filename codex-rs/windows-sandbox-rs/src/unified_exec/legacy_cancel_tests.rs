//! #1909 (#1919 review): a legacy spawn whose future is dropped (the command
//! is cancelled) while its preparation or its start runs on a blocking
//! thread closes the handles that step made: the token, and the started
//! process's process and thread handles.

use super::super::backends::legacy::test_hooks;
use super::*;
use pretty_assertions::assert_eq;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// `PUBLIC_OBJECT_BASIC_INFORMATION`.
#[repr(C)]
struct ObjectBasicInformation {
    attributes: u32,
    granted_access: u32,
    handle_count: u32,
    pointer_count: u32,
    reserved: [u32; 10],
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryObject(
        handle: HANDLE,
        class: i32,
        info: *mut c_void,
        length: u32,
        returned: *mut u32,
    ) -> i32;
}

/// How many handles, in any process, are open to the object `handle` is.
fn handle_count(handle: HANDLE) -> u32 {
    let mut info: ObjectBasicInformation = unsafe { std::mem::zeroed() };
    let mut returned = 0_u32;
    // SAFETY: a valid handle and a buffer of the class's size.
    let status = unsafe {
        NtQueryObject(
            handle,
            /*ObjectBasicInformation*/ 0,
            (&mut info as *mut ObjectBasicInformation).cast(),
            std::mem::size_of::<ObjectBasicInformation>() as u32,
            &mut returned,
        )
    };
    assert_eq!(status, 0, "NtQueryObject");
    info.handle_count
}

fn duplicate(handle: HANDLE) -> HANDLE {
    let mut copy: HANDLE = std::ptr::null_mut();
    // SAFETY: a valid handle of this process.
    let ok = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            handle,
            GetCurrentProcess(),
            &mut copy,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        )
    };
    assert_ne!(ok, 0, "DuplicateHandle");
    copy
}

/// Starts a legacy command and drops its future while the blocking step
/// `step` (0: preparation, 1: start) waits, then lets that step return.
/// For each handle the step made: the handles open to its object while it
/// waited, and once the step's result was dropped.
fn cancelled_during(step: usize) -> Vec<(u32, u32)> {
    let _guard = legacy_process_test_guard();
    let test_root = user_owned_test_root();
    let workspace = test_root.path().join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let marker = format!("rem corbanu-1909-cancel-{step}");
    // Made first so that it is dropped last: if this test panics, the
    // release sender (made below) goes first and the waiting step returns.
    let runtime = current_thread_runtime();
    let (reached_tx, mut reached_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<usize>>();
    let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
    let release_rx = std::sync::Mutex::new(release_rx);
    let calls = std::sync::atomic::AtomicUsize::new(0);
    {
        let marker = marker.clone();
        *test_hooks::AFTER_BLOCKING_STEP.lock().expect("hook") =
            Some(Arc::new(move |command: &[String], handles: &[HANDLE]| {
                if !command.iter().any(|arg| arg.contains(&marker))
                    || calls.fetch_add(1, Ordering::SeqCst) != step
                {
                    return;
                }
                // As addresses: a `HANDLE` is a raw pointer, which a channel can't send.
                let copies = handles
                    .iter()
                    .map(|handle| duplicate(*handle) as usize)
                    .collect();
                let _ = reached_tx.send(copies);
                // Until the future has been dropped.
                let _ = release_rx.lock().expect("release").recv();
            }));
    }
    let copies = runtime.block_on(async {
        let codex_home = sandbox_home("legacy-cancel");
        let profile = PermissionProfile::workspace_write();
        let roots = workspace_roots_for(&workspace);
        let spawn = spawn_windows_sandbox_session_legacy(
            &profile,
            roots.as_slice(),
            codex_home.path(),
            vec![
                "C:\\Windows\\System32\\cmd.exe".to_string(),
                "/d".to_string(),
                "/c".to_string(),
                format!("{marker} & exit /b 0"),
            ],
            &workspace,
            with_session_temp(codex_home.path(), HashMap::new()),
            /*timeout_ms*/ Some(10_000),
            &[],
            &[],
            /*tty*/ false,
            /*stdin_open*/ false,
        );
        tokio::select! {
            spawned = spawn => panic!("the spawn ended: {:?}", spawned.map(|_| ())),
            copies = reached_rx.recv() => copies.expect("the step was reached"),
        }
    });
    let waiting: Vec<u32> = copies.iter().map(|copy| handle_count(*copy as HANDLE)).collect();
    release_tx.send(()).expect("release");
    // Waits for the blocking step to return (its result is dropped).
    drop(runtime);
    *test_hooks::AFTER_BLOCKING_STEP.lock().expect("hook") = None;
    // Other processes (such as csrss) may hold a started process's handles
    // until it has fully ended, which a job closing doesn't wait for.
    let deadline = Instant::now() + Duration::from_secs(10);
    let after: Vec<u32> = copies
        .iter()
        .map(|copy| {
            loop {
                let count = handle_count(*copy as HANDLE);
                if count <= 1 || Instant::now() >= deadline {
                    break count;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        })
        .collect();
    for copy in copies {
        // SAFETY: a handle this test duplicated.
        unsafe {
            CloseHandle(copy as HANDLE);
        }
    }
    waiting.iter().copied().zip(after).collect()
}

/// Whether, for each object, only this test's copy is still open once the
/// dropped step's result was dropped (while the step waited there were
/// more, among them the step's own).
fn closed(counts: &[(u32, u32)]) -> Vec<bool> {
    counts
        .iter()
        .map(|(waiting, after)| *waiting > 1 && *after == 1)
        .collect()
}

#[test]
fn legacy_spawn_cancelled_while_preparing_closes_its_token() {
    let counts = cancelled_during(0);
    assert_eq!(closed(&counts), vec![true], "{counts:?}");
}

#[test]
fn legacy_spawn_cancelled_while_starting_closes_its_handles() {
    let counts = cancelled_during(1);
    assert_eq!(closed(&counts), vec![true; 3], "{counts:?}");
}
