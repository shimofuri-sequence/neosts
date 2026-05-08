mod keyframe;
mod platform;
mod selection;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub use keyframe::{
    ParseKeyframeDataError, ParsedKeyframeData, is_keyframe_data_text, jsx_script, keyframe_data,
    parse_keyframe_data, send_column_to_after_effects,
};
pub use platform::{SendToAfterEffectsError, after_effects_is_available};
pub use selection::{
    AeKeyframe, AeLayerPayload, AePayloadToSheetError, AeReceiveSession, AeSelectionPayload,
    ParseAePayloadError, ReceiveAePayloadError, ReceiveFromAfterEffectsError,
    begin_receive_session, parse_nonce_prefixed_payload, receive_selection_from_after_effects,
    receive_selection_payload_once, selection_payload_to_sheet,
};

const TEMP_JSX_PREFIX: &str = "neosts-";
const TEMP_JSX_SUFFIX: &str = ".jsx";

pub fn cleanup_temp_jsx_files() {
    let Ok(entries) = fs::read_dir(std::env::temp_dir()) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with(TEMP_JSX_PREFIX) || !file_name.ends_with(TEMP_JSX_SUFFIX) {
            continue;
        }

        let _ = fs::remove_file(path);
    }
}

fn generate_nonce() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id() as u128;
    let mixed = nanos
        .wrapping_mul(6364136223846793005)
        .wrapping_add(pid)
        .rotate_left(17);
    format!("{mixed:032x}")
}

fn write_temp_jsx(script: &str) -> Result<PathBuf, io::Error> {
    let mut path = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    path.push(format!("{TEMP_JSX_PREFIX}{timestamp}{TEMP_JSX_SUFFIX}"));
    fs::write(&path, script)?;
    Ok(path)
}

fn javascript_string_literal(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\'' => escaped.push_str("\\'"),
            ch if ch.is_ascii() && !ch.is_control() => escaped.push(ch),
            ch => {
                let mut buf = [0u16; 2];
                for unit in ch.encode_utf16(&mut buf).iter().copied() {
                    escaped.push_str(&format!("\\u{unit:04X}"));
                }
            }
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{TEMP_JSX_PREFIX, TEMP_JSX_SUFFIX};

    #[test]
    fn temp_jsx_file_name_pattern_is_stable() {
        let file_name = format!("{TEMP_JSX_PREFIX}123456{TEMP_JSX_SUFFIX}");
        assert!(file_name.starts_with(TEMP_JSX_PREFIX));
        assert!(file_name.ends_with(TEMP_JSX_SUFFIX));
    }
}
