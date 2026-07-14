use super::*;

#[test]
fn bounded_authorization_parser_accepts_decimal_limits() {
    assert_eq!(
        parse_gpu_authorization("3.25 12 90"),
        Ok((3_250_000, 12_000_000, 90))
    );
}

#[test]
fn bounded_authorization_parser_rejects_missing_negative_and_unbounded_terms() {
    assert!(parse_gpu_authorization("3.25 12").is_err());
    assert!(parse_gpu_authorization("-1 12 90").is_err());
    assert!(parse_gpu_authorization("3.25 12 0").is_err());
    assert!(parse_gpu_authorization("3.25 12 10081").is_err());
}
