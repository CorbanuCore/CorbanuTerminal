use pretty_assertions::assert_eq;

use super::RouteAwareHttpClient;
use super::armed_header_env_var_error;
use crate::protocol::HttpHeader;

fn env_header(env_var: &str) -> HttpHeader {
    HttpHeader {
        name: "authorization".to_string(),
        value: "Bearer ".to_string(),
        value_env_var: Some(env_var.to_string()),
    }
}

fn build_error(env_var: &str) -> String {
    RouteAwareHttpClient::build_headers(vec![env_header(env_var)])
        .expect_err("header should be refused")
        .message
}

/// #2316: Corbanu's provider keys and its own vault, Task Node, plan and
/// helper credentials never reach an MCP server through an executor-side
/// header, at any security level. Other names keep today's behaviour.
#[test]
fn corbanu_and_provider_credentials_are_denied_header_env_vars() {
    let mut denied: Vec<String> = codex_model_provider_info::built_in_provider_api_key_env_vars()
        .iter()
        .flat_map(|name| [name.clone(), name.to_ascii_lowercase()])
        .collect();
    assert!(denied.iter().any(|name| name == "ZAI_API_KEY"));
    assert!(denied.iter().any(|name| name == "ANTHROPIC_API_KEY"));
    assert!(denied.iter().any(|name| name == "OPENROUTER_API_KEY"));
    denied.extend(
        [
            "CLAUDE_CODE_OAUTH_TOKEN",
            "ANTHROPIC_AUTH_TOKEN",
            "AZURE_OPENAI_API_KEY",
            "AWS_BEARER_TOKEN_BEDROCK",
            "CORBANU_VAULT_PASSWORD",
            "CORBANU_VAULT_LABEL",
            "corbanu_vault_password",
            "CORBANU_PLAN_API_KEY",
            "CORBANU_TASKNODE_PROFILE",
            "TASKNODE_API_TOKEN",
            "PFT_ENDPOINT_TOKEN",
            "PFT_TASKNODE_ORIGIN",
            "PFTERMINAL_PLAN_API_KEY",
            "PFTERMINAL_TELEGRAM_TOKEN",
            "CODEX_REMOTE_AUTH_TOKEN",
            "CODEX_GITHUB_TOKEN",
            "OPENAI_API_KEY",
        ]
        .map(str::to_string),
    );
    for name in denied {
        assert_eq!(
            build_error(&name),
            format!(
                "http/request header authorization cannot use executor environment variable {name}"
            ),
        );
    }

    // Names outside those lists are still read (and here reported unset).
    for name in [
        "PF27_HEADER_UNSET_LINEAR_API_KEY",
        "CORBANU_PF27_HEADER_UNSET_THEME",
    ] {
        assert_eq!(
            build_error(name),
            format!(
                "http/request header authorization requires executor environment variable {name}"
            ),
        );
    }
}

/// PF-27-S02 name classes: armed, only ASCII literal names on the launch
/// allowlist pass; unarmed, nothing changes.
#[test]
fn armed_header_env_var_names_must_pass_the_launch_allowlist() {
    for refused in [
        "LINEAR_API_KEY",
        "linear_api_key",
        "GITHUB_TOKEN",
        "ZAI_API_KEY",
        "MCP_SESSION_COOKIE",
        "NODE_OPTIONS",
        "AWS_REGION",
        "PF27_PLAIN",
        "CORBANU_*",
        "CORBANU_HEADER_?",
        // A Kelvin sign keeps the name off the secret-fragment check.
        "CORBANU_\u{212A}EY",
    ] {
        let header = env_header(refused);
        assert_eq!(
            armed_header_env_var_error(&header, /*armed*/ true),
            Some(format!(
                "http/request header authorization cannot use executor environment variable {refused} while protected launch is armed; set the credential in the MCP server's own http_headers configuration"
            )),
        );
        assert_eq!(armed_header_env_var_error(&header, /*armed*/ false), None);
    }
    for allowed in [
        "PF27_HEADER_HOME",
        "XDG_RUNTIME_DIR",
        "CORBANU_HEADER_PREFIX",
    ] {
        assert_eq!(
            armed_header_env_var_error(&env_header(allowed), /*armed*/ true),
            None,
            "{allowed}"
        );
    }
    let literal = HttpHeader {
        name: "authorization".to_string(),
        value: "Bearer configured".to_string(),
        value_env_var: None,
    };
    assert_eq!(armed_header_env_var_error(&literal, /*armed*/ true), None);
}

/// PF-27-S02: once this process arms protected launch (a local environment,
/// or an executor that armed itself), the request runner refuses a header
/// variable outside the launch allowlist; literal headers still work.
#[test]
fn pf_27_s02_armed_runner_refuses_header_env_vars_outside_the_allowlist() {
    const CHILD: &str = "CODEX_EXEC_SERVER_HEADER_ENV_SECRETLESS_CHILD";
    const TEST: &str = "client::http_client::route_aware_http_client::tests::pf_27_s02_armed_runner_refuses_header_env_vars_outside_the_allowlist";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, "1")
            .env("PF27_HEADER_TOKEN", "host-secret")
            .env("PF27_HEADER_HOME", "host-plain")
            .output()
            .expect("run child test");
        assert!(
            output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let header_value = |env_var: &str| {
        RouteAwareHttpClient::build_headers(vec![env_header(env_var)])
            .map(|headers| headers["authorization"].to_str().unwrap().to_string())
            .map_err(|error| error.message)
    };
    assert_eq!(
        header_value("PF27_HEADER_TOKEN"),
        Ok("Bearer host-secret".to_string())
    );
    codex_protocol::secretless_launch::arm();
    assert_eq!(
        header_value("PF27_HEADER_TOKEN"),
        Err("http/request header authorization cannot use executor environment variable PF27_HEADER_TOKEN while protected launch is armed; set the credential in the MCP server's own http_headers configuration".to_string())
    );
    assert_eq!(
        header_value("PF27_HEADER_HOME"),
        Ok("Bearer host-plain".to_string())
    );
    let literal = RouteAwareHttpClient::build_headers(vec![HttpHeader {
        name: "authorization".to_string(),
        value: "Bearer configured".to_string(),
        value_env_var: None,
    }])
    .expect("literal header");
    assert_eq!(literal["authorization"], "Bearer configured");
}
