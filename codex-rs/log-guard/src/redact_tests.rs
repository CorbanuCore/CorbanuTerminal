use std::borrow::Cow;
use std::io::Write;

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn redacts_credential_headers_in_every_log_shape() {
    let cases = [
        // tungstenite's `Debug` of the upgrade request.
        (
            r#"Request: "GET /v1 HTTP/1.1\r\nhost: api\r\nauthorization: Bearer fake-ws-key-0001\r\nupgrade: websocket\r\n\r\n""#,
            r#"Request: "GET /v1 HTTP/1.1\r\nhost: api\r\nauthorization: REDACTED\r\nupgrade: websocket\r\n\r\n""#,
        ),
        // Raw header lines.
        (
            "Authorization: Basic ZmFrZTpmYWtl\r\nCookie: a=fake-cookie; b=c\r\nX-Api-Key: fake-key\r\nAccept: */*\r\n",
            "Authorization: REDACTED\r\nCookie: REDACTED\r\nX-Api-Key: REDACTED\r\nAccept: */*\r\n",
        ),
        // `Debug` and JSON maps, including escaped quotes.
        (
            r#"headers: {"api-key": "fake-azure-key", "content-type": "application/json"}"#,
            r#"headers: {"api-key": "REDACTED", "content-type": "application/json"}"#,
        ),
        (
            r#"{\"proxy-authorization\": \"Basic ZmFrZQ==\"}"#,
            r#"{\"proxy-authorization\": \"REDACTED\"}"#,
        ),
        // Values inside wrapper types and tuples (aws-smithy, Option, Vec).
        (
            r#"{"authorization": HeaderValue { _private: H0("AWS4-HMAC-SHA256 Credential=AKID/x, Signature=fake") }}"#,
            r#"{"authorization": HeaderValue { _private: H0("REDACTED") }}"#,
        ),
        (
            r#"x-api-key: Some("fake-key-0004")"#,
            r#"x-api-key: Some("REDACTED")"#,
        ),
        (
            "authorization: AWS4-HMAC-SHA256 Credential=AKID/x, SignedHeaders=host, Signature=fake\n",
            "authorization: REDACTED\n",
        ),
        (
            r#"[("api-key", "fake-azure-0005"), ("accept", "*/*")]"#,
            r#"[("api-key", "REDACTED"), ("accept", "*/*")]"#,
        ),
        // A SigV4 canonical request.
        (
            "host:bedrock\nx-amz-security-token:fake-session-0006\n",
            "host:bedrock\nx-amz-security-token:REDACTED\n",
        ),
        // A marker hides nothing after it; unquoted values end at one token.
        (
            "authorization: <redacted> x-api-key: fake-real-0007",
            "authorization: <redacted> x-api-key: REDACTED",
        ),
        (
            "authorization=required model=gpt-5",
            "authorization=REDACTED model=gpt-5",
        ),
        // Round-2 review: values after a type-like word, quoted values with
        // spaces, quoted token parameters, markers after a value.
        (
            "x-api-key=abcd1234secret (from env)",
            "x-api-key=REDACTED (from env)",
        ),
        (
            "authorization: Bearer abc123secretxyz {retry}",
            "authorization: REDACTED {retry}",
        ),
        (
            r#"x-api-key=KEY123 (source "env")"#,
            r#"x-api-key=REDACTED (source "REDACTED")"#,
        ),
        (r#""x-api-key": "abc def""#, r#""x-api-key": "REDACTED""#),
        (
            r#"Authorization: Token token="abc123""#,
            "Authorization: REDACTED",
        ),
        ("cookie: sid=abc123 REDACTED", "cookie: REDACTED"),
        (
            r#"("authorization", "secret REDACTED")"#,
            r#"("authorization", "REDACTED")"#,
        ),
        (
            r#"{"access_token":"fake-at-0009","token_type":"bearer"}"#,
            r#"{"access_token":"REDACTED","token_type":"bearer"}"#,
        ),
        (
            "grant_type=refresh_token&refresh_token=fake-rt-0010&client_assertion=fake-ca",
            "grant_type=refresh_token&refresh_token=REDACTED&client_assertion=REDACTED",
        ),
        // URL query credentials.
        (
            "Trying to contact wss://host/v1/realtime?model=gpt&api_key=fake-q-0008 now",
            "Trying to contact wss://host/v1/realtime?model=gpt&api_key=REDACTED now",
        ),
        // Bearer tokens, key formats and JWTs anywhere.
        (
            "retrying with Bearer fake-bearer-token-0002 now",
            "retrying with Bearer REDACTED now",
        ),
        (
            "key sk-proj-AAAAAAAAAAAAAAAAAAAAAAAA end",
            "key REDACTED end",
        ),
        (
            "token=eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM0.c2lnbmF0dXJlMTIz",
            "token=REDACTED",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(redact_credentials(input), expected);
    }
}

#[test]
fn keeps_redacted_values_and_ordinary_text() {
    for text in [
        r#""set-cookie": "REDACTED""#,
        r#"{"authorization": Sensitive, "x-request-id": "req-1"}"#,
        r#""X-Sentinel": "<redacted>""#,
        "experimental_bearer_token: Some(\"<redacted>\")",
        "uses bearer auth for the provider",
        "authorization server metadata discovered",
        "cookies: 3, max_tokens=100",
        r#"auth.header_name="authorization" auth_mode="ApiKey""#,
        r#"api-key: Some(Sensitive), "x-request-id": "req-1""#,
        "has_access_token=true refresh_token: None",
        "url=https://api.example.com/v1/models?key=REDACTED&api-version=REDACTED",
        "task-runner-AAAAAAAAAAAAAAAAAAAAAAAAA",
        // #398: ordinary command lines and token counters.
        "START: cargo test -p codex-core --token-file token.txt --passive",
        r#"["git", "-c", "user.name=x", "push", "https://github.com/o/r.git"]"#,
        "input_tokens=10 output_tokens: 5 max_output_tokens=100 total_tokens=15",
        r#"TokenUsage { input_tokens: 1, cached_input_tokens: 0 }"#,
        "psql -U postgres -h db:5432 && ssh -p 22 host && curl http://host:8080/a@b",
        "env_names=[\"CORBANU_SENTINEL_API_KEY\"]",
        // Review of #398: diagnostics after bare credential words.
        "Failed to refresh token: 401 Unauthorized",
        "docker run -u 1000:1000 image && OLDPWD=/tmp PWD=/home/x glob_pat=*.rs",
        "next_page_tokens=3 has_secret=false",
    ] {
        assert!(
            matches!(redact_credentials(text), Cow::Borrowed(_)),
            "{text}"
        );
    }
}

#[test]
fn writer_redacts_each_write() {
    let mut out = Vec::new();
    {
        let mut writer = RedactingWriter::new(&mut out);
        writer
            .write_all(b"a authorization: Bearer fake-writer-key-0003\n")
            .unwrap();
        writer.write_all(b"plain line\n").unwrap();
    }
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "a authorization: REDACTED\nplain line\n"
    );
}

/// #398: credentials typed into a command line, as sandbox command logs
/// (`START: <argv joined>`) and `Debug` argument lists record them.
#[test]
fn redacts_credentials_in_command_lines() {
    let cases = [
        (
            r#"START: curl -H "Authorization: Bearer fake-cmd-0001" https://api"#,
            r#"START: curl -H "Authorization: REDACTED" https://api"#,
        ),
        (
            "START: curl -H Authorization: Bearer fake-cmd-0002 https://api",
            "START: curl -H Authorization: REDACTED https://api",
        ),
        (
            r#"START: curl -H "Authorization: Bearer fake-cmd-0019" -H "X-Api-Key: fake-cmd-0020" -d "{}""#,
            r#"START: curl -H "Authorization: REDACTED" -H "X-Api-Key: REDACTED" -d "{}""#,
        ),
        (
            r#"curl -H \"Authorization: Bearer fake-cmd-0021\" -H \"X-Api-Key: fake-cmd-0022\""#,
            r#"curl -H \"Authorization: REDACTED\" -H \"X-Api-Key: REDACTED\""#,
        ),
        (
            r#"START: curl -H "X-Api-Key: fake-cmd-0003" https://api"#,
            r#"START: curl -H "X-Api-Key: REDACTED" https://api"#,
        ),
        (
            "START: curl https://api/v1?model=m&api_key=fake-cmd-0004",
            "START: curl https://api/v1?model=m&api_key=REDACTED",
        ),
        (
            "START: mysql --user=root --password=fake-cmd-0005 db",
            "START: mysql --user=root --password=REDACTED db",
        ),
        (
            "START: tool --password fake-cmd-0006 --token=fake-cmd-0007 -v",
            "START: tool --password REDACTED --token=REDACTED -v",
        ),
        (
            r#"START: tool --api-key "fake cmd 0008" --secret='fake-cmd-0009'"#,
            r#"START: tool --api-key "REDACTED" --secret='REDACTED'"#,
        ),
        (
            r#"command=["tool", "--client-secret", "fake-cmd-0010", "-v"]"#,
            r#"command=["tool", "--client-secret", "REDACTED", "-v"]"#,
        ),
        (
            "pwsh -Command Connect-Thing -Token fake-cmd-0011 -Force",
            "pwsh -Command Connect-Thing -Token REDACTED -Force",
        ),
        (
            "START: curl -u admin:fake-cmd-0012 https://host",
            "START: curl -u admin:REDACTED https://host",
        ),
        (
            "START: git clone https://oauth2:fake-cmd-0013@gitlab.example/r.git",
            "START: git clone https://oauth2:REDACTED@gitlab.example/r.git",
        ),
        (
            "START: bash -c OPENAI_API_KEY=fake-cmd-0014 GH_TOKEN='fake cmd 0015' run",
            "START: bash -c OPENAI_API_KEY=REDACTED GH_TOKEN='REDACTED' run",
        ),
        (
            r#"START: pwsh -c $env:AWS_SECRET_ACCESS_KEY = "fake-cmd-0016"; set DB_PASSWORD=fake-cmd-0017"#,
            r#"START: pwsh -c $env:AWS_SECRET_ACCESS_KEY = "REDACTED"; set DB_PASSWORD=REDACTED"#,
        ),
        (
            "START: git push https://ghp_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA@github.com/o/r",
            "START: git push https://REDACTED@github.com/o/r",
        ),
        (
            "START: openssl enc -pass pass:fake-cmd-0018 -in f",
            "START: openssl enc -pass REDACTED -in f",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(redact_credentials(input), expected);
        assert!(contains_credentials(input), "{input}");
    }
    assert!(!contains_credentials("START: cargo test -p codex-core"));
}

/// Review of #398: compact JSON argv, quoted arguments with spaces, names
/// without an underscore, attached short options and more key formats.
#[test]
fn redacts_more_command_line_shapes() {
    let cases = [
        (
            r#"["tool","--password","fake-cmd-0101","-v"]"#,
            r#"["tool","--password","REDACTED","-v"]"#,
        ),
        (
            r#"cmd /c "tool "--password=fake cmd 0102" -v""#,
            r#"cmd /c "tool "--password=REDACTED" -v""#,
        ),
        (
            r#"curl -H "X-Api-Key: fake cmd 0103" -H 'Authorization: Bearer fake cmd 0104'"#,
            r#"curl -H "X-Api-Key: REDACTED" -H 'Authorization: REDACTED'"#,
        ),
        (
            r#"set "OPENAI_API_KEY=fake cmd 0105" && PGPASSWORD=fake-cmd-0106 MYSQL_PWD=fake-cmd-0107 psql"#,
            r#"set "OPENAI_API_KEY=REDACTED" && PGPASSWORD=REDACTED MYSQL_PWD=REDACTED psql"#,
        ),
        (
            "npm config set //registry.npmjs.org/:_authToken=fake-cmd-0108",
            "npm config set //registry.npmjs.org/:_authToken=REDACTED",
        ),
        (
            "mysql -u root -pfake-cmd-0109 db && sshpass -p fake-cmd-0110 ssh host",
            "mysql -u root -pREDACTED db && sshpass -p REDACTED ssh host",
        ),
        (
            "docker login -u me -p fake-cmd-0111 registry",
            "docker login -u me -p REDACTED registry",
        ),
        (
            "openssl rsa -passin pass:fake-cmd-0112 && curl --oauth2-bearer fake-cmd-0113 u",
            "openssl rsa -passin REDACTED && curl --oauth2-bearer REDACTED u",
        ),
        (
            "aws configure set aws_secret_access_key fake-cmd-0114",
            "aws configure set aws_secret_access_key REDACTED",
        ),
        ("password: fake-cmd-0115", "password: REDACTED"),
        (
            "keys npm_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA sk_live_AAAAAAAAAAAAAAAAAAAA",
            "keys REDACTED REDACTED",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(redact_credentials(input), expected);
    }
}

/// Review of #398: argv elements are redacted whole before joining.
#[test]
fn redact_command_hides_values_holding_spaces() {
    let argv = |args: &[&str]| -> Vec<String> { args.iter().map(ToString::to_string).collect() };
    assert_eq!(
        redact_command(&argv(&[
            "tool",
            "--password=fake cmd 0201",
            "--token",
            "fake cmd 0202",
            "-H",
            "Authorization: Bearer fake cmd 0203",
            "--token-file=token.txt",
            "plain words",
        ])),
        "tool --password=REDACTED --token REDACTED -H Authorization: REDACTED --token-file=token.txt plain words"
    );
    assert_eq!(
        redact_command(&argv(&[
            "bash",
            "-lc",
            "curl -H 'X-Api-Key: fake-cmd-0204' u"
        ])),
        "bash -lc curl -H 'X-Api-Key: REDACTED' u"
    );
}
