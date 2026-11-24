use super::*;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// #2872: numbers in JSON-valued config fields (`desktop`, `telegram`,
/// `openrouter_provider`) are exported, diffed and replayed as TOML numbers,
/// never as serde_json's private number tables, and read back unchanged.
#[test]
fn json_valued_numbers_stay_toml_numbers() {
    let config = ConfigToml {
        desktop: Some(HashMap::from([
            ("zoom".to_string(), serde_json::json!(1.25)),
            ("appearanceTheme".to_string(), serde_json::json!("dark")),
        ])),
        telegram: Some(BTreeMap::from([
            (
                "allowed_chat_ids".to_string(),
                serde_json::json!([123456789]),
            ),
            ("media_retention_days".to_string(), serde_json::json!(7)),
        ])),
        openrouter_provider: Some(serde_json::json!({
            "max_price": {"prompt": 0.5, "completion": -2},
        })),
        ..Default::default()
    };
    let lockfile = config_lockfile(config);

    let written = config_lock_toml_string(&lockfile).expect("serialize config lock");
    assert!(!written.contains(JSON_NUMBER_FIELD), "{written}");
    let written: toml::Table = toml::from_str(&written).expect("parse config lock");
    let expected = toml::toml! {
        [desktop]
        zoom = 1.25
        appearanceTheme = "dark"
        [telegram]
        allowed_chat_ids = [123456789]
        media_retention_days = 7
        [openrouter_provider.max_price]
        prompt = 0.5
        completion = -2
    };
    for key in ["desktop", "telegram", "openrouter_provider"] {
        assert_eq!(written["config"][key], expected[key], "{key}");
    }
    let read_back: ConfigLockfileToml =
        toml::from_str(&toml::to_string(&written).expect("serialize")).expect("read back");
    assert_eq!(read_back.config.desktop, lockfile.config.desktop);
    assert_eq!(read_back.config.telegram, lockfile.config.telegram);
    assert_eq!(
        read_back.config.openrouter_provider,
        lockfile.config.openrouter_provider
    );

    // The lock's config layer holds the same numbers.
    let path = AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("x.config.lock.toml"))
        .expect("absolute path");
    let layer = lock_layer_from_config(&path, &lockfile).expect("lock layer");
    for key in ["desktop", "telegram", "openrouter_provider"] {
        assert_eq!(layer.config[key], expected[key], "{key}");
    }
}

/// A number TOML cannot hold exactly keeps serde_json's table, which reads
/// back as the same number; other tables are left as they are.
#[test]
fn only_exact_numbers_are_rewritten() {
    let table = |text: &str| {
        let mut table = toml::Table::new();
        table.insert(
            JSON_NUMBER_FIELD.to_string(),
            toml::Value::String(text.into()),
        );
        table
    };
    for (text, expected) in [
        ("7", Some(toml::Value::Integer(7))),
        ("-2", Some(toml::Value::Integer(-2))),
        ("1.25", Some(toml::Value::Float(1.25))),
        ("18446744073709551615", None),
        ("1e3", None),
        ("007", None),
        ("x", None),
    ] {
        assert_eq!(json_number(&table(text)), expected, "{text}");
    }
    let mut other = table("7");
    other.insert("more".to_string(), toml::Value::Integer(1));
    assert_eq!(json_number(&other), None);
}
