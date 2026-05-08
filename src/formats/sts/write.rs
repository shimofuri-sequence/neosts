use sheet::Sheet;
use std::path::Path;

use super::{STS_MAGIC, StsCel, StsError, StsFile};

/// Build an [`StsFile`] from a [`Sheet`].
///
/// Column names are preserved as-is and encoded as Shift-JIS in the output.
/// Cell values are clamped to `[0, u16::MAX]`.
///
/// The standard STS magic header and zero metadata tail are used.
pub fn sheet_to_sts(sheet: &Sheet) -> StsFile {
    let layer_count = sheet.column_count();
    let frame_count = sheet.row_count();

    let cels = (0..layer_count)
        .map(|col| {
            let name = sheet.column_name(col).to_string();
            let values = (0..frame_count)
                .map(|row| {
                    sheet
                        .cell(col, row)
                        .map(|v| v.as_i64().clamp(0, u16::MAX as i64) as u16)
                        .unwrap_or(0)
                })
                .collect();
            StsCel::new(name, values)
        })
        .collect();

    StsFile::new(STS_MAGIC, layer_count, frame_count, [0, 0], cels)
}

/// Write `sheet` as an STS file to `path`.
pub fn write_sheet_to_path(sheet: &Sheet, path: impl AsRef<Path>) -> Result<(), StsError> {
    sheet_to_sts(sheet).write_to_path(path)
}

#[cfg(test)]
mod tests {
    use super::{sheet_to_sts, write_sheet_to_path};
    use crate::formats::sts::{StsError, StsFile};
    use crate::test_fixtures::fixture_path;
    use sheet::{CellValue, Sheet, SheetColumn};

    fn sample_sheet() -> Sheet {
        Sheet::with_fps(
            vec![
                SheetColumn::new(
                    "A",
                    vec![
                        CellValue::from(1i32),
                        CellValue::from(1i32),
                        CellValue::from(2i32),
                    ],
                ),
                SheetColumn::new(
                    "B",
                    vec![
                        CellValue::from(3i32),
                        CellValue::blank(),
                        CellValue::from(3i32),
                    ],
                ),
            ],
            24,
        )
    }

    #[test]
    fn round_trips_through_bytes() {
        let sheet = sample_sheet();
        let sts = sheet_to_sts(&sheet);

        assert_eq!(sts.layer_count(), 2);
        assert_eq!(sts.frame_count(), 3);
        assert_eq!(sts.cels()[0].name(), "A");
        assert_eq!(sts.cels()[1].name(), "B");
        assert_eq!(sts.cels()[0].values(), &[1, 1, 2]);
        assert_eq!(sts.cels()[1].values(), &[3, 0, 3]);

        let bytes = sts.to_bytes();
        let reparsed = StsFile::from_bytes(&bytes).expect("re-parse should succeed");
        assert_eq!(reparsed.layer_count(), 2);
        assert_eq!(reparsed.frame_count(), 3);
        assert_eq!(reparsed.cels()[0].name(), "A");
        assert_eq!(reparsed.cels()[0].values(), &[1, 1, 2]);
        assert_eq!(reparsed.cels()[1].values(), &[3, 0, 3]);
    }

    #[test]
    fn round_trips_sample_sts_file() {
        let Some(path) = fixture_path("sheet05.sts") else {
            eprintln!("skipping fixture-based test: tests/fixtures/sheet05.sts is missing");
            return;
        };
        let original = StsFile::from_path(&path).expect("should parse");

        let bytes = original.to_bytes();
        let reparsed = StsFile::from_bytes(&bytes).expect("re-serialized bytes should parse");

        assert_eq!(reparsed.layer_count(), original.layer_count());
        assert_eq!(reparsed.frame_count(), original.frame_count());
        for (a, b) in original.cels().iter().zip(reparsed.cels()) {
            assert_eq!(a.name(), b.name());
            assert_eq!(a.values(), b.values());
        }
    }

    #[test]
    fn writes_and_reads_back_from_disk() {
        let sheet = sample_sheet();
        let tmp = std::env::temp_dir().join("sts_writer_test.sts");
        write_sheet_to_path(&sheet, &tmp).expect("write should succeed");

        let read_back = StsFile::from_path(&tmp).expect("read back should succeed");
        assert_eq!(read_back.layer_count(), 2);
        assert_eq!(read_back.frame_count(), 3);
        assert_eq!(read_back.cels()[0].name(), "A");

        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn rejects_names_that_exceed_sts_limit() {
        let long_name = "A".repeat(256);
        let sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                long_name.clone(),
                vec![CellValue::from(1i32)],
            )],
            24,
        );

        let tmp = std::env::temp_dir().join("sts_writer_name_too_long_test.sts");
        let error = write_sheet_to_path(&sheet, &tmp).expect_err("write should fail");

        match error {
            StsError::CelNameTooLong { name, encoded_len } => {
                assert_eq!(name, long_name);
                assert_eq!(encoded_len, 256);
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
