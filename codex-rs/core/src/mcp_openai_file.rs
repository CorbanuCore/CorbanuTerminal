//! Bridges Apps SDK-style `openai/fileParams` metadata into Codex's MCP flow.
//!
//! Strategy:
//! - Inspect `_meta["openai/fileParams"]` to discover which tool arguments are
//!   file inputs.
//! - At tool execution time, read those files from the primary environment,
//!   upload them to OpenAI file storage,
//!   and rewrite only the declared arguments into the provided-file payload
//!   shape expected by the downstream Apps tool.
//!
//! The model-facing local-path schema is owned by `codex-mcp` alongside MCP tool inventory, so this
//! module only handles uploading the files and rewriting the execution-time arguments.

use crate::session::session::Session;
use crate::session::step_context::StepContext;
use codex_api::OPENAI_FILE_UPLOAD_LIMIT_BYTES;
use codex_api::upload_openai_file;
use codex_file_system::FileSystemReadStream;
use codex_file_system::FileSystemSandboxContext;
use codex_http_client::RouteAwareClientPool;
use codex_login::CodexAuth;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub(crate) async fn rewrite_mcp_tool_arguments_for_openai_files(
    sess: &Session,
    step_context: &StepContext,
    arguments_value: Option<JsonValue>,
    openai_file_input_optional_fields: Option<&HashMap<String, Vec<String>>>,
) -> Result<Option<JsonValue>, String> {
    let Some(openai_file_input_optional_fields) = openai_file_input_optional_fields else {
        return Ok(arguments_value);
    };

    let Some(arguments_value) = arguments_value else {
        return Ok(None);
    };
    let Some(arguments) = arguments_value.as_object() else {
        return Ok(Some(arguments_value));
    };
    let auth = sess.services.auth_manager.auth().await;
    // PF-23-S01: after untrusted content these host-side reads honour the
    // same read denials as agent commands.
    let read_policy =
        crate::security::protected_surface::post_taint_read_policy(sess, &step_context.turn);
    // PF-23-S02: the read itself goes through the same sandbox, so a link
    // swapped in after the check above still cannot be read.
    let read_sandbox = read_policy.as_ref().and_then(|_| {
        step_context.environments.primary().map(|environment| {
            crate::security::protected_surface::protect_file_tool_context(
                sess,
                &step_context.turn,
                step_context
                    .turn
                    .file_system_sandbox_context(/*additional_permissions*/ None, environment),
            )
        })
    });
    let mut rewritten_arguments = arguments.clone();

    for (field_name, optional_fields) in openai_file_input_optional_fields {
        let Some(value) = arguments.get(field_name) else {
            continue;
        };
        let Some(uploaded_value) = rewrite_argument_value_for_openai_files(
            step_context,
            read_policy.as_ref(),
            read_sandbox.as_ref(),
            &sess.services.openai_file_upload_client_pool,
            auth.as_ref(),
            field_name,
            optional_fields,
            value,
        )
        .await?
        else {
            continue;
        };
        rewritten_arguments.insert(field_name.clone(), uploaded_value);
    }

    if rewritten_arguments == *arguments {
        return Ok(Some(arguments_value));
    }

    Ok(Some(JsonValue::Object(rewritten_arguments)))
}

async fn rewrite_argument_value_for_openai_files(
    step_context: &StepContext,
    read_policy: Option<&codex_protocol::permissions::FileSystemSandboxPolicy>,
    read_sandbox: Option<&FileSystemSandboxContext>,
    client_pool: &RouteAwareClientPool,
    auth: Option<&CodexAuth>,
    field_name: &str,
    optional_fields: &[String],
    value: &JsonValue,
) -> Result<Option<JsonValue>, String> {
    match value {
        JsonValue::String(file_path) => {
            check_upload_readable(step_context, read_policy, file_path)?;
            let rewritten = build_uploaded_argument_value(
                step_context,
                read_sandbox,
                client_pool,
                auth,
                field_name,
                /*index*/ None,
                optional_fields,
                file_path,
            )
            .await?;
            Ok(Some(rewritten))
        }
        JsonValue::Array(values) => {
            let mut rewritten_values = Vec::with_capacity(values.len());
            for (index, item) in values.iter().enumerate() {
                let Some(file_path) = item.as_str() else {
                    return Ok(None);
                };
                check_upload_readable(step_context, read_policy, file_path)?;
                let rewritten = build_uploaded_argument_value(
                    step_context,
                    read_sandbox,
                    client_pool,
                    auth,
                    field_name,
                    Some(index),
                    optional_fields,
                    file_path,
                )
                .await?;
                rewritten_values.push(rewritten);
            }
            Ok(Some(JsonValue::Array(rewritten_values)))
        }
        _ => Ok(None),
    }
}

/// PF-23-S01: after untrusted content, a file the host reads for upload
/// must be readable under the same read denials as agent commands.
fn check_upload_readable(
    step_context: &StepContext,
    read_policy: Option<&codex_protocol::permissions::FileSystemSandboxPolicy>,
    file_path: &str,
) -> Result<(), String> {
    let Some(policy) = read_policy else {
        return Ok(());
    };
    let Some(turn_environment) = step_context.environments.primary() else {
        return Err("no primary turn environment is available".to_string());
    };
    #[allow(deprecated)]
    let cwd = step_context.turn.cwd.clone();
    // A path this host cannot check (a remote environment) is refused.
    let readable = !turn_environment.environment.is_remote()
        && turn_environment
            .cwd()
            .join(file_path)
            .ok()
            .and_then(|uri| uri.to_abs_path().ok())
            .is_some_and(|path| {
                crate::security::protected_surface::readable_under(
                    policy,
                    path.as_path(),
                    cwd.as_path(),
                )
            });
    if readable {
        Ok(())
    } else {
        Err(format!(
            "failed to upload `{file_path}`: reading this file is blocked after untrusted content entered the session"
        ))
    }
}

async fn build_uploaded_argument_value(
    step_context: &StepContext,
    read_sandbox: Option<&FileSystemSandboxContext>,
    client_pool: &RouteAwareClientPool,
    auth: Option<&CodexAuth>,
    field_name: &str,
    index: Option<usize>,
    optional_fields: &[String],
    file_path: &str,
) -> Result<JsonValue, String> {
    let contextualize_error = |error: String| match index {
        Some(index) => {
            format!("failed to upload `{file_path}` for `{field_name}[{index}]`: {error}")
        }
        None => format!("failed to upload `{file_path}` for `{field_name}`: {error}"),
    };
    let Some(auth) = auth else {
        return Err("ChatGPT auth is required to upload files for Codex Apps tools".to_string());
    };
    if !auth.uses_codex_backend() {
        return Err("ChatGPT auth is required to upload files for Codex Apps tools".to_string());
    }
    let turn_context = &step_context.turn;
    let Some(turn_environment) = step_context.environments.primary() else {
        return Err(contextualize_error(
            "no primary turn environment is available".to_string(),
        ));
    };
    let path_uri = turn_environment
        .cwd()
        .join(file_path)
        .map_err(|error| contextualize_error(error.to_string()))?;
    let fs = turn_environment.environment.get_filesystem();
    let metadata = fs
        .get_metadata(&path_uri, read_sandbox)
        .await
        .map_err(|error| contextualize_error(error.to_string()))?;
    if !metadata.is_file {
        return Err(contextualize_error(format!(
            "path `{}` is not a file",
            path_uri.inferred_native_path_string()
        )));
    }
    if metadata.size > OPENAI_FILE_UPLOAD_LIMIT_BYTES {
        return Err(contextualize_error(format!(
            "file `{}` is too large: {} bytes exceeds the limit of {} bytes",
            path_uri.inferred_native_path_string(),
            metadata.size,
            OPENAI_FILE_UPLOAD_LIMIT_BYTES,
        )));
    }
    let (size, contents) = match read_sandbox {
        // Sandboxed reads do not stream: read the whole file (within the
        // upload limit) through the sandbox.
        Some(sandbox) => {
            let bytes = fs
                .read_file(&path_uri, Some(sandbox))
                .await
                .map_err(|error| contextualize_error(error.to_string()))?;
            let size = bytes.len() as u64;
            if size > OPENAI_FILE_UPLOAD_LIMIT_BYTES {
                return Err(contextualize_error(format!(
                    "file `{}` is too large: {size} bytes exceeds the limit of {} bytes",
                    path_uri.inferred_native_path_string(),
                    OPENAI_FILE_UPLOAD_LIMIT_BYTES,
                )));
            }
            let chunk = futures::stream::once(std::future::ready(Ok(bytes.into())));
            (size, FileSystemReadStream::new(chunk))
        }
        None => (
            metadata.size,
            fs.read_file_stream(&path_uri, /*sandbox*/ None)
                .await
                .map_err(|error| contextualize_error(error.to_string()))?,
        ),
    };
    let file_name = path_uri
        .basename()
        .or_else(|| {
            path_uri.infer_path_convention().and_then(|convention| {
                convention
                    .path_segments(file_path)
                    .rfind(|segment| !segment.is_empty())
                    .map(str::to_string)
            })
        })
        .unwrap_or_else(|| "file".to_string());
    let upload_auth = codex_model_provider::auth_provider_from_auth(auth);
    let uploaded = upload_openai_file(
        turn_context.config.chatgpt_base_url.trim_end_matches('/'),
        upload_auth.as_ref(),
        client_pool,
        file_name,
        size,
        contents,
    )
    .await
    .map_err(|error| contextualize_error(error.to_string()))?;
    let mut payload = serde_json::Map::new();
    payload.insert(
        "download_url".to_string(),
        JsonValue::String(uploaded.download_url),
    );
    payload.insert("file_id".to_string(), JsonValue::String(uploaded.file_id));
    if optional_fields
        .iter()
        .any(|optional_field| optional_field == "mime_type")
        && let Some(mime_type) = uploaded.mime_type
    {
        payload.insert("mime_type".to_string(), JsonValue::String(mime_type));
    }
    if optional_fields
        .iter()
        .any(|optional_field| optional_field == "file_name")
    {
        payload.insert(
            "file_name".to_string(),
            JsonValue::String(uploaded.file_name),
        );
    }
    Ok(JsonValue::Object(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment_selection::TurnEnvironmentState;
    use crate::session::tests::make_session_and_context;
    use crate::session::turn_context::TurnContext;
    use crate::session::turn_context::TurnEnvironment;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use codex_utils_path_uri::PathUri;
    use pretty_assertions::assert_eq;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn set_primary_environment_cwd(turn_context: &mut TurnContext, cwd: &Path) {
        let cwd = AbsolutePathBuf::try_from(cwd).expect("absolute path");
        turn_context.permission_profile = codex_protocol::models::PermissionProfile::Disabled;
        let TurnEnvironmentState::Ready(primary) = &mut turn_context.environments.environments[0]
        else {
            panic!("expected ready primary environment");
        };
        *primary = TurnEnvironment::new(
            primary.environment_id.clone(),
            Arc::clone(&primary.environment),
            PathUri::from_abs_path(&cwd),
            Vec::new(),
            primary.shell.clone(),
        );
    }

    #[tokio::test]
    async fn openai_file_argument_rewrite_requires_declared_file_params() {
        let (session, turn_context) = make_session_and_context().await;
        let step_context = StepContext::for_test(Arc::new(turn_context));
        let arguments = Some(serde_json::json!({
            "file": "/tmp/codex-smoke-file.txt"
        }));

        let rewritten = rewrite_mcp_tool_arguments_for_openai_files(
            &session,
            &step_context,
            arguments.clone(),
            /*openai_file_input_optional_fields*/ None,
        )
        .await
        .expect("rewrite should succeed");

        assert_eq!(rewritten, arguments);
    }

    #[tokio::test]
    async fn build_uploaded_argument_value_uses_environment_that_becomes_ready_during_turn() {
        use wiremock::Mock;
        use wiremock::MockServer;
        use wiremock::ResponseTemplate;
        use wiremock::matchers::body_json;
        use wiremock::matchers::header;
        use wiremock::matchers::method;
        use wiremock::matchers::path;

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files"))
            .and(header("chatgpt-account-id", "account_id"))
            .and(body_json(serde_json::json!({
                "file_name": "file_report.csv",
                "file_size": 5,
                "use_case": "codex",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "file_id": "file_123",
                "upload_url": format!("{}/upload/file_123", server.uri()),
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/upload/file_123"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files/file_123/uploaded"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "download_url": format!("{}/download/file_123", server.uri()),
                "file_name": "file_report.csv",
                "mime_type": "text/csv",
                "file_size_bytes": 5,
            })))
            .expect(1)
            .mount(&server)
            .await;

        let (session, mut turn_context) = make_session_and_context().await;
        let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
        let dir = tempdir().expect("temp dir");
        let local_path = dir.path().join("file_report.csv");
        tokio::fs::write(&local_path, b"hello")
            .await
            .expect("write local file");
        set_primary_environment_cwd(&mut turn_context, dir.path());
        let selection = turn_context
            .environments
            .primary()
            .expect("ready primary environment")
            .selection();
        let environments = crate::environment_selection::ThreadEnvironments::new(
            session.services.turn_environments.environment_manager(),
            crate::shell::default_user_shell(),
            crate::shell_snapshot::ShellSnapshot::disabled(),
            Default::default(),
            /*non_blocking_snapshots*/ true,
        );
        environments.update_selections(std::slice::from_ref(&selection));
        turn_context.environments = environments.snapshot().await;
        turn_context
            .environments
            .starting()
            .next()
            .expect("environment should initially be starting")
            .wait_until_ready()
            .await
            .expect("environment should become ready");
        let step_environments = turn_context.environments.refresh_readiness();

        let mut config = (*turn_context.config).clone();
        config.chatgpt_base_url = format!("{}/backend-api", server.uri());
        turn_context.config = Arc::new(config);
        let mut step_context = StepContext::for_test(Arc::new(turn_context));
        Arc::get_mut(&mut step_context)
            .expect("step context should be uniquely owned")
            .environments = step_environments;

        let rewritten = build_uploaded_argument_value(
            &step_context,
            /*read_sandbox*/ None,
            &session.services.openai_file_upload_client_pool,
            Some(&auth),
            "file",
            /*index*/ None,
            &["mime_type".to_string(), "file_name".to_string()],
            "file_report.csv",
        )
        .await
        .expect("rewrite should upload the local file");

        assert_eq!(
            rewritten,
            serde_json::json!({
                "download_url": format!("{}/download/file_123", server.uri()),
                "file_id": "file_123",
                "mime_type": "text/csv",
                "file_name": "file_report.csv",
            })
        );
    }

    /// PF-23-S02: once the protected-path rules apply, the upload read goes
    /// through the sandbox itself, so a link to a denied file swapped in
    /// after the readability check still reads nothing.
    #[cfg(unix)]
    #[tokio::test]
    async fn pf_23_s02_upload_reads_go_through_the_sandbox() {
        use wiremock::Mock;
        use wiremock::MockServer;
        use wiremock::ResponseTemplate;
        use wiremock::matchers::method;
        use wiremock::matchers::path;

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "file_id": "file_123",
                "upload_url": format!("{}/upload/file_123", server.uri()),
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/upload/file_123"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files/file_123/uploaded"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "download_url": format!("{}/download/file_123", server.uri()),
                "file_name": "plain.txt",
                "file_size_bytes": 5,
            })))
            .mount(&server)
            .await;

        let (session, mut turn_context) = make_session_and_context().await;
        let client = (*session.services.model_client())
            .clone()
            .with_ingress_level(codex_security_policy::SecurityLevel::Moderate)
            .with_source_envelopes(true);
        session.services.replace_model_client(client);
        session
            .services
            .model_client()
            .note_unrecorded_input_for_taint();
        let home = turn_context.config.codex_home.to_path_buf();
        std::fs::create_dir_all(&home).expect("home");
        std::fs::write(home.join("auth.json"), "SECRET").expect("auth");
        let dir = tempdir().expect("temp dir");
        let cwd = dir.path().canonicalize().expect("canonical");
        std::os::unix::fs::symlink(home.join("auth.json"), cwd.join("swapped.txt")).expect("link");
        std::fs::write(cwd.join("plain.txt"), "hello").expect("plain");
        set_primary_environment_cwd(&mut turn_context, &cwd);
        let mut config = (*turn_context.config).clone();
        config.chatgpt_base_url = format!("{}/backend-api", server.uri());
        turn_context.config = Arc::new(config);
        let step_context = StepContext::for_test(Arc::new(turn_context));
        let environment = step_context.environments.primary().expect("environment");
        let sandbox = crate::security::protected_surface::protect_file_tool_context(
            &session,
            &step_context.turn,
            step_context
                .turn
                .file_system_sandbox_context(/*additional_permissions*/ None, environment),
        );
        assert!(sandbox.should_run_in_sandbox());
        let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
        let upload = |sandbox, file: &'static str| {
            build_uploaded_argument_value(
                &step_context,
                sandbox,
                &session.services.openai_file_upload_client_pool,
                Some(&auth),
                "file",
                /*index*/ None,
                &[],
                file,
            )
        };
        // Unit tests have no sandbox helper, so a sandboxed read fails here
        // with that reason: proof that every read took the sandboxed path
        // (the helper is the one in-process file tools use). Nothing is sent.
        for file in ["swapped.txt", "plain.txt"] {
            let error = upload(Some(&sandbox), file)
                .await
                .expect_err("read through the sandbox");
            assert!(error.contains("sandboxed filesystem"), "{error}");
            assert!(!error.contains("SECRET"), "{error}");
        }
        // The same upload without the rules reads directly and is sent once.
        upload(None, "plain.txt")
            .await
            .expect("a plain file uploads");
    }

    #[tokio::test]
    async fn build_uploaded_argument_value_rejects_oversized_file_before_reading() {
        let (session, mut turn_context) = make_session_and_context().await;
        let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("oversized.bin");
        let file = std::fs::File::create(&file_path).expect("create sparse file");
        file.set_len(OPENAI_FILE_UPLOAD_LIMIT_BYTES + 1)
            .expect("size sparse file");
        set_primary_environment_cwd(&mut turn_context, dir.path());
        let step_context = StepContext::for_test(Arc::new(turn_context));

        let error = build_uploaded_argument_value(
            &step_context,
            /*read_sandbox*/ None,
            &session.services.openai_file_upload_client_pool,
            Some(&auth),
            "file",
            /*index*/ None,
            &[],
            "oversized.bin",
        )
        .await
        .expect_err("oversized file should be rejected");

        assert!(error.contains("is too large"));
        assert!(error.contains(&(OPENAI_FILE_UPLOAD_LIMIT_BYTES + 1).to_string()));
    }

    #[tokio::test]
    async fn rewrite_argument_value_for_openai_files_omits_undeclared_optional_fields() {
        use wiremock::Mock;
        use wiremock::MockServer;
        use wiremock::ResponseTemplate;
        use wiremock::matchers::body_json;
        use wiremock::matchers::header;
        use wiremock::matchers::method;
        use wiremock::matchers::path;

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files"))
            .and(header("chatgpt-account-id", "account_id"))
            .and(body_json(serde_json::json!({
                "file_name": "file_report.csv",
                "file_size": 5,
                "use_case": "codex",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "file_id": "file_123",
                "upload_url": format!("{}/upload/file_123", server.uri()),
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/upload/file_123"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files/file_123/uploaded"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "download_url": format!("{}/download/file_123", server.uri()),
                "file_name": "file_report.csv",
                "mime_type": "text/csv",
                "file_size_bytes": 5,
            })))
            .expect(1)
            .mount(&server)
            .await;

        let (session, mut turn_context) = make_session_and_context().await;
        let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
        let dir = tempdir().expect("temp dir");
        let local_path = dir.path().join("file_report.csv");
        tokio::fs::write(&local_path, b"hello")
            .await
            .expect("write local file");
        set_primary_environment_cwd(&mut turn_context, dir.path());

        let mut config = (*turn_context.config).clone();
        config.chatgpt_base_url = format!("{}/backend-api", server.uri());
        turn_context.config = Arc::new(config);
        let step_context = StepContext::for_test(Arc::new(turn_context));
        let rewritten = rewrite_argument_value_for_openai_files(
            &step_context,
            /*read_policy*/ None,
            /*read_sandbox*/ None,
            &session.services.openai_file_upload_client_pool,
            Some(&auth),
            "file",
            &[],
            &serde_json::json!("file_report.csv"),
        )
        .await
        .expect("rewrite should succeed");

        assert_eq!(
            rewritten,
            Some(serde_json::json!({
                "download_url": format!("{}/download/file_123", server.uri()),
                "file_id": "file_123",
            }))
        );
    }

    #[tokio::test]
    async fn rewrite_argument_value_for_openai_files_rewrites_array_paths() {
        use wiremock::Mock;
        use wiremock::MockServer;
        use wiremock::ResponseTemplate;
        use wiremock::matchers::body_json;
        use wiremock::matchers::header;
        use wiremock::matchers::method;
        use wiremock::matchers::path;

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files"))
            .and(header("chatgpt-account-id", "account_id"))
            .and(body_json(serde_json::json!({
                "file_name": "one.csv",
                "file_size": 3,
                "use_case": "codex",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "file_id": "file_1",
                "upload_url": format!("{}/upload/file_1", server.uri()),
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files"))
            .and(header("chatgpt-account-id", "account_id"))
            .and(body_json(serde_json::json!({
                "file_name": "two.csv",
                "file_size": 3,
                "use_case": "codex",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "file_id": "file_2",
                "upload_url": format!("{}/upload/file_2", server.uri()),
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/upload/file_1"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/upload/file_2"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files/file_1/uploaded"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "download_url": format!("{}/download/file_1", server.uri()),
                "file_name": "one.csv",
                "mime_type": "text/csv",
                "file_size_bytes": 3,
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/backend-api/files/file_2/uploaded"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "download_url": format!("{}/download/file_2", server.uri()),
                "file_name": "two.csv",
                "mime_type": "text/csv",
                "file_size_bytes": 3,
            })))
            .expect(1)
            .mount(&server)
            .await;

        let (session, mut turn_context) = make_session_and_context().await;
        let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
        let dir = tempdir().expect("temp dir");
        tokio::fs::write(dir.path().join("one.csv"), b"one")
            .await
            .expect("write first local file");
        tokio::fs::write(dir.path().join("two.csv"), b"two")
            .await
            .expect("write second local file");
        set_primary_environment_cwd(&mut turn_context, dir.path());

        let mut config = (*turn_context.config).clone();
        config.chatgpt_base_url = format!("{}/backend-api", server.uri());
        turn_context.config = Arc::new(config);
        let step_context = StepContext::for_test(Arc::new(turn_context));
        let rewritten = rewrite_argument_value_for_openai_files(
            &step_context,
            /*read_policy*/ None,
            /*read_sandbox*/ None,
            &session.services.openai_file_upload_client_pool,
            Some(&auth),
            "files",
            &[],
            &serde_json::json!(["one.csv", "two.csv"]),
        )
        .await
        .expect("rewrite should succeed");

        assert_eq!(
            rewritten,
            Some(serde_json::json!([
                {
                    "download_url": format!("{}/download/file_1", server.uri()),
                    "file_id": "file_1",
                },
                {
                    "download_url": format!("{}/download/file_2", server.uri()),
                    "file_id": "file_2",
                }
            ]))
        );
    }

    #[tokio::test]
    async fn rewrite_mcp_tool_arguments_for_openai_files_surfaces_upload_failures() {
        let (mut session, turn_context) = make_session_and_context().await;
        session.services.auth_manager = crate::test_support::auth_manager_from_auth(
            CodexAuth::create_dummy_chatgpt_auth_for_testing(),
        );
        let step_context = StepContext::for_test(Arc::new(turn_context));
        let error = rewrite_mcp_tool_arguments_for_openai_files(
            &session,
            &step_context,
            Some(serde_json::json!({
                "file": "/definitely/missing/file.csv",
            })),
            Some(&HashMap::from([("file".to_string(), Vec::new())])),
        )
        .await
        .expect_err("missing file should fail");

        assert!(error.contains("failed to upload"));
        assert!(error.contains("file"));
    }
}
