use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

fn source(index: u8, kind: SourceKind) -> SourceEnvelope {
    SourceEnvelope::host_assigned(
        SourceId::try_from([index; 16]).unwrap(),
        kind,
        b"hostile content",
    )
}

#[test]
fn source_identity_binds_content_without_exposing_it() {
    let source = source(/*index*/ 1, SourceKind::Web);
    assert!(source.matches_content(b"hostile content"));
    assert!(!source.matches_content(b"sanitized content"));
    let encoded = serde_json::to_string(&source).unwrap();
    assert!(!encoded.contains("hostile content"));
    assert!(!encoded.contains("trusted"));
    assert!(SourceId::try_from([0; 16]).is_err());
    assert!(serde_json::from_value::<SourceId>(json!(vec![0; 16])).is_err());
    assert!(serde_json::from_value::<SourceId>(json!("human:trusted")).is_err());
}

#[test]
fn every_ingress_kind_is_tainted_and_unknown_origin_is_explicit() {
    for kind in [
        SourceKind::File,
        SourceKind::Web,
        SourceKind::Document,
        SourceKind::Tool,
        SourceKind::Mcp,
        SourceKind::Connector,
        SourceKind::Email,
        SourceKind::Memory,
        SourceKind::Delegated,
        SourceKind::Unknown,
    ] {
        let source = source(/*index*/ 1, kind);
        let taint = TaintContext::from_host_source(&source);
        assert_eq!(
            taint.sources(),
            &std::collections::BTreeSet::from([source.source_id()])
        );
        assert_eq!(taint.has_unknown_origin(), kind == SourceKind::Unknown);
    }
}

#[test]
fn derivation_and_checkpoint_round_trip_cannot_clear_ancestry() {
    let web = TaintContext::from_host_source(&source(/*index*/ 1, SourceKind::Web));
    let file = TaintContext::from_host_source(&source(/*index*/ 2, SourceKind::File));
    let joined = web.derive(&file);
    assert_eq!(joined, file.derive(&web));
    assert_eq!(joined.derive(&joined), joined);
    assert_eq!(joined.derive(&TaintContext::trusted_input()), joined);
    // Same operation at each native summary/compaction/memory/child/resume join.
    let mut derived = joined.clone();
    for _ in 0..5 {
        derived = serde_json::from_value(json!(derived.derive(&web))).unwrap();
    }
    assert_eq!(derived, joined);
    let unknown = joined.derive(&TaintContext::unknown());
    assert!(unknown.has_unknown_origin());
    assert_eq!(unknown.derive(&joined), unknown);
    assert!(TaintContext::default().has_unknown_origin());
}

#[test]
fn overflow_is_bounded_and_stays_unknown_after_more_derivations() {
    let mut taint = TaintContext::trusted_input();
    for index in 1..=MAX_TAINT_SOURCES as u8 {
        taint = taint.derive(&TaintContext::from_host_source(&source(
            index,
            SourceKind::Web,
        )));
    }
    assert!(!taint.has_unknown_origin());
    taint = taint.derive(&TaintContext::from_host_source(&source(
        /*index*/ 65,
        SourceKind::File,
    )));
    assert!(taint.has_unknown_origin());
    assert_eq!(taint.sources().len(), MAX_TAINT_SOURCES);
    assert_eq!(taint.derive(&TaintContext::trusted_input()), taint);
}

#[test]
fn taint_wire_rejects_unknown_versions_missing_fields_and_unbounded_sources() {
    let valid = json!(TaintContext::from_host_source(&source(
        /*index*/ 1,
        SourceKind::Web
    )));
    for field in ["schema_version", "sources", "unknown_origin"] {
        let mut wire = valid.clone();
        wire.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<TaintContext>(wire).is_err());
    }
    for (field, value) in [
        ("schema_version", json!(2)),
        ("sources", json!(vec![[1; 16]; 65])),
        ("human", json!(true)),
    ] {
        let mut wire = valid.clone();
        wire[field] = value;
        assert!(serde_json::from_value::<TaintContext>(wire).is_err());
    }
}
