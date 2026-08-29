use super::*;
use pretty_assertions::assert_eq;

#[test]
fn image_ids_require_a_full_hash_with_only_an_optional_sha256_prefix() {
    let digest = "a1".repeat(32);
    assert!(is_image_id(&digest));
    assert!(is_image_id(&format!("sha256:{digest}")));
    for invalid in [
        String::new(),
        "a1".repeat(31),
        "a1".repeat(33),
        "g1".repeat(32),
        format!("sha512:{digest}"),
        format!("sha256:sha256:{digest}"),
        format!("{digest} "),
    ] {
        assert!(!is_image_id(&invalid), "{invalid}");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn image_build_binds_the_returned_docker_or_podman_id() {
    use crate::EngineKind;
    use crate::command::EngineCommand;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;

    let digest = "a1".repeat(32);
    for (kind, id, build_id, recipe, expected) in [
        (
            EngineKind::Docker,
            format!("sha256:{digest}"),
            format!("sha256:{digest}"),
            recipe_digest(),
            Ok(format!("sha256:{digest}")),
        ),
        (
            EngineKind::Podman,
            digest.clone(),
            digest.clone(),
            recipe_digest(),
            Ok(digest.clone()),
        ),
        (
            EngineKind::Podman,
            "a1".repeat(31),
            "a1".repeat(31),
            recipe_digest(),
            Err(BrowserError::ContainerMismatch),
        ),
        (
            EngineKind::Podman,
            digest.clone(),
            digest.clone(),
            "wrong-recipe".to_owned(),
            Err(BrowserError::ContainerMismatch),
        ),
        (
            EngineKind::Podman,
            "a2".repeat(32),
            digest,
            recipe_digest(),
            Err(BrowserError::ContainerMismatch),
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("engine");
        std::fs::write(&executable, include_str!("../worker/fixture_engine.py")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let state_path = directory.path().join("state.json");
        std::fs::write(&state_path, serde_json::to_vec(&json!({
            "calls": [],
            "build_id": build_id,
            "image": {
                "Id": id,
                "Config": {
                    "User": "65532:65532",
                    "Labels": {"org.corbanu.browser.recipe": recipe},
                    "Entrypoint": ["/app/.venv/bin/python", "-I", "-u", "/opt/corbanu/worker.py"]
                }
            }
        })).unwrap()).unwrap();
        let engine = ContainerEngine {
            kind,
            command: EngineCommand {
                executable: AbsolutePathBuf::from_absolute_path_checked(executable).unwrap(),
                environment: BTreeMap::from([(
                    "PF30_FIXTURE_STATE".into(),
                    state_path.clone().into_os_string(),
                )]),
            },
        };
        assert_eq!(engine.prepare_image().await, expected);
        let state: serde_json::Value =
            serde_json::from_slice(&std::fs::read(state_path).unwrap()).unwrap();
        let calls = state["calls"].as_array().unwrap();
        let tag = format!("localhost/corbanu-browser:recipe-{}", recipe_digest());
        assert_eq!(calls[0], json!(["image", "inspect", BASE_IMAGE]));
        let build = calls[1].as_array().unwrap();
        assert_eq!(
            &build[..build.len() - 1],
            &json!([
                "build",
                "--network=none",
                "--pull=false",
                "--no-cache",
                "--quiet",
                "--label",
                format!("org.corbanu.browser.recipe={}", recipe_digest()),
                "--tag",
                &tag,
            ])
            .as_array()
            .unwrap()[..]
        );
        assert!(std::path::Path::new(build.last().unwrap().as_str().unwrap()).is_absolute());
        let built_id = state["build_id"].as_str().unwrap();
        if is_image_id(built_id) {
            assert_eq!(calls[2], json!(["image", "inspect", built_id]));
            assert_eq!(calls.len(), 3);
        } else {
            assert_eq!(calls.len(), 2);
        }
    }
}
