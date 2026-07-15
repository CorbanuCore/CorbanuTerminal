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

#[test]
fn gpu_provider_names_are_presented_as_user_facing_marketplaces() {
    assert_eq!(gpu_provider_display_name("runpod"), "RunPod");
    assert_eq!(gpu_provider_display_name("vast"), "Vast.ai");
    assert_eq!(gpu_provider_display_name("unexpected"), "GPU marketplace");
}
