use super::super::quote_observations;
use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

/// A snapshot exactly as a build before 2026-09-21 wrote it: no basis fields.
const BEFORE_BASIS: &str = r#"{"id":"00000000-0000-0000-0000-000000000010","provider":"synthetic","model":"fixture-model","scope":"00000000-0000-0000-0000-000000000004","currency":"USD","unit":"PerMillionTokens","rates":{"noncached":"3","read":"0.3","write":null,"output":null},"source_reference":"00000000-0000-0000-0000-000000000011","source_kind":"ProviderPublished","observed_at_ms":80,"approved_at_ms":90,"effective_from_ms":100,"effective_end_ms":null}"#;

fn snapshot() -> Snapshot {
    serde_json::from_str(BEFORE_BASIS).unwrap()
}

fn attempt() -> Attempt {
    serde_json::from_value(json!({
        "attempt_id": "00000000-0000-0000-0000-000000000001",
        "request_id": "00000000-0000-0000-0000-000000000002",
        "thread_id": "00000000-0000-0000-0000-000000000003",
        "turn": "fixture", "retry_of": null, "provider": "synthetic",
        "model": "fixture-model", "scope": "00000000-0000-0000-0000-000000000004",
        "dialect": "NativeAnthropic", "dispatched_at_ms": 100
    }))
    .unwrap()
}

#[test]
fn both_snapshot_forms_read_and_no_other() {
    let snapshot = snapshot();
    let current = serde_json::to_string(&snapshot).unwrap();
    assert!(current.contains(r#""basis":"Billed""#));
    assert!(snapshot_is_canonical(&snapshot, BEFORE_BASIS).unwrap());
    assert!(snapshot_is_canonical(&snapshot, &current).unwrap());
    // The older form cannot carry a basis, so a record stating one never
    // matches it: reading it that way would hide what it states.
    for change in [
        |s: &mut Snapshot| s.basis = Basis::PlanEquivalent,
        |s: &mut Snapshot| s.basis_source = BasisSource::UserConfig,
    ] {
        let mut stated = snapshot.clone();
        change(&mut stated);
        assert!(!snapshot_is_canonical(&stated, BEFORE_BASIS).unwrap());
    }
}

#[test]
fn both_estimate_forms_read_and_no_other() {
    let rows: Vec<Observation> = vec![
        serde_json::from_value(json!({"revision": 1,
            "source": "00000000-0000-0000-0000-000000000005",
            "sequence": 10, "patch": {"input": 50, "read": 10}}))
        .unwrap(),
    ];
    let quote = quote_observations(&attempt(), &rows, &[snapshot()]).unwrap();
    let current = serde_json::to_string(&quote).unwrap();
    let older = current
        .replacen(r#","basis":"Billed","plan_burn_millis":null"#, "", 1)
        .replacen(
            r#","known_equivalent":"0","all_buckets_equivalent":null,"plan_burn_millis":null,"plan_burn_milli_tokens":null"#,
            "",
            1,
        );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&older).unwrap()["snapshot"],
        serde_json::from_str::<serde_json::Value>(BEFORE_BASIS).unwrap()
    );
    assert!(quote_is_canonical(&quote, &current).unwrap());
    assert!(quote_is_canonical(&quote, &older).unwrap());
    let mut plan = quote.clone();
    plan.plan_burn_millis = Some(1000);
    assert!(!quote_is_canonical(&plan, &older).unwrap());
}

#[test]
fn newer_format_is_named() {
    let error = anyhow::Error::from(NewerLedgerFormat {
        found: 3,
        supported: 2,
    })
    .context("open accounting store");
    assert!(is_newer_format(&error));
    assert!(!is_newer_format(&anyhow::anyhow!("busy")));
}
