//! Pure, checked retention preparation under the private test-only accounting tree.
use super::*;

#[derive(Debug, PartialEq, Eq)]
struct CompactDay {
    values: CompactValues,
    snapshots: BTreeSet<String>,
}

/// Ephemeral caller-snapshot result, never a serialized or transferable apply capability.
#[derive(Debug, PartialEq, Eq)]
struct RetentionPlan {
    previous_as_of_ms: Option<i64>,
    computed_as_of_ms: i64,
    compact_days: BTreeMap<DayKey, CompactDay>,
    expired_days: BTreeSet<DayKey>,
    raw_removals: BTreeMap<Uuid, i64>,
    raw_days: BTreeMap<DayKey, Current<DayTotals>>,
}

/// Caller owns the transaction spanning all reads. This entry never applies its result.
async fn prepare_on_connection(
    conn: &mut SqliteConnection,
    as_of_ms: i64,
) -> anyhow::Result<RetentionPlan> {
    reduce(read_validated_input_on_connection(conn, as_of_ms).await?)
}

fn reduce(input: ValidatedRetentionInput) -> anyhow::Result<RetentionPlan> {
    let mut plan = RetentionPlan {
        previous_as_of_ms: input.previous_as_of_ms,
        computed_as_of_ms: input.as_of_ms,
        compact_days: BTreeMap::new(),
        expired_days: BTreeSet::new(),
        raw_removals: BTreeMap::new(),
        raw_days: BTreeMap::new(),
    };
    for (key, day) in input.compact_days {
        if day.expires_at_ms <= input.as_of_ms {
            plan.expired_days.insert(key);
        } else {
            plan.compact_days.insert(
                key,
                CompactDay {
                    values: day.values,
                    snapshots: day.snapshots,
                },
            );
        }
    }
    let mut stale_days = BTreeSet::new();
    for (id, raw) in input.raw_attempts {
        if raw.detail_expires_at_ms <= input.as_of_ms {
            plan.raw_removals.insert(id, raw.replay_expires_at_ms);
            // Conservative whole-day expiry must not resurrect an expired raw-only day.
            if raw.day_expires_at_ms > input.as_of_ms {
                let mut totals = DayTotals::default();
                totals.add(&raw.quote)?;
                let values = CompactValues::from_day_totals(&totals)?;
                let day = plan.compact_days.entry(raw.day).or_insert(CompactDay {
                    values: CompactValues::from_day_totals(&DayTotals::default())?,
                    snapshots: BTreeSet::new(),
                });
                day.values = day.values.checked_add(&values)?;
                if let Some(snapshot) = raw.quote.snapshot {
                    day.snapshots.insert(snapshot.id.to_string());
                }
            }
        } else {
            if raw.contribution != ContributionFreshness::Current {
                stale_days.insert(raw.day.clone());
            }
            let day = plan
                .raw_days
                .entry(raw.day)
                .or_insert(Current::Ready(DayTotals::default()));
            if let Current::Ready(totals) = day {
                totals.add(&raw.quote)?;
            }
        }
    }
    // Check every raw sum before freshness masks it; missing/stale never means zero.
    for key in stale_days {
        plan.raw_days.insert(key, Current::NeedsRefresh);
    }
    Ok(plan)
}

#[cfg(test)]
#[path = "accounting_retention_reduction_tests.rs"]
mod tests;
