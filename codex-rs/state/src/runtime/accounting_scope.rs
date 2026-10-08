//! Attempts outside the inspected conversation, read beside it.
//!
//! A day view covers one root conversation and its resolved descendants. The
//! same UTC day can hold attempts from other conversations, and without them an
//! empty conversation reads as a zero-cost day. They are read in the same
//! snapshot, after the tree is complete, so they never consume the tree's
//! budget, refuse its read, or enter its total.
use super::InspectionDay;
use super::InspectionWork;
use super::ObservationQuote;
use crate::runtime::accounting::Journal;
use codex_protocol::ThreadId;
use sqlx::SqliteConnection;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Readable attempts from other root conversations on the inspected day.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct OtherConversations {
    /// Root conversations, other than the inspected one, with attempts that day.
    pub conversations: usize,
    /// Of those, conversations whose attempts could not all be read. Their cost
    /// is unknown; nothing is inferred for the unread part.
    pub unavailable: usize,
    /// Readable attempts by logical request. Never part of the inspected total.
    pub requests: BTreeMap<uuid::Uuid, Vec<ObservationQuote>>,
    /// Attempts dispatched this day by conversations since deleted.
    pub deleted_attempts: DeletedAttempts,
}

/// Deleting a conversation removes its attempts and their cost; only a replay
/// fence (tombstone) per attempt survives, which is what this counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletedAttempts {
    Counted(usize),
    /// The day reaches past the raw detail window, where retention's own
    /// tombstones cannot be told apart from deletions.
    PastDetailWindow,
    /// The count could not be read: the work budget ran out, the ledger has no
    /// checkpoint yet, or the read failed.
    Unread,
}

impl Default for DeletedAttempts {
    fn default() -> Self {
        Self::Counted(0)
    }
}

const MAX_ATTEMPTS: usize = 512;
const MAX_BYTES: usize = 4 * 1024 * 1024;
/// Scan-row units these reads may spend: a quarter of the whole call's budget,
/// so a busy host answers about this conversation promptly and states the
/// conversations it did not read rather than reading all of them.
const MAX_ROWS: usize = 1_000_000;

/// How long a deleted attempt's replay fence (tombstone) outlives its dispatch,
/// and how long raw attempt detail is kept. Kept equal to the retention plan's.
const REPLAY_MS: i64 = crate::runtime::accounting::REPLAY_MS;
const DETAIL_MS: i64 = 90 * 86_400_000;

/// Attempts dispatched on `day` whose conversation (or subagent) was deleted.
///
/// A tombstone expires exactly `REPLAY_MS` after its attempt's dispatch, so the
/// dispatch time is recoverable. Deletion is not the only writer: retention
/// leaves one when raw detail ages out, and compact-only imports leave one too,
/// but only for attempts already past the detail window at the time they ran.
/// That time is, on a monotonic host clock, no later than the checkpoint or -
/// for batched expiry that has not advanced the checkpoint yet - than about
/// this reader's clock. A tombstone whose dispatch is inside the window at both
/// is therefore a deletion; a day that reaches past it cannot be counted.
pub(super) async fn deleted_attempts(
    conn: &mut SqliteConnection,
    day: i64,
    read_at_ms: i64,
    work: &mut InspectionWork,
) -> DeletedAttempts {
    // One unindexed scan of the tombstone table, charged like one other scan.
    // Tombstones outlive raw detail, so this can under-charge the budget; the
    // count is cheap either way.
    if !work.scans(/*count*/ 1) {
        return DeletedAttempts::Unread;
    }
    let checkpoint: Option<i64> = match sqlx::query_scalar(
        "SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint WHERE singleton = 1",
    )
    .fetch_optional(&mut *conn)
    .await
    {
        Ok(checkpoint) => checkpoint.flatten(),
        Err(_) => return DeletedAttempts::Unread,
    };
    let (Some(checkpoint), Some(start)) = (checkpoint, day.checked_mul(86_400_000)) else {
        return DeletedAttempts::Unread;
    };
    if start.saturating_add(DETAIL_MS) <= checkpoint.max(read_at_ms) {
        return DeletedAttempts::PastDetailWindow;
    }
    let (Some(from), Some(to)) = (
        start.checked_add(REPLAY_MS),
        start.checked_add(86_400_000 + REPLAY_MS),
    ) else {
        return DeletedAttempts::Unread;
    };
    let count: Result<i64, _> = sqlx::query_scalar(
        "SELECT count(*) FROM draft_accounting_tombstones WHERE expires_at_ms >= ? AND expires_at_ms < ?",
    )
    .bind(from)
    .bind(to)
    .fetch_one(&mut *conn)
    .await;
    count
        .ok()
        .and_then(|count| usize::try_from(count).ok())
        .map_or(DeletedAttempts::Unread, DeletedAttempts::Counted)
}

/// `threads` pairs each unrelated thread with its terminal root conversation.
pub(super) async fn other_conversations(
    conn: &mut SqliteConnection,
    threads: Vec<(String, String)>,
    day: i64,
    read_at_ms: i64,
    work: &mut InspectionWork,
) -> OtherConversations {
    let conversations = threads
        .iter()
        .map(|(_, root)| root.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let mut unavailable = BTreeSet::new();
    let mut requests: BTreeMap<uuid::Uuid, Vec<ObservationQuote>> = BTreeMap::new();
    let (mut attempts, mut bytes) = (0usize, 0usize);
    let floor = work.rows.saturating_sub(MAX_ROWS);
    for (thread, root) in threads {
        if work.rows <= floor {
            unavailable.insert(root);
            continue;
        }
        let read = match ThreadId::from_string(&thread) {
            // A thread that cannot be read is reported as unread, never as a
            // failure of the conversation the user asked about.
            Ok(id) => Journal::inspect_window_on_connection(
                conn, id, day, read_at_ms, /*window*/ None, work,
            )
            .await
            .ok(),
            Err(_) => None,
        };
        let Some(InspectionDay::Ready(view)) = read else {
            unavailable.insert(root);
            continue;
        };
        let quotes: Vec<_> = view.requests.into_iter().collect();
        let size = quotes
            .iter()
            .flat_map(|(_, quotes)| quotes)
            .map(|quote| serde_json::to_vec(quote).map_or(MAX_BYTES, |v| v.len() + 2048))
            .sum::<usize>();
        let count = quotes.iter().map(|(_, quotes)| quotes.len()).sum::<usize>();
        if attempts + count > MAX_ATTEMPTS || bytes + size > MAX_BYTES {
            unavailable.insert(root);
            continue;
        }
        attempts += count;
        bytes += size;
        for (request, quotes) in quotes {
            requests.entry(request).or_default().extend(quotes);
        }
    }
    let deleted_attempts = deleted_attempts(conn, day, read_at_ms, work).await;
    OtherConversations {
        conversations,
        unavailable: unavailable.len(),
        requests,
        deleted_attempts,
    }
}
