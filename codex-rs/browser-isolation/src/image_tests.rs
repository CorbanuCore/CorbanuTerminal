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
async fn cached_image_inspection_preserves_docker_and_podman_ids() {
    use crate::EngineKind;
    use crate::command::EngineCommand;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;

    let digest = "a1".repeat(32);
    for (kind, id, recipe, expected) in [
        (
            EngineKind::Docker,
            format!("sha256:{digest}"),
            recipe_digest(),
            Ok(format!("sha256:{digest}")),
        ),
        (
            EngineKind::Podman,
            digest.clone(),
            recipe_digest(),
            Ok(digest.clone()),
        ),
        (
            EngineKind::Podman,
            "a1".repeat(31),
            recipe_digest(),
            Err(BrowserError::ContainerMismatch),
        ),
        (
            EngineKind::Podman,
            digest,
            "wrong-recipe".to_owned(),
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
        let tag = format!("localhost/corbanu-browser:recipe-{}", recipe_digest());
        assert_eq!(
            state["calls"],
            json!([["image", "inspect", &tag], ["image", "inspect", &tag]])
        );
    }
}
