//! Shared model metadata types exchanged between Codex services and clients.
//!
//! These types are serialized across core, TUI, app-server, and SDK boundaries, so field defaults
//! are used to preserve compatibility when older payloads omit newly introduced attributes.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use schemars::JsonSchema;
use schemars::r#gen::SchemaGenerator;
use schemars::schema::InstanceType;
use schemars::schema::Metadata;
use schemars::schema::Schema;
use schemars::schema::SchemaObject;
use schemars::schema::StringValidation;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::DeserializeOwned;
use serde::de::Error;
use strum_macros::Display;
use strum_macros::EnumIter;
use tracing::warn;
use ts_rs::TS;

use crate::config_types::ReasoningSummary;
use crate::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use crate::config_types::ServiceTier;
use crate::config_types::Verbosity;
use crate::protocol::MultiAgentVersion;

mod access_programs;
mod incremental_tools;
pub use incremental_tools::IncrementalToolMessages;
#[path = "openai_models/memento_v2.rs"]
mod memento_v2;
pub use memento_v2::ContextManagementToolMessages;
pub use memento_v2::HistoryToolMessages;
pub use memento_v2::NotesToolMessages;
#[path = "openai_models/guardian.rs"]
mod guardian;
pub use guardian::GuardianModelPolicy;
pub use guardian::GuardianReviewMode;
pub use guardian::GuardianScope;
pub use guardian::GuardianUnscoredAction;

#[path = "openai_models/guardian_v2.rs"]
mod guardian_v2;
#[path = "openai_models/reasoning_effort.rs"]
mod reasoning_effort;

pub use access_programs::ModelAccessPrograms;
pub use guardian_v2::AsyncClassifierMode;
pub use guardian_v2::GuardianV2ModelConfig;
pub use guardian_v2::GuardianV2TranscriptModelConfig;

/// Backend model-catalog specialty identifying cybersecurity-focused models.
pub const MODEL_SPECIALTY_CYBER: &str = "cyber";
pub const SPEED_TIER_FAST: &str = "fast";

/// OpenAI protocol compatibility, independent of Corbanu's product version.
/// Shared by model discovery and inference so the backend negotiates one contract.
/// Validated against Codex 0.156.0 for Astra, Sol, and Luna. Older negotiation
/// versions can hide available models and reject inference for the same account.
pub const OPENAI_CODEX_COMPAT_VERSION: &str = "0.156.0";

/// See https://platform.openai.com/docs/guides/reasoning?api-mode=responses#get-started-with-reasoning
#[derive(Debug, Default, Clone, PartialEq, Eq, TS, Hash)]
#[ts(type = "string")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
    Ultra,
    Persistent,
    /// A model-defined effort value that this client does not know yet.
    Custom(String),
}

impl ReasoningEffort {
    /// Returns the exact value used on the wire.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
            Self::Max => "max",
            Self::Ultra => "ultra",
            Self::Persistent => "persistent",
            Self::Custom(effort) => effort,
        }
    }
}

impl fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl JsonSchema for ReasoningEffort {
    fn schema_name() -> String {
        "ReasoningEffort".to_string()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A non-empty reasoning effort value advertised by the model.".to_string(),
                ),
                ..Default::default()
            })),
            string: Some(Box::new(StringValidation {
                min_length: Some(1),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

impl Serialize for ReasoningEffort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ReasoningEffort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let effort = String::deserialize(deserializer)?;
        effort.parse().map_err(D::Error::custom)
    }
}

impl FromStr for ReasoningEffort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" => Ok(Self::XHigh),
            "max" => Ok(Self::Max),
            "ultra" => Ok(Self::Ultra),
            "persistent" => Ok(Self::Persistent),
            "" => Err("reasoning_effort must not be empty".to_string()),
            effort => Ok(Self::Custom(effort.to_string())),
        }
    }
}

/// Canonical user-input modality tags advertised by a model.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Display,
    JsonSchema,
    TS,
    EnumIter,
    Hash,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum InputModality {
    /// Plain text turns and tool payloads.
    Text,
    /// Image attachments included in user turns.
    Image,
    /// Audio attachments included in user turns.
    Audio,
}

/// Backward-compatible default when `input_modalities` is omitted on the wire.
///
/// Legacy payloads predate modality metadata, so we conservatively assume both text and images are
/// accepted unless a preset explicitly narrows support.
pub fn default_input_modalities() -> Vec<InputModality> {
    vec![InputModality::Text, InputModality::Image]
}

/// A reasoning effort option that can be surfaced for a model.
#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct ReasoningEffortPreset {
    /// Effort level that the model supports.
    pub effort: ReasoningEffort,
    /// Short human description shown next to the effort in UIs.
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq)]
pub struct ModelUpgrade {
    pub id: String,
    pub migration_config_key: String,
    pub model_link: Option<String>,
    pub upgrade_copy: Option<String>,
    pub migration_markdown: Option<String>,
    /// Informational time when the model associated with this upgrade is scheduled to retire.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_rfc3339_timestamp"
    )]
    #[schemars(with = "Option<String>")]
    #[ts(type = "string", optional)]
    pub retirement_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct ModelAvailabilityNux {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct ModelServiceTier {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Capability class used by orchestration when matching a model to spawned work.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ModelCapabilityTier {
    Frontier,
    Balanced,
    Fast,
    Legacy,
    Unclassified,
}

/// Reasoning-state contract for an OpenAI-compatible Chat Completions model.
#[derive(
    Debug, Default, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq, Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ChatReasoningProtocol {
    /// Reasoning is optional or does not need to be replayed in assistant messages.
    #[default]
    Independent,
    /// Every turn reasons, and returned reasoning content must be replayed with the assistant
    /// message on the next request.
    PreservedRequired,
}

/// Scalar `reasoning_effort` dialect for an OpenAI-compatible Chat Completions model.
///
/// The exact model catalogue selects this dialect; provider and model-name heuristics do not.
#[derive(
    Debug, Default, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq, Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ChatReasoningEffortProtocol {
    #[default]
    ProviderDefault,
    NoneHighMaxDefaultHigh,
    LowHighMaxRequiredDefaultMax,
    HighMaxDefaultHigh,
}

/// Chat Completions protocol capabilities that cannot be inferred from the provider alone.
#[derive(Debug, Default, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct ChatCompletionsCapabilities {
    #[serde(default)]
    pub reasoning_protocol: ChatReasoningProtocol,
    #[serde(default)]
    pub reasoning_effort_protocol: ChatReasoningEffortProtocol,
}

/// UTC weekdays on which a scheduled plan peak applies.
///
/// `ModelBilling::PlanSchedule::peak_weekdays == None` retains the legacy behavior where the
/// peak window applies every day.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct WeekdaySet {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}

impl WeekdaySet {
    pub const fn weekdays_only() -> Self {
        Self {
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
        }
    }

    pub const fn is_empty(self) -> bool {
        !self.monday
            && !self.tuesday
            && !self.wednesday
            && !self.thursday
            && !self.friday
            && !self.saturday
            && !self.sunday
    }

    pub const fn is_every_day(self) -> bool {
        self.monday
            && self.tuesday
            && self.wednesday
            && self.thursday
            && self.friday
            && self.saturday
            && self.sunday
    }

    pub fn names(self) -> Vec<&'static str> {
        [
            (self.monday, "Monday"),
            (self.tuesday, "Tuesday"),
            (self.wednesday, "Wednesday"),
            (self.thursday, "Thursday"),
            (self.friday, "Friday"),
            (self.saturday, "Saturday"),
            (self.sunday, "Sunday"),
        ]
        .into_iter()
        .filter_map(|(enabled, name)| enabled.then_some(name))
        .collect()
    }

    fn contains(self, weekday: chrono::Weekday) -> bool {
        match weekday {
            chrono::Weekday::Mon => self.monday,
            chrono::Weekday::Tue => self.tuesday,
            chrono::Weekday::Wed => self.wednesday,
            chrono::Weekday::Thu => self.thursday,
            chrono::Weekday::Fri => self.friday,
            chrono::Weekday::Sat => self.saturday,
            chrono::Weekday::Sun => self.sunday,
        }
    }
}

/// Whether `at` falls inside a catalogue peak window.
///
/// The window is `[start, end)` in UTC hours and may wrap past midnight. When it
/// wraps, the hours after midnight belong to the window that opened the previous
/// day, so a weekday restriction is tested against that opening day rather than
/// against the calendar day of the instant itself.
fn peak_window_contains(
    at: chrono::DateTime<chrono::Utc>,
    start_hour: u8,
    end_hour: u8,
    weekdays: Option<WeekdaySet>,
) -> bool {
    use chrono::Datelike;
    use chrono::Timelike;
    if start_hour > 23 || end_hour > 24 || start_hour == end_hour {
        return false;
    }
    let hour = u8::try_from(at.hour()).unwrap_or(u8::MAX);
    let (inside, opened_on) = if start_hour < end_hour {
        (hour >= start_hour && hour < end_hour, at.date_naive())
    } else {
        (
            hour >= start_hour,
            if hour >= start_hour {
                at.date_naive()
            } else {
                at.date_naive().pred_opt().unwrap_or(at.date_naive())
            },
        )
    };
    let inside = inside || (start_hour > end_hour && hour < end_hour);
    inside && weekdays.is_none_or(|days| days.contains(opened_on.weekday()))
}

/// Per-token API rates, in milli-USD per million tokens.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct MeteredRates {
    pub input_milli_usd_per_million_tokens: u32,
    pub output_milli_usd_per_million_tokens: u32,
    #[serde(default)]
    pub cached_input_milli_usd_per_million_tokens: Option<u32>,
}

impl MeteredRates {
    fn tuple(self) -> (u32, u32, Option<u32>) {
        (
            self.input_milli_usd_per_million_tokens,
            self.output_milli_usd_per_million_tokens,
            self.cached_input_milli_usd_per_million_tokens,
        )
    }
}

/// API rates for a whole request whose input exceeds a published threshold,
/// such as OpenAI's long-context price for prompts above 272K input tokens.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct LongContextRates {
    /// The rates apply when the request's input tokens (cached and cache
    /// writes included) are strictly more than this.
    pub above_input_tokens: u32,
    pub input_milli_usd_per_million_tokens: u32,
    pub output_milli_usd_per_million_tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cached_input_milli_usd_per_million_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cache_write_milli_usd_per_million_tokens: Option<u32>,
}

/// A `[start, end)` window in whole UTC hours; it may wrap past midnight.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
pub struct UtcHourWindow {
    pub start_utc_hour: u8,
    pub end_utc_hour: u8,
}

/// Billing data for an exact provider/model route.
///
/// Monetary values use milli-USD per million tokens so catalogue metadata remains exact,
/// serializable, and equality-comparable without floating point.
#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelBilling {
    Plan {
        /// Relative subscription-pool burn where 1000 means 1.0x.
        relative_burn_millis: u32,
    },
    /// Subscription-pool burn changes by time of day.
    ///
    /// The normal peak/off-peak schedule remains present when a temporary promotion is active so
    /// clients can stop applying the discount after `promotion_valid_through_utc` without a
    /// catalogue release.
    PlanSchedule {
        /// Normal off-peak relative burn where 1000 means 1.0x.
        off_peak_relative_burn_millis: u32,
        /// Peak relative burn where 1000 means 1.0x.
        peak_relative_burn_millis: u32,
        /// Inclusive peak-window start, expressed as an hour in UTC.
        peak_start_utc_hour: u8,
        /// Exclusive peak-window end, expressed as an hour in UTC.
        peak_end_utc_hour: u8,
        /// UTC weekdays on which the peak window applies. Omitted means every day for backward
        /// compatibility with older catalogue entries.
        #[serde(default)]
        peak_weekdays: Option<WeekdaySet>,
        #[serde(default)]
        promotional_off_peak_relative_burn_millis: Option<u32>,
        #[serde(default)]
        promotion_valid_through_utc: Option<String>,
    },
    Metered {
        input_milli_usd_per_million_tokens: u32,
        output_milli_usd_per_million_tokens: u32,
        #[serde(default)]
        cached_input_milli_usd_per_million_tokens: Option<u32>,
        /// The published cache-write price. Omitted means the catalogue states
        /// none, not that writes are free.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        cache_write_milli_usd_per_million_tokens: Option<u32>,
        /// Rates for a request whose input exceeds a published threshold.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        long_context: Option<LongContextRates>,
        /// The last UTC instant (RFC 3339) the provider says these rates hold,
        /// such as the end of a promotion. Afterwards the row states no price.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        valid_through_utc: Option<String>,
    },
    /// Per-token rates that differ between peak windows and the rest of the week.
    ///
    /// The rate that applies is the one in force at the dispatch instant, so a
    /// ledger states what a request was actually charged, not a range.
    MeteredSchedule {
        off_peak: MeteredRates,
        peak: MeteredRates,
        /// Peak windows. Outside every window the off-peak rates apply.
        peak_windows: Vec<UtcHourWindow>,
        /// UTC weekdays on which the peak windows apply. Omitted means every day.
        #[serde(default)]
        peak_weekdays: Option<WeekdaySet>,
        /// UTC calendar dates (`YYYY-MM-DD`) with no peak at all, such as the
        /// provider's published holidays.
        #[serde(default)]
        off_peak_dates_utc: Vec<String>,
    },
    /// The same provider/model route can use subscription capacity or API-key billing.
    ///
    /// Both sides are exact. Runtime consumers resolve the active side from authentication;
    /// catalogue consumers that do not know authentication must present both, never guess.
    AuthDependent {
        plan_relative_burn_millis: u32,
        api_key_input_milli_usd_per_million_tokens: u32,
        api_key_output_milli_usd_per_million_tokens: u32,
        #[serde(default)]
        api_key_cached_input_milli_usd_per_million_tokens: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        api_key_cache_write_milli_usd_per_million_tokens: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        api_key_long_context: Option<LongContextRates>,
        /// As `Metered::valid_through_utc`, for the API-key side only.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        api_key_valid_through_utc: Option<String>,
    },
    Local,
}

impl ModelBilling {
    /// The subscription-pool burn this row charges for work dispatched at `at_ms`,
    /// where 1000 means 1.0x. `None` means this row has no plan side at all.
    ///
    /// A schedule is resolved against the dispatch instant rather than read as a
    /// range, because a ledger has to state the rate that actually applied to a
    /// request, not the rates the model could have charged that day.
    pub fn plan_burn_millis_at(&self, at_ms: i64) -> Option<u32> {
        match self {
            Self::Plan {
                relative_burn_millis,
            } => Some(*relative_burn_millis),
            Self::AuthDependent {
                plan_relative_burn_millis,
                ..
            } => Some(*plan_relative_burn_millis),
            Self::PlanSchedule {
                off_peak_relative_burn_millis,
                peak_relative_burn_millis,
                peak_start_utc_hour,
                peak_end_utc_hour,
                peak_weekdays,
                promotional_off_peak_relative_burn_millis,
                promotion_valid_through_utc,
            } => {
                let at = chrono::DateTime::from_timestamp_millis(at_ms)?;
                let peak = peak_window_contains(
                    at,
                    *peak_start_utc_hour,
                    *peak_end_utc_hour,
                    *peak_weekdays,
                );
                if peak {
                    return Some(*peak_relative_burn_millis);
                }
                // A promotion applies only while it is valid, and only if its end is
                // a timestamp this client can actually read. An unparsable or absent
                // end is treated as no promotion rather than as an open one.
                let promotion = promotional_off_peak_relative_burn_millis.filter(|_| {
                    promotion_valid_through_utc
                        .as_deref()
                        .and_then(|through| {
                            chrono::DateTime::parse_from_rfc3339(through)
                                .ok()
                                .map(|end| end.timestamp_millis())
                        })
                        .is_some_and(|end| at_ms <= end)
                });
                Some(promotion.unwrap_or(*off_peak_relative_burn_millis))
            }
            Self::Metered { .. } | Self::MeteredSchedule { .. } | Self::Local => None,
        }
    }

    /// The exact API-key rates this row charges for work dispatched at `at_ms`,
    /// in milli-USD per million: input, output, cached input.
    ///
    /// `None` means the catalogue states no API price for this route, and a
    /// client must not invent one. A schedule is resolved against the dispatch
    /// instant; an instant that cannot be read has no stateable rate.
    pub fn api_key_rates_at(&self, at_ms: i64) -> Option<(u32, u32, Option<u32>)> {
        match self {
            Self::MeteredSchedule {
                off_peak,
                peak,
                peak_windows,
                peak_weekdays,
                off_peak_dates_utc,
            } => {
                let at = chrono::DateTime::from_timestamp_millis(at_ms)?;
                let date = at.date_naive().format("%Y-%m-%d").to_string();
                let peak_now = !off_peak_dates_utc.contains(&date)
                    && peak_windows.iter().any(|window| {
                        peak_window_contains(
                            at,
                            window.start_utc_hour,
                            window.end_utc_hour,
                            *peak_weekdays,
                        )
                    });
                Some(if peak_now { peak } else { off_peak }.tuple())
            }
            billing => billing
                .api_key_rates()
                .filter(|_| billing.api_key_rates_valid_at(at_ms)),
        }
    }

    /// Whether the flat API-key rates still hold at `at_ms`. A validity end
    /// this client cannot read is treated as already passed: no price rather
    /// than a price the provider may no longer charge.
    fn api_key_rates_valid_at(&self, at_ms: i64) -> bool {
        let through = match self {
            Self::Metered {
                valid_through_utc, ..
            } => valid_through_utc,
            Self::AuthDependent {
                api_key_valid_through_utc,
                ..
            } => api_key_valid_through_utc,
            _ => return true,
        };
        through.as_deref().is_none_or(|through| {
            chrono::DateTime::parse_from_rfc3339(through)
                .is_ok_and(|end| at_ms <= end.timestamp_millis())
        })
    }

    /// The exact API-key rates this row would charge for the same tokens, in
    /// milli-USD per million: input, output, cached input.
    ///
    /// Present for rows that carry a flat API side. `None` means the catalogue
    /// states no flat API price for this route, and a client must not invent
    /// one; a scheduled row's price depends on the instant, see `api_key_rates_at`.
    pub fn api_key_rates(&self) -> Option<(u32, u32, Option<u32>)> {
        match self {
            Self::Metered {
                input_milli_usd_per_million_tokens,
                output_milli_usd_per_million_tokens,
                cached_input_milli_usd_per_million_tokens,
                ..
            } => Some((
                *input_milli_usd_per_million_tokens,
                *output_milli_usd_per_million_tokens,
                *cached_input_milli_usd_per_million_tokens,
            )),
            Self::AuthDependent {
                api_key_input_milli_usd_per_million_tokens,
                api_key_output_milli_usd_per_million_tokens,
                api_key_cached_input_milli_usd_per_million_tokens,
                ..
            } => Some((
                *api_key_input_milli_usd_per_million_tokens,
                *api_key_output_milli_usd_per_million_tokens,
                *api_key_cached_input_milli_usd_per_million_tokens,
            )),
            Self::Plan { .. }
            | Self::PlanSchedule { .. }
            | Self::MeteredSchedule { .. }
            | Self::Local => None,
        }
    }

    /// The published API-key cache-write rate, in milli-USD per million
    /// tokens. `None` means the catalogue states no price for cache writes.
    pub fn api_key_cache_write(&self) -> Option<u32> {
        match self {
            Self::Metered {
                cache_write_milli_usd_per_million_tokens,
                ..
            } => *cache_write_milli_usd_per_million_tokens,
            Self::AuthDependent {
                api_key_cache_write_milli_usd_per_million_tokens,
                ..
            } => *api_key_cache_write_milli_usd_per_million_tokens,
            _ => None,
        }
    }

    /// The API-key rates for a request whose input exceeds a threshold, if the
    /// provider prices such requests differently.
    pub fn api_key_long_context(&self) -> Option<LongContextRates> {
        match self {
            Self::Metered { long_context, .. } => *long_context,
            Self::AuthDependent {
                api_key_long_context,
                ..
            } => *api_key_long_context,
            _ => None,
        }
    }

    /// Resolve an auth-dependent row for a runtime that knows its active OpenAI auth mode.
    pub fn resolve_auth_mode(&self, api_key: bool) -> Self {
        match self {
            Self::AuthDependent {
                plan_relative_burn_millis: _,
                api_key_input_milli_usd_per_million_tokens,
                api_key_output_milli_usd_per_million_tokens,
                api_key_cached_input_milli_usd_per_million_tokens,
                api_key_cache_write_milli_usd_per_million_tokens,
                api_key_long_context,
                api_key_valid_through_utc,
            } if api_key => Self::Metered {
                input_milli_usd_per_million_tokens: *api_key_input_milli_usd_per_million_tokens,
                output_milli_usd_per_million_tokens: *api_key_output_milli_usd_per_million_tokens,
                cached_input_milli_usd_per_million_tokens:
                    *api_key_cached_input_milli_usd_per_million_tokens,
                cache_write_milli_usd_per_million_tokens:
                    *api_key_cache_write_milli_usd_per_million_tokens,
                long_context: *api_key_long_context,
                valid_through_utc: api_key_valid_through_utc.clone(),
            },
            Self::AuthDependent {
                plan_relative_burn_millis,
                ..
            } => Self::Plan {
                relative_burn_millis: *plan_relative_burn_millis,
            },
            billing => billing.clone(),
        }
    }
}

/// Canonical spawn-allocation metadata carried with a model through every catalogue layer.
#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ModelOrchestrationMetadata {
    /// Spawnable models must declare exact billing and capability metadata.
    Eligible {
        provider_id: String,
        capability: ModelCapabilityTier,
        billing: ModelBilling,
    },
    /// Non-spawnable models remain visible for existing sessions and manual use.
    Disabled {
        provider_id: String,
        capability: ModelCapabilityTier,
        reason: String,
        /// The provider's published price, for cost accounting of manual use.
        /// Never used for spawn allocation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        billing: Option<ModelBilling>,
    },
}

impl ModelOrchestrationMetadata {
    pub fn provider_id(&self) -> &str {
        match self {
            Self::Eligible { provider_id, .. } | Self::Disabled { provider_id, .. } => provider_id,
        }
    }

    pub fn capability(&self) -> ModelCapabilityTier {
        match self {
            Self::Eligible { capability, .. } | Self::Disabled { capability, .. } => *capability,
        }
    }

    pub fn billing(&self) -> Option<&ModelBilling> {
        match self {
            Self::Eligible { billing, .. } => Some(billing),
            Self::Disabled { .. } => None,
        }
    }

    /// The price this row states for cost accounting: an eligible row's
    /// billing, or a disabled row's published price.
    pub fn accounting_billing(&self) -> Option<&ModelBilling> {
        match self {
            Self::Eligible { billing, .. } => Some(billing),
            Self::Disabled { billing, .. } => billing.as_ref(),
        }
    }

    /// Resolve auth-dependent billing without changing provider/capability policy.
    pub fn resolve_billing_auth_mode(&mut self, api_key: bool) {
        if let Self::Eligible { billing, .. } = self {
            *billing = billing.resolve_auth_mode(api_key);
        }
    }

    pub fn is_spawn_eligible(&self) -> bool {
        matches!(self, Self::Eligible { .. })
    }

    pub fn disabled_reason(&self) -> Option<&str> {
        match self {
            Self::Eligible { .. } => None,
            Self::Disabled { reason, .. } => Some(reason),
        }
    }
}

/// Metadata describing a Codex-supported model.
#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema, PartialEq)]
pub struct ModelPreset {
    /// Stable identifier for the preset.
    pub id: String,
    /// Model slug (e.g., "gpt-5").
    pub model: String,
    /// Provider that owns this catalog entry when model naming alone is not authoritative.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    /// Canonical cost, capability, and spawn-policy metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration: Option<ModelOrchestrationMetadata>,
    /// Display name shown in UIs.
    pub display_name: String,
    /// Short human description shown in UIs.
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_specialty: Option<String>,
    /// Reasoning effort applied when none is explicitly chosen.
    pub default_reasoning_effort: ReasoningEffort,
    /// Supported reasoning effort options.
    pub supported_reasoning_efforts: Vec<ReasoningEffortPreset>,
    /// Compatibility field retained for consumers of model presets; personality selection is no
    /// longer supported.
    #[serde(default)]
    pub supports_personality: bool,
    /// Deprecated: use `service_tiers` instead.
    #[serde(default)]
    pub additional_speed_tiers: Vec<String>,
    /// Service tiers this model can run with.
    #[serde(default)]
    pub service_tiers: Vec<ModelServiceTier>,
    /// Catalog default service tier id for this model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_service_tier: Option<String>,
    /// Caller-specific explicit access programs, when discovery resolved them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_access_programs: Option<ModelAccessPrograms>,
    /// Whether this is the default model for new users.
    pub is_default: bool,
    /// recommended upgrade model
    pub upgrade: Option<ModelUpgrade>,
    /// Whether this preset should appear in the picker UI.
    pub show_in_picker: bool,
    /// Multi-agent backend selected when this model starts a new thread.
    #[serde(default, skip_serializing, skip_deserializing)]
    #[schemars(skip)]
    #[ts(skip)]
    pub multi_agent_version: Option<MultiAgentVersion>,
    /// Availability NUX shown when this preset becomes accessible to the user.
    pub availability_nux: Option<ModelAvailabilityNux>,
    /// whether this model is supported in the api
    pub supported_in_api: bool,
    /// Input modalities accepted when composing user turns for this preset.
    #[serde(default = "default_input_modalities")]
    pub input_modalities: Vec<InputModality>,
}

/// Visibility of a model in the picker or APIs.
#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS, JsonSchema, EnumIter, Display,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ModelVisibility {
    List,
    Hide,
    None,
}

/// Shell execution capability for a model.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    TS,
    JsonSchema,
    EnumIter,
    Display,
    Hash,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ConfigShellToolType {
    #[serde(alias = "default", alias = "local", alias = "shell_command")]
    UnifiedExec,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplyPatchToolType {
    Freeform,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, TS, JsonSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolType {
    #[default]
    Text,
    TextAndImage,
}

/// Server-provided truncation policy metadata for a model.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TruncationMode {
    Bytes,
    Tokens,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolMode {
    Direct,
    CodeMode,
    CodeModeOnly,
}

fn deserialize_optional_model_selector<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let Some(value) = Option::<String>::deserialize(deserializer)? else {
        return Ok(None);
    };
    Ok(serde_json::from_value(serde_json::Value::String(value)).ok())
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS, JsonSchema)]
pub struct TruncationPolicyConfig {
    pub mode: TruncationMode,
    pub limit: i64,
}

impl TruncationPolicyConfig {
    pub const fn bytes(limit: i64) -> Self {
        Self {
            mode: TruncationMode::Bytes,
            limit,
        }
    }

    pub const fn tokens(limit: i64) -> Self {
        Self {
            mode: TruncationMode::Tokens,
            limit,
        }
    }
}

/// Semantic version triple encoded as an array in JSON (e.g. [0, 62, 0]).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS, JsonSchema)]
pub struct ClientVersion(pub i32, pub i32, pub i32);

const fn default_effective_context_window_percent() -> i64 {
    95
}

const fn default_true() -> bool {
    true
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_true(value: &bool) -> bool {
    *value
}

/// Model metadata returned by the Codex backend `/models` endpoint.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ModelInfo {
    /// Model-owned approval coverage. Absent preserves legacy settings; an empty map disables
    /// ordinary Guardian review. Keys are computer_use, shell, file_changes, mcp, network,
    /// and permissions. This does not override mandatory safety or administrator requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardian: Option<GuardianModelPolicy>,
    pub slug: String,
    /// Canonical cost, capability, and spawn-policy metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration: Option<ModelOrchestrationMetadata>,
    /// Model-specific Chat Completions wire behavior.
    #[serde(default)]
    pub chat_completions: ChatCompletionsCapabilities,
    pub display_name: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_reasoning_level: Option<ReasoningEffort>,
    pub supported_reasoning_levels: Vec<ReasoningEffortPreset>,
    pub shell_type: ConfigShellToolType,
    pub visibility: ModelVisibility,
    pub supported_in_api: bool,
    pub priority: i32,
    #[serde(default)]
    pub additional_speed_tiers: Vec<String>,
    #[serde(default)]
    pub service_tiers: Vec<ModelServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_service_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_access_programs: Option<ModelAccessPrograms>,
    pub availability_nux: Option<ModelAvailabilityNux>,
    pub upgrade: Option<ModelInfoUpgrade>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_messages: Option<ModelMessages>,
    #[serde(default)]
    pub include_skills_usage_instructions: bool,
    #[serde(default)]
    pub include_plugin_usage_instructions: bool,
    #[serde(default = "default_true")]
    pub include_apps_usage_instructions: bool,
    /// Whether the model accepts the Responses API `reasoning.summary` parameter.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub supports_reasoning_summary_parameter: bool,
    #[serde(default)]
    pub default_reasoning_summary: ReasoningSummary,
    pub support_verbosity: bool,
    pub default_verbosity: Option<Verbosity>,
    pub apply_patch_tool_type: Option<ApplyPatchToolType>,
    #[serde(default)]
    pub web_search_tool_type: WebSearchToolType,
    pub truncation_policy: TruncationPolicyConfig,
    #[serde(default)]
    pub supports_image_detail_original: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<i64>,
    /// Maximum context window allowed for config overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_context_window: Option<i64>,
    /// Maximum output tokens accepted by this exact model route.
    ///
    /// Provider adapters must consume this value instead of imposing a global harness cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,
    /// Token threshold for automatic compaction. When omitted, core derives it
    /// from `context_window` (90%). When provided, core clamps it to 90% of the
    /// context window when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_compact_token_limit: Option<i64>,
    /// Opaque identifier for compaction-compatible model configurations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comp_hash: Option<String>,
    /// Percentage of the context window considered usable for inputs, after
    /// reserving headroom for system prompts, tool overhead, and model output.
    #[serde(default = "default_effective_context_window_percent")]
    pub effective_context_window_percent: i64,
    pub experimental_supported_tools: Vec<String>,
    /// Input modalities accepted by the backend for this model.
    #[serde(default = "default_input_modalities")]
    pub input_modalities: Vec<InputModality>,
    #[serde(default)]
    pub supports_search_tool: bool,
    /// Whether experimental context management may be activated at session startup.
    #[serde(default)]
    pub supports_experimental_context: bool,
    #[serde(default)]
    pub use_responses_lite: bool,
    /// Whether the model accepts reasoning-effort `configuration_update` items.
    /// Missing metadata keeps effort changes on the ordinary request parameter.
    #[serde(default)]
    pub supports_reasoning_effort_updates: bool,
    #[serde(default)]
    pub node_repl_auto_review_required: bool,
    #[serde(default)]
    pub node_repl_disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_review_model_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_specialty: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_model_selector"
    )]
    pub tool_mode: Option<ToolMode>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_model_selector"
    )]
    pub multi_agent_version: Option<MultiAgentVersion>,
    /// Reasoning effort used for multi-agent work when the user selects Ultra.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_agent_reasoning_effort: Option<ReasoningEffort>,
}

/// Where model metadata was obtained during local catalog resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelOrigin {
    Bundled,
    Provider,
    ExplicitConfig,
    /// Generic metadata synthesized when no catalog entry matches the requested model.
    Synthetic,
}

/// Model metadata and its locally determined source, retained together through resolution.
/// Metadata is shared with runtime snapshots and extension stores; mutation uses copy-on-write.
/// This runtime wrapper is not serialized; managers assign its source from local catalog state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogModel {
    pub model: Arc<ModelInfo>,
    pub origin: ModelOrigin,
}

impl std::ops::Deref for CatalogModel {
    type Target = ModelInfo;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl std::ops::DerefMut for CatalogModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.model)
    }
}

impl ModelInfo {
    pub fn resolved_context_window(&self) -> Option<i64> {
        self.context_window.or(self.max_context_window)
    }

    /// Context available to inference after reserving this model's configured headroom.
    pub fn usable_context_window(&self) -> Option<i64> {
        self.resolved_context_window().map(|context_window| {
            context_window.saturating_mul(self.effective_context_window_percent) / 100
        })
    }

    pub fn auto_compact_token_limit(&self) -> Option<i64> {
        let context_limit = self
            .resolved_context_window()
            .map(|context_window| (context_window * 9) / 10);
        let config_limit = self.auto_compact_token_limit;
        if let Some(context_limit) = context_limit {
            return Some(
                config_limit.map_or(context_limit, |limit| std::cmp::min(limit, context_limit)),
            );
        }
        config_limit
    }
}

/// A strongly-typed template for assembling model instructions and developer messages.
///
/// `instructions_template`: Formerly a personality template, now literal text. The name is
/// retained for model catalog compatibility.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ModelMessages {
    /// Optional model-owned Apps wording, transported without runtime interpretation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apps: Option<AppsMessages>,
    /// Developer guidance after a content-filter block. Missing, null, blank, or values over
    /// 512 UTF-8 bytes use the bundled guidance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_filter_guidance: Option<String>,
    /// Additional developer instructions for persistent mode. Missing or null uses the built-in
    /// instructions; an empty string disables them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent_instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolMessages>,
    pub instructions_template: Option<String>,
    pub approvals: Option<ApprovalMessages>,
    pub collaboration_modes: Option<CollaborationModeMessages>,
    pub auto_review: Option<AutoReviewMessages>,
    pub permissions: Option<PermissionMessages>,
    pub multi_agent: Option<MultiAgentMessages>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<ModelTokenBudgetConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardian_v2: Option<GuardianV2ModelConfig>,
    /// Replacement confirmation-policy documents forwarded in actor MCP request metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_policies: Option<ConfirmationPolicies>,
}

/// Optional Apps wording from the model catalog, preserved for future runtime consumption.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct AppsMessages {
    /// Missing and null deserialize as `None`; all strings preserve their exact bytes.
    /// Rendering, size limits, and empty-string behavior are defined by consumers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_instructions: Option<String>,
}

/// Model-owned confirmation-policy Markdown, forwarded unchanged to actor tools.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ConfirmationPolicies {
    /// Replacement Markdown for the Browser Use confirmation-policy document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_use: Option<String>,
    /// Replacement Markdown for the native Computer Use confirmation policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<String>,
}

/// Model-owned tool messages and description prefixes.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ToolMessages {
    /// Native Memento V2 history tool definitions, independent of runtime namespace aliases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryToolMessages>,
    /// Native Memento V2 notes tool definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<NotesToolMessages>,
    /// Memento V2 context-window control tool definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_management: Option<ContextManagementToolMessages>,
    /// Optional wording for incremental catalog notices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incremental_tools: Option<IncrementalToolMessages>,
    /// Prefixes for individual functions in the `functions` namespace, keyed by unqualified name.
    /// Applies to direct declarations, Code Mode, and tool search, including custom tools.
    /// Values are trimmed; empty values and unavailable functions add nothing. The trimmed
    /// values for available functions plus separating blank lines must fit in 256 UTF-8 bytes
    /// in total. This bounds the added text, not the complete definition or its token count.
    /// Other namespaces are never affected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions_namespace_functions_description_prefixes:
        Option<std::collections::BTreeMap<String, String>>,
    /// Optional guidance for indirectly presented tools; missing or empty adds no prefix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indirect_description_prefixes: Option<IndirectDescriptionPrefixes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_user_message_async: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_agent: Option<MultiAgentToolMessages>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_mode: Option<CodeModeToolMessages>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_resources: Option<McpResourceToolMessages>,
}

/// Plain-text prefixes for Code Mode documentation, ALL_TOOLS, and loaded tool-search namespaces.
/// Flat indirect entries carry these before any per-function prefix.
/// Values are trimmed, and selectors for the same final namespace must agree, including empty values.
/// Empty values add no prefix; unregistered targets are ignored.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct IndirectDescriptionPrefixes {
    /// Exact rendered namespaces, including `functions` for plain tools.
    /// Overlap with an MCP server prefix is allowed only when the trimmed values match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespaces: Option<std::collections::BTreeMap<String, String>>,
    /// Configured MCP server names, before callable namespace normalization.
    /// Each prefix applies to all namespaces exposed by that server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<std::collections::BTreeMap<String, String>>,
}

/// Model-owned messages for a built-in tool.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ToolMessage {
    /// Missing or null uses the built-in description; an empty string suppresses its static
    /// text without disabling the tool. Tool-owned runtime guidance is retained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Complete JSON Schema encoded as a string. Consumed by Multi-Agent V2 tools, Code Mode wait,
    /// request_user_input_async (the send_user_message_async catalog key), MCP resource helpers,
    /// and Memento V2 tools.
    /// Uses the harness's supported schema subset; unrecognized keywords are ignored.
    /// Missing, null, invalid or unsupported structures, or a root without `type: "object"`
    /// retains the harness parameters. Schema semantics must remain API-compatible.
    /// Overrides must declare harness-encrypted properties so their annotations can be retained.
    /// Argument handling is unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<String>,
}

/// Model-owned descriptions and parameters for Multi-Agent V2 tools, independent of their namespace.
/// Channel tools consume only the description; their parameter schemas are fixed.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct MultiAgentToolMessages {
    /// Replaces the static description. Missing or null uses the bundled text; an empty string
    /// suppresses it. Generated model information and local usage hints are retained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_agent: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_message: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followup_task: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_agent: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_agent: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_agents: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_channel: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub get_channels: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_threads: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_posts: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_thread: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_post: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsubscribe: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<ToolMessage>,
}

impl MultiAgentToolMessages {
    /// Returns the catalog entry for a tool, independently of its runtime namespace.
    pub fn by_name(&self, name: &str) -> Option<&ToolMessage> {
        match name {
            "spawn_agent" => &self.spawn_agent,
            "send_message" => &self.send_message,
            "followup_task" => &self.followup_task,
            "wait_agent" => &self.wait_agent,
            "interrupt_agent" => &self.interrupt_agent,
            "list_agents" => &self.list_agents,
            "create_channel" => &self.create_channel,
            "get_channels" => &self.get_channels,
            "list_threads" => &self.list_threads,
            "search_posts" => &self.search_posts,
            "read_thread" => &self.read_thread,
            "read_post" => &self.read_post,
            "subscribe" => &self.subscribe,
            "unsubscribe" => &self.unsubscribe,
            "post" => &self.post,
            _ => return None,
        }
        .as_ref()
    }
}

/// Model-owned descriptions and parameters for the built-in MCP resource helpers.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct McpResourceToolMessages {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_mcp_resources: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_mcp_resource_templates: Option<ToolMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_mcp_resource: Option<ToolMessage>,
}

/// Model-owned instructions for Code Mode's exec and wait tools.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct CodeModeToolMessages {
    /// Instructional template supporting `{{ default_exec_yield_time_ms }}` and `{{ image_helper }}`.
    /// Runtime tool declarations are appended. Unknown placeholders remain literal.
    /// Exec accepts raw JavaScript; `parameters` is not consumed and its grammar is unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exec: Option<ToolMessage>,
    /// Complete description and JSON parameter schema, selected independently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait: Option<ToolMessage>,
    /// Literal guidance appended when deferred nested tools exist.
    /// Missing or null uses bundled text; an empty string omits the section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deferred_nested_tools_guidance: Option<String>,
    /// Literal shared TypeScript definitions, emitted when Code Mode Only exposes MCP results.
    /// Missing or null uses bundled definitions; an empty string omits the section.
    /// Custom definitions must remain compatible with the generated tool declarations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_typescript_preamble: Option<String>,
}

/// Model-owned defaults for the context-window token-budget feature.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ModelTokenBudgetConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub use_history_notes_extension: bool,
    pub reminder_threshold_tokens: i64,
    pub reminder_message_template: String,
    pub guidance_message: String,
    pub auto_compact_fallback_prompt: String,
    pub auto_compact_fallback_buffer_tokens: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ApprovalMessages {
    pub on_request: Option<String>,
    pub on_request_auto_review: Option<String>,
    pub never: Option<String>,
    pub unless_trusted: Option<String>,
    /// Optional replacement heading for the request_permissions tool instructions.
    #[serde(
        default,
        alias = "request_permissions_tool",
        skip_serializing_if = "Option::is_none"
    )]
    pub request_permissions: Option<String>,
    /// Optional introduction to the harness-owned automatic escalation review policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_review_suffix: Option<String>,
    /// Optional catalog templates for harness-owned granular approval facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub granular: Option<GranularApprovalMessages>,
}

/// Catalog presentation for granular approval policy; the harness owns the policy and lists.
/// This wire type preserves templates verbatim; validation and rendering are consumer concerns.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct GranularApprovalMessages {
    /// Rendering contract: at most 500 UTF-8 bytes with exactly one `{{ policy }}` marker.
    /// Consumers must use bundled text for missing, null, empty, invalid, or oversized templates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    /// Presentation around `{{ categories }}`, including the fixed prompted-bucket label.
    /// Uses the same byte limit and fallback rules as `intro`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompted_categories: Option<String>,
    /// Presentation around `{{ categories }}`, including the fixed rejected-bucket label.
    /// Uses the same byte limit and fallback rules as `intro`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_categories: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct CollaborationModeMessages {
    pub default: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct AutoReviewMessages {
    pub policy: Option<String>,
    pub policy_template: Option<String>,
    /// Extra developer policy for `node_repl` and `cua_repl` reviews.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_repl_policy: Option<String>,
    pub rejection_instructions: Option<String>,
    pub timeout_instructions: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct PermissionMessages {
    pub danger_full_access: Option<String>,
    pub workspace_write: Option<String>,
    pub read_only: Option<String>,
    /// Supports `{{ approved_prefixes }}`. Missing or null uses the bundled text;
    /// an empty string suppresses the approved-prefix section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_prefixes: Option<String>,
    /// Optional plain heading for the separate saved-command-prefix notification.
    /// The harness formats the prefix list and falls back to its bundled heading for invalid text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_command_prefix_saved: Option<String>,
    /// Single writable-root sentence with exactly one `{{ roots }}` placeholder.
    /// Missing, null, empty, invalid, or oversized templates use the bundled sentence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writable_root: Option<String>,
    /// Multiple writable-roots sentence with exactly one `{{ roots }}` placeholder.
    /// Missing, null, empty, invalid, or oversized templates use the bundled sentence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writable_roots: Option<String>,
    /// Optional presentation before the harness-owned omission and no-bypass warning.
    /// Empty, invalid, or oversized text uses the bundled warning without a lead-in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omitted_paths: Option<String>,
    /// Presentation with exactly one `{{ denied_reads }}` placeholder for the
    /// harness-owned denial explanation, no-escalation rule, and path/glob list.
    /// Empty, invalid, or oversized templates use the bundled presentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denied_reads: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct MultiAgentMessages {
    pub role: Option<MultiAgentRoleMessages>,
    pub mode: Option<MultiAgentModeMessages>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct MultiAgentRoleMessages {
    pub root: Option<String>,
    pub subagent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct MultiAgentModeMessages {
    pub explicit: Option<String>,
    /// Ultra-only mode instructions. Missing or null uses the built-in proactive hint;
    /// an empty string suppresses the mode message. `hint_text` takes precedence.
    pub proactive: Option<String>,
    pub hint_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema)]
pub struct ModelInfoUpgrade {
    pub model: String,
    pub migration_markdown: String,
    /// Informational time when the model associated with this upgrade is scheduled to retire.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_rfc3339_timestamp"
    )]
    #[schemars(with = "Option<String>")]
    #[ts(type = "string", optional)]
    pub retirement_at: Option<DateTime<Utc>>,
}

fn deserialize_optional_rfc3339_timestamp<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc)))
}

impl From<&ModelUpgrade> for ModelInfoUpgrade {
    fn from(upgrade: &ModelUpgrade) -> Self {
        ModelInfoUpgrade {
            model: upgrade.id.clone(),
            migration_markdown: upgrade.migration_markdown.clone().unwrap_or_default(),
            retirement_at: upgrade.retirement_at,
        }
    }
}

/// Response wrapper for `/models`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS, JsonSchema, Default)]
pub struct ModelsResponse {
    #[serde(
        serialize_with = "serialize_model_infos_with_legacy_base",
        deserialize_with = "deserialize_model_infos_with_legacy_base"
    )]
    pub models: Vec<ModelInfo>,
}

#[derive(Serialize)]
struct ModelInfoWithLegacyBaseInstructionsRef<'a> {
    #[serde(flatten)]
    model: &'a ModelInfo,
    base_instructions: String,
}

#[derive(Deserialize)]
struct ModelInfoWithLegacyBaseInstructions {
    #[serde(default)]
    base_instructions: Option<String>,
    #[serde(flatten)]
    model: ModelInfo,
}

/// Serializes catalog models with the deprecated top-level instruction field required by older
/// clients.
#[doc(hidden)]
pub fn serialize_model_infos_with_legacy_base<S>(
    models: &[ModelInfo],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    models
        .iter()
        .map(|model| {
            // Mirror the literal instruction template for clients using the deprecated wire field.
            let base_instructions = if let Some(template) = model
                .model_messages
                .as_ref()
                .and_then(|messages| messages.instructions_template.as_deref())
            {
                template.to_owned()
            } else {
                warn!(
                    model = %model.slug,
                    "Model has no instruction template; returning empty instructions."
                );
                String::new()
            };
            ModelInfoWithLegacyBaseInstructionsRef {
                model,
                base_instructions,
            }
        })
        .collect::<Vec<_>>()
        .serialize(serializer)
}

/// Deserializes catalog models while promoting the legacy top-level instruction field into Model
/// Messages V2 when no canonical instruction template is present.
#[doc(hidden)]
pub fn deserialize_model_infos_with_legacy_base<'de, D>(
    deserializer: D,
) -> Result<Vec<ModelInfo>, D::Error>
where
    D: Deserializer<'de>,
{
    let models = Vec::<ModelInfoWithLegacyBaseInstructions>::deserialize(deserializer)?;
    models
        .into_iter()
        .map(|legacy_model| {
            let ModelInfoWithLegacyBaseInstructions {
                base_instructions,
                mut model,
            } = legacy_model;
            if let Some(base_instructions) = base_instructions
                && model
                    .model_messages
                    .as_ref()
                    .and_then(|messages| messages.instructions_template.as_ref())
                    .is_none()
            {
                let messages = model.model_messages.get_or_insert_default();
                messages.instructions_template = Some(base_instructions);
            }
            if model
                .model_messages
                .as_ref()
                .and_then(|messages| messages.instructions_template.as_ref())
                .is_none()
            {
                let model_slug = &model.slug;
                return Err(D::Error::custom(format!(
                    "model `{model_slug}` is missing both `base_instructions` and \
                     `model_messages.instructions_template`"
                )));
            }
            Ok(model)
        })
        .collect()
}

// convert ModelInfo to ModelPreset
impl From<ModelInfo> for ModelPreset {
    fn from(info: ModelInfo) -> Self {
        let provider_id = info
            .orchestration
            .as_ref()
            .map(|metadata| metadata.provider_id().to_string());
        ModelPreset {
            id: info.slug.clone(),
            model: info.slug.clone(),
            provider_id,
            orchestration: info.orchestration,
            display_name: info.display_name,
            description: info.description.unwrap_or_default(),
            model_specialty: info.model_specialty,
            default_reasoning_effort: info
                .default_reasoning_level
                .unwrap_or(ReasoningEffort::None),
            supported_reasoning_efforts: info.supported_reasoning_levels.clone(),
            supports_personality: false,
            additional_speed_tiers: info.additional_speed_tiers,
            service_tiers: info.service_tiers,
            default_service_tier: info.default_service_tier,
            available_access_programs: info.available_access_programs,
            is_default: false, // default is the highest priority available model
            upgrade: info.upgrade.as_ref().map(|upgrade| ModelUpgrade {
                id: upgrade.model.clone(),
                migration_config_key: info.slug.clone(),
                // todo(aibrahim): add the model link here.
                model_link: None,
                upgrade_copy: None,
                migration_markdown: Some(upgrade.migration_markdown.clone()),
                retirement_at: upgrade.retirement_at,
            }),
            show_in_picker: info.visibility == ModelVisibility::List,
            multi_agent_version: info.multi_agent_version,
            availability_nux: info.availability_nux,
            supported_in_api: info.supported_in_api,
            input_modalities: info.input_modalities,
        }
    }
}

impl ModelPreset {
    pub fn supports_fast_mode(&self) -> bool {
        self.service_tiers
            .iter()
            .any(|tier| tier.id == ServiceTier::Fast.request_value())
            || self
                .additional_speed_tiers
                .iter()
                .any(|tier| tier == SPEED_TIER_FAST)
    }
}

impl ModelInfo {
    pub fn supports_service_tier(&self, service_tier: &str) -> bool {
        // Flex is an API request option, even when the Codex catalog does not advertise it.
        service_tier == ServiceTier::Flex.request_value()
            || self
                .service_tiers
                .iter()
                .any(|tier| tier.id == service_tier)
    }

    pub fn service_tier_for_request(&self, service_tier: Option<String>) -> Option<String> {
        service_tier.filter(|service_tier| {
            service_tier != SERVICE_TIER_DEFAULT_REQUEST_VALUE
                && self.supports_service_tier(service_tier)
        })
    }
}

impl ModelPreset {
    /// Filter models based on authentication mode.
    ///
    /// In ChatGPT mode, all models are visible. Otherwise, only API-supported models are shown.
    pub fn filter_by_auth(models: Vec<ModelPreset>, chatgpt_mode: bool) -> Vec<ModelPreset> {
        models
            .into_iter()
            .filter(|model| chatgpt_mode || model.supported_in_api)
            .collect()
    }

    /// Recompute the single default preset using picker visibility.
    ///
    /// The first picker-visible model wins; if none are picker-visible, the first model wins.
    pub fn mark_default_by_picker_visibility(models: &mut [ModelPreset]) {
        for preset in models.iter_mut() {
            preset.is_default = false;
        }
        if let Some(default) = models.iter_mut().find(|preset| preset.show_in_picker) {
            default.is_default = true;
        } else if let Some(default) = models.first_mut() {
            default.is_default = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::from_str;
    use serde_json::to_string;

    /// DeepSeek V4.1 Flash as published: off-peak half of peak; peak 01-04 and
    /// 06-10 UTC Monday-Friday; no peak on Chinese public holidays.
    #[test]
    fn metered_schedule_charges_the_rate_in_force_at_dispatch() {
        let rates = |input, output, cached| MeteredRates {
            input_milli_usd_per_million_tokens: input,
            output_milli_usd_per_million_tokens: output,
            cached_input_milli_usd_per_million_tokens: Some(cached),
        };
        let billing: ModelBilling = from_str(
            r#"{"kind":"metered_schedule",
                "off_peak":{"input_milli_usd_per_million_tokens":150,"output_milli_usd_per_million_tokens":600,"cached_input_milli_usd_per_million_tokens":3},
                "peak":{"input_milli_usd_per_million_tokens":300,"output_milli_usd_per_million_tokens":1200,"cached_input_milli_usd_per_million_tokens":6},
                "peak_windows":[{"start_utc_hour":1,"end_utc_hour":4},{"start_utc_hour":6,"end_utc_hour":10}],
                "peak_weekdays":{"monday":true,"tuesday":true,"wednesday":true,"thursday":true,"friday":true,"saturday":false,"sunday":false},
                "off_peak_dates_utc":["2026-10-01"]}"#,
        )
        .expect("schedule parses");
        let at = |text: &str| {
            chrono::DateTime::parse_from_rfc3339(text)
                .expect("fixture instant")
                .timestamp_millis()
        };
        let peak = Some(rates(300, 1_200, 6).tuple());
        let off_peak = Some(rates(150, 600, 3).tuple());
        for (instant, expected) in [
            ("2026-09-21T01:00:00Z", peak),     // Monday, first window opens
            ("2026-09-21T03:59:59Z", peak),     // still inside
            ("2026-09-21T04:00:00Z", off_peak), // end is exclusive
            ("2026-09-21T05:30:00Z", off_peak), // between the windows
            ("2026-09-22T06:00:00Z", peak),     // Tuesday, second window
            ("2026-09-22T09:59:59Z", peak),
            ("2026-09-22T10:00:00Z", off_peak),
            ("2026-09-22T00:59:59Z", off_peak),
            ("2026-09-26T02:00:00Z", off_peak), // Saturday
            ("2026-09-27T07:00:00Z", off_peak), // Sunday
            ("2026-10-01T02:00:00Z", off_peak), // Thursday, a listed holiday
            ("2026-10-08T02:00:00Z", peak),     // Thursday after it
        ] {
            assert_eq!(billing.api_key_rates_at(at(instant)), expected, "{instant}");
        }
        // A schedule has no flat rate: nothing may read one without an instant.
        assert_eq!(billing.api_key_rates(), None);
        assert_eq!(
            billing.plan_burn_millis_at(at("2026-09-21T02:00:00Z")),
            None
        );
        // An unreadable instant has no stateable rate.
        assert_eq!(billing.api_key_rates_at(i64::MAX), None);
        // Flat rows are unchanged at every instant.
        let flat = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 1,
            output_milli_usd_per_million_tokens: 2,
            cached_input_milli_usd_per_million_tokens: None,
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: None,
        };
        assert_eq!(
            flat.api_key_rates_at(at("2026-09-21T02:00:00Z")),
            Some((1, 2, None))
        );
        assert_eq!(billing, from_str(&to_string(&billing).unwrap()).unwrap());
    }

    fn at(text: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(text)
            .expect("fixture instant")
            .timestamp_millis()
    }

    #[test]
    fn plan_burn_resolves_flat_rows_and_refuses_metered_ones() {
        let plan = ModelBilling::Plan {
            relative_burn_millis: 2000,
        };
        assert_eq!(
            plan.plan_burn_millis_at(at("2026-09-21T12:00:00Z")),
            Some(2000)
        );
        assert_eq!(plan.api_key_rates(), None);

        let auth = ModelBilling::AuthDependent {
            plan_relative_burn_millis: 500,
            api_key_input_milli_usd_per_million_tokens: 2500,
            api_key_output_milli_usd_per_million_tokens: 15000,
            api_key_cached_input_milli_usd_per_million_tokens: Some(250),
            api_key_cache_write_milli_usd_per_million_tokens: None,
            api_key_long_context: None,
            api_key_valid_through_utc: None,
        };
        assert_eq!(
            auth.plan_burn_millis_at(at("2026-09-21T12:00:00Z")),
            Some(500)
        );
        assert_eq!(auth.api_key_rates(), Some((2500, 15000, Some(250))));

        let metered = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 760,
            output_milli_usd_per_million_tokens: 2420,
            cached_input_milli_usd_per_million_tokens: Some(140),
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: None,
        };
        assert_eq!(
            metered.plan_burn_millis_at(at("2026-09-21T12:00:00Z")),
            None
        );
        assert_eq!(metered.api_key_rates(), Some((760, 2420, Some(140))));
        assert_eq!(
            ModelBilling::Local.plan_burn_millis_at(at("2026-09-21T12:00:00Z")),
            None
        );
        assert_eq!(ModelBilling::Local.api_key_rates(), None);
    }

    #[test]
    fn stated_cache_write_long_context_and_validity_end() {
        let tier = LongContextRates {
            above_input_tokens: 272_000,
            input_milli_usd_per_million_tokens: 8000,
            output_milli_usd_per_million_tokens: 30000,
            cached_input_milli_usd_per_million_tokens: Some(800),
            cache_write_milli_usd_per_million_tokens: Some(10000),
        };
        let auth = ModelBilling::AuthDependent {
            plan_relative_burn_millis: 1000,
            api_key_input_milli_usd_per_million_tokens: 4000,
            api_key_output_milli_usd_per_million_tokens: 20000,
            api_key_cached_input_milli_usd_per_million_tokens: Some(400),
            api_key_cache_write_milli_usd_per_million_tokens: Some(5000),
            api_key_long_context: Some(tier),
            api_key_valid_through_utc: Some("2026-11-21T23:59:59Z".to_string()),
        };
        assert_eq!(auth.api_key_cache_write(), Some(5000));
        assert_eq!(auth.api_key_long_context(), Some(tier));
        // The promotional rates hold through their stated end, and no later.
        assert_eq!(
            auth.api_key_rates_at(at("2026-11-21T23:59:59Z")),
            Some((4000, 20000, Some(400)))
        );
        assert_eq!(auth.api_key_rates_at(at("2026-11-22T00:00:00Z")), None);
        // The plan side is unaffected by the API price's end.
        assert_eq!(
            auth.plan_burn_millis_at(at("2026-11-22T00:00:00Z")),
            Some(1000)
        );
        // An API-key runtime keeps every stated field.
        let ModelBilling::Metered {
            cache_write_milli_usd_per_million_tokens,
            long_context,
            valid_through_utc,
            ..
        } = auth.resolve_auth_mode(/*api_key*/ true)
        else {
            panic!("api-key side resolves to a metered row");
        };
        assert_eq!(
            (cache_write_milli_usd_per_million_tokens, long_context),
            (Some(5000), Some(tier))
        );
        assert_eq!(valid_through_utc.as_deref(), Some("2026-11-21T23:59:59Z"));
        // An end this client cannot read states no price.
        let unreadable = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 1,
            output_milli_usd_per_million_tokens: 2,
            cached_input_milli_usd_per_million_tokens: None,
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: Some("soon".to_string()),
        };
        assert_eq!(
            unreadable.api_key_rates_at(at("2026-01-01T00:00:00Z")),
            None
        );
        // Unstated fields are omitted, so earlier rows keep their bytes.
        let plain = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 1,
            output_milli_usd_per_million_tokens: 2,
            cached_input_milli_usd_per_million_tokens: None,
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: None,
        };
        assert_eq!(
            to_string(&plain).unwrap(),
            r#"{"kind":"metered","input_milli_usd_per_million_tokens":1,"output_milli_usd_per_million_tokens":2,"cached_input_milli_usd_per_million_tokens":null}"#
        );
        assert_eq!(auth, from_str(&to_string(&auth).unwrap()).unwrap());
    }

    #[test]
    fn plan_schedule_resolves_peak_weekdays_and_promotion_expiry() {
        // The shipped glm-5.3 row: peak 06:00-10:00 UTC on weekdays only.
        let weekday_peak = ModelBilling::PlanSchedule {
            off_peak_relative_burn_millis: 1000,
            peak_relative_burn_millis: 3000,
            peak_start_utc_hour: 6,
            peak_end_utc_hour: 10,
            peak_weekdays: Some(WeekdaySet::weekdays_only()),
            promotional_off_peak_relative_burn_millis: None,
            promotion_valid_through_utc: None,
        };
        // 2026-09-21 is a Monday, 2026-09-19 a Saturday.
        for (instant, expected) in [
            ("2026-09-21T05:59:59Z", 1000),
            ("2026-09-21T06:00:00Z", 3000),
            ("2026-09-21T09:59:59Z", 3000),
            ("2026-09-21T10:00:00Z", 1000),
            ("2026-09-19T08:00:00Z", 1000),
        ] {
            assert_eq!(
                weekday_peak.plan_burn_millis_at(at(instant)),
                Some(expected),
                "{instant}"
            );
        }

        // The shipped glm-5.2 row: every day, with a promotion that ends.
        let promoted = ModelBilling::PlanSchedule {
            off_peak_relative_burn_millis: 2000,
            peak_relative_burn_millis: 3000,
            peak_start_utc_hour: 6,
            peak_end_utc_hour: 10,
            peak_weekdays: None,
            promotional_off_peak_relative_burn_millis: Some(1000),
            promotion_valid_through_utc: Some("2026-09-30T23:59:59Z".into()),
        };
        for (instant, expected) in [
            ("2026-09-19T08:00:00Z", 3000),
            ("2026-09-30T23:59:59Z", 1000),
            ("2026-10-01T00:00:00Z", 2000),
        ] {
            assert_eq!(
                promoted.plan_burn_millis_at(at(instant)),
                Some(expected),
                "{instant}"
            );
        }

        // A promotion with no end, or an end this client cannot parse, is not a
        // promotion: the discount must stop without a catalogue release.
        for through in [None, Some("whenever".to_string())] {
            let unbounded = ModelBilling::PlanSchedule {
                off_peak_relative_burn_millis: 2000,
                peak_relative_burn_millis: 3000,
                peak_start_utc_hour: 6,
                peak_end_utc_hour: 10,
                peak_weekdays: None,
                promotional_off_peak_relative_burn_millis: Some(1000),
                promotion_valid_through_utc: through,
            };
            assert_eq!(
                unbounded.plan_burn_millis_at(at("2026-09-21T12:00:00Z")),
                Some(2000)
            );
        }
    }

    #[test]
    fn plan_schedule_peak_window_may_wrap_past_midnight() {
        // 22:00-02:00 opening on Fridays: the Saturday small hours still belong to
        // the Friday window, and Saturday night does not open one.
        let overnight = ModelBilling::PlanSchedule {
            off_peak_relative_burn_millis: 1000,
            peak_relative_burn_millis: 4000,
            peak_start_utc_hour: 22,
            peak_end_utc_hour: 2,
            peak_weekdays: Some(WeekdaySet {
                monday: false,
                tuesday: false,
                wednesday: false,
                thursday: false,
                friday: true,
                saturday: false,
                sunday: false,
            }),
            promotional_off_peak_relative_burn_millis: None,
            promotion_valid_through_utc: None,
        };
        // 2026-09-18 is a Friday.
        for (instant, expected) in [
            ("2026-09-18T21:59:59Z", 1000),
            ("2026-09-18T22:30:00Z", 4000),
            ("2026-09-19T01:59:59Z", 4000),
            ("2026-09-19T02:00:00Z", 1000),
            ("2026-09-19T23:00:00Z", 1000),
        ] {
            assert_eq!(
                overnight.plan_burn_millis_at(at(instant)),
                Some(expected),
                "{instant}"
            );
        }

        // A degenerate window charges off-peak rather than charging peak forever.
        let degenerate = ModelBilling::PlanSchedule {
            off_peak_relative_burn_millis: 1000,
            peak_relative_burn_millis: 4000,
            peak_start_utc_hour: 6,
            peak_end_utc_hour: 6,
            peak_weekdays: None,
            promotional_off_peak_relative_burn_millis: None,
            promotion_valid_through_utc: None,
        };
        assert_eq!(
            degenerate.plan_burn_millis_at(at("2026-09-21T06:30:00Z")),
            Some(1000)
        );
    }

    #[test]
    fn legacy_shell_model_metadata_deserializes_as_unified_exec() {
        for legacy_shell_type in ["default", "local", "shell_command"] {
            assert_eq!(
                from_str::<ConfigShellToolType>(&format!("\"{legacy_shell_type}\""))
                    .expect("legacy shell type"),
                ConfigShellToolType::UnifiedExec
            );
        }
        assert_eq!(
            to_string(&ConfigShellToolType::UnifiedExec).expect("serialize unified shell type"),
            "\"unified_exec\""
        );
    }

    pub(super) fn test_model(spec: Option<ModelMessages>) -> ModelInfo {
        ModelInfo {
            slug: "test-model".to_string(),
            display_name: "Test Model".to_string(),
            description: None,
            orchestration: None,
            chat_completions: Default::default(),
            default_reasoning_level: None,
            supported_reasoning_levels: vec![],
            shell_type: ConfigShellToolType::UnifiedExec,
            visibility: ModelVisibility::List,
            supported_in_api: true,
            priority: 1,
            additional_speed_tiers: Vec::new(),
            service_tiers: Vec::new(),
            default_service_tier: None,
            available_access_programs: None,
            availability_nux: None,
            upgrade: None,
            model_messages: spec,
            include_skills_usage_instructions: false,
            include_plugin_usage_instructions: false,
            include_apps_usage_instructions: false,
            supports_reasoning_summary_parameter: true,
            default_reasoning_summary: ReasoningSummary::Auto,
            support_verbosity: false,
            default_verbosity: None,
            apply_patch_tool_type: None,
            web_search_tool_type: WebSearchToolType::Text,
            truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
            supports_image_detail_original: false,
            context_window: None,
            max_context_window: None,
            max_output_tokens: None,
            auto_compact_token_limit: None,
            comp_hash: None,
            effective_context_window_percent: 95,
            experimental_supported_tools: vec![],
            input_modalities: default_input_modalities(),
            supports_search_tool: false,
            supports_experimental_context: false,
            use_responses_lite: false,
            supports_reasoning_effort_updates: false,
            guardian: None,
            node_repl_auto_review_required: false,
            node_repl_disabled: false,
            auto_review_model_override: None,
            model_specialty: None,
            tool_mode: None,
            multi_agent_version: None,
            multi_agent_reasoning_effort: None,
        }
    }
    #[test]
    fn model_messages_deserialize_without_optional_sections() {
        let messages: ModelMessages =
            from_str(r#"{"instructions_template":null,"persistent_instructions":null}"#)
                .expect("model messages should deserialize");

        assert_eq!(messages, ModelMessages::default());
    }

    #[test]
    fn apps_messages_preserve_nullable_and_literal_wire_values() {
        for apps in [
            serde_json::Value::Null,
            serde_json::json!({}),
            serde_json::json!({"usage_instructions": null}),
            serde_json::json!({"usage_instructions": ""}),
            serde_json::json!({"usage_instructions": " \r\n\t"}),
            serde_json::json!({"usage_instructions": "  {{literal}}\r\n"}),
            serde_json::json!({"usage_instructions": "é".repeat(501)}),
        ] {
            let messages: ModelMessages =
                serde_json::from_value(serde_json::json!({"apps": apps})).unwrap();
            let serialized = serde_json::to_value(messages).unwrap();
            let mut expected = serde_json::to_value(ModelMessages::default()).unwrap();
            if !apps.is_null() {
                expected["apps"] = if apps["usage_instructions"].is_null() {
                    serde_json::json!({})
                } else {
                    apps
                };
            }
            assert_eq!(serialized, expected);
        }
    }

    #[test]
    fn send_user_message_async_description_preserves_missing_null_and_empty_values() {
        for (value, expected) in [
            (serde_json::json!({}), None),
            (serde_json::json!({"tools": null}), None),
            (serde_json::json!({"tools": {}}), Some(None)),
            (
                serde_json::json!({"tools": {"send_user_message_async": null}}),
                Some(None),
            ),
            (
                serde_json::json!({"tools": {"send_user_message_async": {}}}),
                Some(Some(ToolMessage::default())),
            ),
            (
                serde_json::json!({"tools": {"send_user_message_async": {"description": null}}}),
                Some(Some(ToolMessage::default())),
            ),
            (
                serde_json::json!({"tools": {"send_user_message_async": {"description": ""}}}),
                Some(Some(ToolMessage {
                    description: Some(String::new()),
                    ..Default::default()
                })),
            ),
            (
                serde_json::json!({"tools": {"send_user_message_async": {"description": "Catalog description"}}}),
                Some(Some(ToolMessage {
                    description: Some("Catalog description".to_string()),
                    ..Default::default()
                })),
            ),
        ] {
            let messages: ModelMessages =
                serde_json::from_value(value).expect("model messages should deserialize");
            let serialized =
                serde_json::to_value(&messages).expect("model messages should serialize");
            assert_eq!(
                (
                    messages.tools,
                    serialized
                        .get("tools")
                        .map(|tools| tools.get("send_user_message_async").cloned()),
                ),
                (
                    expected.as_ref().map(|tool| ToolMessages {
                        send_user_message_async: tool.clone(),
                        ..Default::default()
                    }),
                    expected.map(|tool| tool.map(|tool| match tool.description {
                        Some(description) => serde_json::json!({"description": description}),
                        None => serde_json::json!({}),
                    })),
                )
            );
        }
    }

    #[test]
    fn spawn_agent_messages_preserve_sparse_and_empty_values() {
        for (value, expected) in [
            (serde_json::json!({}), serde_json::json!({})),
            (
                serde_json::json!({"multi_agent": null}),
                serde_json::json!({}),
            ),
            (
                serde_json::json!({"multi_agent": {"spawn_agent": null}}),
                serde_json::json!({"multi_agent": {}}),
            ),
            (
                serde_json::json!({"multi_agent": {"spawn_agent": {"description": null}}}),
                serde_json::json!({"multi_agent": {"spawn_agent": {}}}),
            ),
            (
                serde_json::json!({"multi_agent": {"spawn_agent": {"description": "Catalog spawn"}}}),
                serde_json::json!({"multi_agent": {"spawn_agent": {"description": "Catalog spawn"}}}),
            ),
            (
                serde_json::json!({"multi_agent": {"spawn_agent": {"description": ""}}}),
                serde_json::json!({"multi_agent": {"spawn_agent": {"description": ""}}}),
            ),
        ] {
            let tools: ToolMessages =
                serde_json::from_value(value).expect("deserialize tool messages");
            assert_eq!(
                serde_json::to_value(&tools).expect("serialize tool messages"),
                expected
            );
            let restored: ToolMessages =
                serde_json::from_value(expected).expect("restore tool messages");
            assert_eq!(restored, tools);
        }
    }

    #[test]
    fn approval_messages_preserve_missing_and_empty_values() {
        let messages: ModelMessages = from_str(
            r#"{
                "instructions_template": null,
                "approvals": {
                    "on_request": "",
                    "never": "",
                    "request_permissions": "",
                    "auto_review_suffix": null
                }
            }"#,
        )
        .expect("approval messages should deserialize");

        assert_eq!(
            messages.approvals,
            Some(ApprovalMessages {
                on_request: Some(String::new()),
                request_permissions: Some(String::new()),
                on_request_auto_review: None,
                never: Some(String::new()),
                unless_trusted: None,
                granular: None,
                ..Default::default()
            })
        );

        for field in ["request_permissions", "request_permissions_tool"] {
            for heading in [
                serde_json::Value::Null,
                serde_json::json!(""),
                serde_json::json!("Permission requests"),
            ] {
                let approvals: ApprovalMessages =
                    serde_json::from_value(serde_json::json!({ field: heading }))
                        .expect("permission heading should deserialize with either field name");
                let mut expected = serde_json::json!({
                    "on_request": null,
                    "on_request_auto_review": null,
                    "never": null,
                    "unless_trusted": null,
                });
                if !heading.is_null() {
                    expected["request_permissions"] = heading;
                }
                assert_eq!(
                    serde_json::to_value(approvals).expect("serialize approvals"),
                    expected
                );
            }
        }

        for (granular, expected) in [
            (serde_json::json!(null), None),
            (serde_json::json!({}), Some(serde_json::json!({}))),
            (
                serde_json::json!({"intro": null, "prompted_categories": null, "rejected_categories": null}),
                Some(serde_json::json!({})),
            ),
            (
                serde_json::json!({"intro": ""}),
                Some(serde_json::json!({"intro": ""})),
            ),
            (
                serde_json::json!({
                    "intro": "Catalog: {{ policy }}",
                    "prompted_categories": "Allowed: {{ categories }}",
                    "rejected_categories": "Denied: {{ categories }}",
                }),
                Some(serde_json::json!({
                    "intro": "Catalog: {{ policy }}",
                    "prompted_categories": "Allowed: {{ categories }}",
                    "rejected_categories": "Denied: {{ categories }}",
                })),
            ),
        ] {
            let wire = serde_json::json!({"approvals": {
                "on_request": "existing override", "granular": granular,
            }});
            let messages: ModelMessages =
                serde_json::from_value(wire).expect("sparse granular messages");
            let roundtrip = serde_json::to_value(messages.approvals).expect("serialize approvals");
            let mut expected_approvals = serde_json::json!({
                "on_request": "existing override",
                "on_request_auto_review": null,
                "never": null,
                "unless_trusted": null,
            });
            if let Some(expected) = expected {
                expected_approvals["granular"] = expected;
            }
            assert_eq!(roundtrip, expected_approvals);
        }
    }

    #[test]
    fn auto_review_messages_preserve_missing_and_empty_values() {
        let missing_template: ModelMessages = from_str(
            r#"{
                "instructions_template": null,
                "auto_review": {
                    "policy": "policy"
                }
            }"#,
        )
        .expect("auto-review messages should deserialize without a policy template");
        let empty_template: ModelMessages = from_str(
            r#"{
                "instructions_template": null,
                "auto_review": {
                    "policy": "policy",
                    "policy_template": "",
                    "node_repl_policy": "",
                    "rejection_instructions": "",
                    "timeout_instructions": ""
                }
            }"#,
        )
        .expect("auto-review messages should deserialize with an empty policy template");

        assert_eq!(
            missing_template.auto_review,
            Some(AutoReviewMessages {
                policy: Some("policy".to_string()),
                policy_template: None,
                node_repl_policy: None,
                rejection_instructions: None,
                timeout_instructions: None,
            })
        );
        assert_eq!(
            empty_template.auto_review,
            Some(AutoReviewMessages {
                policy: Some("policy".to_string()),
                policy_template: Some(String::new()),
                node_repl_policy: Some(String::new()),
                rejection_instructions: Some(String::new()),
                timeout_instructions: Some(String::new()),
            })
        );
    }

    #[test]
    fn permission_messages_preserve_missing_and_empty_values() {
        let messages: ModelMessages = from_str(
            r#"{
                "instructions_template": null,
                "permissions": {
                    "workspace_write": ""
                }
            }"#,
        )
        .expect("permission messages should deserialize");

        assert_eq!(
            messages.permissions,
            Some(PermissionMessages {
                danger_full_access: None,
                workspace_write: Some(String::new()),
                read_only: None,
                approved_prefixes: None,
                ..PermissionMessages::default()
            })
        );
    }

    #[test]
    fn permission_messages_preserve_sparse_wire_values() {
        for input in [
            serde_json::json!({}),
            serde_json::json!({"approved_command_prefix_saved": null}),
            serde_json::json!({"approved_command_prefix_saved": ""}),
            serde_json::json!({"approved_command_prefix_saved": "Saved", "approved_prefixes": "Allowed: {{ approved_prefixes }}"}),
            serde_json::json!({"writable_root": null, "writable_roots": null}),
            serde_json::json!({"writable_root": "", "writable_roots": "Roots: {{ roots }}."}),
            serde_json::json!({"omitted_paths": null, "denied_reads": null}),
            serde_json::json!({"omitted_paths": "", "denied_reads": "Blocked:\n{{ denied_reads }}"}),
        ] {
            let messages: PermissionMessages = serde_json::from_value(input.clone()).unwrap();
            let mut expected = serde_json::json!({
                "danger_full_access": null,
                "workspace_write": null,
                "read_only": null,
            });
            for key in [
                "approved_command_prefix_saved",
                "approved_prefixes",
                "writable_root",
                "writable_roots",
                "omitted_paths",
                "denied_reads",
            ] {
                if let Some(value) = input.get(key).filter(|value| !value.is_null()) {
                    expected[key] = value.clone();
                }
            }
            assert_eq!(serde_json::to_value(messages).unwrap(), expected);
        }
    }

    #[test]
    fn multi_agent_messages_preserve_missing_and_empty_values() {
        let messages: ModelMessages = from_str(
            r#"{"instructions_template":null,"multi_agent":{"role":{"root":"","subagent":"subagent base"},"mode":{"explicit":"explicit mode","proactive":"","hint_text":""}}}"#,
        )
        .expect("multi-agent messages should deserialize");

        assert_eq!(
            messages.multi_agent,
            Some(MultiAgentMessages {
                role: Some(MultiAgentRoleMessages {
                    root: Some(String::new()),
                    subagent: Some("subagent base".to_string()),
                }),
                mode: Some(MultiAgentModeMessages {
                    explicit: Some("explicit mode".to_string()),
                    proactive: Some(String::new()),
                    hint_text: Some(String::new()),
                }),
            })
        );
    }

    #[test]
    fn collaboration_mode_messages_preserve_missing_and_empty_values() {
        let messages: ModelMessages = from_str(
            r#"{
                "instructions_template": "legacy catalog instructions",
                "instructions_variables": {
                    "personality_default": "default",
                    "personality_friendly": "friendly",
                    "personality_pragmatic": "pragmatic"
                },
                "collaboration_modes": {
                    "default": ""
                }
            }"#,
        )
        .expect("collaboration mode messages should deserialize");

        assert_eq!(
            messages,
            ModelMessages {
                instructions_template: Some("legacy catalog instructions".to_string()),
                collaboration_modes: Some(CollaborationModeMessages {
                    default: Some(String::new()),
                    plan: None,
                }),
                ..Default::default()
            }
        );
        let roundtripped: ModelMessages = serde_json::from_value(
            serde_json::to_value(&messages).expect("serialize model messages"),
        )
        .expect("deserialize model messages");
        assert_eq!(roundtripped, messages);
    }

    #[test]
    fn reasoning_effort_accepts_known_and_custom_values() {
        let custom = ReasoningEffort::Custom("future".to_string());
        let deserialized = from_str::<ReasoningEffort>(r#""future""#)
            .expect("custom reasoning effort should deserialize");
        let serialized = to_string(&custom).expect("custom reasoning effort should serialize");
        let serialized_max = to_string(&ReasoningEffort::Max).expect("Max should serialize");
        let serialized_ultra = to_string(&ReasoningEffort::Ultra).expect("Ultra should serialize");
        let serialized_persistent =
            to_string(&ReasoningEffort::Persistent).expect("Persistent should serialize");

        assert_eq!(
            (
                "high".parse(),
                "max".parse(),
                "ultra".parse(),
                "persistent".parse(),
                "future".parse(),
                deserialized,
                serialized,
                serialized_max,
                serialized_ultra,
                serialized_persistent,
                custom.to_string(),
            ),
            (
                Ok(ReasoningEffort::High),
                Ok(ReasoningEffort::Max),
                Ok(ReasoningEffort::Ultra),
                Ok(ReasoningEffort::Persistent),
                Ok(custom.clone()),
                custom,
                r#""future""#.to_string(),
                r#""max""#.to_string(),
                r#""ultra""#.to_string(),
                r#""persistent""#.to_string(),
                "future".to_string(),
            )
        );
    }

    #[test]
    fn plan_schedule_without_peak_weekdays_keeps_legacy_daily_semantics() {
        let billing: ModelBilling = from_str(
            r#"{
                "kind": "plan_schedule",
                "off_peak_relative_burn_millis": 1000,
                "peak_relative_burn_millis": 3000,
                "peak_start_utc_hour": 6,
                "peak_end_utc_hour": 10
            }"#,
        )
        .expect("legacy plan schedule should deserialize");

        assert_eq!(
            billing,
            ModelBilling::PlanSchedule {
                off_peak_relative_burn_millis: 1_000,
                peak_relative_burn_millis: 3_000,
                peak_start_utc_hour: 6,
                peak_end_utc_hour: 10,
                peak_weekdays: None,
                promotional_off_peak_relative_burn_millis: None,
                promotion_valid_through_utc: None,
            }
        );
        assert_eq!(
            WeekdaySet::weekdays_only().names(),
            vec!["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
        );
    }

    #[test]
    fn reasoning_effort_rejects_empty_values() {
        assert_eq!(
            "".parse::<ReasoningEffort>(),
            Err("reasoning_effort must not be empty".to_string())
        );
    }

    #[test]
    fn reasoning_effort_json_schema_is_an_open_string() {
        let mut effort_generator = SchemaGenerator::default();

        assert_eq!(
            ReasoningEffort::json_schema(&mut effort_generator),
            Schema::Object(SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "A non-empty reasoning effort value advertised by the model.".to_string(),
                    ),
                    ..Default::default()
                })),
                string: Some(Box::new(StringValidation {
                    min_length: Some(1),
                    ..Default::default()
                })),
                ..Default::default()
            })
        );
    }

    #[test]
    fn models_response_promotes_legacy_base_instructions() {
        let mut value = serde_json::to_value(ModelsResponse {
            models: vec![test_model(/*spec*/ None)],
        })
        .expect("serialize models response");
        value["models"][0]["base_instructions"] = serde_json::json!("legacy instructions");

        let response: ModelsResponse =
            serde_json::from_value(value).expect("deserialize legacy models response");
        let model = &response.models[0];

        assert_eq!(
            model.model_messages,
            Some(ModelMessages {
                instructions_template: Some("legacy instructions".to_string()),
                ..Default::default()
            })
        );
        let serialized = serde_json::to_value(response).expect("serialize canonical response");
        assert_eq!(
            serialized["models"][0]["base_instructions"],
            "legacy instructions"
        );
    }

    #[test]
    fn models_response_rejects_model_without_instruction_source() {
        let mut value = serde_json::to_value(ModelsResponse {
            models: vec![test_model(/*spec*/ None)],
        })
        .expect("serialize models response");
        value["models"][0]
            .as_object_mut()
            .expect("model should serialize as an object")
            .remove("base_instructions")
            .expect("serialized model should include legacy base instructions");

        let error = serde_json::from_value::<ModelsResponse>(value)
            .expect_err("model without instructions should be rejected");

        assert_eq!(
            error.to_string(),
            "model `test-model` is missing both `base_instructions` and \
             `model_messages.instructions_template`"
        );
    }

    #[test]
    fn models_response_serializes_literal_legacy_base_instructions() {
        for (template, expected) in [
            (
                Some("before {{ personality }} after"),
                "before {{ personality }} after",
            ),
            (Some(""), ""),
            (None, ""),
        ] {
            let response = ModelsResponse {
                models: vec![test_model(Some(ModelMessages {
                    instructions_template: template.map(str::to_owned),
                    ..Default::default()
                }))],
            };
            let serialized = serde_json::to_value(response).expect("serialize models response");
            assert_eq!(serialized["models"][0]["base_instructions"], expected);
        }
    }

    #[test]
    fn models_response_prefers_template_and_preserves_message_siblings() {
        let messages = ModelMessages {
            apps: Some(AppsMessages {
                usage_instructions: Some("Literal Apps {{guidance}}".to_string()),
            }),
            content_filter_guidance: Some("Offer a permitted alternative.".to_string()),
            persistent_instructions: Some("Persistent catalog instructions".to_string()),
            tools: Some(ToolMessages {
                send_user_message_async: Some(ToolMessage {
                    description: Some("Catalog description".to_string()),
                    ..Default::default()
                }),
                multi_agent: Some(MultiAgentToolMessages {
                    spawn_agent: Some(ToolMessage {
                        description: Some("Catalog spawn description".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            instructions_template: None,
            approvals: Some(ApprovalMessages {
                on_request: Some("approval".to_string()),
                on_request_auto_review: None,
                never: Some("never approval".to_string()),
                unless_trusted: Some("unless-trusted approval".to_string()),
                granular: Some(GranularApprovalMessages {
                    intro: Some("Catalog: {{ policy }}".to_string()),
                    prompted_categories: Some("Allowed: {{ categories }}".to_string()),
                    rejected_categories: Some("Denied: {{ categories }}".to_string()),
                }),
                ..Default::default()
            }),
            collaboration_modes: Some(CollaborationModeMessages {
                default: Some("default collaboration".to_string()),
                plan: Some("plan collaboration".to_string()),
            }),
            auto_review: Some(AutoReviewMessages {
                policy: Some("policy".to_string()),
                policy_template: None,
                node_repl_policy: None,
                rejection_instructions: Some("rejection instructions".to_string()),
                timeout_instructions: Some("timeout instructions".to_string()),
            }),
            permissions: Some(PermissionMessages {
                danger_full_access: None,
                workspace_write: Some("workspace".to_string()),
                read_only: None,
                approved_prefixes: None,
                ..PermissionMessages::default()
            }),
            multi_agent: None,
            token_budget: None,
            confirmation_policies: Some(ConfirmationPolicies {
                browser_use: Some(
                    "# Browser confirmations\n\nKeep {{literal_markdown}}.\n".to_string(),
                ),
                computer_use: Some(
                    "  # Native confirmations\r\n\nKeep ${native_markdown}.\n".to_string(),
                ),
            }),
            guardian_v2: Some(GuardianV2ModelConfig {
                classifier_instructions: Some("Guardian classification".to_string()),
                review_threshold_basis_points: Some(7_500),
                reasoning_effort: Some(ReasoningEffort::Minimal),
                transcript: Some(GuardianV2TranscriptModelConfig {
                    sources: Some(vec!["reasoning".to_string()]),
                    max_tool_entry_tokens: Some(500),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };
        let mut value = serde_json::to_value(ModelsResponse {
            models: vec![test_model(Some(messages.clone()))],
        })
        .expect("serialize models response");
        assert_eq!(
            value["models"][0]["model_messages"]["confirmation_policies"],
            serde_json::json!({
                "browser_use": "# Browser confirmations\n\nKeep {{literal_markdown}}.\n",
                "computer_use": "  # Native confirmations\r\n\nKeep ${native_markdown}.\n",
            })
        );
        value["models"][0]["base_instructions"] = serde_json::json!("legacy instructions");

        let response: ModelsResponse =
            serde_json::from_value(value).expect("deserialize legacy models response");
        let mut expected_messages = messages;
        expected_messages.instructions_template = Some("legacy instructions".to_string());
        assert_eq!(response.models[0].model_messages, Some(expected_messages));

        let canonical_messages = ModelMessages {
            persistent_instructions: Some(String::new()),
            tools: Some(ToolMessages {
                send_user_message_async: Some(ToolMessage {
                    description: Some(String::new()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            instructions_template: Some("canonical instructions".to_string()),
            ..Default::default()
        };
        let mut value = serde_json::to_value(ModelsResponse {
            models: vec![test_model(Some(canonical_messages.clone()))],
        })
        .expect("serialize models response");
        value["models"][0]["base_instructions"] = serde_json::json!("legacy instructions");

        let response: ModelsResponse =
            serde_json::from_value(value).expect("deserialize mixed models response");
        assert_eq!(response.models[0].model_messages, Some(canonical_messages));
    }

    #[test]
    fn model_info_defaults_availability_nux_to_none_when_omitted() {
        let model: ModelInfo = serde_json::from_value(serde_json::json!({
            "slug": "test-model",
            "display_name": "Test Model",
            "description": null,
            "supported_reasoning_levels": [],
            "shell_type": "unified_exec",
            "visibility": "list",
            "supported_in_api": true,
            "priority": 1,
            "upgrade": null,
            "model_messages": null,
            "default_reasoning_summary": "auto",
            "support_verbosity": false,
            "default_verbosity": null,
            "apply_patch_tool_type": null,
            "truncation_policy": {
                "mode": "bytes",
                "limit": 10000
            },
            "supports_image_detail_original": false,
            "context_window": null,
            "auto_compact_token_limit": null,
            "effective_context_window_percent": 95,
            "experimental_supported_tools": []
        }))
        .expect("deserialize model info");

        assert_eq!(model.availability_nux, None);
        assert_eq!(
            model.input_modalities,
            vec![InputModality::Text, InputModality::Image]
        );
        assert!(!model.include_skills_usage_instructions);
        assert!(!model.include_plugin_usage_instructions);
        assert!(model.include_apps_usage_instructions);
        assert!(model.supports_reasoning_summary_parameter);
        assert!(!model.supports_image_detail_original);
        assert_eq!(model.web_search_tool_type, WebSearchToolType::Text);
        assert!(!model.supports_search_tool);
        assert!(!model.supports_experimental_context);
        assert!(!model.use_responses_lite);
        assert!(!model.supports_reasoning_effort_updates);
        assert!(!model.node_repl_auto_review_required);
        assert!(!model.node_repl_disabled);
        assert_eq!(model.comp_hash, None);
        assert_eq!(model.auto_review_model_override, None);
        assert_eq!(model.tool_mode, None);
        assert_eq!(model.multi_agent_reasoning_effort, None);
    }

    #[test]
    fn model_info_deserializes_multi_agent_reasoning_effort() {
        let mut value =
            serde_json::to_value(test_model(/*spec*/ None)).expect("serialize test model");
        value
            .as_object_mut()
            .expect("model info should be an object")
            .insert(
                "multi_agent_reasoning_effort".to_string(),
                serde_json::Value::String("high".to_string()),
            );

        let model = serde_json::from_value::<ModelInfo>(value)
            .expect("deserialize multi-agent reasoning effort");

        assert_eq!(
            model.multi_agent_reasoning_effort,
            Some(ReasoningEffort::High)
        );
    }

    #[test]
    fn model_info_deserializes_optional_upgrade_retirement_at() {
        let base = serde_json::to_value(test_model(/*spec*/ None))
            .expect("serialize test model without retirement time");

        let mut absent = base.clone();
        absent
            .as_object_mut()
            .expect("model info should be an object")
            .insert(
                "upgrade".to_string(),
                serde_json::json!({
                    "model": "replacement-model",
                    "migration_markdown": "Use the replacement model."
                }),
            );
        let absent = serde_json::from_value::<ModelInfo>(absent)
            .expect("deserialize model info without upgrade retirement time");
        assert_eq!(
            absent
                .upgrade
                .as_ref()
                .and_then(|upgrade| upgrade.retirement_at.as_ref())
                .map(DateTime::timestamp),
            None
        );

        let mut null = base.clone();
        null.as_object_mut()
            .expect("model info should be an object")
            .insert(
                "upgrade".to_string(),
                serde_json::json!({
                    "model": "replacement-model",
                    "migration_markdown": "Use the replacement model.",
                    "retirement_at": null
                }),
            );
        let null = serde_json::from_value::<ModelInfo>(null)
            .expect("deserialize model info with null upgrade retirement time");
        assert_eq!(
            null.upgrade
                .as_ref()
                .and_then(|upgrade| upgrade.retirement_at.as_ref())
                .map(DateTime::timestamp),
            None
        );

        let mut populated = base;
        populated
            .as_object_mut()
            .expect("model info should be an object")
            .insert(
                "upgrade".to_string(),
                serde_json::json!({
                    "model": "replacement-model",
                    "migration_markdown": "Use the replacement model.",
                    "retirement_at": "2030-01-01T00:00:00Z"
                }),
            );
        let populated = serde_json::from_value::<ModelInfo>(populated)
            .expect("deserialize model info with upgrade retirement time");
        assert_eq!(
            populated
                .upgrade
                .as_ref()
                .and_then(|upgrade| upgrade.retirement_at.as_ref())
                .map(DateTime::timestamp),
            Some(1_893_456_000)
        );

        let mut malformed = serde_json::to_value(test_model(/*spec*/ None))
            .expect("serialize test model for malformed retirement time");
        malformed
            .as_object_mut()
            .expect("model info should be an object")
            .insert(
                "upgrade".to_string(),
                serde_json::json!({
                    "model": "replacement-model",
                    "migration_markdown": "Use the replacement model.",
                    "retirement_at": "not-a-timestamp"
                }),
            );
        let malformed = serde_json::from_value::<ModelInfo>(malformed)
            .expect("tolerate malformed upgrade retirement time");
        assert_eq!(
            malformed
                .upgrade
                .as_ref()
                .and_then(|upgrade| upgrade.retirement_at.as_ref())
                .map(DateTime::timestamp),
            None
        );
    }

    #[test]
    fn model_info_preserves_explicit_apps_guidance_opt_out() {
        let value = serde_json::to_value(test_model(/*spec*/ None))
            .expect("serialize model info with explicit apps guidance opt-out");
        assert_eq!(value["include_apps_usage_instructions"], false);

        let model = serde_json::from_value::<ModelInfo>(value).expect("deserialize model info");
        assert!(!model.include_apps_usage_instructions);
    }

    #[test]
    fn model_info_deserializes_known_tool_mode() {
        let mut value =
            serde_json::to_value(test_model(/*spec*/ None)).expect("serialize test model");
        let object = value
            .as_object_mut()
            .expect("model info should be an object");
        object.insert(
            "tool_mode".to_string(),
            serde_json::Value::String("code_mode_only".to_string()),
        );
        let model = serde_json::from_value::<ModelInfo>(value).expect("deserialize model info");

        assert_eq!(model.tool_mode, Some(ToolMode::CodeModeOnly));
    }

    #[test]
    fn model_info_treats_unknown_tool_mode_as_omitted() {
        let mut value =
            serde_json::to_value(test_model(/*spec*/ None)).expect("serialize test model");
        let object = value
            .as_object_mut()
            .expect("model info should be an object");
        object.insert(
            "tool_mode".to_string(),
            serde_json::Value::String("future_tool_mode".to_string()),
        );
        let model = serde_json::from_value::<ModelInfo>(value).expect("deserialize model info");

        assert_eq!(model.tool_mode, None);
        let serialized = serde_json::to_value(model).expect("serialize model info");
        let object = serialized
            .as_object()
            .expect("model info should be an object");
        assert!(!object.contains_key("tool_mode"));
    }

    #[test]
    fn model_info_treats_unknown_multi_agent_version_as_omitted() {
        let mut value =
            serde_json::to_value(test_model(/*spec*/ None)).expect("serialize test model");
        let object = value
            .as_object_mut()
            .expect("model info should be an object");
        object.insert(
            "multi_agent_version".to_string(),
            serde_json::Value::String("future_multi_agent_version".to_string()),
        );
        let model = serde_json::from_value::<ModelInfo>(value).expect("deserialize model info");

        assert_eq!(model.multi_agent_version, None);
    }

    #[test]
    fn resolved_context_window_prefers_context_window() {
        let model = ModelInfo {
            context_window: Some(273_000),
            max_context_window: Some(400_000),
            ..test_model(/*spec*/ None)
        };

        assert_eq!(model.resolved_context_window(), Some(273_000));
    }

    #[test]
    fn resolved_context_window_falls_back_to_max_context_window() {
        let model = ModelInfo {
            context_window: None,
            max_context_window: Some(400_000),
            ..test_model(/*spec*/ None)
        };

        assert_eq!(model.resolved_context_window(), Some(400_000));
        assert_eq!(model.usable_context_window(), Some(380_000));
        assert_eq!(model.auto_compact_token_limit(), Some(360_000));
    }

    #[test]
    fn model_context_window_limits_preserve_their_distinct_meanings() {
        let model = ModelInfo {
            context_window: Some(272_000),
            max_context_window: Some(400_000),
            auto_compact_token_limit: Some(250_000),
            effective_context_window_percent: 95,
            ..test_model(/*spec*/ None)
        };

        assert_eq!(
            (
                model.resolved_context_window(),
                model.usable_context_window(),
                model.auto_compact_token_limit(),
            ),
            (Some(272_000), Some(258_400), Some(244_800))
        );
    }

    #[test]
    fn model_preset_preserves_availability_nux() {
        let preset = ModelPreset::from(ModelInfo {
            availability_nux: Some(ModelAvailabilityNux {
                message: "Try Spark.".to_string(),
            }),
            additional_speed_tiers: vec![SPEED_TIER_FAST.to_string()],
            default_service_tier: Some(ServiceTier::Fast.request_value().to_string()),
            service_tiers: Vec::new(),
            ..test_model(/*spec*/ None)
        });

        assert_eq!(
            preset.availability_nux,
            Some(ModelAvailabilityNux {
                message: "Try Spark.".to_string(),
            })
        );
        assert!(preset.supports_fast_mode());
        assert_eq!(
            preset.default_service_tier,
            Some(ServiceTier::Fast.request_value().to_string())
        );
    }

    #[test]
    fn model_access_programs_preserve_explicit_selection_availability() {
        use crate::turn_input::CyberAccessProgram;

        for (metadata, expected) in [
            (None, None),
            (Some(serde_json::Value::Null), None),
            (
                Some(serde_json::json!({ "cyber": [] })),
                Some(ModelAccessPrograms { cyber: vec![] }),
            ),
            (
                Some(serde_json::json!({
                    "cyber": ["standard", "daybreak_blue", "future_program", "daybreak_red"]
                })),
                Some(ModelAccessPrograms {
                    cyber: vec![
                        CyberAccessProgram::Standard,
                        CyberAccessProgram::DaybreakBlue,
                        CyberAccessProgram::DaybreakRed,
                    ],
                }),
            ),
        ] {
            let mut value = serde_json::to_value(test_model(/*spec*/ None)).unwrap();
            if let Some(metadata) = metadata {
                value["available_access_programs"] = metadata;
            }
            let model: ModelInfo = serde_json::from_value(value).unwrap();
            assert_eq!(model.available_access_programs, expected);
            assert_eq!(ModelPreset::from(model).available_access_programs, expected);
        }
    }

    #[test]
    fn model_preset_supports_fast_mode_from_service_tiers() {
        let preset = ModelPreset::from(ModelInfo {
            service_tiers: vec![ModelServiceTier {
                id: ServiceTier::Fast.request_value().to_string(),
                name: "Fast".to_string(),
                description: "Priority processing.".to_string(),
            }],
            ..test_model(/*spec*/ None)
        });

        assert!(preset.supports_fast_mode());
    }

    #[test]
    fn service_tier_for_request_omits_explicit_default_tier() {
        let model = ModelInfo {
            default_service_tier: Some(ServiceTier::Fast.request_value().to_string()),
            service_tiers: vec![ModelServiceTier {
                id: ServiceTier::Fast.request_value().to_string(),
                name: "Fast".to_string(),
                description: "Priority processing.".to_string(),
            }],
            ..test_model(/*spec*/ None)
        };

        assert_eq!(
            model.service_tier_for_request(Some(SERVICE_TIER_DEFAULT_REQUEST_VALUE.to_string())),
            None
        );
    }

    #[test]
    fn service_tier_for_request_preserves_flex_without_catalog_support() {
        let model = ModelInfo {
            service_tiers: Vec::new(),
            ..test_model(/*spec*/ None)
        };

        assert_eq!(
            model.service_tier_for_request(Some(ServiceTier::Flex.request_value().to_string())),
            Some(ServiceTier::Flex.request_value().to_string())
        );
    }

    #[test]
    fn service_tier_for_request_filters_unsupported_tiers() {
        let model = ModelInfo {
            default_service_tier: Some(ServiceTier::Fast.request_value().to_string()),
            service_tiers: vec![ModelServiceTier {
                id: ServiceTier::Fast.request_value().to_string(),
                name: "Fast".to_string(),
                description: "Priority processing.".to_string(),
            }],
            ..test_model(/*spec*/ None)
        };

        assert_eq!(
            model.service_tier_for_request(Some(ServiceTier::Fast.request_value().to_string())),
            Some(ServiceTier::Fast.request_value().to_string())
        );
        assert_eq!(
            model.service_tier_for_request(Some("unsupported".to_string())),
            None
        );
        assert_eq!(model.service_tier_for_request(/*service_tier*/ None), None);
    }
}
