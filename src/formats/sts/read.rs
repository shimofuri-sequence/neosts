use crate::{settings::AppLocale, strings};
use encoding_rs::SHIFT_JIS;
use sheet::{CellValue, Sheet, SheetColumn, SheetError};
use std::{fs, io, path::Path};
use thiserror::Error;

/// STS file magic header: length-prefixed ASCII string "ShiraheitTimeSheet"
pub const STS_MAGIC: [u8; HEADER_SIZE] = [
    17, b'S', b'h', b'i', b'r', b'a', b'h', b'e', b'i', b'T', b'i', b'm', b'e', b'S', b'h', b'e',
    b'e', b't',
];

const HEADER_SIZE: usize = 18;
const METADATA_SIZE: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StsCel {
    name: String,
    values: Vec<u16>,
}

impl StsCel {
    pub fn new(name: impl Into<String>, values: Vec<u16>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> &[u16] {
        &self.values
    }
}

#[derive(Clone, Debug)]
pub struct StsFile {
    header: [u8; HEADER_SIZE],
    layer_count: usize,
    frame_count: usize,
    metadata_tail: [u8; 2],
    cels: Vec<StsCel>,
}

impl StsFile {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, StsError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| StsError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_reader(reader: &mut impl io::Read) -> Result<Self, StsError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StsError> {
        if bytes.len() < HEADER_SIZE + METADATA_SIZE {
            return Err(StsError::InvalidHeader {
                actual_len: bytes.len(),
            });
        }

        let mut offset = 0;
        let header = bytes[offset..offset + HEADER_SIZE]
            .try_into()
            .expect("header slice size should match");
        offset += HEADER_SIZE;

        let metadata = &bytes[offset..offset + METADATA_SIZE];
        offset += METADATA_SIZE;

        let layer_count = metadata[0] as usize;
        let frame_count = u16::from_le_bytes([metadata[1], metadata[2]]) as usize;
        let metadata_tail = [metadata[3], metadata[4]];

        let cel_bytes_len = layer_count
            .checked_mul(frame_count)
            .and_then(|count| count.checked_mul(2))
            .ok_or(StsError::CellDataTooLarge)?;

        if bytes.len() < offset + cel_bytes_len {
            return Err(StsError::UnexpectedEndOfCellData {
                expected_len: cel_bytes_len,
                actual_len: bytes.len().saturating_sub(offset),
            });
        }

        let mut cels = Vec::with_capacity(layer_count);
        let mut cel_values = Vec::with_capacity(layer_count);
        for _ in 0..layer_count {
            let cel_bytes = &bytes[offset..offset + frame_count * 2];
            offset += frame_count * 2;

            let values = cel_bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<_>>();
            cel_values.push(values);
        }

        let mut names = Vec::new();
        while offset < bytes.len() {
            let count = bytes[offset] as usize;
            offset += 1;

            if bytes.len() < offset + count {
                return Err(StsError::UnexpectedEndOfNameData {
                    expected_len: count,
                    actual_len: bytes.len().saturating_sub(offset),
                });
            }

            let name_bytes = &bytes[offset..offset + count];
            offset += count;
            names.push(decode_sjis(name_bytes));
        }

        if names.len() < layer_count {
            return Err(StsError::MissingCelNames {
                expected: layer_count,
                actual: names.len(),
            });
        }

        for (name, values) in names
            .into_iter()
            .zip(cel_values.into_iter())
            .take(layer_count)
        {
            cels.push(StsCel::new(name.trim_end_matches('\r'), values));
        }

        Ok(Self {
            header,
            layer_count,
            frame_count,
            metadata_tail,
            cels,
        })
    }

    pub fn header(&self) -> &[u8; HEADER_SIZE] {
        &self.header
    }

    pub fn layer_count(&self) -> usize {
        self.layer_count
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn metadata_tail(&self) -> [u8; 2] {
        self.metadata_tail
    }

    pub fn cels(&self) -> &[StsCel] {
        &self.cels
    }

    pub fn into_cels(self) -> Vec<StsCel> {
        self.cels
    }

    /// Construct an `StsFile` from raw parts.
    pub fn new(
        header: [u8; HEADER_SIZE],
        layer_count: usize,
        frame_count: usize,
        metadata_tail: [u8; 2],
        cels: Vec<StsCel>,
    ) -> Self {
        Self {
            header,
            layer_count,
            frame_count,
            metadata_tail,
            cels,
        }
    }

    /// Serialize this `StsFile` to its binary representation.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.try_to_bytes()
            .expect("sts serialization should succeed for validated data")
    }

    /// Serialize this `StsFile` to its binary representation with format validation.
    pub fn try_to_bytes(&self) -> Result<Vec<u8>, StsError> {
        if self.layer_count > u8::MAX as usize {
            return Err(StsError::TooManyLayers {
                actual: self.layer_count,
            });
        }
        if self.frame_count > u16::MAX as usize {
            return Err(StsError::TooManyFrames {
                actual: self.frame_count,
            });
        }
        if self.cels.len() != self.layer_count {
            return Err(StsError::CelCountMismatch {
                expected: self.layer_count,
                actual: self.cels.len(),
            });
        }

        let mut buf = Vec::new();

        buf.extend_from_slice(&self.header);
        buf.push(self.layer_count as u8);
        buf.extend_from_slice(&(self.frame_count as u16).to_le_bytes());
        buf.extend_from_slice(&self.metadata_tail);

        for cel in &self.cels {
            if cel.values().len() != self.frame_count {
                return Err(StsError::CelValueCountMismatch {
                    name: cel.name().to_owned(),
                    expected: self.frame_count,
                    actual: cel.values().len(),
                });
            }
            for &value in cel.values() {
                buf.extend_from_slice(&value.to_le_bytes());
            }
        }

        for cel in &self.cels {
            let (encoded, _, _) = SHIFT_JIS.encode(cel.name());
            if encoded.len() > u8::MAX as usize {
                return Err(StsError::CelNameTooLong {
                    name: cel.name().to_owned(),
                    encoded_len: encoded.len(),
                });
            }
            buf.push(encoded.len() as u8);
            buf.extend_from_slice(&encoded);
        }

        Ok(buf)
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), StsError> {
        let path = path.as_ref();
        let bytes = self.try_to_bytes()?;
        fs::write(path, bytes).map_err(|source| StsError::WriteFile {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn to_sheet(&self) -> Result<Sheet, StsError> {
        self.to_sheet_with_fps(24)
    }

    pub fn to_sheet_with_fps(&self, fps: u32) -> Result<Sheet, StsError> {
        let columns = self
            .cels
            .iter()
            .map(|cel| {
                let values = cel
                    .values
                    .iter()
                    .map(|value| CellValue::from(i64::from(*value)))
                    .collect();
                SheetColumn::new(cel.name.clone(), values)
            })
            .collect();

        Sheet::try_with_fps(columns, fps).map_err(StsError::InvalidSheetModel)
    }
}

fn decode_sjis(bytes: &[u8]) -> String {
    let (text, _, _) = SHIFT_JIS.decode(bytes);
    text.into_owned()
}

#[derive(Debug, Error)]
pub enum StsError {
    #[error("failed to read sts file `{path}`")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write sts file `{path}`")]
    WriteFile {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read sts bytes")]
    ReadBytes(#[from] io::Error),
    #[error("invalid sts header: expected at least 23 bytes but got {actual_len}")]
    InvalidHeader { actual_len: usize },
    #[error("sts cell data is too large")]
    CellDataTooLarge,
    #[error("sts layer count {actual} exceeds file format limit of 255")]
    TooManyLayers { actual: usize },
    #[error("sts frame count {actual} exceeds file format limit of 65535")]
    TooManyFrames { actual: usize },
    #[error("sts layer count mismatch: expected {expected} cels but got {actual}")]
    CelCountMismatch { expected: usize, actual: usize },
    #[error("sts cel `{name}` has {actual} values but expected {expected}")]
    CelValueCountMismatch {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error("sts cel name `{name}` is too long after Shift-JIS encoding: {encoded_len} bytes")]
    CelNameTooLong { name: String, encoded_len: usize },
    #[error("unexpected end of cel data: expected {expected_len} bytes but got {actual_len}")]
    UnexpectedEndOfCellData {
        expected_len: usize,
        actual_len: usize,
    },
    #[error("unexpected end of name data: expected {expected_len} bytes but got {actual_len}")]
    UnexpectedEndOfNameData {
        expected_len: usize,
        actual_len: usize,
    },
    #[error("cel names are missing: expected {expected} names but got {actual}")]
    MissingCelNames { expected: usize, actual: usize },
    #[error("failed to convert sts to sheet")]
    InvalidSheetModel(#[source] SheetError),
}

impl StsError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::ReadFile { path, source } => {
                strings::lowlevel_read_file(locale, "STS", path, source)
            }
            Self::WriteFile { path, source } => {
                strings::lowlevel_write_file(locale, "STS", path, source)
            }
            Self::ReadBytes(source) => strings::lowlevel_read_bytes(locale, "STS", source),
            Self::InvalidHeader { actual_len } => match locale {
                AppLocale::Japanese => {
                    format!(
                        "STS ヘッダが不正です。23 バイト以上必要ですが、実際は {actual_len} バイトでした"
                    )
                }
                AppLocale::English => {
                    format!("Invalid STS header: expected at least 23 bytes but got {actual_len}")
                }
            },
            Self::CellDataTooLarge => match locale {
                AppLocale::Japanese => "STS のセルデータサイズが大きすぎます".to_owned(),
                AppLocale::English => "STS cell data is too large".to_owned(),
            },
            Self::TooManyLayers { actual } => match locale {
                AppLocale::Japanese => {
                    format!("STS の列数 {actual} は形式上限の 255 を超えています")
                }
                AppLocale::English => {
                    format!("STS layer count {actual} exceeds file format limit of 255")
                }
            },
            Self::TooManyFrames { actual } => match locale {
                AppLocale::Japanese => {
                    format!("STS のコマ数 {actual} は形式上限の 65535 を超えています")
                }
                AppLocale::English => {
                    format!("STS frame count {actual} exceeds file format limit of 65535")
                }
            },
            Self::CelCountMismatch { expected, actual } => match locale {
                AppLocale::Japanese => {
                    format!("STS の列数が一致しません。期待値は {expected}、実際は {actual} です")
                }
                AppLocale::English => {
                    format!("STS layer count mismatch: expected {expected} cels but got {actual}")
                }
            },
            Self::CelValueCountMismatch {
                name,
                expected,
                actual,
            } => match locale {
                AppLocale::Japanese => {
                    format!(
                        "STS セル `{name}` の値数が不正です。期待値は {expected}、実際は {actual} です"
                    )
                }
                AppLocale::English => {
                    format!("STS cel `{name}` has {actual} values but expected {expected}")
                }
            },
            Self::CelNameTooLong { name, encoded_len } => match locale {
                AppLocale::Japanese => {
                    format!(
                        "STS セル名 `{name}` は Shift-JIS 変換後の長さ {encoded_len} バイトで長すぎます"
                    )
                }
                AppLocale::English => {
                    format!(
                        "STS cel name `{name}` is too long after Shift-JIS encoding: {encoded_len} bytes"
                    )
                }
            },
            Self::UnexpectedEndOfCellData {
                expected_len,
                actual_len,
            } => match locale {
                AppLocale::Japanese => {
                    format!(
                        "セルデータが途中で終わっています。期待値は {expected_len} バイト、実際は {actual_len} バイトです"
                    )
                }
                AppLocale::English => {
                    format!(
                        "Unexpected end of cel data: expected {expected_len} bytes but got {actual_len}"
                    )
                }
            },
            Self::UnexpectedEndOfNameData {
                expected_len,
                actual_len,
            } => match locale {
                AppLocale::Japanese => {
                    format!(
                        "名前データが途中で終わっています。期待値は {expected_len} バイト、実際は {actual_len} バイトです"
                    )
                }
                AppLocale::English => {
                    format!(
                        "Unexpected end of name data: expected {expected_len} bytes but got {actual_len}"
                    )
                }
            },
            Self::MissingCelNames { expected, actual } => match locale {
                AppLocale::Japanese => {
                    format!("セル名が足りません。期待値は {expected}、実際は {actual} です")
                }
                AppLocale::English => {
                    format!("Cel names are missing: expected {expected} names but got {actual}")
                }
            },
            Self::InvalidSheetModel(source) => {
                strings::lowlevel_invalid_sheet_model(locale, "STS", source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StsFile;
    use crate::test_fixtures::fixture_path;

    #[test]
    fn parses_sample_sts_into_sheet() {
        let Some(path) = fixture_path("sheet05.sts") else {
            eprintln!("skipping fixture-based test: tests/fixtures/sheet05.sts is missing");
            return;
        };
        let sts = StsFile::from_path(&path).expect("sample sts should parse");

        assert_eq!(sts.layer_count(), 4);
        assert_eq!(sts.frame_count(), 72);
        assert_eq!(sts.cels()[0].name(), "A");
        assert_eq!(sts.cels()[1].name(), "B");

        let sheet = sts.to_sheet().expect("sheet conversion should succeed");
        assert_eq!(sheet.column_count(), 4);
        assert_eq!(sheet.row_count(), 72);
        assert_eq!(sheet.fps(), 24);
        assert_eq!(
            sheet.cell(0, 0).map(ToString::to_string).as_deref(),
            Some("1")
        );
        assert_eq!(
            sheet.cell(3, 71).map(ToString::to_string).as_deref(),
            Some("1")
        );
    }
}
