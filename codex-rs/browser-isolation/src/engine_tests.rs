use super::*;
use pretty_assertions::assert_eq;

#[test]
fn accepts_local_podman_socket_and_loopback_machine_only() {
    for endpoint in [
        "unix:///run/user/1000/podman/podman.sock",
        "ssh://core@127.0.0.1:5000/run/podman.sock",
        "ssh://core@[::1]:5000/run/podman.sock",
    ] {
        assert_eq!(verify_podman_endpoint(endpoint), Ok(()));
    }
    for endpoint in [
        "tcp://127.0.0.1:2375",
        "ssh://core@example.com/run/podman.sock",
        "ssh://core:secret@localhost/run/podman.sock",
        "https://example.com/",
    ] {
        assert_eq!(
            verify_podman_endpoint(endpoint),
            Err(BrowserError::UnsupportedRuntime)
        );
    }
}

#[test]
fn docker_client_only_json_is_unavailable_not_a_qualified_server() {
    assert_eq!(
        verify_docker_info(&serde_json::json!({"OSType":"","ServerErrors":["fixture"]})),
        Err(BrowserError::RuntimeUnavailable)
    );
    assert_eq!(
        verify_docker_info(&serde_json::json!({"OSType":"windows"})),
        Err(BrowserError::UnsupportedRuntime)
    );
    assert_eq!(
        verify_docker_info(&serde_json::json!({"OSType":"linux"})),
        Ok(())
    );
}
