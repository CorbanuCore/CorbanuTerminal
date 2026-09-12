use anyhow::ensure;
use codex_protocol::ThreadId;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

/// Exact nonnegative SQLite integer; serde rejects booleans and floating point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub(super) struct Count(i64);

impl TryFrom<i64> for Count {
    type Error = anyhow::Error;
    fn try_from(value: i64) -> anyhow::Result<Self> {
        ensure!(value >= 0, "negative count or position");
        Ok(Self(value))
    }
}

impl From<Count> for i64 {
    fn from(value: Count) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(super) enum Presence {
    #[default]
    Missing,
    Null,
    Number(Count),
}

impl<'de> Deserialize<'de> for Presence {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Option::<Count>::deserialize(deserializer)?.map_or(Self::Null, Self::Number))
    }
}

impl Presence {
    fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Patch {
    #[serde(skip_serializing_if = "Presence::is_missing")]
    pub input: Presence,
    #[serde(skip_serializing_if = "Presence::is_missing")]
    pub read: Presence,
    #[serde(skip_serializing_if = "Presence::is_missing")]
    pub write: Presence,
    #[serde(skip_serializing_if = "Presence::is_missing")]
    pub output: Presence,
    #[serde(skip_serializing_if = "Presence::is_missing")]
    pub reasoning: Presence,
    #[serde(skip_serializing_if = "Presence::is_missing")]
    pub total: Presence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Dialect {
    Inclusive,
    NativeAnthropic,
    UnknownCompatible,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Attempt {
    pub attempt_id: Uuid,
    pub request_id: Uuid,
    pub thread_id: ThreadId,
    pub turn: String,
    pub retry_of: Option<Uuid>,
    pub provider: String,
    pub model: String,
    pub scope: Uuid,
    pub dialect: Dialect,
    pub dispatched_at_ms: Count,
}

impl Attempt {
    pub fn validate(&self) -> anyhow::Result<()> {
        for text in [&self.turn, &self.provider, &self.model] {
            ensure!(
                !text.trim().is_empty() && text.len() <= 128 && !text.chars().any(char::is_control),
                "invalid bounded identity metadata"
            );
        }
        ensure!(self.retry_of != Some(self.attempt_id), "self retry");
        Ok(())
    }

    pub fn same_owner(&self, other: &Self) -> bool {
        self.request_id == other.request_id
            && self.thread_id == other.thread_id
            && self.turn == other.turn
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Observation {
    pub revision: Count,
    pub source: Uuid,
    pub sequence: Count,
    pub patch: Patch,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Usage {
    pub input: Option<i64>,
    pub noncached: Option<i64>,
    pub read: Option<i64>,
    pub write: Option<i64>,
    pub output: Option<i64>,
    pub reasoning: Option<i64>,
    pub total: Option<i64>,
}

/// Replay cumulative patches in source revision order, validating every prefix.
pub(super) fn replay(dialect: Dialect, observations: &[Observation]) -> anyhow::Result<Usage> {
    let mut raw = [None; 6];
    let mut usage = Usage::default();
    let mut previous = 0;
    for observation in observations {
        let revision = i64::from(observation.revision);
        ensure!(
            revision > previous,
            "replay requires unique ordered revisions"
        );
        previous = revision;
        let p = &observation.patch;
        for (slot, value) in
            raw.iter_mut()
                .zip([p.input, p.read, p.write, p.output, p.reasoning, p.total])
        {
            if let Presence::Number(count) = value {
                *slot = Some(i64::from(count));
            }
        }
        let [input, read, write, output, reasoning, total] = raw;
        let known_cache = read.unwrap_or(0).checked_add(write.unwrap_or(0));
        let known_cache = known_cache.ok_or_else(|| anyhow::anyhow!("cache overflow"))?;
        let (input, noncached) = match dialect {
            Dialect::Inclusive => {
                ensure!(
                    input.is_none_or(|n| known_cache <= n),
                    "cache exceeds inclusive input"
                );
                (
                    input,
                    input.zip(read).zip(write).map(|((i, r), w)| i - r - w),
                )
            }
            Dialect::NativeAnthropic => {
                let sum = input.unwrap_or(0).checked_add(known_cache);
                ensure!(sum.is_some(), "input overflow");
                (input.zip(read).zip(write).and(sum), input)
            }
            Dialect::UnknownCompatible => (None, None),
        };
        // Unknown dialects retain raw evidence without asserting subset/total semantics.
        let mut measured_total = total;
        let mut measured_reasoning = reasoning;
        if dialect != Dialect::UnknownCompatible {
            ensure!(
                reasoning.zip(output).is_none_or(|(r, o)| r <= o),
                "reasoning exceeds output"
            );
            if let Some((i, o)) = input.zip(output) {
                let sum = i
                    .checked_add(o)
                    .ok_or_else(|| anyhow::anyhow!("total overflow"))?;
                ensure!(total.is_none_or(|t| t == sum), "conflicting total");
                if dialect == Dialect::NativeAnthropic {
                    measured_total = Some(sum);
                }
            }
        } else {
            measured_total = None;
            measured_reasoning = None;
        }
        usage = Usage {
            input,
            noncached,
            read,
            write,
            output,
            reasoning: measured_reasoning,
            total: measured_total,
        };
    }
    Ok(usage)
}
