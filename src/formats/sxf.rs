use crate::{settings::AppLocale, strings};
use encoding_rs::SHIFT_JIS;
use sheet::{CellValue, Sheet, SheetColumn, SheetError};
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
enum SxfCelValue {
    Number(i64),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct SxfCel {
    name: String,
    values: Vec<SxfCelValue>,
}

impl SxfCel {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct SxfFile {
    frame_count: usize,
    memo: Vec<String>,
    cels: Vec<SxfCel>,
    dialog: Vec<String>,
}

impl SxfFile {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, SxfError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| SxfError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, SxfError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SxfError> {
        let mut offset = 0usize;

        macro_rules! skip {
            ($n:expr) => {{
                let n: usize = $n;
                let end = offset.checked_add(n).ok_or(SxfError::UnexpectedEof)?;
                if end > bytes.len() {
                    return Err(SxfError::UnexpectedEof);
                }
                offset = end;
            }};
        }

        macro_rules! read_bytes {
            ($n:expr) => {{
                let n: usize = $n;
                let end = offset.checked_add(n).ok_or(SxfError::UnexpectedEof)?;
                if end > bytes.len() {
                    return Err(SxfError::UnexpectedEof);
                }
                let slice = &bytes[offset..end];
                offset = end;
                slice
            }};
        }

        macro_rules! read_u16_be {
            () => {{
                let b = read_bytes!(2);
                u16::from_be_bytes([b[0], b[1]]) as usize
            }};
        }

        macro_rules! read_u32_be {
            () => {{
                let b = read_bytes!(4);
                u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize
            }};
        }

        // 8-byte file header
        skip!(8);

        // FF01: フレーム数
        skip!(2); // section marker
        let section1_len = read_u32_be!();
        skip!(2);
        let frame_count = read_u32_be!();
        skip!(section1_len.saturating_sub(6));

        // FF02: メモ
        skip!(2);
        let section2_len = read_u32_be!();
        let memo = parse_memo(read_bytes!(section2_len));

        // FF03: 原画シート（出力には使用しない）
        skip!(2);
        let section3_len = read_u32_be!();
        let mut count = 0usize;
        while count < section3_len {
            let line_len = read_u32_be!();
            let name_len = read_u16_be!();
            skip!(name_len);
            skip!(4); // padding
            let cel_size = read_u32_be!();
            skip!(frame_count * cel_size);
            count += line_len + 4;
        }

        // FF04: 動画シート
        skip!(2);
        let section4_len = read_u32_be!();
        let mut count = 0usize;
        let mut cels = Vec::new();
        while count < section4_len {
            let line_len = read_u32_be!();
            let name_len = read_u16_be!();
            let cel_name = decode_sjis(read_bytes!(name_len));
            skip!(4); // padding
            let cel_size = read_u32_be!();
            let mut raw_values: Vec<Option<SxfCelValue>> = Vec::with_capacity(frame_count);
            for _ in 0..frame_count {
                raw_values.push(parse_douga_cel(read_bytes!(cel_size)));
            }
            let values = fill_cel_column(raw_values);
            cels.push(SxfCel {
                name: cel_name,
                values,
            });
            count += line_len + 4;
        }

        // FF05: 台詞
        skip!(2);
        let section5_len = read_u32_be!();
        let dialog_size = read_u32_be!();
        let mut dialog = Vec::new();
        let mut count = 0usize;
        while dialog_size > 0 && count < section5_len.saturating_sub(dialog_size) {
            let raw = read_bytes!(dialog_size);
            let trimmed_len = raw
                .iter()
                .rposition(|&b| b != 0)
                .map(|i| i + 1)
                .unwrap_or(0);
            dialog.push(decode_sjis(&raw[..trimmed_len]));
            count += dialog_size;
        }

        // FF06: スキップ
        skip!(2);
        let section6_len = read_u32_be!();
        skip!(section6_len);

        // FF07: カメラワーク（スキップ）
        skip!(2);
        let section7_len = read_u32_be!();
        skip!(section7_len);
        let _parsed_len = offset;

        Ok(Self {
            frame_count,
            memo,
            cels,
            dialog,
        })
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn memo(&self) -> &[String] {
        &self.memo
    }

    pub fn cels(&self) -> &[SxfCel] {
        &self.cels
    }

    pub fn dialog(&self) -> &[String] {
        &self.dialog
    }

    pub fn to_sheet(&self) -> Result<Sheet, SxfError> {
        self.to_sheet_with_fps(24)
    }

    pub fn to_sheet_with_fps(&self, fps: u32) -> Result<Sheet, SxfError> {
        let columns = self
            .cels
            .iter()
            .map(|cel| {
                let values = cel
                    .values
                    .iter()
                    .map(|v| match v {
                        SxfCelValue::Number(n) => CellValue::from(*n),
                        SxfCelValue::Text(_) => CellValue::blank(),
                    })
                    .collect();
                SheetColumn::new(cel.name.clone(), values)
            })
            .collect();

        Sheet::try_with_fps(columns, fps).map_err(SxfError::InvalidSheetModel)
    }
}

fn decode_sjis(bytes: &[u8]) -> String {
    let (text, _, _) = SHIFT_JIS.decode(bytes);
    text.into_owned()
}

fn parse_memo(data: &[u8]) -> Vec<String> {
    if data.len() < 6 {
        return vec![];
    }
    // data[2..-4] を \r で分割
    let inner = &data[2..data.len() - 4];
    inner.split(|&b| b == b'\r').map(decode_sjis).collect()
}

fn parse_douga_cel(data: &[u8]) -> Option<SxfCelValue> {
    if data.len() < 2 {
        return None;
    }
    let head = u16::from_be_bytes([data[0], data[1]]);
    let raw = &data[2..];
    let trimmed_len = raw
        .iter()
        .rposition(|&b| b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    let value = decode_sjis(&raw[..trimmed_len]);

    match head {
        0 => None,
        1 => {
            if value.is_empty() {
                None
            } else if value.chars().all(|c| c.is_ascii_digit()) {
                Some(SxfCelValue::Number(value.parse().unwrap_or(0)))
            } else {
                Some(SxfCelValue::Text(value))
            }
        }
        8 => Some(SxfCelValue::Number(0)),
        _ => None,
    }
}

/// 空セルを直前の値で埋める（Python の douga_cel_column に対応）
fn fill_cel_column(mut cels: Vec<Option<SxfCelValue>>) -> Vec<SxfCelValue> {
    // 先頭が空なら 0 に置き換え
    if cels.first() == Some(&None) {
        cels[0] = Some(SxfCelValue::Number(0));
    }

    let mut result = Vec::with_capacity(cels.len());
    let mut prev = SxfCelValue::Number(0);

    for cel in cels {
        let current = cel.unwrap_or_else(|| prev.clone());
        result.push(current.clone());
        prev = current;
    }

    result
}

#[derive(Debug, Error)]
pub enum SxfError {
    #[error("failed to read sxf file `{path}`")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read sxf bytes")]
    ReadBytes(#[from] io::Error),
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("failed to convert sxf to sheet")]
    InvalidSheetModel(#[source] SheetError),
}

impl SxfError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::ReadFile { path, source } => {
                strings::lowlevel_read_file(locale, "SXF", path, source)
            }
            Self::ReadBytes(source) => strings::lowlevel_read_bytes(locale, "SXF", source),
            Self::UnexpectedEof => match locale {
                AppLocale::Japanese => "SXF ファイルが途中で終わっています".to_owned(),
                AppLocale::English => "Unexpected end of SXF file".to_owned(),
            },
            Self::InvalidSheetModel(source) => {
                strings::lowlevel_invalid_sheet_model(locale, "SXF", source)
            }
        }
    }
}
