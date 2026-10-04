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
}

const MAX_ATTEMPTS: usize = 512;
const MAX_BYTES: usize = 4 * 1024 * 1024;
/// Scan-row units these reads may spend: a quarter of the whole call's budget,
/// so a busy host answers about this conversation promptly and states the
/// conversations it did not read rather than reading all of them.
const MAX_ROWS: usize = 1_000_000;

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
    OtherConversations {
        conversations,
        unavailable: unavailable.len(),
        requests,
    }
}
