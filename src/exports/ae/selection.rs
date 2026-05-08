use super::generate_nonce;
use super::platform::{SendToAfterEffectsError, launch_after_effects_script};
use super::write_temp_jsx;
use crate::{AeKaraCellMode, AppLocale, strings};
use serde::{Deserialize, Serialize};
use sheet::{CellValue, Sheet, SheetColumn};
use std::fs;
use std::io;
use std::io::Read;
use std::net::TcpListener;

const RECEIVE_FROM_AE_PORT: u16 = 31715;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AeSelectionResult {
    Ok,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AeSelectionPayload {
    pub result: AeSelectionResult,
    pub fps: f64,
    pub comp_name: String,
    pub comp_duration: f64,
    #[serde(default)]
    pub error: Option<String>,
    pub layers: Vec<AeLayerPayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AeLayerPayload {
    pub name: String,
    pub index: usize,
    pub in_point: f64,
    pub out_point: f64,
    pub source_duration: Option<f64>,
    pub source_frame_duration: Option<f64>,
    pub is_comp_layer: bool,
    #[serde(default)]
    pub time_remap: Vec<AeKeyframe>,
    #[serde(default)]
    pub blinds: Vec<AeKeyframe>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AeKeyframe {
    pub time: f64,
    pub value: f64,
    #[serde(default)]
    pub kara_cell: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AeReceiveSession {
    pub nonce: String,
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseAePayloadError {
    #[error("payload is missing nonce separator")]
    MissingNonceSeparator,
    #[error("unexpected nonce")]
    NonceMismatch,
    #[error("payload JSON is invalid")]
    InvalidJson(#[from] serde_json::Error),
    #[error("After Effects reported an error: {0}")]
    AfterEffects(String),
}

impl ParseAePayloadError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::MissingNonceSeparator => match locale {
                AppLocale::Japanese => "受信データに nonce 区切りがありません".to_owned(),
                AppLocale::English => "Payload is missing a nonce separator".to_owned(),
            },
            Self::NonceMismatch => match locale {
                AppLocale::Japanese => "受信データの nonce が一致しません".to_owned(),
                AppLocale::English => "Unexpected nonce".to_owned(),
            },
            Self::InvalidJson(source) => match locale {
                AppLocale::Japanese => format!("受信データの JSON が不正です: {source}"),
                AppLocale::English => format!("Payload JSON is invalid: {source}"),
            },
            Self::AfterEffects(message) => match locale {
                AppLocale::Japanese => format!("After Effects がエラーを返しました: {message}"),
                AppLocale::English => format!("After Effects reported an error: {message}"),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveAePayloadError {
    #[error("failed to bind localhost listener")]
    Bind(#[source] io::Error),
    #[error("failed to accept localhost connection")]
    Accept(#[source] io::Error),
    #[error("failed to read localhost payload")]
    Read(#[source] io::Error),
    #[error(transparent)]
    Parse(#[from] ParseAePayloadError),
}

impl ReceiveAePayloadError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::Bind(source) => match locale {
                AppLocale::Japanese => {
                    format!("localhost リスナーの開始に失敗しました: {source}")
                }
                AppLocale::English => {
                    format!("Failed to bind localhost listener: {source}")
                }
            },
            Self::Accept(source) => match locale {
                AppLocale::Japanese => {
                    format!("localhost 接続の受け入れに失敗しました: {source}")
                }
                AppLocale::English => {
                    format!("Failed to accept localhost connection: {source}")
                }
            },
            Self::Read(source) => match locale {
                AppLocale::Japanese => {
                    format!("localhost ペイロードの読込に失敗しました: {source}")
                }
                AppLocale::English => {
                    format!("Failed to read localhost payload: {source}")
                }
            },
            Self::Parse(source) => source.localized_message(locale),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveFromAfterEffectsError {
    #[error("After Effects script execution is only supported on Windows")]
    UnsupportedPlatform,
    #[error("failed to write temporary JSX file")]
    WriteScript(#[source] io::Error),
    #[error("failed to launch After Effects")]
    Launch(#[source] io::Error),
    #[error(transparent)]
    Receive(#[from] ReceiveAePayloadError),
}

impl ReceiveFromAfterEffectsError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::UnsupportedPlatform => match locale {
                AppLocale::Japanese => {
                    "After Effects スクリプト実行は Windows のみ対応です".to_owned()
                }
                AppLocale::English => {
                    "After Effects script execution is only supported on Windows".to_owned()
                }
            },
            Self::WriteScript(source) => match locale {
                AppLocale::Japanese => {
                    format!("一時 JSX ファイルの書き込みに失敗しました: {source}")
                }
                AppLocale::English => {
                    format!("Failed to write temporary JSX file: {source}")
                }
            },
            Self::Launch(source) => match locale {
                AppLocale::Japanese => format!("After Effects の起動に失敗しました: {source}"),
                AppLocale::English => format!("Failed to launch After Effects: {source}"),
            },
            Self::Receive(source) => source.localized_message(locale),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AePayloadToSheetError {
    #[error("After Effects reported an error: {0}")]
    AfterEffects(String),
    #[error("invalid fps in AE payload")]
    InvalidFps,
}

impl AePayloadToSheetError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::AfterEffects(message) => match locale {
                AppLocale::Japanese => format!("After Effects がエラーを返しました: {message}"),
                AppLocale::English => format!("After Effects reported an error: {message}"),
            },
            Self::InvalidFps => match locale {
                AppLocale::Japanese => "AE ペイロードの fps が不正です".to_owned(),
                AppLocale::English => "Invalid fps in AE payload".to_owned(),
            },
        }
    }
}

pub fn begin_receive_session(port: u16) -> AeReceiveSession {
    AeReceiveSession {
        nonce: generate_nonce(),
        port,
    }
}

pub fn parse_nonce_prefixed_payload(
    payload: &str,
    expected_nonce: &str,
) -> Result<AeSelectionPayload, ParseAePayloadError> {
    let Some(separator_index) = payload.find(":{") else {
        return Err(ParseAePayloadError::MissingNonceSeparator);
    };
    let nonce = &payload[..separator_index];
    let json = &payload[separator_index + 1..];
    if nonce != expected_nonce {
        return Err(ParseAePayloadError::NonceMismatch);
    }

    let payload: AeSelectionPayload =
        serde_json::from_str(json).map_err(ParseAePayloadError::InvalidJson)?;
    if payload.result == AeSelectionResult::Error {
        return Err(ParseAePayloadError::AfterEffects(
            payload
                .error
                .unwrap_or_else(|| strings::ae_error_unknown(AppLocale::English).to_owned()),
        ));
    }
    Ok(payload)
}

pub fn receive_selection_payload_once(
    listener: TcpListener,
    session: &AeReceiveSession,
) -> Result<AeSelectionPayload, ReceiveAePayloadError> {
    let (mut stream, _) = listener.accept().map_err(ReceiveAePayloadError::Accept)?;
    let mut payload = String::new();
    stream
        .read_to_string(&mut payload)
        .map_err(ReceiveAePayloadError::Read)?;
    parse_nonce_prefixed_payload(&payload, &session.nonce).map_err(ReceiveAePayloadError::Parse)
}

pub fn receive_selection_from_after_effects(
    kara_cell_mode: AeKaraCellMode,
    owner_hwnd: Option<isize>,
    locale: AppLocale,
) -> Result<AeSelectionPayload, ReceiveFromAfterEffectsError> {
    let session = begin_receive_session(RECEIVE_FROM_AE_PORT);
    let listener = TcpListener::bind(("127.0.0.1", session.port)).map_err(|error| {
        ReceiveFromAfterEffectsError::Receive(ReceiveAePayloadError::Bind(error))
    })?;
    let script = receive_selection_jsx(&session, kara_cell_mode, locale);
    let script_path = write_temp_jsx(&script).map_err(ReceiveFromAfterEffectsError::WriteScript)?;
    let launch_result =
        launch_after_effects_script(&script_path, owner_hwnd).map_err(|error| match error {
            SendToAfterEffectsError::UnsupportedPlatform => {
                ReceiveFromAfterEffectsError::UnsupportedPlatform
            }
            SendToAfterEffectsError::Launch(source) => ReceiveFromAfterEffectsError::Launch(source),
            SendToAfterEffectsError::WriteScript(source) => {
                ReceiveFromAfterEffectsError::WriteScript(source)
            }
            SendToAfterEffectsError::WindowNotFound
            | SendToAfterEffectsError::ExecutablePathUnavailable => {
                ReceiveFromAfterEffectsError::Launch(io::Error::new(
                    io::ErrorKind::NotFound,
                    strings::ae_error_could_not_connect(locale),
                ))
            }
        });

    if let Err(error) = launch_result {
        let _ = fs::remove_file(&script_path);
        return Err(error);
    }

    let result = receive_selection_payload_once(listener, &session);
    let _ = fs::remove_file(&script_path);
    result.map_err(ReceiveFromAfterEffectsError::Receive)
}

pub fn selection_payload_to_sheet(
    payload: &AeSelectionPayload,
    kara_cell_mode: AeKaraCellMode,
) -> Result<Sheet, AePayloadToSheetError> {
    if payload.result == AeSelectionResult::Error {
        return Err(AePayloadToSheetError::AfterEffects(
            payload
                .error
                .clone()
                .unwrap_or_else(|| strings::ae_error_unknown(AppLocale::English).to_owned()),
        ));
    }

    let fps = payload.fps.round() as u32;
    if fps == 0 {
        return Err(AePayloadToSheetError::InvalidFps);
    }

    let row_count = duration_to_frame_count(payload.comp_duration, fps).max(1);
    let columns = payload
        .layers
        .iter()
        .map(|layer| {
            let mut values = vec![CellValue::blank(); row_count];
            apply_time_remap_to_cells(&mut values, payload, layer, fps, kara_cell_mode);
            apply_blinds_to_cells(&mut values, layer, fps);
            apply_in_out_points_to_cells(&mut values, layer, fps);
            SheetColumn::new(layer.name.clone(), values)
        })
        .collect::<Vec<_>>();

    Ok(Sheet::with_fps(columns, fps))
}

fn duration_to_frame_count(duration: f64, fps: u32) -> usize {
    (duration.max(0.0) * fps as f64).round() as usize
}

fn time_to_frame_index(time: f64, fps: u32) -> usize {
    (time.max(0.0) * fps as f64).round() as usize
}

fn remap_seconds_to_cell_value(seconds: f64, units_per_second: f64) -> i64 {
    ((seconds * units_per_second).round() as i64 + 1).max(1)
}

fn apply_time_remap_to_cells(
    values: &mut [CellValue],
    _payload: &AeSelectionPayload,
    layer: &AeLayerPayload,
    fps: u32,
    kara_cell_mode: AeKaraCellMode,
) {
    if layer.time_remap.is_empty() {
        return;
    }

    let active_start = time_to_frame_index(layer.in_point, fps).min(values.len());
    let active_end = time_to_frame_index(layer.out_point, fps).min(values.len());
    if active_start >= active_end {
        return;
    }

    let mut keyframes = layer.time_remap.clone();
    keyframes.sort_by(|left, right| left.time.total_cmp(&right.time));
    for (index, keyframe) in keyframes.iter().enumerate() {
        let start = time_to_frame_index(keyframe.time, fps)
            .max(active_start)
            .min(values.len());
        let end = keyframes
            .get(index + 1)
            .map(|next| time_to_frame_index(next.time, fps))
            .unwrap_or(active_end)
            .max(start)
            .min(active_end);
        if start >= end {
            continue;
        }

        let mut cell_value = if keyframe.kara_cell {
            0
        } else {
            remap_seconds_to_cell_value(keyframe.value, fps as f64)
        };
        if cell_value > 0
            && kara_cell_mode == AeKaraCellMode::MaxFrameCount
            && source_frame_count(layer)
                .is_some_and(|source_frame_count| cell_value > source_frame_count)
        {
            cell_value = 0;
        }

        for cell in &mut values[start..end] {
            *cell = CellValue::Int(cell_value);
        }
    }
}

fn source_frame_count(layer: &AeLayerPayload) -> Option<i64> {
    let duration = layer.source_duration?;
    let frame_duration = layer.source_frame_duration?;
    if !duration.is_finite() || !frame_duration.is_finite() || frame_duration <= 0.0 {
        return None;
    }

    Some(((duration / frame_duration).round() as i64).max(0))
}

fn apply_blinds_to_cells(values: &mut [CellValue], layer: &AeLayerPayload, fps: u32) {
    if layer.blinds.is_empty() {
        return;
    }

    let mut keyframes = layer.blinds.clone();
    keyframes.sort_by(|left, right| left.time.total_cmp(&right.time));
    for (index, keyframe) in keyframes.iter().enumerate() {
        let start = time_to_frame_index(keyframe.time, fps).min(values.len());
        let end = keyframes
            .get(index + 1)
            .map(|next| time_to_frame_index(next.time, fps))
            .unwrap_or(values.len())
            .max(start)
            .min(values.len());
        if start >= end || keyframe.value < 100.0 {
            continue;
        }

        for cell in &mut values[start..end] {
            *cell = CellValue::Int(0);
        }
    }
}

fn apply_in_out_points_to_cells(values: &mut [CellValue], layer: &AeLayerPayload, fps: u32) {
    let active_start = time_to_frame_index(layer.in_point, fps).min(values.len());
    let active_end = time_to_frame_index(layer.out_point, fps).min(values.len());

    for cell in &mut values[..active_start] {
        *cell = CellValue::Int(0);
    }
    for cell in &mut values[active_end..] {
        *cell = CellValue::Int(0);
    }
}

fn receive_selection_jsx(
    session: &AeReceiveSession,
    kara_cell_mode: AeKaraCellMode,
    locale: AppLocale,
) -> String {
    let kara_cell_mode = match kara_cell_mode {
        AeKaraCellMode::Blinds => "blinds",
        AeKaraCellMode::MaxFrameCount => "max_frame_count",
    };

    format!(
        r#"(function() {{
var NEOSTS_PORT = {port};
var NEOSTS_NONCE = "{nonce}";
var karaCellMode = "{kara_cell_mode}";

function escapeString(str) {{
    if (str == null) return "";
    str = String(str);
    return str.replace(/\\/g, "\\\\")
              .replace(/"/g, '\\"')
              .replace(/\n/g, "\\n")
              .replace(/\r/g, "\\r")
              .replace(/\t/g, "\\t");
}}

function sendJson(json) {{
    var socket = new Socket();
    socket.encoding = "UTF-8";
    if (socket.open("127.0.0.1:" + NEOSTS_PORT)) {{
        socket.write(NEOSTS_NONCE + ":" + json);
        socket.close();
    }} else {{
        alert("{connect_error}");
    }}
}}

function sendError(message) {{
    sendJson('{{"result":"error","fps":0,"compName":"error","compDuration":0,"error":"' + escapeString(message) + '","layers":[]}}');
}}

function pushKeyframes(prop, dest) {{
    if (!prop || prop.numKeys === 0) {{
        return;
    }}
    for (var i = 1; i <= prop.numKeys; i++) {{
        dest.push({{
            time: prop.keyTime(i),
            value: prop.keyValue(i)
        }});
    }}
}}

function sourceFrameDuration(layer, comp) {{
    if (layer.source && typeof layer.source.frameDuration === "number" && isFinite(layer.source.frameDuration) && layer.source.frameDuration > 0) {{
        return layer.source.frameDuration;
    }}
    return comp.frameDuration;
}}

function sourceMaxTime(layer, comp) {{
    if (layer.source && typeof layer.source.duration === "number" && isFinite(layer.source.duration) && layer.source.duration >= 0) {{
        return layer.source.duration;
    }}
    return comp.duration;
}}

function isKaraCellValueForNeoSTS(layer, comp, value) {{
    if (karaCellMode === "blinds") {{
        return false;
    }}

    if (karaCellMode === "max_frame_count") {{
        var frameDuration = sourceFrameDuration(layer, comp);
        if (!(frameDuration > 0)) {{
            return false;
        }}

        var sourceFrameCount = Math.max(0, Math.round(sourceMaxTime(layer, comp) / frameDuration));
        var remapFrameIndex = Math.max(0, Math.round(value / frameDuration));
        var remapFrame = remapFrameIndex + 1;
        return remapFrame > sourceFrameCount;
    }}

    return false;
}}

function clearSelectedLayers(comp) {{
    for (var layerIndex = 1; layerIndex <= comp.numLayers; ++layerIndex) {{
        comp.layer(layerIndex).selected = false;
    }}
}}

try {{
    var comp = app.project.activeItem;
    if (!comp || !(comp instanceof CompItem)) {{
        throw new Error("{select_comp_error}");
    }}

    var layers = comp.selectedLayers;
    if (!layers || layers.length === 0) {{
        throw new Error("{select_layers_error}");
    }}

    var json = '{{';
    json += '"result":"ok",';
    json += '"fps":' + comp.frameRate + ',';
    json += '"compName":"' + escapeString(comp.name) + '",';
    json += '"compDuration":' + comp.duration + ',';
    json += '"layers":[';

    var added = 0;
    for (var i = 0; i < layers.length; i++) {{
        var layer = layers[i];
        if (!(layer instanceof AVLayer)) {{
            continue;
        }}

        var timeRemap = layer.property("ADBE Time Remapping");
        var blindProp = null;
        var blindEffect = layer.property("ADBE Effect Parade");
        if (blindEffect) {{
            var blinds = blindEffect.property("ADBE Venetian Blinds");
            if (blinds) {{
                blindProp = blinds.property("ADBE Venetian Blinds-0001");
            }}
        }}

        if (added > 0) json += ',';
        json += '{{';
        json += '"name":"' + escapeString(layer.name) + '",';
        json += '"index":' + layer.index + ',';
        json += '"inPoint":' + layer.inPoint + ',';
        json += '"outPoint":' + layer.outPoint + ',';
        json += '"sourceDuration":' + ((layer.source && typeof layer.source.duration === "number") ? layer.source.duration : 'null') + ',';
        json += '"sourceFrameDuration":' + ((layer.source && typeof layer.source.frameDuration === "number") ? layer.source.frameDuration : 'null') + ',';
        json += '"isCompLayer":' + ((layer.source instanceof CompItem) ? 'true' : 'false') + ',';

        var timeRemapKeys = [];
        pushKeyframes(timeRemap, timeRemapKeys);
        json += '"timeRemap":[';
        for (var tr = 0; tr < timeRemapKeys.length; tr++) {{
            if (tr > 0) json += ',';
            json += '{{"time":' + timeRemapKeys[tr].time + ',"value":' + timeRemapKeys[tr].value + ',"karaCell":' +
                (isKaraCellValueForNeoSTS(layer, comp, timeRemapKeys[tr].value) ? 'true' : 'false') + '}}';
        }}
        json += '],';

        var blindKeys = [];
        pushKeyframes(blindProp, blindKeys);
        json += '"blinds":[';
        for (var bi = 0; bi < blindKeys.length; bi++) {{
            if (bi > 0) json += ',';
            json += '{{"time":' + blindKeys[bi].time + ',"value":' + blindKeys[bi].value + '}}';
        }}
        json += ']';
        json += '}}';
        added += 1;
    }}

    if (added === 0) {{
        throw new Error("{select_avlayer_error}");
    }}

    json += ']}}';
    sendJson(json);
    clearSelectedLayers(comp);
}} catch (error) {{
    sendError(error.toString());
}}
}})();
"#,
        port = session.port,
        nonce = session.nonce,
        kara_cell_mode = kara_cell_mode,
        connect_error = strings::ae_error_could_not_connect(locale),
        select_comp_error = strings::ae_error_select_composition(locale),
        select_layers_error = strings::ae_error_select_layers(locale),
        select_avlayer_error = strings::ae_error_select_av_layer(locale)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AeKeyframe, AeLayerPayload, AePayloadToSheetError, AeSelectionPayload, AeSelectionResult,
        ParseAePayloadError, begin_receive_session, parse_nonce_prefixed_payload,
        receive_selection_jsx, selection_payload_to_sheet,
    };
    use crate::AeKaraCellMode;
    use crate::AppLocale;
    use sheet::CellValue;

    #[test]
    fn begin_receive_session_creates_nonce_and_keeps_port() {
        let session = begin_receive_session(31715);
        assert_eq!(session.port, 31715);
        assert_eq!(session.nonce.len(), 32);
        assert!(session.nonce.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn parses_nonce_prefixed_ae_selection_payload() {
        let payload = parse_nonce_prefixed_payload(
            r#"abc123:{"result":"ok","fps":24.0,"compName":"Comp 1","compDuration":3.0,"layers":[{"name":"A-1","index":3,"inPoint":0.5,"outPoint":2.5,"sourceDuration":2.0,"isCompLayer":true,"timeRemap":[{"time":0.0,"value":0.0}],"blinds":[{"time":0.0,"value":100.0}]}]}"#,
            "abc123",
        )
        .unwrap();

        assert_eq!(
            payload,
            AeSelectionPayload {
                result: AeSelectionResult::Ok,
                fps: 24.0,
                comp_name: "Comp 1".to_owned(),
                comp_duration: 3.0,
                error: None,
                layers: vec![AeLayerPayload {
                    name: "A-1".to_owned(),
                    index: 3,
                    in_point: 0.5,
                    out_point: 2.5,
                    source_duration: Some(2.0),
                    source_frame_duration: None,
                    is_comp_layer: true,
                    time_remap: vec![AeKeyframe {
                        time: 0.0,
                        value: 0.0,
                        kara_cell: false,
                    }],
                    blinds: vec![AeKeyframe {
                        time: 0.0,
                        value: 100.0,
                        kara_cell: false,
                    }],
                }],
            }
        );
    }

    #[test]
    fn rejects_ae_payload_with_wrong_nonce() {
        let result = parse_nonce_prefixed_payload(
            r#"actual:{"result":"ok","fps":24.0,"compName":"Comp 1","compDuration":1.0,"layers":[]}"#,
            "expected",
        );
        assert!(matches!(result, Err(ParseAePayloadError::NonceMismatch)));
    }

    #[test]
    fn rejects_ae_payload_without_nonce_separator() {
        let result = parse_nonce_prefixed_payload(
            r#"{"result":"ok","fps":24.0,"compName":"Comp 1","compDuration":1.0,"layers":[]}"#,
            "expected",
        );
        assert!(matches!(
            result,
            Err(ParseAePayloadError::MissingNonceSeparator)
        ));
    }

    #[test]
    fn receive_selection_jsx_embeds_port_and_nonce() {
        let session = begin_receive_session(31715);
        let script =
            receive_selection_jsx(&session, AeKaraCellMode::MaxFrameCount, AppLocale::Japanese);
        assert!(script.contains("var NEOSTS_PORT = 31715;"));
        assert!(script.contains(&format!("var NEOSTS_NONCE = \"{}\";", session.nonce)));
        assert!(script.contains("var karaCellMode = \"max_frame_count\";"));
        assert!(
            script
                .contains("var remapFrameIndex = Math.max(0, Math.round(value / frameDuration));")
        );
        assert!(script.contains("\"result\":\"ok\""));
        assert!(script.contains("\"timeRemap\":["));
        assert!(script.contains("\"blinds\":["));
        assert!(script.contains("comp.layer(layerIndex).selected = false;"));
    }

    #[test]
    fn converts_ae_selection_payload_into_sheet() {
        let payload = AeSelectionPayload {
            result: AeSelectionResult::Ok,
            fps: 24.0,
            comp_name: "Comp 1".to_owned(),
            comp_duration: 0.25,
            error: None,
            layers: vec![AeLayerPayload {
                name: "A-1".to_owned(),
                index: 1,
                in_point: 0.0,
                out_point: 0.25,
                source_duration: Some(0.25),
                source_frame_duration: Some(1.0 / 24.0),
                is_comp_layer: false,
                time_remap: vec![
                    AeKeyframe {
                        time: 0.0,
                        value: 0.0,
                        kara_cell: false,
                    },
                    AeKeyframe {
                        time: 2.0 / 24.0,
                        value: 1.0 / 24.0,
                        kara_cell: false,
                    },
                ],
                blinds: vec![AeKeyframe {
                    time: 3.0 / 24.0,
                    value: 100.0,
                    kara_cell: false,
                }],
            }],
        };

        let sheet = selection_payload_to_sheet(&payload, AeKaraCellMode::Blinds).unwrap();
        assert_eq!(sheet.fps(), 24);
        assert_eq!(sheet.column_count(), 1);
        assert_eq!(sheet.row_count(), 6);
        assert_eq!(sheet.column_name(0), "A-1");
        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(0));
    }

    #[test]
    fn rejects_invalid_fps_payloads() {
        let payload = AeSelectionPayload {
            result: AeSelectionResult::Ok,
            fps: 0.0,
            comp_name: "Comp 1".to_owned(),
            comp_duration: 1.0,
            error: None,
            layers: Vec::new(),
        };

        assert!(matches!(
            selection_payload_to_sheet(&payload, AeKaraCellMode::Blinds),
            Err(AePayloadToSheetError::InvalidFps)
        ));
    }

    #[test]
    fn rounds_comp_duration_to_the_nearest_frame() {
        let payload = AeSelectionPayload {
            result: AeSelectionResult::Ok,
            fps: 24.0,
            comp_name: "Comp 10f".to_owned(),
            comp_duration: (10.0 / 24.0) + 1e-9,
            error: None,
            layers: vec![AeLayerPayload {
                name: "A-1".to_owned(),
                index: 1,
                in_point: 0.0,
                out_point: 10.0 / 24.0,
                source_duration: Some(10.0 / 24.0),
                source_frame_duration: Some(1.0 / 24.0),
                is_comp_layer: false,
                time_remap: vec![AeKeyframe {
                    time: 0.0,
                    value: 0.0,
                    kara_cell: false,
                }],
                blinds: Vec::new(),
            }],
        };

        let sheet = selection_payload_to_sheet(&payload, AeKaraCellMode::Blinds).unwrap();
        assert_eq!(sheet.row_count(), 10);
    }

    #[test]
    fn rejects_after_effects_error_payloads() {
        let result = parse_nonce_prefixed_payload(
            r#"abc123:{"result":"error","fps":0,"compName":"error","compDuration":0,"error":"AVLayerを選択してください","layers":[]}"#,
            "abc123",
        );
        assert!(matches!(
            result,
            Err(ParseAePayloadError::AfterEffects(message)) if message == "AVLayerを選択してください"
        ));
    }

    #[test]
    fn converts_script_marked_max_frame_count_kara_cells_into_zero() {
        let payload = AeSelectionPayload {
            result: AeSelectionResult::Ok,
            fps: 24.0,
            comp_name: "Comp 1".to_owned(),
            comp_duration: 0.25,
            error: None,
            layers: vec![AeLayerPayload {
                name: "A-1".to_owned(),
                index: 1,
                in_point: 0.0,
                out_point: 0.25,
                source_duration: Some(0.25),
                source_frame_duration: Some(1.0 / 24.0),
                is_comp_layer: true,
                time_remap: vec![
                    AeKeyframe {
                        time: 0.0,
                        value: 0.0,
                        kara_cell: false,
                    },
                    AeKeyframe {
                        time: 2.0 / 24.0,
                        value: 0.25,
                        kara_cell: true,
                    },
                ],
                blinds: Vec::new(),
            }],
        };

        let sheet = selection_payload_to_sheet(&payload, AeKaraCellMode::MaxFrameCount).unwrap();
        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(0));
    }

    #[test]
    fn keeps_source_duration_value_as_last_cell_when_not_marked_as_kara() {
        let payload = AeSelectionPayload {
            result: AeSelectionResult::Ok,
            fps: 24.0,
            comp_name: "Comp 1".to_owned(),
            comp_duration: 0.25,
            error: None,
            layers: vec![AeLayerPayload {
                name: "A-1".to_owned(),
                index: 1,
                in_point: 0.0,
                out_point: 0.25,
                source_duration: Some(0.25),
                source_frame_duration: Some(1.0 / 24.0),
                is_comp_layer: true,
                time_remap: vec![
                    AeKeyframe {
                        time: 0.0,
                        value: 0.0,
                        kara_cell: false,
                    },
                    AeKeyframe {
                        time: 2.0 / 24.0,
                        value: 0.25,
                        kara_cell: false,
                    },
                ],
                blinds: Vec::new(),
            }],
        };

        let sheet = selection_payload_to_sheet(&payload, AeKaraCellMode::Blinds).unwrap();
        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(7));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(7));
    }

    #[test]
    fn converts_values_past_source_frame_count_into_zero_in_max_frame_count_mode() {
        let payload = AeSelectionPayload {
            result: AeSelectionResult::Ok,
            fps: 24.0,
            comp_name: "Comp 1".to_owned(),
            comp_duration: 0.5,
            error: None,
            layers: vec![AeLayerPayload {
                name: "A-1".to_owned(),
                index: 1,
                in_point: 0.0,
                out_point: 0.5,
                source_duration: Some(10.0 / 24.0),
                source_frame_duration: Some(1.0 / 24.0),
                is_comp_layer: true,
                time_remap: vec![
                    AeKeyframe {
                        time: 0.0,
                        value: 0.0,
                        kara_cell: false,
                    },
                    AeKeyframe {
                        time: 1.0 / 24.0,
                        value: 10.0 / 24.0,
                        kara_cell: false,
                    },
                ],
                blinds: Vec::new(),
            }],
        };

        let sheet = selection_payload_to_sheet(&payload, AeKaraCellMode::MaxFrameCount).unwrap();
        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(0));
    }
}
