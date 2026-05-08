use crate::{settings::AppLocale, strings};
use encoding_rs::SHIFT_JIS;
use sheet::{CellValue, Sheet, SheetColumn, SheetError};
use std::{collections::HashMap, fs, io, path::Path};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArdCell {
    column_index: usize,
    frames: Vec<(usize, i64)>,
}

impl ArdCell {
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    pub fn frames(&self) -> &[(usize, i64)] {
        &self.frames
    }
}

#[derive(Clone, Debug)]
pub struct ArdFile {
    layer_count: usize,
    frame_count: usize,
    cmp_fps: Option<u32>,
    page_sec: Option<f32>,
    cell_names: Vec<String>,
    cells: Vec<ArdCell>,
}

impl ArdFile {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ArdError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| ArdError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, ArdError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ArdError> {
        let (text, _, _) = SHIFT_JIS.decode(bytes);
        Self::from_text(&text)
    }

    pub fn from_text(text: &str) -> Result<Self, ArdError> {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let lines = normalized.lines().collect::<Vec<_>>();

        let mut layer_count = None;
        let mut frame_count = None;
        let mut cmp_fps = None;
        let mut page_sec = None;
        let mut cell_names = Vec::new();
        let mut cells = Vec::new();

        let mut index = 0usize;
        while index < lines.len() {
            let line = lines[index].trim();

            if line.is_empty() {
                index += 1;
                continue;
            }

            if let Some((key, value)) = split_tab_pair(line) {
                match key.to_ascii_lowercase().as_str() {
                    "layercount" => {
                        layer_count = Some(parse_usize(value, "LayerCount")?);
                    }
                    "framecount" => {
                        frame_count = Some(parse_usize(value, "FrameCount")?);
                    }
                    "cmpfps" => {
                        cmp_fps = Some(parse_u32(value, "CmpFps")?);
                    }
                    "pagesec" => {
                        page_sec = Some(parse_f32(value, "PageSec")?);
                    }
                    _ => {}
                }
                index += 1;
                continue;
            }

            if line.eq_ignore_ascii_case("*CellName") {
                index += 1;
                while index < lines.len() {
                    let current = lines[index].trim();
                    if current.is_empty() {
                        index += 1;
                        continue;
                    }
                    if current.starts_with('*') {
                        break;
                    }
                    let (name_index, name) = split_tab_pair(current)
                        .ok_or_else(|| ArdError::InvalidCellNameLine(current.to_owned()))?;
                    let parsed_index = parse_usize(name_index, "CellName index")?;
                    ensure_vec_len(&mut cell_names, parsed_index + 1);
                    cell_names[parsed_index] = name.trim().to_owned();
                    index += 1;
                }
                continue;
            }

            if line.eq_ignore_ascii_case("*CellDataStart") {
                index += 1;
                while index < lines.len() {
                    let current = lines[index].trim();
                    if current.is_empty() {
                        index += 1;
                        continue;
                    }
                    if current.eq_ignore_ascii_case("*End") {
                        break;
                    }
                    let Some(rest) = current.strip_prefix("*Cell\t") else {
                        return Err(ArdError::InvalidCellBlockStart(current.to_owned()));
                    };
                    let column_index = parse_usize(rest.trim(), "Cell index")?;
                    index += 1;

                    let mut frames = Vec::new();
                    while index < lines.len() {
                        let content = lines[index].trim();
                        if content.is_empty() {
                            index += 1;
                            continue;
                        }
                        if let Some(end_rest) = content.strip_prefix("*CellEnd\t") {
                            let end_index = parse_usize(end_rest.trim(), "CellEnd index")?;
                            if end_index != column_index {
                                return Err(ArdError::MismatchedCellBlockEnd {
                                    start: column_index,
                                    end: end_index,
                                });
                            }
                            index += 1;
                            break;
                        }

                        let (frame, value) = split_tab_pair(content)
                            .ok_or_else(|| ArdError::InvalidCellFrameLine(content.to_owned()))?;
                        frames.push((
                            parse_usize(frame, "frame number")?,
                            parse_i64(value, "cell number")?,
                        ));
                        index += 1;
                    }

                    cells.push(ArdCell {
                        column_index,
                        frames,
                    });
                }
                continue;
            }

            index += 1;
        }

        let layer_count = layer_count.ok_or(ArdError::MissingField("LayerCount"))?;
        let frame_count = frame_count.ok_or(ArdError::MissingField("FrameCount"))?;

        Ok(Self {
            layer_count,
            frame_count,
            cmp_fps,
            page_sec,
            cell_names,
            cells,
        })
    }

    pub fn layer_count(&self) -> usize {
        self.layer_count
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn cmp_fps(&self) -> Option<u32> {
        self.cmp_fps
    }

    pub fn page_sec(&self) -> Option<f32> {
        self.page_sec
    }

    pub fn cell_names(&self) -> &[String] {
        &self.cell_names
    }

    pub fn cells(&self) -> &[ArdCell] {
        &self.cells
    }

    pub fn to_sheet(&self) -> Result<Sheet, ArdError> {
        self.to_sheet_with_fps(24)
    }

    pub fn to_sheet_with_fps(&self, fallback_fps: u32) -> Result<Sheet, ArdError> {
        let fps = self.cmp_fps.unwrap_or(fallback_fps).max(1);
        let cell_map = self
            .cells
            .iter()
            .map(|cell| (cell.column_index, cell))
            .collect::<HashMap<_, _>>();

        let columns = (0..self.layer_count)
            .map(|column_index| {
                let name = self
                    .cell_names
                    .get(column_index)
                    .filter(|name| !name.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| default_column_name(column_index));
                let values = match cell_map.get(&column_index) {
                    Some(cell) => expand_frames(self.frame_count, &cell.frames)?,
                    None => vec![CellValue::blank(); self.frame_count],
                };
                Ok(SheetColumn::new(name, values))
            })
            .collect::<Result<Vec<_>, ArdError>>()?;

        Sheet::try_with_fps(columns, fps).map_err(ArdError::InvalidSheetModel)
    }
}

fn split_tab_pair(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '\t');
    Some((parts.next()?.trim(), parts.next()?.trim()))
}

fn parse_usize(value: &str, field: &'static str) -> Result<usize, ArdError> {
    value.parse().map_err(|_| ArdError::InvalidNumber {
        field,
        value: value.to_owned(),
    })
}

fn parse_u32(value: &str, field: &'static str) -> Result<u32, ArdError> {
    value.parse().map_err(|_| ArdError::InvalidNumber {
        field,
        value: value.to_owned(),
    })
}

fn parse_i64(value: &str, field: &'static str) -> Result<i64, ArdError> {
    value.parse().map_err(|_| ArdError::InvalidNumber {
        field,
        value: value.to_owned(),
    })
}

fn parse_f32(value: &str, field: &'static str) -> Result<f32, ArdError> {
    value.parse().map_err(|_| ArdError::InvalidNumber {
        field,
        value: value.to_owned(),
    })
}

fn ensure_vec_len(vec: &mut Vec<String>, len: usize) {
    while vec.len() < len {
        vec.push(String::new());
    }
}

fn expand_frames(frame_count: usize, frames: &[(usize, i64)]) -> Result<Vec<CellValue>, ArdError> {
    let mut values = vec![CellValue::blank(); frame_count];

    for (index, &(start_frame, value)) in frames.iter().enumerate() {
        if start_frame == 0 {
            return Err(ArdError::FrameNumberStartsAtZero);
        }
        if start_frame > frame_count {
            return Err(ArdError::FrameNumberOutOfRange {
                frame: start_frame,
                frame_count,
            });
        }
        if index > 0 && frames[index - 1].0 >= start_frame {
            return Err(ArdError::FrameNumberNotAscending {
                previous: frames[index - 1].0,
                current: start_frame,
            });
        }

        let start_index = start_frame - 1;
        let end_index = frames
            .get(index + 1)
            .map(|(next_frame, _)| next_frame.saturating_sub(1))
            .unwrap_or(frame_count);
        for item in values.iter_mut().take(end_index).skip(start_index) {
            *item = CellValue::from(value);
        }
    }

    Ok(values)
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
pub enum ArdError {
    #[error("failed to read ard file `{path}`")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read ard bytes")]
    ReadBytes(#[from] io::Error),
    #[error("required field `{0}` is missing")]
    MissingField(&'static str),
    #[error("invalid numeric value for `{field}`: `{value}`")]
    InvalidNumber { field: &'static str, value: String },
    #[error("invalid cell name line: `{0}`")]
    InvalidCellNameLine(String),
    #[error("invalid cell block start: `{0}`")]
    InvalidCellBlockStart(String),
    #[error("cell block start/end index mismatch: start={start}, end={end}")]
    MismatchedCellBlockEnd { start: usize, end: usize },
    #[error("invalid cell frame line: `{0}`")]
    InvalidCellFrameLine(String),
    #[error("frame numbers must start from 1")]
    FrameNumberStartsAtZero,
    #[error("frame number {frame} is out of range for {frame_count} frames")]
    FrameNumberOutOfRange { frame: usize, frame_count: usize },
    #[error("frame numbers must be strictly ascending: previous={previous}, current={current}")]
    FrameNumberNotAscending { previous: usize, current: usize },
    #[error("failed to convert ard to sheet")]
    InvalidSheetModel(#[source] SheetError),
}

impl ArdError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::ReadFile { path, source } => {
                strings::lowlevel_read_file(locale, "ARD", path, source)
            }
            Self::ReadBytes(source) => strings::lowlevel_read_bytes(locale, "ARD", source),
            Self::MissingField(field) => match locale {
                AppLocale::Japanese => format!("必須項目 `{field}` がありません"),
                AppLocale::English => format!("Missing required field `{field}`"),
            },
            Self::InvalidNumber { field, value } => match locale {
                AppLocale::Japanese => format!("`{field}` の数値が不正です: `{value}`"),
                AppLocale::English => format!("Invalid numeric value for `{field}`: `{value}`"),
            },
            Self::InvalidCellNameLine(line) => match locale {
                AppLocale::Japanese => format!("セル名行の形式が不正です: `{line}`"),
                AppLocale::English => format!("Invalid cell name line: `{line}`"),
            },
            Self::InvalidCellBlockStart(line) => match locale {
                AppLocale::Japanese => format!("セルブロック開始行の形式が不正です: `{line}`"),
                AppLocale::English => format!("Invalid cell block start: `{line}`"),
            },
            Self::MismatchedCellBlockEnd { start, end } => match locale {
                AppLocale::Japanese => {
                    format!("セルブロックの開始/終了番号が一致しません: start={start}, end={end}")
                }
                AppLocale::English => {
                    format!("Cell block start/end index mismatch: start={start}, end={end}")
                }
            },
            Self::InvalidCellFrameLine(line) => match locale {
                AppLocale::Japanese => format!("セルコマ行の形式が不正です: `{line}`"),
                AppLocale::English => format!("Invalid cell frame line: `{line}`"),
            },
            Self::FrameNumberStartsAtZero => match locale {
                AppLocale::Japanese => "コマ番号は 1 から始まる必要があります".to_owned(),
                AppLocale::English => "Frame numbers must start from 1".to_owned(),
            },
            Self::FrameNumberOutOfRange { frame, frame_count } => match locale {
                AppLocale::Japanese => {
                    format!("コマ番号 {frame} は全 {frame_count} コマの範囲外です")
                }
                AppLocale::English => {
                    format!("Frame number {frame} is out of range for {frame_count} frames")
                }
            },
            Self::FrameNumberNotAscending { previous, current } => match locale {
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
            Self::InvalidSheetModel(source) => {
                strings::lowlevel_invalid_sheet_model(locale, "ARD", source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ArdFile;
    use crate::test_fixtures::fixture_path;

    #[test]
    fn parses_sample_ard_into_sheet() {
        let Some(path) = fixture_path("136.ard") else {
            eprintln!("skipping fixture-based test: tests/fixtures/136.ard is missing");
            return;
        };
        let ard = ArdFile::from_path(&path).expect("sample ard should parse");

        assert_eq!(ard.layer_count(), 12);
        assert_eq!(ard.frame_count(), 243);
        assert_eq!(ard.cmp_fps(), Some(24));
        assert_eq!(ard.page_sec(), Some(6.0));
        assert_eq!(ard.cell_names()[0], "A");
        assert_eq!(ard.cell_names()[11], "L");

        let sheet = ard.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.column_count(), 12);
        assert_eq!(sheet.row_count(), 243);
        assert_eq!(sheet.fps(), 24);
        assert_eq!(
            sheet.cell(0, 0).map(ToString::to_string).as_deref(),
            Some("1")
        );
        assert_eq!(
            sheet.cell(0, 3).map(ToString::to_string).as_deref(),
            Some("2")
        );
        assert_eq!(
            sheet.cell(1, 120).map(ToString::to_string).as_deref(),
            Some("1")
        );
        assert!(sheet.cell(10, 10).is_some_and(|value| value.is_blank()));
    }
}
