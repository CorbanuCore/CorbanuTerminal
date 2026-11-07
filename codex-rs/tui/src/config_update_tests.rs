use super::*;
<<<<<<< HEAD
use crate::legacy_core::config::ConfigBuilder;
||||||| 413492cd6c
=======
use crate::history_cell::HistoryCell;
use crate::legacy_core::config::ConfigBuilder;
use crate::legacy_core::config::ConfigOverrides;
use codex_app_server_client::AppServerClient;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use color_eyre::eyre::WrapErr;
use pretty_assertions::assert_eq;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn app_scoped_key_path_quotes_dotted_app_ids() {
    assert_eq!(
        app_scoped_key_path("plugin.linear", "enabled"),
        "apps.\"plugin.linear\".enabled"
    );
}

#[test]
fn trusted_project_edit_targets_project_trust_level() {
    assert_eq!(
        trusted_project_edit(Path::new("/workspace/team.project")),
        ConfigEdit {
            key_path: "projects.\"/workspace/team.project\".trust_level".to_string(),
            value: serde_json::json!("trusted"),
            merge_strategy: MergeStrategy::Replace,
        }
    );
}

#[tokio::test]
async fn remote_project_trust_guards_thread_start_and_preserves_repository_decisions() -> Result<()>
{
    let temp_dir = tempfile::tempdir()?;
    let codex_home = temp_dir.path().join("codex-home");
    let project_root = temp_dir
        .path()
        .join("project as a trusted project in sibling");
    let project_cwd = project_root.join("nested");
    std::fs::create_dir_all(&codex_home)?;
    std::fs::create_dir_all(&project_cwd)?;
    std::fs::create_dir(project_root.join(".git"))?;
    std::fs::write(project_root.join(".git/HEAD"), "ref: refs/heads/main\n")?;
    std::fs::create_dir(project_cwd.join(".codex"))?;
    let undecided_config = format!(
        "[{}]\n",
        trusted_project_edit(&project_root)
            .key_path
            .trim_end_matches(".trust_level")
    );
    std::fs::write(codex_home.join("config.toml"), &undecided_config)?;
    std::fs::write(
        project_cwd.join(".codex/config.toml"),
        "model_reasoning_effort = \"high\"\n",
    )?;
    let config = ConfigBuilder::default()
        .codex_home(codex_home.clone())
        .harness_overrides(ConfigOverrides {
            cwd: Some(codex_home.clone()),
            ..ConfigOverrides::default()
        })
        .build()
        .await?;
    let app_server =
        AppServerClient::InProcess(crate::tests::start_test_embedded_app_server(config).await?);
    let read_trust = async |cwd: &Path, host| {
        read_remote_project_trust(app_server.request_handle(), cwd, host).await
    };
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let non_utf8 = project_root.join(std::ffi::OsString::from_vec(vec![0xff]));
        let error = read_trust(&non_utf8, ProjectTrustHost::Local)
            .await
            .expect_err("trust checks must not inspect a lossy replacement path");
        assert!(
            error
                .to_string()
                .contains("directory path is not valid UTF-8")
        );
    }
    let relative_cwd = pathdiff::diff_paths(&project_cwd, std::env::current_dir()?)
        .ok_or_else(|| color_eyre::eyre::eyre!("failed to calculate relative project path"))?;

    assert_eq!(
        read_trust(&relative_cwd, ProjectTrustHost::Remote).await?,
        Some(RemoteProjectTrust {
            trust_level: None,
            cwd: project_cwd.clone(),
            trust_target: PathBuf::from(project_trust_key(&project_root)),
        })
    );
    assert_eq!(
        std::fs::read_to_string(codex_home.join("config.toml"))?,
        undecided_config
    );

    write_trusted_project(app_server.request_handle(), &project_root).await?;
    let persisted_config: toml::Value =
        toml::from_str(&std::fs::read_to_string(codex_home.join("config.toml"))?)?;
    assert_eq!(
        persisted_config["projects"][project_trust_key(&project_root)]["trust_level"].as_str(),
        Some("trusted")
    );

    let _: ThreadStartResponse = app_server
        .request_typed(ClientRequest::ThreadStart {
            request_id: RequestId::Integer(1),
            params: ThreadStartParams {
                cwd: Some(project_cwd.to_string_lossy().into_owned()),
                ephemeral: Some(true),
                ..ThreadStartParams::default()
            },
        })
        .await?;
    assert_eq!(
        read_trust(&project_cwd, ProjectTrustHost::Remote).await?,
        None
    );

    let mut untrusted_project = trusted_project_edit(&project_root);
    untrusted_project.value = serde_json::json!("untrusted");
    write_config_batch(app_server.request_handle(), vec![untrusted_project]).await?;
    assert_eq!(
        read_trust(&project_cwd, ProjectTrustHost::Remote).await?,
        Some(RemoteProjectTrust {
            trust_level: Some(TrustLevel::Untrusted),
            cwd: project_cwd.clone(),
            trust_target: project_cwd.clone(),
        })
    );

    let response: JsonValue = app_server
        .request_typed(ClientRequest::ConfigRead {
            request_id: RequestId::String("untrusted-project-warning".to_string()),
            params: ConfigReadParams {
                include_layers: true,
                cwd: Some(project_cwd.to_string_lossy().into_owned()),
            },
        })
        .await?;
    let reason = response["layers"]
        .as_array()
        .expect("config layers")
        .iter()
        .find(|layer| layer["name"]["type"] == "project")
        .and_then(|layer| layer["disabledReason"].as_str())
        .expect("explicitly untrusted project warning");
    let warning = crate::history_cell::new_warning_event(
        reason.replace(&project_trust_key(&project_root), "<PROJECT>"),
    );
    insta::assert_snapshot!(
        warning
            .transcript_lines(/*width*/ 80)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
        @r"
    ⚠ <PROJECT> is marked as untrusted in the effective configuration. To load
      project-local config, hooks, and exec policies, update its trust setting. If
      that setting is managed by your organization, contact your administrator.
    "
    );

    std::fs::create_dir(project_root.join(".codex"))?;
    std::fs::write(
        project_root.join(".codex/config.toml"),
        "model_reasoning_effort = \"low\"\n",
    )?;
    for (parent, child) in [("untrusted", "trusted"), ("trusted", "untrusted")] {
        write_config_batch(
            app_server.request_handle(),
            vec![replace_config_value(
                "projects",
                serde_json::json!({
                    project_trust_key(&project_root): {"trust_level": parent},
                    project_trust_key(&project_cwd): {"trust_level": child}
                }),
            )],
        )
        .await?;
        let consent = read_trust(&project_cwd, ProjectTrustHost::Local).await?;
        assert_eq!(
            consent.map(|project| project.trust_level),
            (child == "untrusted").then_some(Some(TrustLevel::Untrusted))
        );
    }
    std::fs::remove_dir_all(project_root.join(".codex"))?;
    let untrusted_projects = replace_config_value(
        "projects",
        serde_json::json!({
            project_trust_key(&project_root): {"trust_level": "untrusted"}
        }),
    );
    write_config_batch(
        app_server.request_handle(),
        vec![untrusted_projects.clone()],
    )
    .await?;

    std::fs::remove_file(project_cwd.join(".codex/config.toml"))?;
    std::fs::remove_dir(project_cwd.join(".codex"))?;
    let canonical_project_cwd = PathBuf::from(project_trust_key(&project_root)).join("nested");
    let error = read_trust(&canonical_project_cwd, ProjectTrustHost::Remote)
        .await
        .expect_err("an untrusted repository must not be overridden by its subdirectory");
    assert!(error.to_string().contains("explicitly untrusted project"));

    assert_eq!(
        read_trust(&canonical_project_cwd, ProjectTrustHost::Local).await?,
        Some(RemoteProjectTrust {
            trust_level: Some(TrustLevel::Untrusted),
            cwd: canonical_project_cwd.clone(),
            trust_target: PathBuf::from(project_trust_key(&project_root)),
        })
    );
    write_trusted_project(app_server.request_handle(), &project_root).await?;
    assert_eq!(
        read_trust(&canonical_project_cwd, ProjectTrustHost::Local).await?,
        None
    );

    let key = project_trust_key(&project_root);
    #[cfg(windows)]
    let (legacy_key, lookup_cwd) = (key.to_ascii_uppercase(), canonical_project_cwd.clone());
    #[cfg(unix)]
    let (legacy_key, lookup_cwd) = {
        let alias = temp_dir.path().join("project-alias");
        std::os::unix::fs::symlink(&project_root, &alias)?;
        (alias.to_string_lossy().into_owned(), alias.join("nested"))
    };
    for (trust_level, canonical_key) in [
        ("trusted", true),
        ("untrusted", true),
        ("trusted", false),
        ("untrusted", false),
    ] {
        // Canonical decisions win over legacy symlink or case aliases.
        let mut projects = serde_json::json!({&legacy_key: {"trust_level": trust_level}});
        projects[&key] = serde_json::json!({});
        if canonical_key {
            projects[&legacy_key]["trust_level"] = serde_json::json!(if trust_level == "trusted" {
                "untrusted"
            } else {
                "trusted"
            });
            projects[&key] = serde_json::json!({"trust_level": trust_level});
        }
        write_config_batch(
            app_server.request_handle(),
            vec![replace_config_value("projects", projects)],
        )
        .await?;
        let consent = read_trust(&lookup_cwd, ProjectTrustHost::Local).await?;
        assert_eq!(
            consent.map(|project| project.trust_level),
            (trust_level == "untrusted").then_some(Some(TrustLevel::Untrusted)),
        );
    }
    std::fs::remove_dir_all(project_root.join(".git"))?;
    write_config_batch(app_server.request_handle(), vec![untrusted_projects]).await?;
    for marked_root in [false, true] {
        if marked_root {
            std::fs::write(project_root.join(".project-root"), "")?;
            write_config_batch(
                app_server.request_handle(),
                vec![replace_config_value(
                    "project_root_markers",
                    serde_json::json!([".project-root"]),
                )],
            )
            .await?;
        }
        assert_eq!(
            read_trust(&canonical_project_cwd, ProjectTrustHost::Local).await?,
            marked_root.then(|| RemoteProjectTrust {
                trust_level: Some(TrustLevel::Untrusted),
                cwd: canonical_project_cwd.clone(),
                trust_target: PathBuf::from(project_trust_key(&project_root)),
            })
        );
    }
    // A gitfile checkout is still a project when configured markers exclude Git.
    write_config_batch(
        app_server.request_handle(),
        vec![replace_config_value(
            "project_root_markers",
            serde_json::json!([]),
        )],
    )
    .await?;
    std::fs::write(project_cwd.join(".git"), "gitdir: ../separate-git-dir\n")?;
    assert!(
        read_trust(&canonical_project_cwd, ProjectTrustHost::Local)
            .await?
            .is_some()
    );
    app_server.shutdown().await?;
    Ok(())
}

#[test]
fn onboarding_provider_selection_persists_provider_and_compatible_model() {
    let edits = build_onboarding_provider_selection_edits(
        Some(codex_model_provider_info::AMBIENT_DEFAULT_MODEL),
        codex_model_provider_info::ANTHROPIC_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![
            ConfigEdit {
                key_path: "model".to_string(),
                value: serde_json::json!(codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL),
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_reasoning_effort".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::ANTHROPIC_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            },
        ]
    );
}

#[test]
fn onboarding_openai_without_model_persists_only_provider() {
    let edits = build_onboarding_provider_selection_edits(
        /*current_model*/ None,
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![ConfigEdit {
            key_path: "model_provider".to_string(),
            value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
            merge_strategy: MergeStrategy::Replace,
        }]
    );
}

#[test]
fn onboarding_openai_clears_fresh_inherited_ambient_model() {
    let edits = build_onboarding_provider_selection_edits(
        Some(codex_model_provider_info::AMBIENT_DEFAULT_MODEL),
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![
            ConfigEdit {
                key_path: "model".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_reasoning_effort".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            },
        ]
    );
}

#[test]
fn onboarding_openai_clears_explicit_claude_model_and_effort() {
    let existing_config = serde_json::json!({
        "model": codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL,
        "model_reasoning_effort": "high",
    });
    let edits = build_onboarding_provider_selection_edits(
        existing_config["model"].as_str(),
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![
            ConfigEdit {
                key_path: "model".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_reasoning_effort".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            },
        ]
    );
}

#[test]
fn onboarding_returning_openai_model_preserves_high_reasoning_effort() {
    let existing_config = serde_json::json!({
        "model": "gpt-5.6-sol",
        "model_reasoning_effort": "high",
    });
    let edits = build_onboarding_provider_selection_edits(
        existing_config["model"].as_str(),
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(existing_config["model_reasoning_effort"], "high");
    assert_eq!(
        edits,
        vec![ConfigEdit {
            key_path: "model_provider".to_string(),
            value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
            merge_strategy: MergeStrategy::Replace,
        }]
    );
}

#[test]
fn onboarding_openai_preserves_catalog_and_custom_compatible_models() {
    for model in ["codex-auto-fast", "o3"] {
        let existing_config = serde_json::json!({
            "model": model,
            "model_reasoning_effort": "high",
        });
        let edits = build_onboarding_provider_selection_edits(
            existing_config["model"].as_str(),
            codex_model_provider_info::OPENAI_PROVIDER_ID,
        );

        assert_eq!(
            edits,
            vec![ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            }],
            "unexpected edits for OpenAI-compatible model {model}"
        );
    }
}

#[tokio::test]
async fn onboarding_uses_user_model_instead_of_runtime_override() {
    let codex_home = tempdir().expect("temp codex home");
    std::fs::write(
        codex_home.path().join("config.toml"),
        "model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"high\"\n",
    )
    .expect("write user config");
    let config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .cli_overrides(vec![(
            "model".to_string(),
            toml::Value::String(codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL.to_string()),
        )])
        .build()
        .await
        .expect("config with runtime override");

    assert_eq!(
        config.model.as_deref(),
        Some(codex_model_provider_info::CLAUDE_PLAN_MODEL)
    );
    let user_model = onboarding_user_model(&config);
    assert_eq!(user_model.as_deref(), Some("gpt-5.6-sol"));
    assert_eq!(
        build_onboarding_provider_selection_edits(
            user_model.as_deref(),
            codex_model_provider_info::OPENAI_PROVIDER_ID,
        ),
        vec![ConfigEdit {
            key_path: "model_provider".to_string(),
            value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
            merge_strategy: MergeStrategy::Replace,
        }]
    );
}

#[test]
fn onboarding_same_anthropic_model_preserves_high_reasoning_effort() {
    let existing_config = serde_json::json!({
        "model": codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL,
        "model_reasoning_effort": "high",
    });
    let edits = build_onboarding_provider_selection_edits(
        existing_config["model"].as_str(),
        codex_model_provider_info::ANTHROPIC_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![ConfigEdit {
            key_path: "model_provider".to_string(),
            value: serde_json::json!(codex_model_provider_info::ANTHROPIC_PROVIDER_ID),
            merge_strategy: MergeStrategy::Replace,
        }]
    );
}

#[test]
fn format_config_error_preserves_server_validation_message() {
    let err = Err::<(), _>(color_eyre::eyre::eyre!(
        "config/batchWrite failed: Invalid configuration: features.fast_mode=true violates \
         managed requirements; allowed set [fast_mode=false]"
    ))
    .wrap_err("config/batchWrite failed in TUI")
    .unwrap_err();

    assert_eq!(
        format_config_error(&err),
        "config/batchWrite failed in TUI: config/batchWrite failed: Invalid configuration: \
         features.fast_mode=true violates managed requirements; allowed set [fast_mode=false]"
    );
}

#[test]
fn effective_defaults_accept_foreign_layer_paths() -> Result<()> {
    let layers = serde_json::json!([
        {"name": {"type": "system", "file": "/etc/codex/config.toml"}, "version": "1", "config": {}},
        {"name": {"type": "project", "dotCodexFolder": "C:\\work\\.codex"}, "disabledReason": "untrusted", "version": "1", "config": {}}
    ]);
    let response: EffectiveConfig = serde_json::from_value(serde_json::json!({
        "config": {"model": "server-model"},
        "origins": {"model": {"name": {"type": "user", "file": "C:\\Users\\user\\.codex\\config.toml", "profile": null}, "version": "1"}},
        "layers": layers,
    }))?;
    assert_eq!(
        (
            response.config.model.as_deref(),
            serde_json::to_value(response.layers)?
        ),
        (Some("server-model"), layers)
    );
    Ok(())
}
