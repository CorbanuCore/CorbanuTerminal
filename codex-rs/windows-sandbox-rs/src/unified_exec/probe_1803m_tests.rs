//! PROBE ONLY (win-1803m, never merged): #1803. Workspace A runs an elevated
//! command (in a ConPTY, and without one for comparison) that prints a known
//! output marker and reads a typed input marker, as a prompt would. Workspace
//! B's elevated command then opens A's command, runner and the runner's
//! console hosts for read rights, scans their memory for both markers, and
//! opens their threads for `THREAD_GET_CONTEXT`. Reports only (`W1803M|`
//! lines); asserts nothing.

use super::*;
use windows_sys::Win32::System::Diagnostics::Debug::CONTEXT;
use windows_sys::Win32::System::Diagnostics::Debug::GetThreadContext;
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::System::Memory::MEM_COMMIT;
use windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
use windows_sys::Win32::System::Memory::PAGE_GUARD;
use windows_sys::Win32::System::Memory::PAGE_NOACCESS;
use windows_sys::Win32::System::Memory::VirtualQueryEx;
use windows_sys::Win32::System::Threading::OpenThread;

const HOLD_1803: &str =
    "unified_exec::tests::command_identity::elevated::probe_1803m::probe_1803m_hold_entry";
const PROBE_1803: &str =
    "unified_exec::tests::command_identity::elevated::probe_1803m::probe_1803m_probe_entry";
const HOLD_DIR_ENV: &str = "CODEX_W1803M_HOLD";
const TARGET_PID_ENV: &str = "CODEX_W1803M_TARGET";
const MARKER_ENV: &str = "CODEX_W1803M_MARKER";
const OUT_FILE_ENV: &str = "CODEX_W1803M_OUT";
/// Upper bound on the bytes read from one process.
const SCAN_CAP: usize = 1 << 30;

fn session() -> &'static str {
    if crate::setup::is_elevated().expect("elevation") {
        "admin"
    } else {
        "normal"
    }
}

/// Workspace A's command: prints the output marker, reads one typed line
/// (a ConPTY's cooked mode echoes it), then waits for its release file.
#[test]
fn probe_1803m_hold_entry() {
    let Some(dir) = std::env::var_os(HOLD_DIR_ENV).map(PathBuf::from) else {
        return;
    };
    let id = fs::read_to_string(dir.join("marker")).expect("marker");
    println!("W1803OUT{}", id.trim());
    fs::write(dir.join("hold.pid"), std::process::id().to_string()).expect("write hold pid");
    let mut line = String::new();
    let got = std::io::stdin().read_line(&mut line);
    fs::write(
        dir.join("hold.got"),
        format!("read={:?} len={}", got.map_err(|e| e.kind()), line.trim().len()),
    )
    .expect("write hold got");
    wait_for_path(&dir.join("hold.release"), Duration::from_secs(/*secs*/ 180));
}

/// Workspace B's command: reports on A's command, its runner and the
/// runner's console hosts.
#[test]
fn probe_1803m_probe_entry() {
    let Some(target) = std::env::var_os(TARGET_PID_ENV) else {
        return;
    };
    let pid: u32 = target.to_string_lossy().parse().expect("numeric pid");
    let id = std::env::var(MARKER_ENV).expect("marker id");
    let markers = [
        ("out", format!("W1803OUT{id}")),
        ("in", format!("W1803IN{id}")),
    ];
    let (runner, runner_image) = parent_of(pid);
    let mut targets = vec![
        ("command".to_string(), pid),
        (format!("runner:{runner_image}"), runner),
    ];
    for (host, image) in console_hosts_started_by(runner) {
        targets.push((format!("host:{image}"), host));
    }
    let line = targets
        .iter()
        .map(|(name, pid)| format!("target={name} {}", read_report(*pid, &markers)))
        .collect::<Vec<_>>()
        .join(" || ");
    if let Some(out) = std::env::var_os(OUT_FILE_ENV) {
        fs::write(out, &line).expect("write probe result");
    }
    println!("\nW1803PROBE {line}");
}

fn read_report(pid: u32, markers: &[(&str, String)]) -> String {
    let mut parts = vec![format!("dacl={}", process_dacl_sddl(pid))];
    for (name, access) in [
        ("query_limited", 0x1000),
        ("query_info", 0x0400),
        ("vm_read", 0x0010),
        ("vm_read_query", 0x0410),
        ("vm_write", 0x0028),
        ("dup_handle", 0x0040),
    ] {
        parts.push(format!("{name}={}", open_result(pid, access)));
    }
    parts.push(scan_report(pid, markers));
    parts.push(threads_report(pid));
    parts.join(" ")
}

/// Reads every committed, readable region of `pid` (up to [`SCAN_CAP`]) and
/// says which markers it holds, as ASCII and as UTF-16.
fn scan_report(pid: u32, markers: &[(&str, String)]) -> String {
    let needles: Vec<(String, Vec<u8>)> = markers
        .iter()
        .flat_map(|(name, text)| {
            let wide: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
            [
                (format!("{name}_ascii"), text.as_bytes().to_vec()),
                (format!("{name}_utf16"), wide),
            ]
        })
        .collect();
    let overlap = needles.iter().map(|(_, n)| n.len()).max().unwrap_or(0);
    let mut found = vec![0usize; needles.len()];
    let (mut read, mut regions, mut failed) = (0usize, 0usize, 0usize);
    // SAFETY: the handle is closed below; every buffer outlives its call.
    unsafe {
        let process = windows_sys::Win32::System::Threading::OpenProcess(0x0410, 0, pid);
        if process == 0 {
            return format!("scan=open_err{}", GetLastError());
        }
        let mut address = 0usize;
        let mut buffer = vec![0u8; 1 << 20];
        loop {
            let mut info: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
            if VirtualQueryEx(
                process,
                address as *const _,
                &mut info,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            ) == 0
            {
                break;
            }
            let base = info.BaseAddress as usize;
            let size = info.RegionSize;
            if info.State == MEM_COMMIT
                && info.Protect & (PAGE_GUARD | PAGE_NOACCESS) == 0
                && info.Protect != 0
                && read < SCAN_CAP
            {
                regions += 1;
                let mut carry: Vec<u8> = Vec::new();
                let mut offset = 0usize;
                while offset < size {
                    let chunk = (size - offset).min(buffer.len());
                    let mut got = 0usize;
                    if ReadProcessMemory(
                        process,
                        (base + offset) as *const _,
                        buffer.as_mut_ptr().cast(),
                        chunk,
                        &mut got,
                    ) == 0
                        || got == 0
                    {
                        failed += 1;
                        break;
                    }
                    read += got;
                    let kept = carry.len();
                    carry.extend_from_slice(&buffer[..got]);
                    for (index, (_, needle)) in needles.iter().enumerate() {
                        found[index] += count(&carry, needle, kept);
                    }
                    let keep = carry.len().min(overlap.saturating_sub(1));
                    carry.drain(..carry.len() - keep);
                    offset += got;
                }
            }
            let next = base.saturating_add(size);
            if next <= address {
                break;
            }
            address = next;
        }
        CloseHandle(process);
    }
    let hits = needles
        .iter()
        .zip(&found)
        .map(|((name, _), n)| format!("{name}={n}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("scan=ok bytes_read={read} regions={regions} region_read_errors={failed} {hits}")
}

/// Occurrences of `needle` in `haystack` that end past `kept` (the tail
/// carried over from the previous chunk, already searched).
fn count(haystack: &[u8], needle: &[u8], kept: usize) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    let first = needle[0];
    (0..=haystack.len() - needle.len())
        .filter(|&i| {
            i + needle.len() > kept
                && haystack[i] == first
                && &haystack[i..i + needle.len()] == needle
        })
        .count()
}

/// How many of `pid`'s threads open for `THREAD_GET_CONTEXT` and
/// `THREAD_QUERY_INFORMATION`, and whether `GetThreadContext` succeeds.
fn threads_report(pid: u32) -> String {
    let threads = threads_of(&std::collections::HashSet::from([pid]));
    let (mut get_context, mut query, mut context_read) = (0, 0, 0);
    let mut first_error = String::new();
    for (thread, _) in &threads {
        // SAFETY: each handle is closed right away; the context is aligned
        // by its type.
        unsafe {
            let handle = OpenThread(0x0040, 0, *thread);
            if handle != 0 {
                query += 1;
                CloseHandle(handle);
            }
            let handle = OpenThread(0x0008, 0, *thread);
            if handle == 0 {
                if first_error.is_empty() {
                    first_error = format!("err{}", GetLastError());
                }
                continue;
            }
            get_context += 1;
            let mut context: CONTEXT = std::mem::zeroed();
            // CONTEXT_CONTROL | CONTEXT_INTEGER (AMD64).
            context.ContextFlags = 0x0010_0003;
            if GetThreadContext(handle, &mut context) != 0 {
                context_read += 1;
            } else if first_error.is_empty() {
                first_error = format!("ctx_err{}", GetLastError());
            }
            CloseHandle(handle);
        }
    }
    format!(
        "threads={} thread_query={query} thread_get_context={get_context} context_read={context_read} thread_first_error={first_error}",
        threads.len()
    )
}

fn measure(tty: bool, codex_home: &Path) {
    let holder_root = user_owned_test_root();
    let root = user_owned_test_root();
    let hold_dir = holder_root.path().join("hold1803");
    fs::create_dir_all(&hold_dir).expect("hold dir");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let id = format!("{:016x}", (nanos as u64) ^ (u64::from(std::process::id()) << 40));
    fs::write(hold_dir.join("marker"), &id).expect("write marker");
    let out = root.path().join("probe1803.out");
    let runtime = current_thread_runtime();
    let (line, got) = runtime.block_on(async {
        let release = Release(hold_dir.join("hold.release"));
        let holder = spawn_elevated(
            holder_root.path(),
            codex_home,
            probe_argv(HOLD_1803),
            &[(HOLD_DIR_ENV, hold_dir.display().to_string())],
            tty,
        )
        .await;
        let deadline = Instant::now() + Duration::from_secs(/*secs*/ 60);
        let pid = loop {
            if let Ok(pid) = fs::read_to_string(hold_dir.join("hold.pid"))
                && let Ok(pid) = pid.trim().parse::<u32>()
            {
                break Some(pid);
            }
            if Instant::now() >= deadline {
                break None;
            }
            tokio::time::sleep(Duration::from_millis(/*millis*/ 100)).await;
        };
        let Some(pid) = pid else {
            drop(release);
            let (stdout, code) =
                collect_stdout_and_exit(holder, codex_home, Duration::from_secs(/*secs*/ 90))
                    .await;
            return (
                format!(
                    "holder never started exit={code} {}",
                    String::from_utf8_lossy(&stdout)
                ),
                String::new(),
            );
        };
        if tty {
            // Typed as a user would, Enter included.
            let _ = holder
                .session
                .writer_sender()
                .send(format!("W1803IN{id}\r").into_bytes())
                .await;
        }
        let got_path = hold_dir.join("hold.got");
        wait_for_path(&got_path, Duration::from_secs(/*secs*/ 20));
        tokio::time::sleep(Duration::from_secs(/*secs*/ 1)).await;
        let got = fs::read_to_string(&got_path).unwrap_or_else(|_| "no hold.got".to_string());
        let (stdout, code) = run_elevated(
            root.path(),
            codex_home,
            probe_argv(PROBE_1803),
            &[
                (TARGET_PID_ENV, pid.to_string()),
                (MARKER_ENV, id.clone()),
                (OUT_FILE_ENV, out.display().to_string()),
            ],
            /*tty*/ false,
        )
        .await;
        let line = fs::read_to_string(&out)
            .unwrap_or_else(|_| format!("no result exit={code} {stdout}"));
        drop(release);
        let _ = collect_stdout_and_exit(holder, codex_home, Duration::from_secs(/*secs*/ 90)).await;
        (line, got)
    });
    let mode = if tty { "conpty" } else { "pipes" };
    println!("\nW1803M|{}|{mode}|holder {got}", session());
    for part in line.split(" || ") {
        println!("\nW1803M|{}|{mode}|{part}", session());
    }
}

#[test]
fn probe_1803m_elevated_cross_workspace_read() {
    let Some(codex_home) = elevated_home("w1803m") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    measure(/*tty*/ true, codex_home.path());
    measure(/*tty*/ false, codex_home.path());
    record_setup(codex_home.path());
}
