use std::fs::OpenOptions;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const PANIC_DIAGNOSTIC_FILE_NAME: &str = "tui-panics.log";

static DIAGNOSTIC_PATH: OnceLock<PathBuf> = OnceLock::new();
static TERMINAL_OWNED: AtomicBool = AtomicBool::new(false);
static TERMINAL_OWNER_THREAD: Mutex<Option<std::thread::ThreadId>> = Mutex::new(None);
static PANIC_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static PENDING_PANICS: Mutex<Vec<PendingPanic>> = Mutex::new(Vec::new());

#[derive(Clone, Copy)]
struct PendingPanic {
    sequence: u64,
    restoring: bool,
}

pub(super) fn configure(log_dir: PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(&log_dir)?;
    let _ = DIAGNOSTIC_PATH.set(log_dir.join(PANIC_DIAGNOSTIC_FILE_NAME));
    Ok(())
}

pub(super) fn terminal_acquired() {
    *TERMINAL_OWNER_THREAD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(std::thread::current().id());
    TERMINAL_OWNED.store(true, Ordering::Release);
}

pub(super) fn terminal_released() {
    TERMINAL_OWNED.store(false, Ordering::Release);
    *TERMINAL_OWNER_THREAD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

pub(super) fn panic_is_on_terminal_owner_thread() -> bool {
    TERMINAL_OWNED.load(Ordering::Acquire)
        && TERMINAL_OWNER_THREAD
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some_and(|owner| owner == std::thread::current().id())
}

pub(super) fn record_panic(panic_info: &PanicHookInfo<'_>, restoring: bool) {
    let sequence = PANIC_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1;
    let terminal_owned = TERMINAL_OWNED.load(Ordering::Acquire);
    let location = panic_info
        .location()
        .map(|location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        })
        .unwrap_or_else(|| "unknown".to_string());
    let thread = std::thread::current();
    let thread_name = sanitized_thread_name(thread.name());
    let thread_id = format!("{:?}", thread.id());
    let task_id = tokio::task::try_id()
        .map(|id| id.to_string())
        .unwrap_or_else(|| "none".to_string());
    let mode_disposition = if restoring { "restoring" } else { "preserved" };

    append_diagnostic(&format!(
        "timestamp_ms={} panic_sequence={sequence} phase=observed disposition=pending \
         terminal_owned={terminal_owned} mode_disposition={mode_disposition} thread_name={thread_name} \
         thread_id={thread_id} task_id={task_id} source={location}",
        timestamp_ms()
    ));
    tracing::error!(
        panic_sequence = sequence,
        terminal_owned,
        mode_disposition,
        thread_name,
        thread_id,
        task_id,
        source = location,
        "TUI panic observed; terminal recovery follows the ownership disposition"
    );
    PENDING_PANICS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(PendingPanic {
            sequence,
            restoring,
        });
}

pub(super) fn record_tui_survived() {
    let sequences = take_pending(|pending| !pending.restoring);
    classify_sequences(sequences, "contained_or_background", "preserved");
}

pub(super) fn record_owner_drop() {
    let pending = take_pending(|_| true);
    for pending in pending {
        let (disposition, mode_disposition) = if pending.restoring {
            ("fatal_foreground", "restoring")
        } else if std::thread::panicking() {
            ("contained_or_background", "preserved")
        } else {
            ("contained_or_background_before_exit", "restoring")
        };
        append_classification(pending.sequence, disposition, mode_disposition);
    }
}

fn take_pending(predicate: impl Fn(PendingPanic) -> bool) -> Vec<PendingPanic> {
    let mut pending = PENDING_PANICS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut matching = Vec::new();
    let mut index = 0;
    while index < pending.len() {
        if predicate(pending[index]) {
            matching.push(pending.remove(index));
        } else {
            index += 1;
        }
    }
    matching
}

fn classify_sequences(sequences: Vec<PendingPanic>, disposition: &str, mode_disposition: &str) {
    for pending in sequences {
        append_classification(pending.sequence, disposition, mode_disposition);
    }
}

fn append_classification(sequence: u64, disposition: &str, mode_disposition: &str) {
    let terminal_owned = TERMINAL_OWNED.load(Ordering::Acquire);
    append_diagnostic(&format!(
        "timestamp_ms={} panic_sequence={sequence} phase=classified disposition={disposition} \
         terminal_owned={terminal_owned} mode_disposition={mode_disposition}",
        timestamp_ms()
    ));
}

fn append_diagnostic(line: &str) {
    let Some(path) = DIAGNOSTIC_PATH.get() else {
        return;
    };
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    if let Ok(mut file) = options.open(path) {
        let record = format!("{line}\n");
        let _ = file.write_all(record.as_bytes());
    }
}

fn sanitized_thread_name(name: Option<&str>) -> String {
    let name = name.unwrap_or("unnamed");
    let sanitized: String = name
        .chars()
        .take(64)
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' => character,
            _ => '_',
        })
        .collect();
    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
