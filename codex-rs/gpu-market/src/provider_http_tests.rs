use super::*;

#[test]
fn monetary_parser_rejects_unknown_or_negative_prices() {
    assert_eq!(
        parse_usd_micros(&serde_json::json!("2.75")),
        Some(2_750_000)
    );
    assert_eq!(parse_usd_micros(&serde_json::json!(-1)), None);
    assert_eq!(parse_usd_micros(&serde_json::json!("unknown")), None);
}
