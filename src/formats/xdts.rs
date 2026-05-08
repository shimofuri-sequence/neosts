use crate::{settings::AppLocale, strings};
use serde::Deserialize;
use sheet::{CellValue, Sheet, SheetColumn, SheetError};
use std::{fs, io, path::Path};
use thiserror::Error;

const XDTS_SIGNATURE: &str = "exchangeDigitalTimeSheet Save Data";
const NULL_CELL_SYMBOL: &str = "SYMBOL_NULL_CELL";

#[derive(Clone, Debug)]
pub struct XdtsFile {
    time_tables: Vec<XdtsTimeTable>,
}

impl XdtsFile {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, XdtsError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| XdtsError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, XdtsError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, XdtsError> {
        let text = std::str::from_utf8(bytes).map_err(XdtsError::InvalidUtf8)?;
        Self::from_text(text)
    }

    pub fn from_text(text: &str) -> Result<Self, XdtsError> {
        let json = strip_signature_line(text)?;
        let parsed: XdtsRoot = serde_json::from_str(json).map_err(XdtsError::InvalidJson)?;
        Ok(Self {
            time_tables: parsed.time_tables,
        })
    }

    pub fn time_tables(&self) -> &[XdtsTimeTable] {
        &self.time_tables
    }

    pub fn to_sheet(&self) -> Result<Sheet, XdtsError> {
        self.to_sheet_with_fps(24)
    }

    pub fn to_sheet_with_fps(&self, fps: u32) -> Result<Sheet, XdtsError> {
        let time_table = self
            .time_tables
            .first()
            .ok_or(XdtsError::MissingTimeTable)?;

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

                field
                    .tracks
                    .iter()
                    .enumerate()
                    .map(move |(track_index, track)| {
                        let name = header_names
                            .and_then(|names| names.get(track_index))
                            .filter(|name| !name.trim().is_empty())
                            .cloned()
                            .unwrap_or_else(|| default_column_name(track.track_no));
                        let values = expand_track(duration, &track.frames)?;
                        Ok(SheetColumn::new(name, values))
                    })
            })
            .collect::<Result<Vec<_>, XdtsError>>()?;

        Sheet::try_with_fps(columns, fps).map_err(XdtsError::InvalidSheetModel)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct XdtsRoot {
    #[serde(rename = "timeTables")]
    time_tables: Vec<XdtsTimeTable>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct XdtsTimeTable {
    duration: usize,
    fields: Vec<XdtsField>,
    #[serde(rename = "timeTableHeaders")]
    time_table_headers: Vec<XdtsHeader>,
}

#[derive(Clone, Debug, Deserialize)]
struct XdtsField {
    #[serde(rename = "fieldId")]
    field_id: usize,
    tracks: Vec<XdtsTrack>,
}

#[derive(Clone, Debug, Deserialize)]
struct XdtsTrack {
    #[serde(rename = "trackNo")]
    track_no: usize,
    frames: Vec<XdtsFrame>,
}

#[derive(Clone, Debug, Deserialize)]
struct XdtsFrame {
    frame: usize,
    data: Vec<XdtsFrameData>,
}

#[derive(Clone, Debug, Deserialize)]
struct XdtsFrameData {
    values: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct XdtsHeader {
    #[serde(rename = "fieldId")]
    field_id: usize,
    names: Vec<String>,
}

fn strip_signature_line(text: &str) -> Result<&str, XdtsError> {
    let normalized = text.trim_start_matches('\u{feff}');
    let Some((first_line, rest)) = normalized.split_once('\n') else {
        return Err(XdtsError::MissingSignatureLine);
    };
    if first_line.trim_end_matches('\r').trim() != XDTS_SIGNATURE {
        return Err(XdtsError::InvalidSignatureLine);
    }
    Ok(rest)
}

fn expand_track(duration: usize, frames: &[XdtsFrame]) -> Result<Vec<CellValue>, XdtsError> {
    let mut values = vec![CellValue::blank(); duration];

    for (index, frame) in frames.iter().enumerate() {
        if index > 0 && frames[index - 1].frame >= frame.frame {
            return Err(XdtsError::FrameNotAscending {
                previous: frames[index - 1].frame,
                current: frame.frame,
            });
        }

        let value = parse_frame_value(frame)?;
        if frame.frame > duration {
            if value.is_blank() {
                continue;
            }
            return Err(XdtsError::FrameOutOfRange {
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

fn parse_frame_value(frame: &XdtsFrame) -> Result<CellValue, XdtsError> {
    let data = frame
        .data
        .first()
        .ok_or(XdtsError::MissingFrameData { frame: frame.frame })?;
    let raw = data
        .values
        .first()
        .ok_or(XdtsError::MissingFrameValue { frame: frame.frame })?;

    if raw == NULL_CELL_SYMBOL {
        return Ok(CellValue::blank());
    }

    raw.parse::<i64>()
        .map(CellValue::from)
        .map_err(|_| XdtsError::InvalidCellValue {
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
pub enum XdtsError {
    #[error("failed to read xdts file `{path}`")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read xdts bytes")]
    ReadBytes(#[from] io::Error),
    #[error("xdts file is not valid UTF-8")]
    InvalidUtf8(#[source] std::str::Utf8Error),
    #[error("xdts signature line is missing")]
    MissingSignatureLine,
    #[error("xdts signature line is invalid")]
    InvalidSignatureLine,
    #[error("xdts body is not valid JSON")]
    InvalidJson(#[source] serde_json::Error),
    #[error("xdts contains no time tables")]
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
    #[error("failed to convert xdts to sheet")]
    InvalidSheetModel(#[source] SheetError),
}

impl XdtsError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::ReadFile { path, source } => {
                strings::lowlevel_read_file(locale, "XDTS", path, source)
            }
            Self::ReadBytes(source) => strings::lowlevel_read_bytes(locale, "XDTS", source),
            Self::InvalidUtf8(source) => match locale {
                AppLocale::Japanese => format!("XDTS ファイルは UTF-8 として不正です: {source}"),
                AppLocale::English => format!("XDTS file is not valid UTF-8: {source}"),
            },
            Self::MissingSignatureLine => match locale {
                AppLocale::Japanese => "XDTS のシグネチャ行がありません".to_owned(),
                AppLocale::English => "XDTS signature line is missing".to_owned(),
            },
            Self::InvalidSignatureLine => match locale {
                AppLocale::Japanese => "XDTS のシグネチャ行が不正です".to_owned(),
                AppLocale::English => "XDTS signature line is invalid".to_owned(),
            },
            Self::InvalidJson(source) => match locale {
                AppLocale::Japanese => format!("XDTS 本文の JSON が不正です: {source}"),
                AppLocale::English => format!("XDTS body is not valid JSON: {source}"),
            },
            Self::MissingTimeTable => match locale {
                AppLocale::Japanese => "XDTS にタイムテーブルがありません".to_owned(),
                AppLocale::English => "XDTS contains no time tables".to_owned(),
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
                strings::lowlevel_invalid_sheet_model(locale, "XDTS", source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::XdtsFile;
    use crate::test_fixtures::fixture_path;

    #[test]
    fn parses_sample_xdts_into_sheet() {
        let Some(path) = fixture_path("07_223B.xdts") else {
            eprintln!("skipping fixture-based test: tests/fixtures/07_223B.xdts is missing");
            return;
        };
        let xdts = XdtsFile::from_path(&path).expect("sample xdts should parse");

        assert_eq!(xdts.time_tables().len(), 1);

        let sheet = xdts.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.column_count(), 5);
        assert_eq!(sheet.row_count(), 192);
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(2), "C_go");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(0, 191).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(3, 46).map(|value| value.as_i64()), Some(2));
        assert_eq!(sheet.cell(3, 47).map(|value| value.as_i64()), Some(2));
        assert_eq!(sheet.cell(3, 48).map(|value| value.as_i64()), Some(3));
    }

    #[test]
    fn null_only_track_becomes_blank_column() {
        let source = r#"exchangeDigitalTimeSheet Save Data
{
  "timeTables": [
    {
      "duration": 3,
      "fields": [
        {
          "fieldId": 0,
          "tracks": [
            {
              "trackNo": 0,
              "frames": [
                { "frame": 0, "data": [{ "id": 0, "values": ["SYMBOL_NULL_CELL"] }] },
                { "frame": 3, "data": [{ "id": 0, "values": ["SYMBOL_NULL_CELL"] }] }
              ]
            }
          ]
        }
      ],
      "timeTableHeaders": [
        { "fieldId": 0, "names": ["A"] }
      ]
    }
  ]
}"#;

        let sheet = XdtsFile::from_text(source)
            .expect("inline xdts should parse")
            .to_sheet()
            .expect("sheet conversion should succeed");

        assert_eq!(sheet.column_count(), 1);
        assert_eq!(sheet.row_count(), 3);
        assert!(sheet.cell(0, 0).is_some_and(|value| value.is_blank()));
        assert!(sheet.cell(0, 1).is_some_and(|value| value.is_blank()));
        assert!(sheet.cell(0, 2).is_some_and(|value| value.is_blank()));
    }

    #[test]
    fn ignores_trailing_blank_frame_past_duration() {
        let Some(path) = fixture_path("c-15.xdts") else {
            eprintln!("skipping fixture-based test: tests/fixtures/c-15.xdts is missing");
            return;
        };
        let sheet = XdtsFile::from_path(&path)
            .expect("sample xdts should parse")
            .to_sheet()
            .expect("sheet conversion should succeed");

        assert_eq!(sheet.column_count(), 1);
        assert_eq!(sheet.row_count(), 79);
        assert_eq!(sheet.column_name(0), "Bセル");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(0, 4).map(|value| value.as_i64()), Some(2));
        assert_eq!(sheet.cell(0, 76).map(|value| value.as_i64()), Some(20));
        assert_eq!(sheet.cell(0, 78).map(|value| value.as_i64()), Some(20));
    }
}
