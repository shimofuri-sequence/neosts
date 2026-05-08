use crate::{settings::AppLocale, strings};
use serde::Deserialize;
use sheet::{CellValue, Sheet, SheetColumn, SheetError};
use std::{fs, io, path::Path};
use thiserror::Error;

const TDTS_SIGNATURE: &str = "toeiDigitalTimeSheet Save Data";
const NULL_CELL_SYMBOL: &str = "SYMBOL_NULL_CELL";

#[derive(Clone, Debug)]
pub struct TdtsFile {
    time_sheets: Vec<TdtsTimeSheet>,
}

impl TdtsFile {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, TdtsError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| TdtsError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, TdtsError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TdtsError> {
        let text = std::str::from_utf8(bytes).map_err(TdtsError::InvalidUtf8)?;
        Self::from_text(text)
    }

    pub fn from_text(text: &str) -> Result<Self, TdtsError> {
        let json = strip_signature_line(text)?;
        let parsed: TdtsRoot = serde_json::from_str(json).map_err(TdtsError::InvalidJson)?;
        Ok(Self {
            time_sheets: parsed.time_sheets,
        })
    }

    pub fn time_sheets(&self) -> &[TdtsTimeSheet] {
        &self.time_sheets
    }

    pub fn first_sheet_name(&self) -> Option<&str> {
        self.time_sheets
            .first()
            .and_then(|sheet| sheet.time_tables.first())
            .map(|table| table.name.trim())
            .filter(|name| !name.is_empty())
    }

    pub fn to_sheet(&self) -> Result<Sheet, TdtsError> {
        self.to_sheet_with_fps(24)
    }

    pub fn to_sheet_with_fps(&self, fps: u32) -> Result<Sheet, TdtsError> {
        let time_sheet = self.time_sheets.first().ok_or(TdtsError::MissingTimeSheet)?;
        let time_table = time_sheet
            .time_tables
            .first()
            .ok_or(TdtsError::MissingTimeTable)?;

        let duration = time_table.duration;
        let columns = time_table
            .fields
            .iter()
            .flat_map(|field| {
                let header_names = time_table
                    .time_table_headers
                    .iter()
                    .find(|header| header.field_id == field.field_id)
                    .map(|header| header.names.as_slice());

                field.tracks.iter().enumerate().map(move |(track_index, track)| {
                    let name = header_names
                        .and_then(|names| names.get(track_index))
                        .filter(|name| !name.trim().is_empty())
                        .cloned()
                        .unwrap_or_else(|| default_column_name(track.track_no));
                    let values = expand_track(duration, &track.frames)?;
                    Ok(SheetColumn::new(name, values))
                })
            })
            .collect::<Result<Vec<_>, TdtsError>>()?;

        Sheet::try_with_fps(columns, fps).map_err(TdtsError::InvalidSheetModel)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct TdtsRoot {
    #[serde(rename = "timeSheets")]
    time_sheets: Vec<TdtsTimeSheet>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TdtsTimeSheet {
    #[serde(rename = "timeTables", default)]
    time_tables: Vec<TdtsTimeTable>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TdtsTimeTable {
    duration: usize,
    fields: Vec<TdtsField>,
    name: String,
    #[serde(rename = "timeTableHeaders")]
    time_table_headers: Vec<TdtsHeader>,
}

#[derive(Clone, Debug, Deserialize)]
struct TdtsField {
    #[serde(rename = "fieldId")]
    field_id: usize,
    tracks: Vec<TdtsTrack>,
}

#[derive(Clone, Debug, Deserialize)]
struct TdtsTrack {
    #[serde(rename = "trackNo")]
    track_no: usize,
    frames: Vec<TdtsFrame>,
}

#[derive(Clone, Debug, Deserialize)]
struct TdtsFrame {
    frame: usize,
    data: Vec<TdtsFrameData>,
}

#[derive(Clone, Debug, Deserialize)]
struct TdtsFrameData {
    values: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct TdtsHeader {
    #[serde(rename = "fieldId")]
    field_id: usize,
    names: Vec<String>,
}

fn strip_signature_line(text: &str) -> Result<&str, TdtsError> {
    let normalized = text.trim_start_matches('\u{feff}');
    let Some((first_line, rest)) = normalized.split_once('\n') else {
        return Err(TdtsError::MissingSignatureLine);
    };
    if first_line.trim_end_matches('\r').trim() != TDTS_SIGNATURE {
        return Err(TdtsError::InvalidSignatureLine);
    }
    Ok(rest)
}

fn expand_track(duration: usize, frames: &[TdtsFrame]) -> Result<Vec<CellValue>, TdtsError> {
    let mut values = vec![CellValue::blank(); duration];

    for (index, frame) in frames.iter().enumerate() {
        if index > 0 && frames[index - 1].frame >= frame.frame {
            return Err(TdtsError::FrameNotAscending {
                previous: frames[index - 1].frame,
                current: frame.frame,
            });
        }

        let value = parse_frame_value(frame)?;
        if frame.frame > duration {
            if value.is_blank() {
                continue;
            }
            return Err(TdtsError::FrameOutOfRange {
                frame: frame.frame,
                duration,
            });
        }

        let start = frame.frame;
        let end = frames
            .get(index + 1)
            .map(|next| next.frame)
            .unwrap_or(duration)
            .min(duration);
        for item in values.iter_mut().take(end).skip(start) {
            *item = value.clone();
        }
    }

    Ok(values)
}

fn parse_frame_value(frame: &TdtsFrame) -> Result<CellValue, TdtsError> {
    let data = frame
        .data
        .first()
        .ok_or(TdtsError::MissingFrameData { frame: frame.frame })?;
    let raw = data
        .values
        .first()
        .ok_or(TdtsError::MissingFrameValue { frame: frame.frame })?;

    if raw == NULL_CELL_SYMBOL {
        return Ok(CellValue::blank());
    }

    raw.parse::<i64>()
        .map(CellValue::from)
        .map_err(|_| TdtsError::InvalidCellValue {
            frame: frame.frame,
            value: raw.clone(),
        })
}

fn default_column_name(index: usize) -> String {
    let mut index = index + 1;
    let mut name = String::new();

    while index > 0 {
        let rem = (index - 1) % 26;
        name.insert(0, (b'A' + rem as u8) as char);
        index = (index - 1) / 26;
    }

    name
}

#[derive(Debug, Error)]
pub enum TdtsError {
    #[error("failed to read tdts file `{path}`")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read tdts bytes")]
    ReadBytes(#[from] io::Error),
    #[error("tdts file is not valid UTF-8")]
    InvalidUtf8(#[source] std::str::Utf8Error),
    #[error("tdts signature line is missing")]
    MissingSignatureLine,
    #[error("tdts signature line is invalid")]
    InvalidSignatureLine,
    #[error("tdts body is not valid JSON")]
    InvalidJson(#[source] serde_json::Error),
    #[error("tdts contains no time sheets")]
    MissingTimeSheet,
    #[error("tdts contains no time tables")]
    MissingTimeTable,
    #[error("frame {frame} is out of range for duration {duration}")]
    FrameOutOfRange { frame: usize, duration: usize },
    #[error("frame numbers must be strictly ascending: previous={previous}, current={current}")]
    FrameNotAscending { previous: usize, current: usize },
    #[error("frame {frame} is missing data")]
    MissingFrameData { frame: usize },
    #[error("frame {frame} is missing a value")]
    MissingFrameValue { frame: usize },
    #[error("frame {frame} has an invalid cell value `{value}`")]
    InvalidCellValue { frame: usize, value: String },
    #[error("failed to convert tdts to sheet")]
    InvalidSheetModel(#[source] SheetError),
}

impl TdtsError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::ReadFile { path, source } => {
                strings::lowlevel_read_file(locale, "TDTS", path, source)
            }
            Self::ReadBytes(source) => strings::lowlevel_read_bytes(locale, "TDTS", source),
            Self::InvalidUtf8(source) => match locale {
                AppLocale::Japanese => format!("TDTS ファイルは UTF-8 として不正です: {source}"),
                AppLocale::English => format!("TDTS file is not valid UTF-8: {source}"),
            },
            Self::MissingSignatureLine => match locale {
                AppLocale::Japanese => "TDTS のシグネチャ行がありません".to_owned(),
                AppLocale::English => "TDTS signature line is missing".to_owned(),
            },
            Self::InvalidSignatureLine => match locale {
                AppLocale::Japanese => "TDTS のシグネチャ行が不正です".to_owned(),
                AppLocale::English => "TDTS signature line is invalid".to_owned(),
            },
            Self::InvalidJson(source) => match locale {
                AppLocale::Japanese => format!("TDTS 本文の JSON が不正です: {source}"),
                AppLocale::English => format!("TDTS body is not valid JSON: {source}"),
            },
            Self::MissingTimeSheet => match locale {
                AppLocale::Japanese => "TDTS にタイムシートがありません".to_owned(),
                AppLocale::English => "TDTS contains no time sheets".to_owned(),
            },
            Self::MissingTimeTable => match locale {
                AppLocale::Japanese => "TDTS にタイムテーブルがありません".to_owned(),
                AppLocale::English => "TDTS contains no time tables".to_owned(),
            },
            Self::FrameOutOfRange { frame, duration } => match locale {
                AppLocale::Japanese => format!("コマ {frame} は尺 {duration} の範囲外です"),
                AppLocale::English => {
                    format!("Frame {frame} is out of range for duration {duration}")
                }
            },
            Self::FrameNotAscending { previous, current } => match locale {
                AppLocale::Japanese => {
                    format!(
                        "コマ番号は昇順である必要があります: previous={previous}, current={current}"
                    )
                }
                AppLocale::English => {
                    format!(
                        "Frame numbers must be strictly ascending: previous={previous}, current={current}"
                    )
                }
            },
            Self::MissingFrameData { frame } => match locale {
                AppLocale::Japanese => format!("コマ {frame} にデータがありません"),
                AppLocale::English => format!("Frame {frame} is missing data"),
            },
            Self::MissingFrameValue { frame } => match locale {
                AppLocale::Japanese => format!("コマ {frame} に値がありません"),
                AppLocale::English => format!("Frame {frame} is missing a value"),
            },
            Self::InvalidCellValue { frame, value } => match locale {
                AppLocale::Japanese => format!("コマ {frame} のセル値 `{value}` が不正です"),
                AppLocale::English => format!("Frame {frame} has an invalid cell value `{value}`"),
            },
            Self::InvalidSheetModel(source) => {
                strings::lowlevel_invalid_sheet_model(locale, "TDTS", source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TdtsFile;
    use crate::test_fixtures::fixture_path;

    #[test]
    fn parses_sample_tdts_into_sheet() {
        let Some(path) = fixture_path("tdts_test1.tdts") else {
            eprintln!("skipping fixture-based test: tests/fixtures/tdts_test1.tdts is missing");
            return;
        };
        let tdts = TdtsFile::from_path(&path).expect("sample tdts should parse");

        assert_eq!(tdts.time_sheets().len(), 1);
        assert_eq!(tdts.first_sheet_name(), Some("sheet1"));

        let sheet = tdts.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.column_count(), 1);
        assert_eq!(sheet.row_count(), 144);
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(0, 2).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(0, 3).map(|value| value.as_i64()), Some(2));
        assert_eq!(sheet.cell(0, 27).map(|value| value.as_i64()), Some(10));
    }

    #[test]
    fn accepts_utf8_bom_prefixed_tdts() {
        let tdts = TdtsFile::from_text(
            "\u{feff}toeiDigitalTimeSheet Save Data\n\
             {\n\
               \"timeSheets\": [\n\
                 {\n\
                   \"timeTables\": [\n\
                     {\n\
                       \"name\": \"シートA\",\n\
                       \"duration\": 2,\n\
                       \"fields\": [\n\
                         {\n\
                           \"fieldId\": 0,\n\
                           \"tracks\": [\n\
                             {\n\
                               \"trackNo\": 0,\n\
                               \"frames\": [\n\
                                 { \"frame\": 0, \"data\": [{ \"values\": [\"1\"] }] },\n\
                                 { \"frame\": 1, \"data\": [{ \"values\": [\"2\"] }] }\n\
                               ]\n\
                             }\n\
                           ]\n\
                         }\n\
                       ],\n\
                       \"timeTableHeaders\": [{ \"fieldId\": 0, \"names\": [\"A\"] }]\n\
                     }\n\
                   ]\n\
                 }\n\
               ]\n\
             }",
        )
        .expect("bom-prefixed tdts should parse");

        assert_eq!(tdts.first_sheet_name(), Some("シートA"));
        let sheet = tdts.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(0, 1).map(|value| value.as_i64()), Some(2));
    }
}
