use crate::{settings::AppLocale, strings};
use serde::Deserialize;
use sheet::{CellValue, RowKind, Sheet, SheetColumn, SheetError};
use std::{collections::HashSet, fs, io, path::Path};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct DitisFile {
    version: Option<String>,
    sheets: Vec<DitisSheet>,
}

impl DitisFile {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DitisError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| DitisError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, DitisError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DitisError> {
        let text = std::str::from_utf8(bytes).map_err(DitisError::InvalidUtf8)?;
        Self::from_text(text)
    }

    pub fn from_text(text: &str) -> Result<Self, DitisError> {
        let normalized = text.trim_start_matches('\u{feff}');
        let parsed: DitisRoot =
            serde_json::from_str(normalized).map_err(DitisError::InvalidJson)?;
        Ok(Self {
            version: parsed.version,
            sheets: parsed.sheets,
        })
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn sheets(&self) -> &[DitisSheet] {
        &self.sheets
    }

    pub fn first_sheet_name(&self) -> Option<&str> {
        self.sheets
            .first()
            .and_then(|sheet| sheet.name.as_deref())
            .map(str::trim)
            .filter(|name| !name.is_empty())
    }

    pub fn to_sheet(&self) -> Result<Sheet, DitisError> {
        self.to_sheet_with_fps(24)
    }

    pub fn to_sheet_with_fps(&self, fallback_fps: u32) -> Result<Sheet, DitisError> {
        let sheet_data = self.sheets.first().ok_or(DitisError::MissingSheet)?;
        let fps = match sheet_data.fps {
            Some(0) => fallback_fps,
            Some(fps) => fps,
            None => fallback_fps,
        };

        let column_count = sheet_data.layers.len().max(sheet_data.data.len());
        let columns = (0..column_count)
            .map(|index| {
                let name = sheet_data
                    .layers
                    .get(index)
                    .filter(|name| !name.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| default_column_name(index));
                let values = build_column_values(sheet_data, index)?;
                Ok(SheetColumn::new(name, values))
            })
            .collect::<Result<Vec<_>, DitisError>>()?;

        let mut sheet = Sheet::try_with_fps(columns, fps).map_err(DitisError::InvalidSheetModel)?;
        apply_row_kinds(&mut sheet, sheet_data)?;
        Ok(sheet)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct DitisRoot {
    version: Option<String>,
    sheets: Vec<DitisSheet>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DitisSheet {
    pub name: Option<String>,
    pub fps: Option<u32>,
    #[serde(default)]
    pub layers: Vec<String>,
    pub frames: usize,
    #[serde(default)]
    pub frame_page_size: Option<usize>,
    #[serde(default)]
    pub data: Vec<Vec<String>>,
    #[serde(default)]
    pub inserted_frames: Vec<usize>,
    #[serde(default)]
    pub disabled_frames: Vec<usize>,
    #[serde(default)]
    pub inserted_frame_map: std::collections::BTreeMap<String, usize>,
}

fn build_column_values(sheet_data: &DitisSheet, index: usize) -> Result<Vec<CellValue>, DitisError> {
    let raw_values = sheet_data.data.get(index);
    let value_count = raw_values.map_or(0, Vec::len);
    if value_count > sheet_data.frames {
        return Err(DitisError::TooManyFrameValues {
            column: index,
            actual: value_count,
            frames: sheet_data.frames,
        });
    }

    let mut values = Vec::with_capacity(sheet_data.frames);
    if let Some(raw_values) = raw_values {
        for (frame_index, raw) in raw_values.iter().enumerate() {
            values.push(parse_cell_value(index, frame_index, raw)?);
        }
    }
    values.resize(sheet_data.frames, CellValue::blank());
    Ok(values)
}

fn parse_cell_value(column: usize, frame_index: usize, raw: &str) -> Result<CellValue, DitisError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(CellValue::blank());
    }

    trimmed
        .parse::<i64>()
        .map(CellValue::from)
        .map_err(|_| DitisError::InvalidCellValue {
            column,
            frame: frame_index + 1,
            value: raw.to_owned(),
        })
}

fn apply_row_kinds(sheet: &mut Sheet, sheet_data: &DitisSheet) -> Result<(), DitisError> {
    let inserted_rows = to_row_index_set(&sheet_data.inserted_frames, sheet.row_count(), true)?;
    let disabled_rows = to_row_index_set(&sheet_data.disabled_frames, sheet.row_count(), false)?;

    if let Some(conflicting_row) = inserted_rows.intersection(&disabled_rows).next() {
        return Err(DitisError::ConflictingFrameKinds {
            frame: conflicting_row + 1,
        });
    }

    for row in disabled_rows {
        let _ = sheet.set_row_kind(row, RowKind::Punched);
    }
    for row in inserted_rows {
        let _ = sheet.set_row_kind(row, RowKind::SpecialInserted);
    }
    Ok(())
}

fn to_row_index_set(
    frames: &[usize],
    row_count: usize,
    inserted: bool,
) -> Result<HashSet<usize>, DitisError> {
    frames
        .iter()
        .copied()
        .map(|frame| {
            if frame == 0 || frame > row_count {
                return Err(if inserted {
                    DitisError::InsertedFrameOutOfRange { frame, frames: row_count }
                } else {
                    DitisError::DisabledFrameOutOfRange { frame, frames: row_count }
                });
            }
            Ok(frame - 1)
        })
        .collect()
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
pub enum DitisError {
    #[error("failed to read ditis file `{path}`")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read ditis bytes")]
    ReadBytes(#[from] io::Error),
    #[error("ditis file is not valid UTF-8")]
    InvalidUtf8(#[source] std::str::Utf8Error),
    #[error("ditis body is not valid JSON")]
    InvalidJson(#[source] serde_json::Error),
    #[error("ditis contains no sheets")]
    MissingSheet,
    #[error("column {column} has {actual} values but the sheet declares {frames} frames")]
    TooManyFrameValues {
        column: usize,
        actual: usize,
        frames: usize,
    },
    #[error("column {column}, frame {frame} has an invalid cell value `{value}`")]
    InvalidCellValue {
        column: usize,
        frame: usize,
        value: String,
    },
    #[error("inserted frame {frame} is out of range for {frames} frames")]
    InsertedFrameOutOfRange { frame: usize, frames: usize },
    #[error("disabled frame {frame} is out of range for {frames} frames")]
    DisabledFrameOutOfRange { frame: usize, frames: usize },
    #[error("frame {frame} cannot be both inserted and disabled")]
    ConflictingFrameKinds { frame: usize },
    #[error("failed to convert ditis to sheet")]
    InvalidSheetModel(#[source] SheetError),
}

impl DitisError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::ReadFile { path, source } => {
                strings::lowlevel_read_file(locale, "DITIS", path, source)
            }
            Self::ReadBytes(source) => strings::lowlevel_read_bytes(locale, "DITIS", source),
            Self::InvalidUtf8(source) => match locale {
                AppLocale::Japanese => format!("DITIS ファイルは UTF-8 として不正です: {source}"),
                AppLocale::English => format!("DITIS file is not valid UTF-8: {source}"),
            },
            Self::InvalidJson(source) => match locale {
                AppLocale::Japanese => format!("DITIS 本文の JSON が不正です: {source}"),
                AppLocale::English => format!("DITIS body is not valid JSON: {source}"),
            },
            Self::MissingSheet => match locale {
                AppLocale::Japanese => "DITIS にシートがありません".to_owned(),
                AppLocale::English => "DITIS contains no sheets".to_owned(),
            },
            Self::TooManyFrameValues {
                column,
                actual,
                frames,
            } => match locale {
                AppLocale::Japanese => format!(
                    "列 {column} の値数 {actual} が、定義フレーム数 {frames} を超えています"
                ),
                AppLocale::English => format!(
                    "Column {column} has {actual} values but the sheet declares {frames} frames"
                ),
            },
            Self::InvalidCellValue {
                column,
                frame,
                value,
            } => match locale {
                AppLocale::Japanese => {
                    format!("列 {column} の {frame} コマ目の値 `{value}` が不正です")
                }
                AppLocale::English => {
                    format!("Column {column}, frame {frame} has an invalid cell value `{value}`")
                }
            },
            Self::InsertedFrameOutOfRange { frame, frames } => match locale {
                AppLocale::Japanese => {
                    format!("継ぎ足しコマ {frame} は尺 {frames} の範囲外です")
                }
                AppLocale::English => {
                    format!("Inserted frame {frame} is out of range for {frames} frames")
                }
            },
            Self::DisabledFrameOutOfRange { frame, frames } => match locale {
                AppLocale::Japanese => {
                    format!("中抜きコマ {frame} は尺 {frames} の範囲外です")
                }
                AppLocale::English => {
                    format!("Disabled frame {frame} is out of range for {frames} frames")
                }
            },
            Self::ConflictingFrameKinds { frame } => match locale {
                AppLocale::Japanese => {
                    format!("コマ {frame} が継ぎ足しと中抜きの両方に指定されています")
                }
                AppLocale::English => {
                    format!("Frame {frame} cannot be both inserted and disabled")
                }
            },
            Self::InvalidSheetModel(source) => {
                strings::lowlevel_invalid_sheet_model(locale, "DITIS", source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DitisFile;
    use crate::test_fixtures::fixture_path;

    #[test]
    fn parses_sample_ditis_into_sheet() {
        let Some(path) = fixture_path("Sheet1.ditis") else {
            eprintln!("skipping fixture-based test: tests/fixtures/Sheet1.ditis is missing");
            return;
        };
        let ditis = DitisFile::from_path(&path).expect("sample ditis should parse");

        assert_eq!(ditis.version(), Some("1.0"));
        assert_eq!(ditis.sheets().len(), 1);
        assert_eq!(ditis.sheets()[0].name.as_deref(), Some("Sheet3"));
        assert_eq!(ditis.first_sheet_name(), Some("Sheet3"));
        assert_eq!(ditis.sheets()[0].frame_page_size, Some(144));
        assert_eq!(ditis.sheets()[0].inserted_frame_map.len(), 6);

        let sheet = ditis.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.fps(), 24);
        assert_eq!(sheet.column_count(), 3);
        assert_eq!(sheet.row_count(), 54);
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(1), "B");
        assert_eq!(sheet.column_name(2), "C");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(0, 16).map(|value| value.as_i64()), Some(5));
        assert_eq!(sheet.cell(2, 0).map(|value| value.as_i64()), Some(1));
        assert_eq!(sheet.cell(2, 53).map(|value| value.as_i64()), Some(9));
        assert!(sheet.cell(0, 17).is_some_and(|value| value.is_blank()));
        assert!(sheet.is_inserted_row(17));
        assert!(sheet.is_inserted_row(22));
        assert!(sheet.is_punched_row(6));
        assert!(sheet.is_punched_row(11));
        assert!(sheet.cell(1, 0).is_some_and(|value| value.is_blank()));
        assert!(sheet.cell(1, 53).is_some_and(|value| value.is_blank()));
    }

    #[test]
    fn preserves_multiple_inserted_and_disabled_frame_groups() {
        let Some(path) = fixture_path("Sheet2.ditis") else {
            eprintln!("skipping fixture-based test: tests/fixtures/Sheet2.ditis is missing");
            return;
        };
        let sheet = DitisFile::from_path(&path)
            .expect("sample ditis should parse")
            .to_sheet()
            .expect("sheet conversion should succeed");

        assert_eq!(sheet.column_count(), 3);
        assert_eq!(sheet.row_count(), 57);
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(1), "B");
        assert_eq!(sheet.column_name(2), "C");

        assert!(sheet.is_inserted_row(3));
        assert!(sheet.is_inserted_row(8));
        assert_eq!(sheet.inserted_row_group_position(3), Some(1));
        assert_eq!(sheet.inserted_row_group_position(8), Some(6));

        assert!(sheet.is_inserted_row(36));
        assert!(sheet.is_inserted_row(38));
        assert_eq!(sheet.inserted_row_group_position(36), Some(1));
        assert_eq!(sheet.inserted_row_group_position(38), Some(3));

        assert!(sheet.is_punched_row(12));
        assert!(sheet.is_punched_row(17));
        assert!(sheet.is_punched_row(24));
        assert!(sheet.is_punched_row(29));
        assert_eq!(sheet.cell(1, 0).map(|value| value.as_i64()), Some(0));
        assert_eq!(sheet.cell(2, 0).map(|value| value.as_i64()), Some(0));
    }

    #[test]
    fn treats_missing_data_columns_as_blank_columns() {
        let ditis = DitisFile::from_text(
            r#"{
  "version": "1.0",
  "sheets": [
    {
      "name": "MissingData",
      "fps": 24,
      "layers": ["A", "B"],
      "frames": 3,
      "data": [["1", "2", "3"]]
    }
  ]
}"#,
        )
        .expect("inline ditis should parse");

        let sheet = ditis.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.column_count(), 2);
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(1), "B");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
        assert!(sheet.cell(1, 0).is_some_and(|value| value.is_blank()));
        assert!(sheet.cell(1, 2).is_some_and(|value| value.is_blank()));
    }

    #[test]
    fn first_sheet_name_ignores_blank_names() {
        let ditis = DitisFile::from_text(
            r#"{
  "version": "1.0",
  "sheets": [
    {
      "name": "   ",
      "fps": 24,
      "layers": ["A"],
      "frames": 1,
      "data": [["1"]]
    }
  ]
}"#,
        )
        .expect("inline ditis should parse");

        assert_eq!(ditis.first_sheet_name(), None);
    }

    #[test]
    fn accepts_utf8_bom_prefixed_json() {
        let ditis = DitisFile::from_text(
            "\u{feff}\
             {\n\
               \"version\": \"1.0\",\n\
               \"sheets\": [\n\
                 {\n\
                   \"name\": \"シートA\",\n\
                   \"fps\": 24,\n\
                   \"layers\": [\"A\"],\n\
                   \"frames\": 1,\n\
                   \"data\": [[\"1\"]]\n\
                 }\n\
               ]\n\
             }",
        )
        .expect("bom-prefixed ditis should parse");

        assert_eq!(ditis.first_sheet_name(), Some("シートA"));
        let sheet = ditis.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.cell(0, 0).map(|value| value.as_i64()), Some(1));
    }
}
