use super::*;

#[test]
fn remote_gateway_requires_https() {
    let result = gateway_client_for_origin("http://gateway.example.test".to_string());
    assert!(result.is_err());
}

#[test]
fn loopback_gateway_allows_http() {
    let result = gateway_client_for_origin("http://127.0.0.1:4021".to_string());
    assert!(result.is_ok());
}
