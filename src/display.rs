mod display_logic;
mod edit_ops;

use crate::settings::editor::DisplayMode;
pub(crate) use display_logic::{
    SecondDividerKind, is_odd_second_band_for_sheet, row_header_labels_for_sheet,
    second_divider_kind_for_sheet,
};
pub(crate) use edit_ops::{
    apply_decrement_from_above_to_sheet, apply_delete_to_sheet, apply_enter_to_sheet,
    apply_increment_from_above_to_sheet, apply_paste_to_sheet, copy_range_as_tsv_for_sheet,
    repeat_range_down_on_sheet, transfer_range_on_sheet,
};
use sheet::{CellValue, Sheet};
use std::ops::{Deref, DerefMut};

pub const DEFAULT_SHEET_NAME: &str = "新規タイムシート";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplaySheetState {
    sheet: Sheet,
    name: String,
}

impl DisplaySheetState {
    pub fn new(sheet: Sheet) -> Self {
        Self {
            sheet,
            name: DEFAULT_SHEET_NAME.to_owned(),
        }
    }

    pub fn replace_sheet(&mut self, sheet: Sheet) {
        self.sheet = sheet;
    }

    pub fn replace_with_blank_sheet(&mut self, column_count: u32, frame_count: u32, fps: u32) {
        self.sheet = Sheet::blank_with_fps(
            column_count.max(1) as usize,
            frame_count.max(1) as usize,
            fps,
        );
        self.name = DEFAULT_SHEET_NAME.to_owned();
    }

    pub fn resize_sheet(&mut self, column_count: usize, frame_count: usize) {
        self.sheet.resize_rows(frame_count);
        self.sheet.resize_columns(column_count);
    }

    pub fn sheet(&self) -> &Sheet {
        &self.sheet
    }

    pub fn sheet_mut(&mut self) -> &mut Sheet {
        &mut self.sheet
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let trimmed = name.into().trim().to_owned();
        if !trimmed.is_empty() {
            self.name = trimmed;
        }
    }

    pub fn duration_text(&self) -> String {
        format_sheet_duration(self.sheet.effective_frame_count(), self.sheet.fps())
    }

    pub fn apply_enter(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
        input_value: Option<i64>,
    ) {
        apply_enter_to_sheet(
            &mut self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
            input_value,
        );
    }

    pub fn apply_increment_from_above(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
    ) {
        apply_increment_from_above_to_sheet(
            &mut self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
        );
    }

    pub fn apply_decrement_from_above(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
    ) {
        apply_decrement_from_above_to_sheet(
            &mut self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
        );
    }

    pub fn apply_delete(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
    ) {
        apply_delete_to_sheet(
            &mut self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
        );
    }

    pub fn copy_range_as_tsv(
        &self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
    ) -> String {
        copy_range_as_tsv_for_sheet(
            &self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
        )
    }

    pub fn apply_paste(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        start_row: usize,
        values: &[Vec<CellValue>],
    ) {
        apply_paste_to_sheet(&mut self.sheet, display_mode, start_col, start_row, values);
    }

    pub fn transfer_range(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
        target_col: usize,
        target_row: usize,
        delete_source: bool,
    ) {
        transfer_range_on_sheet(
            &mut self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
            target_col,
            target_row,
            delete_source,
        );
    }

    pub fn repeat_range_down(
        &mut self,
        display_mode: DisplayMode,
        start_col: usize,
        end_col: usize,
        start_row: usize,
        end_row: usize,
    ) {
        repeat_range_down_on_sheet(
            &mut self.sheet,
            display_mode,
            start_col,
            end_col,
            start_row,
            end_row,
        );
    }
}

impl Default for DisplaySheetState {
    fn default() -> Self {
        Self::new(Sheet::new(Vec::new()))
    }
}

impl Deref for DisplaySheetState {
    type Target = Sheet;

    fn deref(&self) -> &Self::Target {
        &self.sheet
    }
}

impl DerefMut for DisplaySheetState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sheet
    }
}

fn format_sheet_duration(frame_count: usize, fps: u32) -> String {
    let fps = fps as usize;
    if fps == 0 {
        return format!("0+{frame_count}");
    }

    let seconds = frame_count / fps;
    let frames = frame_count % fps;
    format!("{seconds}+{frames}")
}

#[cfg(test)]
mod tests {
    use super::format_sheet_duration;

    #[test]
    fn formats_sheet_duration_as_seconds_plus_frames() {
        assert_eq!(format_sheet_duration(4, 24), "0+4");
        assert_eq!(format_sheet_duration(23, 24), "0+23");
        assert_eq!(format_sheet_duration(24, 24), "1+0");
        assert_eq!(format_sheet_duration(36, 24), "1+12");
    }
}
