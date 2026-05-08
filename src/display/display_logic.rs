use crate::settings::table::{FrameHeaderMode, HeaderDisplayDensity, SegmentHeaderMode};
use sheet::Sheet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RowHeaderLabels {
    pub frame_label: Option<String>,
    pub segment_label: Option<String>,
    pub frame_is_alert: bool,
    pub frame_is_inserted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SecondDividerKind {
    Quarter,
    Half,
    Full,
}

pub(crate) fn row_header_labels_for_sheet(
    sheet: &Sheet,
    row: usize,
    frames_per_page: usize,
    frame_mode: FrameHeaderMode,
    segment_mode: SegmentHeaderMode,
    frame_density: HeaderDisplayDensity,
    segment_density: HeaderDisplayDensity,
) -> RowHeaderLabels {
    if sheet.is_inserted_row(row) {
        let group_pos = sheet.inserted_row_group_position(row).unwrap_or(1);
        let frame_label = match frame_mode {
            FrameHeaderMode::AbsoluteFrame => sheet
                .inserted_absolute_frame_number_at(row)
                .map(|frame| frame.to_string()),
            FrameHeaderMode::SecondFrame => Some(group_pos.to_string()),
            FrameHeaderMode::PageFrame => Some(group_pos.to_string()),
        };
        return RowHeaderLabels {
            frame_label,
            segment_label: Some(format_segment_header_label(
                group_pos,
                sheet.fps(),
                frames_per_page,
                segment_mode,
            )),
            frame_is_alert: false,
            frame_is_inserted: true,
        };
    }

    let real_frame = sheet.real_frame_number_at(row);
    let frame_label_frame = match frame_mode {
        FrameHeaderMode::AbsoluteFrame => sheet.absolute_frame_number_at(row),
        _ => real_frame,
    };
    let is_punched_row = sheet.is_punched_row(row);
    let punched_absolute_frame = frame_mode == FrameHeaderMode::AbsoluteFrame && is_punched_row;
    let frame_label = if punched_absolute_frame {
        Some("-".to_owned())
    } else {
        should_show_row_header_label(frame_label_frame, frame_density).then(|| {
            format_frame_header_label(frame_label_frame, sheet.fps(), frames_per_page, frame_mode)
        })
    };
    RowHeaderLabels {
        frame_label,
        segment_label: if punched_absolute_frame {
            None
        } else {
            should_show_row_header_label(real_frame, segment_density).then(|| {
                format_segment_header_label(real_frame, sheet.fps(), frames_per_page, segment_mode)
            })
        },
        frame_is_alert: is_punched_row,
        frame_is_inserted: false,
    }
}

pub(crate) fn second_divider_kind_for_sheet(
    sheet: &Sheet,
    row: usize,
) -> Option<SecondDividerKind> {
    if sheet.is_inserted_row(row) {
        return None;
    }

    let fps = sheet.fps() as usize;
    if fps == 0 {
        return None;
    }

    let real_frame = sheet.real_frame_number_at(row);
    if real_frame % fps == 0 {
        Some(SecondDividerKind::Full)
    } else if fps % 2 == 0 && real_frame % (fps / 2) == 0 {
        Some(SecondDividerKind::Half)
    } else if fps % 4 == 0 && real_frame % (fps / 4) == 0 {
        Some(SecondDividerKind::Quarter)
    } else {
        None
    }
}

fn format_frame_header_label(
    frame: usize,
    fps: u32,
    frames_per_page: usize,
    mode: FrameHeaderMode,
) -> String {
    match mode {
        FrameHeaderMode::SecondFrame => format_second_frame(frame, fps),
        FrameHeaderMode::AbsoluteFrame => frame.to_string(),
        FrameHeaderMode::PageFrame => format_page_frame(frame, frames_per_page),
    }
}

pub(crate) fn is_odd_second_band_for_sheet(sheet: &Sheet, row: usize) -> bool {
    let fps = (sheet.fps() as usize).max(1);
    let real_frame = sheet.real_frame_number_at(row);
    ((real_frame - 1) / fps) % 2 == 1
}

fn format_segment_header_label(
    frame: usize,
    fps: u32,
    frames_per_page: usize,
    mode: SegmentHeaderMode,
) -> String {
    match mode {
        SegmentHeaderMode::Seconds => format!("{}s", format_second_number(frame, fps)),
        SegmentHeaderMode::Pages => format_page_number(frame, frames_per_page),
    }
}

fn format_second_frame(frame: usize, fps: u32) -> String {
    let fps = (fps as usize).max(1);
    (((frame - 1) % fps) + 1).to_string()
}

fn format_page_frame(frame: usize, frames_per_page: usize) -> String {
    let frames_per_page = frames_per_page.max(1);
    let frame_in_page = ((frame - 1) % frames_per_page) + 1;
    frame_in_page.to_string()
}

#[cfg(test)]
fn format_seconds_and_frames(frame: usize, fps: u32) -> String {
    let fps = (fps as usize).max(1);
    let seconds = (frame - 1) / fps;
    let frame_in_second = ((frame - 1) % fps) + 1;
    format!("{seconds}/{frame_in_second}")
}

fn format_second_number(frame: usize, fps: u32) -> String {
    let fps = (fps as usize).max(1);
    ((frame - 1) / fps).to_string()
}

fn format_page_number(frame: usize, frames_per_page: usize) -> String {
    let frames_per_page = frames_per_page.max(1);
    let page = ((frame - 1) / frames_per_page) + 1;
    page.to_string()
}

fn should_show_row_header_label(frame: usize, density: HeaderDisplayDensity) -> bool {
    match density {
        HeaderDisplayDensity::All => true,
        HeaderDisplayDensity::Odd => frame % 2 == 1,
        HeaderDisplayDensity::Even => frame % 2 == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FrameHeaderMode, HeaderDisplayDensity, RowHeaderLabels, SecondDividerKind,
        SegmentHeaderMode, format_frame_header_label, format_page_frame, format_page_number,
        format_second_number, format_seconds_and_frames, format_segment_header_label,
        row_header_labels_for_sheet, second_divider_kind_for_sheet, should_show_row_header_label,
    };
    use sheet::{CellValue, RowKind, Sheet, SheetColumn};

    #[test]
    fn formats_row_header_labels_for_each_mode() {
        assert_eq!(
            format_frame_header_label(4, 24, 144, FrameHeaderMode::SecondFrame),
            "4"
        );
        assert_eq!(
            format_frame_header_label(25, 24, 144, FrameHeaderMode::SecondFrame),
            "1"
        );
        assert_eq!(
            format_frame_header_label(4, 24, 144, FrameHeaderMode::AbsoluteFrame),
            "4"
        );
        assert_eq!(
            format_frame_header_label(145, 24, 144, FrameHeaderMode::PageFrame),
            "1"
        );
        assert_eq!(
            format_segment_header_label(24, 24, 144, SegmentHeaderMode::Seconds),
            "0s"
        );
        assert_eq!(
            format_segment_header_label(145, 24, 144, SegmentHeaderMode::Pages),
            "2"
        );
    }

    #[test]
    fn formats_row_header_seconds_and_frames() {
        assert_eq!(format_seconds_and_frames(1, 24), "0/1");
        assert_eq!(format_seconds_and_frames(24, 24), "0/24");
        assert_eq!(format_seconds_and_frames(25, 24), "1/1");
    }

    #[test]
    fn formats_segment_second_numbers() {
        assert_eq!(format_second_number(1, 24), "0");
        assert_eq!(format_second_number(24, 24), "0");
        assert_eq!(format_second_number(25, 24), "1");
    }

    #[test]
    fn formats_segment_page_numbers() {
        assert_eq!(format_page_number(1, 144), "1");
        assert_eq!(format_page_number(144, 144), "1");
        assert_eq!(format_page_number(145, 144), "2");
    }

    #[test]
    fn formats_page_frame_labels() {
        assert_eq!(format_page_frame(1, 144), "1");
        assert_eq!(format_page_frame(80, 144), "80");
        assert_eq!(format_page_frame(144, 144), "144");
        assert_eq!(format_page_frame(145, 144), "1");
    }

    #[test]
    fn filters_row_header_labels_by_density() {
        assert!(should_show_row_header_label(1, HeaderDisplayDensity::All));
        assert!(should_show_row_header_label(1, HeaderDisplayDensity::Odd));
        assert!(!should_show_row_header_label(1, HeaderDisplayDensity::Even));
        assert!(should_show_row_header_label(2, HeaderDisplayDensity::Even));
        assert!(!should_show_row_header_label(2, HeaderDisplayDensity::Odd));
    }

    #[test]
    fn row_header_labels_skip_special_inserted_rows() {
        let mut sheet = Sheet::new(vec![SheetColumn::new(
            "A",
            vec![CellValue::from(1), CellValue::from(2), CellValue::from(3)],
        )]);
        sheet.set_row_kind(1, RowKind::SpecialInserted);

        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                0,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            ),
            RowHeaderLabels {
                frame_label: Some("1".to_string()),
                segment_label: Some("0s".to_string()),
                frame_is_alert: false,
                frame_is_inserted: false,
            }
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            ),
            RowHeaderLabels {
                frame_label: Some("2".to_string()),
                segment_label: Some("0s".to_string()),
                frame_is_alert: false,
                frame_is_inserted: true,
            }
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                2,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            ),
            RowHeaderLabels {
                frame_label: Some("3".to_string()),
                segment_label: Some("0s".to_string()),
                frame_is_alert: false,
                frame_is_inserted: false,
            }
        );
    }

    #[test]
    fn absolute_frame_labels_continue_through_special_inserted_rows() {
        let mut sheet = Sheet::new(vec![SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
                CellValue::from(5),
            ],
        )]);
        sheet.set_row_kind(1, RowKind::SpecialInserted);
        sheet.set_row_kind(2, RowKind::SpecialInserted);

        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("2".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                2,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("3".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                3,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("4".to_owned())
        );
    }

    #[test]
    fn absolute_frame_labels_for_leading_special_inserted_rows_start_at_one() {
        let mut sheet = Sheet::new(vec![SheetColumn::new(
            "A",
            vec![CellValue::from(1), CellValue::from(2), CellValue::from(3)],
        )]);
        sheet.set_row_kind(0, RowKind::SpecialInserted);
        sheet.set_row_kind(1, RowKind::SpecialInserted);

        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                0,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("1".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("2".to_owned())
        );
    }

    #[test]
    fn page_frame_labels_for_special_inserted_rows_restart_at_group_start() {
        let mut sheet = Sheet::new(vec![SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
                CellValue::from(5),
            ],
        )]);
        sheet.set_row_kind(1, RowKind::SpecialInserted);
        sheet.set_row_kind(2, RowKind::SpecialInserted);

        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::PageFrame,
                SegmentHeaderMode::Pages,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("1".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                2,
                144,
                FrameHeaderMode::PageFrame,
                SegmentHeaderMode::Pages,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("2".to_owned())
        );
    }

    #[test]
    fn second_frame_labels_for_special_inserted_rows_restart_at_group_start() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![
                    CellValue::from(1),
                    CellValue::from(2),
                    CellValue::from(3),
                    CellValue::from(4),
                    CellValue::from(5),
                ],
            )],
            24,
        );
        sheet.set_row_kind(1, RowKind::SpecialInserted);
        sheet.set_row_kind(2, RowKind::SpecialInserted);

        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::SecondFrame,
                SegmentHeaderMode::Pages,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("1".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                2,
                144,
                FrameHeaderMode::SecondFrame,
                SegmentHeaderMode::Pages,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("2".to_owned())
        );
    }

    #[test]
    fn absolute_frame_labels_do_not_count_punched_rows() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![
                    CellValue::from(1),
                    CellValue::from(2),
                    CellValue::from(3),
                    CellValue::from(4),
                ],
            )],
            24,
        );
        sheet.set_row_kind(1, RowKind::Punched);

        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                0,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("1".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("-".to_owned())
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                1,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            ),
            RowHeaderLabels {
                frame_label: Some("-".to_owned()),
                segment_label: None,
                frame_is_alert: true,
                frame_is_inserted: false,
            }
        );
        assert_eq!(
            row_header_labels_for_sheet(
                &sheet,
                2,
                144,
                FrameHeaderMode::AbsoluteFrame,
                SegmentHeaderMode::Seconds,
                HeaderDisplayDensity::All,
                HeaderDisplayDensity::All,
            )
            .frame_label,
            Some("2".to_owned())
        );
    }

    #[test]
    fn second_divider_kind_skips_special_inserted_rows() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![
                    CellValue::from(1),
                    CellValue::from(2),
                    CellValue::from(3),
                    CellValue::from(4),
                ],
            )],
            4,
        );
        sheet.set_row_kind(1, RowKind::SpecialInserted);

        assert_eq!(
            second_divider_kind_for_sheet(&sheet, 0),
            Some(SecondDividerKind::Quarter)
        );
        assert_eq!(second_divider_kind_for_sheet(&sheet, 1), None);
        assert_eq!(
            second_divider_kind_for_sheet(&sheet, 2),
            Some(SecondDividerKind::Half)
        );
        assert_eq!(
            second_divider_kind_for_sheet(&sheet, 3),
            Some(SecondDividerKind::Quarter)
        );
    }

    #[test]
    fn second_divider_kind_shows_only_full_seconds_for_non_divisible_fps() {
        let sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                (1..=30).map(CellValue::from).collect(),
            )],
            30,
        );

        assert_eq!(second_divider_kind_for_sheet(&sheet, 6), None);
        assert_eq!(
            second_divider_kind_for_sheet(&sheet, 14),
            Some(SecondDividerKind::Half)
        );
        assert_eq!(second_divider_kind_for_sheet(&sheet, 21), None);
        assert_eq!(
            second_divider_kind_for_sheet(&sheet, 29),
            Some(SecondDividerKind::Full)
        );
    }

    #[test]
    fn second_divider_kind_hides_fractional_dividers_when_fps_is_odd() {
        let sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                (1..=15).map(CellValue::from).collect(),
            )],
            15,
        );

        assert_eq!(second_divider_kind_for_sheet(&sheet, 4), None);
        assert_eq!(second_divider_kind_for_sheet(&sheet, 6), None);
        assert_eq!(
            second_divider_kind_for_sheet(&sheet, 14),
            Some(SecondDividerKind::Full)
        );
    }
}
