use super::platform::{SendToAfterEffectsError, launch_after_effects_script};
use super::{javascript_string_literal, write_temp_jsx};
use crate::{AeKaraCellMode, AeKeyframeDataLocale, AppLocale};
use sheet::CellValue;

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedKeyframeData {
    pub start_frame: usize,
    pub units_per_second: f64,
    pub values: Vec<CellValue>,
}

#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum ParseKeyframeDataError {
    #[error("text is not Adobe After Effects keyframe data")]
    InvalidHeader,
    #[error("missing Units Per Second")]
    MissingUnitsPerSecond,
    #[error("invalid Units Per Second value")]
    InvalidUnitsPerSecond,
    #[error("missing Time Remap section")]
    MissingTimeRemapSection,
    #[error("missing Time Remap keyframes")]
    MissingTimeRemapKeyframes,
    #[error("invalid frame number in {section} section")]
    InvalidFrame { section: &'static str },
    #[error("invalid numeric value in {section} section")]
    InvalidNumber { section: &'static str },
    #[error("duplicate frame {frame} in {section} section")]
    DuplicateFrame { section: &'static str, frame: usize },
}

impl ParseKeyframeDataError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::InvalidHeader => match locale {
                AppLocale::Japanese => {
                    "Adobe After Effects のキーフレームデータ形式ではありません".to_owned()
                }
                AppLocale::English => "Text is not Adobe After Effects keyframe data".to_owned(),
            },
            Self::MissingUnitsPerSecond => match locale {
                AppLocale::Japanese => "`Units Per Second` がありません".to_owned(),
                AppLocale::English => "Missing `Units Per Second`".to_owned(),
            },
            Self::InvalidUnitsPerSecond => match locale {
                AppLocale::Japanese => "`Units Per Second` の値が不正です".to_owned(),
                AppLocale::English => "Invalid `Units Per Second` value".to_owned(),
            },
            Self::MissingTimeRemapSection => match locale {
                AppLocale::Japanese => "`Time Remap` セクションがありません".to_owned(),
                AppLocale::English => "Missing `Time Remap` section".to_owned(),
            },
            Self::MissingTimeRemapKeyframes => match locale {
                AppLocale::Japanese => "`Time Remap` のキーフレームがありません".to_owned(),
                AppLocale::English => "Missing `Time Remap` keyframes".to_owned(),
            },
            Self::InvalidFrame { section } => match locale {
                AppLocale::Japanese => format!("`{section}` セクションのコマ番号が不正です"),
                AppLocale::English => format!("Invalid frame number in `{section}` section"),
            },
            Self::InvalidNumber { section } => match locale {
                AppLocale::Japanese => format!("`{section}` セクションの数値が不正です"),
                AppLocale::English => format!("Invalid numeric value in `{section}` section"),
            },
            Self::DuplicateFrame { section, frame } => match locale {
                AppLocale::Japanese => {
                    format!("`{section}` セクションに重複したコマ {frame} があります")
                }
                AppLocale::English => {
                    format!("Duplicate frame {frame} in `{section}` section")
                }
            },
        }
    }
}

pub fn keyframe_data(
    _column_name: &str,
    values: &[CellValue],
    fps: u32,
    keyframe_version: &str,
    locale: AeKeyframeDataLocale,
    kara_cell_mode: AeKaraCellMode,
    kara_cell_value: i64,
) -> String {
    let fps = fps.max(1);
    let mut last_value: Option<i64> = None;
    let mut has_karacel = false;
    let mut output = String::new();

    output.push_str(&format!(
        "Adobe After Effects {keyframe_version} Keyframe Data\r\n"
    ));
    output.push_str("\r\n");
    output.push_str(&format!("\tUnits Per Second\t{fps}\r\n"));
    output.push_str("\tSource Width\t640\r\n");
    output.push_str("\tSource Height\t480\r\n");
    output.push_str("\tSource Pixel Aspect Ratio\t1\r\n");
    output.push_str("\tComp Pixel Aspect Ratio\t1\r\n");
    output.push_str("\r\n\r\n");
    output.push_str("Time Remap\r\n");
    output.push_str("\tFrame\tseconds\t\r\n");

    for (frame, value) in values.iter().enumerate() {
        let value = numeric_cell_value(value);
        if is_karacel_value(&values[frame], kara_cell_value) {
            has_karacel = true;
        }
        if last_value != Some(value) {
            let remap_time = remap_time_for_cell_value(
                value,
                fps,
                values.len(),
                kara_cell_mode,
                kara_cell_value,
            );
            output.push_str(&format!("\t{frame}\t{remap_time:.7}\t\r\n"));
            last_value = Some(value);
        }
    }

    output.push_str("\r\n");
    if has_karacel && kara_cell_mode == AeKaraCellMode::Blinds {
        let mut last_opacity: Option<i64> = None;
        let (blind_name, completion_name, percent_label) = match locale {
            AeKeyframeDataLocale::Japanese => ("ブラインド #1", "変換終了 #2", "パーセント"),
            AeKeyframeDataLocale::English => {
                ("Venetian Blinds #1", "Transition Completion #2", "percent")
            }
        };
        output.push_str(&format!("Effects\t{blind_name}\t{completion_name}\t\r\n"));
        output.push_str(&format!("\tFrame\t{percent_label}\r\n"));

        for (frame, value) in values.iter().enumerate() {
            let opacity = if is_karacel_value(value, kara_cell_value) {
                100
            } else {
                0
            };
            if last_opacity != Some(opacity) {
                output.push_str(&format!("\t{frame}\t{opacity}\t\r\n"));
                last_opacity = Some(opacity);
            }
        }
    }

    output.push_str("\r\n\r\nEnd of Keyframe Data\r\n\r\n");
    output
}

pub fn jsx_script(
    column_name: &str,
    values: &[CellValue],
    fps: u32,
    kara_cell_mode: AeKaraCellMode,
) -> String {
    let fps = fps.max(1);
    let mut values_js = String::new();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            values_js.push_str(", ");
        }
        values_js.push_str(&value.as_i64().to_string());
    }
    let column_name = javascript_string_literal(column_name);
    let kara_cell_mode = match kara_cell_mode {
        AeKaraCellMode::Blinds => "blinds",
        AeKaraCellMode::MaxFrameCount => "max_frame_count",
    };

    format!(
        r#"(function() {{
var columnName = "{column_name}";
var sheetData = [{values_js}];
var sourceFps = {fps};
var karaCellMode = "{kara_cell_mode}";

function escapeRegExp(text) {{
    return text.replace(/[.*+?^${{}}()|[\]\\]/g, '\\$&');
}}

function precompNameForLayer(layerName) {{
    var cleaned = layerName.replace(/(?:_?[\[\{{]\d+-\d+[\]\}}]|[A-Za-z]?_?\d+)(\.[^.\s]+)?$/i, "");
    if (!cleaned) {{
        cleaned = layerName;
    }}
    return cleaned;
}}

function prepareLayerForKaraCellMode(layer) {{
    if (karaCellMode !== "max_frame_count") {{
        return layer;
    }}
    if (layer.source instanceof CompItem) {{
        return layer;
    }}

    var comp = layer.containingComp;
    var layerIndex = layer.index;
    var precompName = precompNameForLayer(layer.name);
    comp.layers.precompose([layerIndex], precompName, false);
    return comp.layer(layerIndex);
}}

function applySheetCel(layer, sheetArray, fps) {{
    layer = prepareLayerForKaraCellMode(layer);
    var isStill = false;
    if (layer.source instanceof FootageItem && layer.source.mainSource.isStill) {{
        isStill = true;
    }}

    var compFps = 1 / layer.containingComp.frameDuration;
    var layerFps = (isStill)
        ? 1 / layer.containingComp.frameDuration
        : 1 / layer.source.frameDuration;

    var hasKaracel = false;
    var celDst = null;
    var opacityDst = null;
    var celTimes = [];
    var celValues = [];
    var opacityTimes = [];
    var opacityValues = [];
    var useBlinds = karaCellMode === "blinds";
    var sourceMaxFrame = Math.max(0, Math.round(layer.source.duration * layerFps));
    var sourceMaxTime = sourceMaxFrame / layerFps;

    for (var i = 0; i < sheetArray.length; ++i) {{
        var val = sheetArray[i];
        if (val === 0) {{
            hasKaracel = true;
        }}

        var timeValue = sourceMaxTime;
        if (val > 0) {{
            var celFrame = Math.max(0, Math.round(val) - 1);
            timeValue = celFrame / layerFps;
        }} else if (useBlinds) {{
            timeValue = 0;
        }}
        var opacityValue = useBlinds && val === 0 ? 100 : 0;

        if (celDst !== val) {{
            timeValue = Math.min(sourceMaxTime, timeValue);
            celTimes.push(i / compFps);
            celValues.push(timeValue);
            celDst = val;
        }}

        if (useBlinds && opacityDst !== opacityValue) {{
            opacityTimes.push(i / compFps);
            opacityValues.push(opacityValue);
            opacityDst = opacityValue;
        }}
    }}

    if (!isStill) {{
        var remap = layer.property("ADBE Time Remapping");
        layer.timeRemapEnabled = false;
        layer.timeRemapEnabled = true;
        if (remap.numKeys >= 2) {{
            remap.removeKey(2);
        }}
        remap.setValuesAtTimes(celTimes, celValues);
        for (var keyIndex = 1; keyIndex <= remap.numKeys; ++keyIndex) {{
            remap.setInterpolationTypeAtKey(
                keyIndex,
                KeyframeInterpolationType.HOLD,
                KeyframeInterpolationType.HOLD
            );
        }}
    }}

    layer.outPoint = sheetArray.length / compFps;

    var effectIndex = layer.effect.numProperties;
    while (effectIndex > 0) {{
        if (layer.effect(effectIndex).matchName === "ADBE Venetian Blinds") {{
            layer.effect(effectIndex).remove();
        }}
        --effectIndex;
    }}

    if (!hasKaracel || !useBlinds) {{
        return;
    }}

    var blindEffect = layer.effect.addProperty("ADBE Venetian Blinds");
    blindEffect(1).setValuesAtTimes(opacityTimes, opacityValues);
    for (var blindKeyIndex = 1; blindKeyIndex <= blindEffect(1).numKeys; ++blindKeyIndex) {{
        blindEffect(1).setInterpolationTypeAtKey(
            blindKeyIndex,
            KeyframeInterpolationType.HOLD,
            KeyframeInterpolationType.HOLD
        );
    }}
}}

function collectTargetLayers(comp, columnName) {{
    if (comp.selectedLayers.length > 0) {{
        var selected = [];
        for (var selectedIndex = 0; selectedIndex < comp.selectedLayers.length; ++selectedIndex) {{
            var selectedLayer = comp.selectedLayers[selectedIndex];
            if (selectedLayer instanceof AVLayer) {{
                selected.push(selectedLayer);
            }}
        }}
        return selected;
    }}

    var pattern = new RegExp("^[_\\s]*" + escapeRegExp(columnName) + "$", "i");
    var matched = [];
    for (var layerIndex = 1; layerIndex <= comp.numLayers; ++layerIndex) {{
        var layer = comp.layer(layerIndex);
        if ((layer instanceof AVLayer) && pattern.test(layer.name)) {{
            matched.push(layer);
        }}
    }}
    return matched;
}}

function clearSelectedLayers(comp) {{
    for (var layerIndex = 1; layerIndex <= comp.numLayers; ++layerIndex) {{
        comp.layer(layerIndex).selected = false;
    }}
}}

app.beginUndoGroup("NeoSTS Apply Sheet");
try {{
    if (!(app.project && app.project.activeItem instanceof CompItem)) {{
        throw new Error("Please open a composition in After Effects.");
    }}

    var comp = app.project.activeItem;
    if (!(comp instanceof CompItem)) {{
        throw new Error("Please open a composition in After Effects.");
    }}
    var compFps = 1 / comp.frameDuration;
    if (Math.abs(compFps - sourceFps) > 0.0001) {{
        alert(
            "NeoSTS: Sheet fps (" + sourceFps + ") and comp fps (" + compFps + ") differ.\n" +
            "Keyframe timing will follow the comp fps."
        );
    }}
    var targetLayers = collectTargetLayers(comp, columnName);
    if (!targetLayers.length) {{
        throw new Error("Select a layer or create one named '" + columnName + "'.");
    }}

    for (var i = 0; i < targetLayers.length; ++i) {{
        applySheetCel(targetLayers[i], sheetData, sourceFps);
    }}
    clearSelectedLayers(comp);
}} catch (error) {{
    alert("NeoSTS: " + error.toString());
}} finally {{
    app.endUndoGroup();
}}
}})();
"#
    )
}

pub fn send_column_to_after_effects(
    column_name: &str,
    values: &[CellValue],
    fps: u32,
    kara_cell_mode: AeKaraCellMode,
    owner_hwnd: Option<isize>,
) -> Result<std::path::PathBuf, SendToAfterEffectsError> {
    let script = jsx_script(column_name, values, fps, kara_cell_mode);
    let script_path = write_temp_jsx(&script).map_err(SendToAfterEffectsError::WriteScript)?;
    launch_after_effects_script(&script_path, owner_hwnd)?;
    Ok(script_path)
}

pub fn is_keyframe_data_text(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("Adobe After Effects ") else {
        return false;
    };
    let Some((version, line_end)) = rest.split_once(" Keyframe Data") else {
        return false;
    };

    !version.is_empty()
        && version.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        && matches!(line_end, "\r\n" | "\n" | "\r" | _ if line_end.starts_with("\r\n") || line_end.starts_with('\n') || line_end.starts_with('\r'))
}

pub fn parse_keyframe_data(text: &str) -> Result<ParsedKeyframeData, ParseKeyframeDataError> {
    if !is_keyframe_data_text(text) {
        return Err(ParseKeyframeDataError::InvalidHeader);
    }

    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    let units_per_second = find_units_per_second(&lines)?;
    let time_remap_index = lines
        .iter()
        .position(|line| line.trim() == "Time Remap")
        .ok_or(ParseKeyframeDataError::MissingTimeRemapSection)?;
    let effect_index = lines
        .iter()
        .position(|line| line.trim_start().starts_with("Effects"));
    let time_remap_end = effect_index.unwrap_or(lines.len());

    let time_remap_keyframes =
        parse_section_keyframes(&lines[time_remap_index + 1..time_remap_end], "Time Remap")?;
    if time_remap_keyframes.is_empty() {
        return Err(ParseKeyframeDataError::MissingTimeRemapKeyframes);
    }

    let opacity_keyframes = if let Some(effect_index) = effect_index {
        if is_blinds_effect_header(lines[effect_index]) {
            parse_section_keyframes(&lines[effect_index + 1..], "Opacity")?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let frame_offset = time_remap_keyframes
        .first()
        .map(|(frame, _)| *frame)
        .unwrap_or(0);
    let time_remap_keyframes: Vec<(usize, f64)> = time_remap_keyframes
        .into_iter()
        .map(|(frame, value)| (frame.saturating_sub(frame_offset), value))
        .collect();
    let opacity_keyframes: Vec<(usize, f64)> = opacity_keyframes
        .into_iter()
        .filter(|(frame, _)| *frame >= frame_offset)
        .map(|(frame, value)| (frame - frame_offset, value))
        .collect();

    let last_frame = time_remap_keyframes
        .last()
        .map(|(frame, _)| *frame)
        .unwrap_or(0)
        .max(
            opacity_keyframes
                .last()
                .map(|(frame, _)| *frame)
                .unwrap_or(0),
        );
    let mut values = vec![CellValue::blank(); last_frame + 1];

    for (index, (frame, seconds)) in time_remap_keyframes.iter().enumerate() {
        let next_frame = time_remap_keyframes
            .get(index + 1)
            .map(|(next_frame, _)| *next_frame)
            .unwrap_or(last_frame + 1);
        let value = remap_seconds_to_cell_value(*seconds, units_per_second);
        for cell in &mut values[*frame..next_frame] {
            *cell = CellValue::Int(value);
        }
    }

    for (index, (frame, percent)) in opacity_keyframes.iter().enumerate() {
        let next_frame = opacity_keyframes
            .get(index + 1)
            .map(|(next_frame, _)| *next_frame)
            .unwrap_or(last_frame + 1);
        if *percent != 0.0 {
            for cell in &mut values[*frame..next_frame] {
                *cell = CellValue::Int(0);
            }
        }
    }

    Ok(ParsedKeyframeData {
        start_frame: frame_offset,
        units_per_second,
        values,
    })
}

fn numeric_cell_value(value: &CellValue) -> i64 {
    value.as_i64().max(0)
}

fn is_karacel_value(value: &CellValue, kara_cell_value: i64) -> bool {
    value.as_i64() == kara_cell_value
}

fn remap_time_for_cell_value(
    value: i64,
    fps: u32,
    frame_count: usize,
    kara_cell_mode: AeKaraCellMode,
    kara_cell_value: i64,
) -> f64 {
    if value != kara_cell_value {
        return (value - 1) as f64 / fps as f64;
    }

    match kara_cell_mode {
        AeKaraCellMode::Blinds => value as f64 / fps as f64,
        AeKaraCellMode::MaxFrameCount => frame_count as f64 / fps as f64,
    }
}

fn find_units_per_second(lines: &[&str]) -> Result<f64, ParseKeyframeDataError> {
    let value = lines
        .iter()
        .find_map(|line| {
            let columns = split_nonempty_columns(line);
            (columns.first().copied() == Some("Units Per Second")
                || columns.first().copied() == Some("Units Per Seconds"))
            .then(|| columns.get(1).copied())
            .flatten()
        })
        .ok_or(ParseKeyframeDataError::MissingUnitsPerSecond)?;
    let units_per_second = value
        .parse::<f64>()
        .map_err(|_| ParseKeyframeDataError::InvalidUnitsPerSecond)?;
    if !units_per_second.is_finite() || units_per_second <= 0.0 {
        return Err(ParseKeyframeDataError::InvalidUnitsPerSecond);
    }
    Ok(units_per_second)
}

fn is_blinds_effect_header(line: &str) -> bool {
    let columns = split_nonempty_columns(line);
    if columns.first().copied() != Some("Effects") {
        return false;
    }

    let has_japanese_blinds = columns.iter().any(|column| column.contains("ブラインド"))
        && columns.iter().any(|column| column.contains("変換終了"));
    let has_english_blinds = columns
        .iter()
        .any(|column| column.contains("Venetian Blinds"))
        && columns
            .iter()
            .any(|column| column.contains("Transition Completion"));

    has_japanese_blinds || has_english_blinds
}

fn parse_section_keyframes(
    lines: &[&str],
    section: &'static str,
) -> Result<Vec<(usize, f64)>, ParseKeyframeDataError> {
    let mut keyframes = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !keyframes.is_empty() {
                break;
            }
            continue;
        }
        if trimmed == "End of Keyframe Data" {
            break;
        }

        let columns = split_nonempty_columns(line);
        if columns.len() < 2 {
            continue;
        }
        if columns[0].eq_ignore_ascii_case("frame") {
            continue;
        }

        let frame = columns[0]
            .parse::<usize>()
            .map_err(|_| ParseKeyframeDataError::InvalidFrame { section })?;
        let value = columns[1]
            .parse::<f64>()
            .map_err(|_| ParseKeyframeDataError::InvalidNumber { section })?;
        if keyframes
            .last()
            .is_some_and(|(previous_frame, _)| *previous_frame == frame)
        {
            return Err(ParseKeyframeDataError::DuplicateFrame { section, frame });
        }
        keyframes.push((frame, value));
    }

    Ok(keyframes)
}

fn split_nonempty_columns(line: &str) -> Vec<&str> {
    line.split('\t')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .collect()
}

fn remap_seconds_to_cell_value(seconds: f64, units_per_second: f64) -> i64 {
    ((seconds * units_per_second).round() as i64 + 1).max(1)
}

#[cfg(test)]
mod tests {
    use super::{
        is_keyframe_data_text, javascript_string_literal, jsx_script, keyframe_data,
        parse_keyframe_data,
    };
    use crate::{AeKaraCellMode, AeKeyframeDataLocale};
    use sheet::CellValue;

    #[test]
    fn accepts_after_effects_keyframe_headers() {
        assert!(is_keyframe_data_text(
            "Adobe After Effects 9.0 Keyframe Data\r\n\r\n"
        ));
        assert!(is_keyframe_data_text(
            "Adobe After Effects 7.0 Keyframe Data\r\nLayer\r\n"
        ));
        assert!(is_keyframe_data_text(
            "Adobe After Effects 4.0 Keyframe Data\n"
        ));
    }

    #[test]
    fn rejects_non_keyframe_headers() {
        assert!(!is_keyframe_data_text(
            "Adobe After Effects Keyframe Data\r\n"
        ));
        assert!(!is_keyframe_data_text(
            "Adobe After Effects nine Keyframe Data\r\n"
        ));
        assert!(!is_keyframe_data_text("Something else entirely"));
    }

    #[test]
    fn parses_time_remap_into_column_values() {
        let parsed = parse_keyframe_data(
            "Adobe After Effects 9.0 Keyframe Data\r\n\
             \r\n\
             \tUnits Per Second\t24\r\n\
             \tSource Width\t640\r\n\
             \tSource Height\t480\r\n\
             \tSource Pixel Aspect Ratio\t1\r\n\
             \tComp Pixel Aspect Ratio\t1\r\n\
             \r\n\
             Time Remap\r\n\
             \tFrame\tseconds\t\r\n\
             \t0\t0.0000000\t\r\n\
             \t3\t0.0833333\t\r\n\
             \t5\t0.1666667\t\r\n\
             \r\n\
             End of Keyframe Data\r\n",
        )
        .unwrap();

        assert_eq!(parsed.start_frame, 0);
        assert_eq!(parsed.units_per_second, 24.0);
        assert_eq!(
            parsed.values,
            vec![
                CellValue::Int(1),
                CellValue::Int(1),
                CellValue::Int(1),
                CellValue::Int(3),
                CellValue::Int(3),
                CellValue::Int(5),
            ]
        );
    }

    #[test]
    fn parses_opacity_section_as_kara_cells() {
        let parsed = parse_keyframe_data(
            "Adobe After Effects 7.0 Keyframe Data\r\n\
             \r\n\
             \tUnits Per Second\t24\r\n\
             \r\n\
             Time Remap\r\n\
             \tFrame\tseconds\t\r\n\
             \t0\t0.0000000\t\r\n\
             \t2\t0.0416667\t\r\n\
             \r\n\
             Effects\tブラインド #1\t変換終了 #2\t\r\n\
             \tFrame\tパーセント\r\n\
             \t0\t0\t\r\n\
             \t1\t100\t\r\n\
             \t2\t0\t\r\n\
             \r\n\
             End of Keyframe Data\r\n",
        )
        .unwrap();

        assert_eq!(parsed.start_frame, 0);
        assert_eq!(
            parsed.values,
            vec![CellValue::Int(1), CellValue::Int(0), CellValue::Int(2)]
        );
    }

    #[test]
    fn parses_english_opacity_section_as_kara_cells() {
        let parsed = parse_keyframe_data(
            "Adobe After Effects 9.0 Keyframe Data\r\n\
             \r\n\
             \tUnits Per Second\t24\r\n\
             \r\n\
             Time Remap\r\n\
             \tFrame\tseconds\t\r\n\
             \t0\t0.0000000\t\r\n\
             \t2\t0.0416667\t\r\n\
             \r\n\
             Effects\tVenetian Blinds #1\tTransition Completion #2\t\r\n\
             \tFrame\tpercent\t\r\n\
             \t0\t0\t\r\n\
             \t1\t100\t\r\n\
             \t2\t0\t\r\n\
             \r\n\
             End of Keyframe Data\r\n",
        )
        .unwrap();

        assert_eq!(parsed.start_frame, 0);
        assert_eq!(
            parsed.values,
            vec![CellValue::Int(1), CellValue::Int(0), CellValue::Int(2)]
        );
    }

    #[test]
    fn ignores_non_blinds_effects_section_for_kara_cells() {
        let parsed = parse_keyframe_data(
            "Adobe After Effects 9.0 Keyframe Data\r\n\
             \r\n\
             \tUnits Per Second\t24\r\n\
             \r\n\
             Time Remap\r\n\
             \tFrame\tseconds\t\r\n\
             \t0\t0.0000000\t\r\n\
             \t2\t0.0416667\t\r\n\
             \r\n\
             Effects\tGlow #1\tGlow Radius #2\t\r\n\
             \tFrame\tpercent\t\r\n\
             \t0\t0\t\r\n\
             \t1\t100\t\r\n\
             \t2\t0\t\r\n\
             \r\n\
             End of Keyframe Data\r\n",
        )
        .unwrap();

        assert_eq!(parsed.start_frame, 0);
        assert_eq!(
            parsed.values,
            vec![CellValue::Int(1), CellValue::Int(1), CellValue::Int(2)]
        );
    }

    #[test]
    fn emits_english_blinds_section_for_english_locale() {
        let text = keyframe_data(
            "A-1",
            &[CellValue::Int(0), CellValue::Int(1), CellValue::Int(1)],
            24,
            "9.0",
            AeKeyframeDataLocale::English,
            AeKaraCellMode::Blinds,
            0,
        );

        assert!(text.contains("Effects\tVenetian Blinds #1\tTransition Completion #2\t"));
        assert!(text.contains("\tFrame\tpercent\r\n"));
    }

    #[test]
    fn omits_blinds_section_for_max_frame_count_mode() {
        let text = keyframe_data(
            "A-1",
            &[CellValue::Int(100), CellValue::Int(1), CellValue::Int(1)],
            24,
            "9.0",
            AeKeyframeDataLocale::Japanese,
            AeKaraCellMode::MaxFrameCount,
            100,
        );

        assert!(!text.contains("Effects\t"));
        assert!(text.contains("\t0\t0.1250000\t\r\n"));
    }

    #[test]
    fn normalizes_nonzero_start_frame_to_zero() {
        let parsed = parse_keyframe_data(
            "Adobe After Effects 9.0 Keyframe Data\r\n\
             \r\n\
             \tUnits Per Second\t24\r\n\
             \r\n\
             Time Remap\r\n\
             \tFrame\tseconds\t\r\n\
             \t12\t0.125\t\r\n\
             \t16\t0.166667\t\r\n\
             \r\n\
             End of Keyframe Data\r\n",
        )
        .unwrap();

        assert_eq!(parsed.start_frame, 12);
        assert_eq!(
            parsed.values,
            vec![
                CellValue::Int(4),
                CellValue::Int(4),
                CellValue::Int(4),
                CellValue::Int(4),
                CellValue::Int(5),
            ]
        );
    }

    #[test]
    fn escapes_javascript_string_literals_as_ascii() {
        assert_eq!(
            javascript_string_literal("A\"B\\C\nあ"),
            "A\\\"B\\\\C\\n\\u3042"
        );
    }

    #[test]
    fn jsx_script_embeds_column_name_and_values() {
        let script = jsx_script(
            "動画A",
            &[1.into(), 0.into(), CellValue::blank()],
            24,
            AeKaraCellMode::Blinds,
        );

        assert!(script.contains("var columnName = \"\\u52D5\\u753BA\";"));
        assert!(script.contains("var sheetData = [1, 0, 0];"));
        assert!(script.contains("var sourceFps = 24;"));
        assert!(script.contains(
            "var sourceMaxFrame = Math.max(0, Math.round(layer.source.duration * layerFps));"
        ));
        assert!(script.contains(
            "layerName.replace(/(?:_?[\\[\\{]\\d+-\\d+[\\]\\}]|[A-Za-z]?_?\\d+)(\\.[^.\\s]+)?$/i, \"\")"
        ));
        assert!(script.contains("comp.layer(layerIndex).selected = false;"));
    }
}
