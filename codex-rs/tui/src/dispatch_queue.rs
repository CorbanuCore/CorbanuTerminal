use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

pub(crate) const MAX_DISPATCH_TASK_BYTES: usize = 32 * 1024;
pub(crate) const MAX_TARGET_DISPATCH_ITEMS: usize = 256;
pub(crate) const MAX_TARGET_DISPATCH_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_GLOBAL_DISPATCH_ITEMS: usize = 1024;
pub(crate) const MAX_GLOBAL_DISPATCH_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_DELIVERY_BATCH_ITEMS: usize = 16;
pub(crate) const MAX_DELIVERY_BATCH_BYTES: usize = 128 * 1024;

const LEGACY_BATCH_HEADER: &str = "Multiple spawn dispatches were queued while you were busy. Execute each task below in order, do not skip any task, and treat every section as assigned work.\n\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpawnDispatchAck {
    pub(crate) seq: u64,
    pub(crate) source_node_id: String,
    pub(crate) target_node_id: String,
    pub(crate) target_title: String,
    #[serde(default)]
    pub(crate) attempt: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingDispatchEnqueueResult {
    Queued,
    Duplicate {
        acks: Vec<SpawnDispatchAck>,
        notify: bool,
    },
    Rejected {
        acks: Vec<SpawnDispatchAck>,
        reason: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DispatchOrigin {
    #[serde(default)]
    pub(crate) origin_id: String,
    #[serde(default)]
    pub(crate) source_turn_id: Option<String>,
    #[serde(default)]
    pub(crate) ordinal: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DispatchState {
    #[default]
    Queued,
    Submitting {
        delivery_id: String,
        ordered_dispatch_ids: Vec<String>,
    },
    Accepted {
        delivery_id: String,
        destination_turn_id: String,
        accepted_at_ms: i64,
    },
    Failed {
        reason: String,
        failed_at_ms: i64,
    },
}

impl DispatchState {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Submitting { .. } => "submitting",
            Self::Accepted { .. } => "accepted",
            Self::Failed { .. } => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PendingSpawnDispatch {
    pub(crate) dispatch_id: String,
    pub(crate) origin: DispatchOrigin,
    pub(crate) source_pane_id: String,
    pub(crate) target_pane_id: String,
    pub(crate) task: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) acks: Vec<SpawnDispatchAck>,
    pub(crate) created_at_ms: i64,
    pub(crate) state: DispatchState,
    #[serde(default)]
    pub(crate) duplicate_suppressed_notified: bool,
}

impl PendingSpawnDispatch {
    pub(crate) fn new(task: String, acks: Vec<SpawnDispatchAck>) -> Self {
        Self {
            dispatch_id: String::new(),
            origin: DispatchOrigin::default(),
            source_pane_id: String::new(),
            target_pane_id: String::new(),
            task,
            acks,
            created_at_ms: chrono::Utc::now().timestamp_millis(),
            state: DispatchState::Queued,
            duplicate_suppressed_notified: false,
        }
    }

    pub(crate) fn assign_identity(
        &mut self,
        seq: u64,
        source_pane_id: &str,
        target_pane_id: &str,
        origin_id: Option<&str>,
    ) {
        if self.dispatch_id.is_empty() {
            self.dispatch_id = format!("dispatch-{seq:020}");
        }
        if self.origin.origin_id.is_empty() {
            self.origin.origin_id = origin_id
                .map(str::to_owned)
                .unwrap_or_else(|| format!("host-seq-{seq:020}"));
        }
        if self.source_pane_id.is_empty() {
            self.source_pane_id = source_pane_id.to_string();
        }
        if self.target_pane_id.is_empty() {
            self.target_pane_id = target_pane_id.to_string();
        }
    }

    pub(crate) fn migrate_legacy_identity(&mut self, target_pane_id: &str, ordinal: usize) {
        let source_pane_id = self
            .acks
            .first()
            .map(|ack| ack.source_node_id.as_str())
            .unwrap_or("unknown-source");
        let mut digest = Sha256::new();
        digest.update(target_pane_id.as_bytes());
        digest.update([0]);
        digest.update(source_pane_id.as_bytes());
        digest.update([0]);
        digest.update(self.created_at_ms.to_le_bytes());
        digest.update(ordinal.to_le_bytes());
        digest.update(self.task.as_bytes());
        let digest = format!("{:x}", digest.finalize());
        if self.dispatch_id.is_empty() {
            self.dispatch_id = format!("legacy-{}", &digest[..24]);
        }
        if self.origin.origin_id.is_empty() {
            self.origin.origin_id = format!("legacy-origin-{}", &digest[..24]);
        }
        if self.source_pane_id.is_empty() {
            self.source_pane_id = source_pane_id.to_string();
        }
        if self.target_pane_id.is_empty() {
            self.target_pane_id = target_pane_id.to_string();
        }
    }

    pub(crate) fn payload_bytes(&self) -> usize {
        self.task.len()
    }
}

impl<'de> Deserialize<'de> for PendingSpawnDispatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Fields {
            #[serde(default)]
            dispatch_id: String,
            #[serde(default)]
            origin: DispatchOrigin,
            #[serde(default)]
            source_pane_id: String,
            #[serde(default)]
            target_pane_id: String,
            task: String,
            #[serde(default)]
            acks: Vec<SpawnDispatchAck>,
            #[serde(default = "now_ms")]
            created_at_ms: i64,
            #[serde(default)]
            state: DispatchState,
            #[serde(default)]
            duplicate_suppressed_notified: bool,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Full(Fields),
            Legacy(String),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Full(fields) => Ok(Self {
                dispatch_id: fields.dispatch_id,
                origin: fields.origin,
                source_pane_id: fields.source_pane_id,
                target_pane_id: fields.target_pane_id,
                task: fields.task,
                acks: fields.acks,
                created_at_ms: fields.created_at_ms,
                state: fields.state,
                duplicate_suppressed_notified: fields.duplicate_suppressed_notified,
            }),
            Repr::Legacy(task) => Ok(Self::new(task, Vec::new())),
        }
    }
}

pub(crate) fn expand_legacy_batch(task: &str) -> Option<Vec<String>> {
    let mut rest = task.strip_prefix(LEGACY_BATCH_HEADER)?;
    let mut tasks = Vec::new();
    let mut index = 1;
    while !rest.is_empty() {
        let heading_prefix = format!("## Queued dispatch {index} (bytes=");
        rest = rest.strip_prefix(&heading_prefix)?;
        let (length, after_length) = rest.split_once(")\n")?;
        let length = length.parse::<usize>().ok()?;
        if after_length.len() < length || !after_length.is_char_boundary(length) {
            return None;
        }
        let (component, tail) = after_length.split_at(length);
        tasks.push(component.to_string());
        rest = if tail.is_empty() {
            tail
        } else {
            tail.strip_prefix('\n')?
        };
        index += 1;
    }
    (!tasks.is_empty()).then_some(tasks)
}

pub(crate) fn queue_payload_bytes<'a>(
    dispatches: impl IntoIterator<Item = &'a PendingSpawnDispatch>,
) -> usize {
    dispatches
        .into_iter()
        .map(PendingSpawnDispatch::payload_bytes)
        .sum()
}

pub(crate) fn bounded_delivery_batch(
    dispatches: impl IntoIterator<Item = PendingSpawnDispatch>,
) -> Vec<PendingSpawnDispatch> {
    let mut batch = Vec::new();
    let mut bytes = 0usize;
    for dispatch in dispatches {
        let next_bytes = bytes.saturating_add(dispatch.payload_bytes());
        if batch.len() >= MAX_DELIVERY_BATCH_ITEMS || next_bytes > MAX_DELIVERY_BATCH_BYTES {
            break;
        }
        bytes = next_bytes;
        batch.push(dispatch);
    }
    batch
}

pub(crate) fn queue_bound_violation(
    task_bytes: usize,
    target_items: usize,
    target_bytes: usize,
    global_items: usize,
    global_bytes: usize,
) -> Option<String> {
    if task_bytes > MAX_DISPATCH_TASK_BYTES {
        return Some(format!(
            "task is {task_bytes} bytes; maximum is {MAX_DISPATCH_TASK_BYTES} bytes"
        ));
    }
    if target_items >= MAX_TARGET_DISPATCH_ITEMS {
        return Some(format!(
            "target queue contains {target_items} items; maximum is {MAX_TARGET_DISPATCH_ITEMS}"
        ));
    }
    if target_bytes.saturating_add(task_bytes) > MAX_TARGET_DISPATCH_BYTES {
        return Some(format!(
            "target queue would exceed {MAX_TARGET_DISPATCH_BYTES} bytes"
        ));
    }
    if global_items >= MAX_GLOBAL_DISPATCH_ITEMS {
        return Some(format!(
            "global queue contains {global_items} items; maximum is {MAX_GLOBAL_DISPATCH_ITEMS}"
        ));
    }
    if global_bytes.saturating_add(task_bytes) > MAX_GLOBAL_DISPATCH_BYTES {
        return Some(format!(
            "global queue would exceed {MAX_GLOBAL_DISPATCH_BYTES} bytes"
        ));
    }
    None
}

pub(crate) fn model_dispatch_origin_id(
    source_pane_id: &str,
    source_turn_id: &str,
    ordinal: u32,
) -> String {
    let mut digest = Sha256::new();
    digest.update(source_pane_id.as_bytes());
    digest.update([0]);
    digest.update(source_turn_id.as_bytes());
    digest.update([0]);
    digest.update(ordinal.to_le_bytes());
    format!("model-origin-{:x}", digest.finalize())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
#[path = "dispatch_queue_tests.rs"]
mod tests;
