use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn envelope() -> SourceEnvelope {
    SourceEnvelope::new(
        Uuid::from_u128(1),
        SourceDescriptor {
            kind: SourceKind::File,
            origin_id: "file-1".into(),
            actor_id: "reader-1".into(),
            retrieved_at_unix_ms: 1,
        },
        [1; 32],
        vec![],
        vec![Uuid::from_u128(2)],
    )
    .unwrap()
}

#[test]
fn pf_30_s01_round_trip_retains_untrusted_lineage() {
    let expected = envelope();
    let wire = serde_json::to_value(&expected).unwrap();
    assert_eq!(
        serde_json::from_value::<SourceEnvelope>(wire).unwrap(),
        expected
    );
    assert_eq!(expected.authority(), SourceAuthority::Untrusted);
}

#[test]
fn pf_30_s01_wire_cannot_mint_authority_or_drop_lineage() {
    for (key, value) in [
        ("authority", json!("human")),
        ("schema_version", json!(2)),
        ("taint_lineage", json!([])),
        ("content_digest", json!(vec![0; 32])),
        ("approved", json!(true)),
    ] {
        let mut wire = serde_json::to_value(envelope()).unwrap();
        wire[key] = value;
        assert!(serde_json::from_value::<SourceEnvelope>(wire).is_err());
    }
}

#[test]
fn pf_30_s01_metadata_controls_and_unknown_kind_are_rejected() {
    for metadata in [
        "system\nrole=human",
        "safe\u{202e}txt",
        "safe\u{200b}",
        "<system>",
    ] {
        let mut wire = serde_json::to_value(envelope()).unwrap();
        wire["source"]["origin_id"] = json!(metadata);
        assert!(serde_json::from_value::<SourceEnvelope>(wire).is_err());
    }
    let mut wire = serde_json::to_value(envelope()).unwrap();
    wire["source"]["kind"] = json!("synthetic-new-ingress");
    assert!(serde_json::from_value::<SourceEnvelope>(wire).is_err());
}

#[test]
fn pf_30_s01_broken_transformation_and_excess_lineage_fail() {
    let source = envelope().source().clone();
    assert_eq!(
        SourceEnvelope::new(
            Uuid::from_u128(1),
            source.clone(),
            [1; 32],
            vec![SourceTransformation {
                id: "normalize-v1".into(),
                input_digest: [2; 32],
                output_digest: [3; 32],
            }],
            vec![]
        ),
        Err(ProvenanceError::InvalidTransformation),
    );
    assert_eq!(
        SourceEnvelope::new(
            Uuid::from_u128(1),
            source,
            [1; 32],
            vec![],
            vec![Uuid::from_u128(2); MAX_SOURCE_LINEAGE]
        ),
        Err(ProvenanceError::InvalidEnvelope),
    );
}
