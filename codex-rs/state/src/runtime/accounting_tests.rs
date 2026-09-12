use super::types::*;
use super::*;
use pretty_assertions::assert_eq;
use std::path::PathBuf;
use std::sync::Arc;

struct Fixture {
    home: PathBuf,
    runtime: Arc<StateRuntime>,
}

impl Fixture {
    async fn new() -> anyhow::Result<Self> {
        let home = super::super::test_support::unique_temp_dir();
        let runtime = StateRuntime::init_for_testing(home.clone(), "synthetic".into()).await?;
        Ok(Self { home, runtime })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

fn attempt() -> Attempt {
    Attempt {
        attempt_id: Uuid::from_u128(1),
        request_id: Uuid::from_u128(2),
        thread_id: codex_protocol::ThreadId::from_string("00000000-0000-0000-0000-000000000003")
            .unwrap(),
        turn: "turn-1".into(),
        retry_of: None,
        provider: "synthetic".into(),
        model: "fixture-model".into(),
        scope: Uuid::from_u128(4),
        dialect: Dialect::Inclusive,
        dispatched_at_ms: Count::try_from(1_700_000_000_000).unwrap(),
    }
}

fn observation(revision: i64, patch: &str) -> Observation {
    Observation {
        revision: Count::try_from(revision).unwrap(),
        source: Uuid::from_u128(5),
        sequence: Count::try_from(revision * 10).unwrap(),
        patch: serde_json::from_str(patch).unwrap(),
    }
}

#[tokio::test]
async fn normal_init_does_not_create_draft_tables() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE name LIKE 'draft_accounting_%'")
            .fetch_all(f.runtime.pool.as_ref())
            .await?;
    assert_eq!(tables, Vec::<String>::new());
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn identity_is_immutable_and_request_owner_cannot_change() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let j = Journal::create_for_tests(&f.runtime).await?;
    let a = attempt();
    j.begin_attempt(&a).await?;
    j.begin_attempt(&a).await?;
    let expected = Some((a.clone(), vec![]));
    for (key, value) in [
        ("request_id", serde_json::json!(Uuid::from_u128(20))),
        ("thread_id", serde_json::json!(Uuid::from_u128(21))),
        ("turn", serde_json::json!("other-turn")),
        ("provider", serde_json::json!("other-provider")),
        ("model", serde_json::json!("other-model")),
        ("scope", serde_json::json!(Uuid::from_u128(22))),
        ("dialect", serde_json::json!("NativeAnthropic")),
        ("dispatched_at_ms", serde_json::json!(1)),
        ("retry_of", serde_json::json!(Uuid::from_u128(23))),
    ] {
        let mut changed = serde_json::to_value(&a)?;
        changed[key] = value;
        assert!(
            j.begin_attempt(&serde_json::from_value(changed)?)
                .await
                .is_err(),
            "{key}"
        );
        assert_eq!(j.read_observations(a.attempt_id).await?, expected);
    }
    let mut other = a.clone();
    other.attempt_id = Uuid::from_u128(30);
    other.turn = "stolen-owner".into();
    assert!(j.begin_attempt(&other).await.is_err());
    assert_eq!(j.read_observations(other.attempt_id).await?, None);
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn interrupted_intent_and_retry_survive_real_close_reopen() -> anyhow::Result<()> {
    let mut f = Fixture::new().await?;
    let a = attempt();
    let mut retry = a.clone();
    retry.attempt_id = Uuid::from_u128(10);
    retry.retry_of = Some(a.attempt_id);
    {
        let j = Journal::create_for_tests(&f.runtime).await?;
        assert!(j.begin_attempt(&retry).await.is_err());
        j.begin_attempt(&a).await?;
        j.begin_attempt(&retry).await?;
    }
    f.runtime.close().await;
    assert!(f.runtime.pool.is_closed());
    f.runtime = StateRuntime::init_for_testing(f.home.clone(), "synthetic".into()).await?;
    let j = Journal {
        runtime: &f.runtime,
    };
    for identity in [a, retry] {
        assert_eq!(
            j.read_observations(identity.attempt_id).await?,
            Some((identity.clone(), vec![]))
        );
        assert_eq!(replay(identity.dialect, &[])?, Usage::default());
    }
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn presence_gaps_reordering_and_two_replays_after_reopen() -> anyhow::Result<()> {
    let mut f = Fixture::new().await?;
    let a = attempt();
    let rows = vec![
        observation(1, r#"{"input":100,"read":10}"#),
        observation(3, r#"{"input":null,"read":0,"output":20}"#),
        observation(7, r#"{"input":120,"output":30}"#),
    ];
    {
        let j = Journal::create_for_tests(&f.runtime).await?;
        j.append_observation(&a, &[rows[2].clone(), rows[0].clone(), rows[1].clone()])
            .await?;
        let canonical = observation(1, r#"{"read":10,"input":100}"#);
        j.append_observation(&a, &[canonical]).await?;
    }
    f.runtime.close().await;
    assert!(f.runtime.pool.is_closed());
    f.runtime = StateRuntime::init_for_testing(f.home.clone(), "synthetic".into()).await?;
    let j = Journal {
        runtime: &f.runtime,
    };
    for _ in 0..2 {
        assert_eq!(
            j.read_observations(a.attempt_id).await?,
            Some((a.clone(), rows.clone()))
        );
        j.append_observation(&a, &rows).await?;
        let (_, persisted) = j.read_observations(a.attempt_id).await?.unwrap();
        assert_eq!(
            replay(a.dialect, &persisted)?,
            Usage {
                input: Some(120),
                read: Some(0),
                output: Some(30),
                ..Usage::default()
            }
        );
    }
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn conflicting_batch_rolls_back_identity_rows_and_positions() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let j = Journal::create_for_tests(&f.runtime).await?;
    let a = attempt();
    let first = observation(1, r#"{"input":10}"#);
    let conflict = observation(1, r#"{"input":11}"#);
    assert!(
        j.append_observation(&a, &[first.clone(), conflict.clone()])
            .await
            .is_err()
    );
    assert_eq!(j.read_observations(a.attempt_id).await?, None);
    j.append_observation(&a, &[first.clone()]).await?;
    let second = observation(2, r#"{"output":2}"#);
    assert!(
        j.append_observation(&a, &[second.clone(), conflict])
            .await
            .is_err()
    );
    assert_eq!(
        j.read_observations(a.attempt_id).await?,
        Some((a.clone(), vec![first.clone()]))
    );
    let mut other = a.clone();
    other.attempt_id = Uuid::from_u128(9);
    assert!(
        j.append_observation(&other, &[first.clone()])
            .await
            .is_err()
    );
    assert_eq!(j.read_observations(other.attempt_id).await?, None);
    j.append_observation(&other, &[second.clone()]).await?;
    assert_eq!(
        j.read_observations(other.attempt_id).await?,
        Some((other, vec![second]))
    );
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn duplicate_revision_cannot_change_source_position_or_presence() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let j = Journal::create_for_tests(&f.runtime).await?;
    let a = attempt();
    let row = observation(1, "{}");
    j.append_observation(&a, &[row.clone()]).await?;
    let mut moved = row.clone();
    moved.sequence = Count::try_from(99)?;
    let mut source = row.clone();
    source.source = Uuid::from_u128(99);
    for changed in [
        moved,
        source,
        observation(1, r#"{"input":null}"#),
        observation(1, r#"{"input":0}"#),
    ] {
        assert!(j.append_observation(&a, &[changed]).await.is_err());
        assert_eq!(
            j.read_observations(a.attempt_id).await?,
            Some((a.clone(), vec![row.clone()]))
        );
    }
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn concurrent_same_key_writers_deduplicate_and_conflict() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let j = Journal::create_for_tests(&f.runtime).await?;
    let a = attempt();
    let batch = [observation(1, r#"{"output":2}"#)];
    let (left, right) = tokio::join!(
        j.append_observation(&a, &batch),
        j.append_observation(&a, &batch)
    );
    left?;
    right?;
    let left_batch = [observation(2, r#"{"output":3}"#)];
    let right_batch = [observation(2, r#"{"output":4}"#)];
    let (left, right) = tokio::join!(
        j.append_observation(&a, &left_batch),
        j.append_observation(&a, &right_batch)
    );
    assert_ne!(left.is_ok(), right.is_ok());
    let winner = if left.is_ok() {
        left_batch[0].clone()
    } else {
        right_batch[0].clone()
    };
    assert_eq!(
        j.read_observations(a.attempt_id).await?,
        Some((a, vec![batch[0].clone(), winner]))
    );
    f.runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn reordered_prefix_validation_rolls_back_without_erasing_knowns() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let j = Journal::create_for_tests(&f.runtime).await?;
    let a = attempt();
    let final_row = observation(3, r#"{"input":30,"read":20}"#);
    j.append_observation(&a, &[final_row.clone()]).await?;
    let early = observation(1, r#"{"input":10,"read":20}"#);
    assert!(j.append_observation(&a, &[early]).await.is_err());
    let null_input = observation(4, r#"{"input":null,"read":31}"#);
    assert!(j.append_observation(&a, &[null_input]).await.is_err());
    assert_eq!(
        j.read_observations(a.attempt_id).await?,
        Some((a, vec![final_row]))
    );
    f.runtime.close().await;
    Ok(())
}

#[test]
fn numeric_presence_rejects_nonintegers_overflow_and_arbitrary_fields() {
    for raw in [
        "-1",
        "1.5",
        "1.0",
        "true",
        "\"1\"",
        "9223372036854775808",
        "18446744073709551616",
    ] {
        assert!(
            serde_json::from_str::<Patch>(&format!(r#"{{"input":{raw}}}"#)).is_err(),
            "{raw}"
        );
    }
    assert!(serde_json::from_str::<Patch>(r#"{"prompt":"forbidden"}"#).is_err());
    for raw in [
        "{}",
        r#"{"input":null}"#,
        r#"{"input":0}"#,
        r#"{"input":9223372036854775807}"#,
    ] {
        let patch: Patch = serde_json::from_str(raw).unwrap();
        assert_eq!(serde_json::to_string(&patch).unwrap(), raw);
    }
}

#[test]
fn native_anthropic_partial_input_and_unknown_dialect_remain_distinct() -> anyhow::Result<()> {
    let rows = [observation(1, r#"{"input":50,"read":10}"#)];
    assert_eq!(
        replay(Dialect::NativeAnthropic, &rows)?,
        Usage {
            noncached: Some(50),
            read: Some(10),
            ..Usage::default()
        }
    );
    assert_eq!(
        replay(Dialect::UnknownCompatible, &rows)?,
        Usage {
            read: Some(10),
            ..Usage::default()
        }
    );
    let mut complete = rows.to_vec();
    complete.push(observation(2, r#"{"write":0,"output":5}"#));
    assert_eq!(
        replay(Dialect::NativeAnthropic, &complete)?,
        Usage {
            input: Some(60),
            noncached: Some(50),
            read: Some(10),
            write: Some(0),
            output: Some(5),
            total: Some(65),
            reasoning: None
        }
    );
    Ok(())
}

#[test]
fn inclusive_subsets_totals_and_arithmetic_overflow_are_validated() {
    for patch in [
        r#"{"input":10,"read":20}"#,
        r#"{"input":10,"write":20}"#,
        r#"{"input":10,"read":6,"write":5}"#,
        r#"{"output":10,"reasoning":11}"#,
        r#"{"input":1,"output":1,"total":3}"#,
        r#"{"read":9223372036854775807,"write":1}"#,
        r#"{"input":9223372036854775807,"output":1}"#,
    ] {
        assert!(
            replay(Dialect::Inclusive, &[observation(1, patch)]).is_err(),
            "{patch}"
        );
    }
    assert!(
        replay(
            Dialect::NativeAnthropic,
            &[observation(1, r#"{"input":9223372036854775807,"read":1}"#)]
        )
        .is_err()
    );
    let valid = observation(
        1,
        r#"{"input":10,"read":6,"write":4,"output":2,"reasoning":2,"total":12}"#,
    );
    assert_eq!(
        replay(Dialect::Inclusive, &[valid]).unwrap(),
        Usage {
            input: Some(10),
            noncached: Some(0),
            read: Some(6),
            write: Some(4),
            output: Some(2),
            reasoning: Some(2),
            total: Some(12)
        }
    );
}

#[tokio::test]
async fn invalid_metadata_revision_and_batch_leave_no_intent() -> anyhow::Result<()> {
    let f = Fixture::new().await?;
    let j = Journal::create_for_tests(&f.runtime).await?;
    for text in [
        "".into(),
        " ".into(),
        "x".repeat(129),
        "bad\nmetadata".into(),
    ] {
        let mut a = attempt();
        a.provider = text;
        assert!(j.begin_attempt(&a).await.is_err());
    }
    let a = attempt();
    let mut self_retry = a.clone();
    self_retry.retry_of = Some(a.attempt_id);
    assert!(j.begin_attempt(&self_retry).await.is_err());
    assert!(
        j.append_observation(&a, &[observation(0, "{}")])
            .await
            .is_err()
    );
    assert!(
        j.append_observation(&a, &vec![observation(1, "{}"); 257])
            .await
            .is_err()
    );
    assert_eq!(j.read_observations(a.attempt_id).await?, None);
    f.runtime.close().await;
    Ok(())
}
