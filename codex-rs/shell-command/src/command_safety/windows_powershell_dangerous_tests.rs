use super::is_dangerous_command_windows;
use base64::Engine as _;

#[test]
fn joined_decoded_and_declined_scripts_keep_dangerous_warnings() {
    let script = "Remove-Item -Recurse -Force C:\\fixture; $a";
    let bytes: Vec<_> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    for args in [
        vec![
            "-Command",
            "Remove-Item -Recurse -Force C:\\fixture",
            "; $a",
        ],
        vec!["-Command", "$a;", "Remove-Item -Recurse -Force C:\\fixture"],
        vec!["-comm", "Remove-Item", "-Force", "C:\\fixture; $a"],
        vec!["-ep", "Bypass", "-c", script],
        vec!["-EncodedCommand", encoded.as_str()],
        vec!["-ec", encoded.as_str()],
        vec!["-UnknownSwitch", "-Command", script],
        vec!["-Command", "Remove-Item -Force C:\\fixture", "; '"],
        vec!["-Command:Remove-Item -Force C:\\fixture"],
    ] {
        let command: Vec<String> = std::iter::once("powershell.exe")
            .chain(args)
            .map(str::to_owned)
            .collect();
        assert!(is_dangerous_command_windows(&command), "{command:?}");
    }
}

#[test]
fn joined_ordinary_scripts_are_not_dangerous() {
    for args in [
        vec!["-Command", "git", "status"],
        vec!["-Command", "Get-Content", "Cargo.toml"],
        vec!["-Command", "Remove-Item", "C:\\fixture"],
    ] {
        let command: Vec<String> = std::iter::once("powershell.exe")
            .chain(args)
            .map(str::to_owned)
            .collect();
        assert!(!is_dangerous_command_windows(&command), "{command:?}");
    }
}
