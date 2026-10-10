//! #391: under security level Aggressive, `corbanu exec` holds the provider
//! key in the credential broker by default; an explicit setting still wins.
//! The provider points at a closed local HTTPS port, so no model answers.

use core_test_support::test_codex_exec::test_codex_exec;

const KEY_ENV: &str = "SEC391_PROVIDER_KEY";
const HANDED_OVER: &str = "provider keys handed to the credential broker and removed from the environment: SEC391_PROVIDER_KEY";
const NOT_STARTED: &str =
    "model requests are refused: the isolated credential broker did not start";

/// One turn under Aggressive; returns stdout and stderr together.
fn run(extra: &[&str]) -> String {
    let test = test_codex_exec();
    std::fs::write(
        test.home_path().join("config.toml"),
        "[security]\nversion = 1\nlevel = \"aggressive\"\n\n[features]\nsource_envelopes = true\n",
    )
    .expect("config");
    let mut cmd = test.cmd();
    cmd.env(KEY_ENV, "sec391-synthetic-key")
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .env("RUST_LOG", "codex_core::model_broker_auth=info")
        .env_remove("CORBANU_HOME")
        .env_remove("PFTERMINAL_HOME")
        .arg("--skip-git-repo-check")
        .arg("-c")
        .arg(format!(
            "model_providers.sec391={{name=\"sec391\",base_url=\"https://127.0.0.1:9/v1\",wire_api=\"responses\",env_key=\"{KEY_ENV}\",supports_websockets=false,request_max_retries=0,stream_max_retries=0}}"
        ))
        .arg("-c")
        .arg("model_provider=\"sec391\"");
    for config in extra {
        cmd.arg("-c").arg(config);
    }
    let output = cmd
        .arg("hi")
        .timeout(std::time::Duration::from_secs(120))
        .output()
        .expect("run codex-exec");
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn sec_391_aggressive_brokers_the_provider_key_by_default() {
    // Either the broker took the key, or it could not start here and every
    // request was refused; both mean the level turned the broker on.
    let output = run(&[]);
    assert!(
        output.contains(HANDED_OVER) || output.contains(NOT_STARTED),
        "{output}"
    );

    let output = run(&["features.broker_model_auth=false"]);
    assert!(
        !output.contains(HANDED_OVER) && !output.contains(NOT_STARTED),
        "{output}"
    );
}
