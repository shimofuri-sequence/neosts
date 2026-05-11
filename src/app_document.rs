use super::{TableApp, current_window_hwnd};
use arboard::Clipboard;
use eframe::egui;
use neosts::{DisplayMode, TableSelection, ae, strings};
use sheet::CellValue;

impl TableApp {
    const STANDARD_EDIT_MODE: DisplayMode = DisplayMode::FullFrame;

    fn sync_sheet_name_to_path(&mut self, path: &std::path::Path) {
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            self.sheet.set_name(stem);
        }
    }

    pub(super) fn after_effects_is_available(&self, frame: &eframe::Frame) -> bool {
        ae::after_effects_is_available(current_window_hwnd(frame))
    }

    pub(super) fn set_opened_file_status(&mut self) {
        let locale = self.locale();
        self.status_message = self
            .file_state
            .current_source()
            .map(|path| strings::status_opened_path(locale, path));
    }

    pub(super) fn record_current_file_as_recent(&mut self) {
        if let Some(path) = self.file_state.current_source() {
            self.app_settings.record_recent_file(path);
        }
    }

    pub(super) fn selected_column_index(&self) -> Option<usize> {
        self.selected_header_column_index().or_else(|| {
            self.table_view.selected_range().and_then(|selection| {
                (selection.start_col == selection.end_col).then_some(selection.start_col)
            })
        })
    }

    pub(super) fn selected_header_column_index(&self) -> Option<usize> {
        self.table_view.selected_header_column()
    }

    pub(super) fn send_selected_column_to_after_effects(&mut self, frame: &eframe::Frame) {
        let Some(col) = self.selected_header_column_index() else {
            self.status_message =
                Some(strings::status_select_column_to_send_ae(self.locale()).to_owned());
            return;
        };
        self.send_column_to_after_effects(col, frame);
    }

    pub(super) fn send_column_to_after_effects(&mut self, col: usize, frame: &eframe::Frame) {
        if !self.after_effects_is_available(frame) {
            self.status_message =
                Some(strings::status_ae_not_found_for_send(self.locale()).to_owned());
            return;
        }

        let values = self.sheet.column_values_skipping_visual_only(col);
        let column_name = self.sheet.column_name(col).to_owned();

        match ae::send_column_to_after_effects(
            &column_name,
            &values,
            self.sheet.fps(),
            self.editor_settings.ae_kara_cell_mode,
            current_window_hwnd(frame),
        ) {
            Ok(_) => {
                self.status_message = Some(strings::status_sent_column_to_ae(
                    self.locale(),
                    &column_name,
                ));
            }
            Err(error) => {
                self.status_message = Some(strings::status_failed_send_to_ae(
                    self.locale(),
                    error.localized_message(self.locale()),
                ));
            }
        }
    }

    pub(super) fn save_sheet_as_sts_dialog(&mut self, suggested_name: &str, frame: &eframe::Frame) {
        let locale = self.locale();
        if self.file_state.save_sheet_as_sts_dialog(
            &self.sheet,
            &mut self.current_sheet_loaded_from_sts,
            suggested_name,
            frame,
            locale,
        ) {
            if let Some(path) = self.file_state.current_source_owned() {
                self.sync_sheet_name_to_path(&path);
            }
            self.reset_document_history();
            self.record_current_file_as_recent();
        }
    }

    pub(super) fn save_sheet(&mut self, frame: &eframe::Frame) {
        if self.file_state.current_source().is_some() {
            let locale = self.locale();
            match self.file_state.save_sheet_to_current_path(
                &self.sheet,
                &mut self.current_sheet_loaded_from_sts,
                locale,
            ) {
                Ok(saved_path) => {
                    self.sync_sheet_name_to_path(&saved_path);
                    self.reset_document_history();
                    self.record_current_file_as_recent();
                    self.status_message =
                        Some(strings::status_saved_path(self.locale(), &saved_path));
                }
                Err(error) => {
                    self.status_message = Some(error);
                }
            }
            return;
        }

        let suggested_name = self.sheet.name().to_owned();
        self.save_sheet_as_sts_dialog(&suggested_name, frame);
    }

    pub(super) fn save_sheet_as(&mut self, frame: &eframe::Frame) {
        let suggested_name = self.sheet.name().to_owned();
        self.save_sheet_as_sts_dialog(&suggested_name, frame);
    }

    pub(super) fn copy_selection_to_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(text) = self.selection_text_for_clipboard() {
            ctx.copy_text(text);
            self.status_message = Some(strings::status_copied_selection(self.locale()).to_owned());
        }
    }

    pub(super) fn copy_selected_column_to_clipboard(&mut self, ctx: &egui::Context) {
        let Some(col) = self.selected_column_index() else {
            return;
        };
        let text = self.sheet.copy_range_as_tsv(
            Self::STANDARD_EDIT_MODE,
            col,
            col,
            0,
            self.sheet.row_count().saturating_sub(1),
        );
        ctx.copy_text(text);
        self.status_message = Some(strings::status_copied_column_values(self.locale()).to_owned());
    }

    pub(super) fn cut_selection_to_clipboard(&mut self, ctx: &egui::Context) {
        let Some(selection) = self.table_view.selected_range() else {
            return;
        };
        let Some(text) = self.selection_text(selection) else {
            return;
        };

        self.sheet.apply_delete(
            Self::STANDARD_EDIT_MODE,
            selection.start_col,
            selection.end_col,
            selection.start_row,
            selection.end_row,
        );
        ctx.copy_text(text);
        self.status_message = Some(strings::status_cut_selection(self.locale()).to_owned());
    }

    pub(super) fn paste_from_clipboard(&mut self, ctx: &egui::Context) {
        let Ok(text) = Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) else {
            self.status_message =
                Some(strings::status_failed_clipboard_read(self.locale()).to_owned());
            return;
        };

        self.paste_text_into_selection(ctx, &text);
    }

    pub(super) fn paste_text_into_selection(&mut self, ctx: &egui::Context, text: &str) {
        let Some(selection) = self.table_view.selected_range() else {
            return;
        };
        if !selection_has_editable_rows(selection, &self.sheet) {
            self.status_message =
                Some(strings::status_cannot_paste_into_ignored_rows(self.locale()).to_owned());
            return;
        }

        if let Some((start_row, values)) =
            parse_ae_keyframe_paste(text, selection, self.sheet.row_count())
        {
            self.sheet.apply_paste(
                Self::STANDARD_EDIT_MODE,
                selection.start_col,
                start_row,
                &values,
            );
            self.status_message = Some(strings::status_pasted_clipboard(self.locale()).to_owned());
            ctx.request_repaint();
            return;
        }

        if let Some(values) = parse_clipboard_cells(text) {
            self.sheet.apply_paste(
                Self::STANDARD_EDIT_MODE,
                selection.start_col,
                selection.start_row,
                &values,
            );
            self.status_message = Some(strings::status_pasted_clipboard(self.locale()).to_owned());
            ctx.request_repaint();
        }
    }

    pub(super) fn repeat_selection_down(&mut self, ctx: &egui::Context) {
        let Some(selection) = self.table_view.selected_range() else {
            return;
        };
        if !selection_has_editable_rows(selection, &self.sheet) {
            self.status_message =
                Some(strings::status_cannot_repeat_down_ignored_only(self.locale()).to_owned());
            return;
        }
        if selection.end_row + 1 >= self.sheet.row_count() {
            self.status_message =
                Some(strings::status_no_rows_below_selection(self.locale()).to_owned());
            return;
        }

        self.sheet.repeat_range_down(
            Self::STANDARD_EDIT_MODE,
            selection.start_col,
            selection.end_col,
            selection.start_row,
            selection.end_row,
        );
        self.status_message =
            Some(strings::status_repeated_selection_down(self.locale()).to_owned());
        ctx.request_repaint();
    }

    fn selection_text_for_clipboard(&self) -> Option<String> {
        let selection = self.table_view.selected_range()?;
        self.selection_text(selection)
    }

    fn selection_text(&self, selection: TableSelection) -> Option<String> {
        if self.sheet.column_count() == 0 || self.sheet.row_count() == 0 {
            return None;
        }

        Some(self.sheet.copy_range_as_tsv(
            Self::STANDARD_EDIT_MODE,
            selection.start_col,
            selection.end_col,
            selection.start_row,
            selection.end_row,
        ))
    }

    pub(super) fn delete_selected_column_values(&mut self) {
        let Some(col) = self.selected_column_index() else {
            return;
        };
        let last_row = self.sheet.row_count().saturating_sub(1);
        self.sheet
            .apply_delete(Self::STANDARD_EDIT_MODE, col, col, 0, last_row);
        self.status_message = Some(strings::status_deleted_column_values(self.locale()).to_owned());
    }

    pub(super) fn new_sheet_from_after_effects_selection(
        &mut self,
        frame: &eframe::Frame,
        ctx: &egui::Context,
    ) {
        if !self.after_effects_is_available(frame) {
            self.status_message =
                Some(strings::status_ae_not_found_for_receive(self.locale()).to_owned());
            return;
        }

        match ae::receive_selection_from_after_effects(
            self.editor_settings.ae_kara_cell_mode,
            current_window_hwnd(frame),
            self.locale(),
        ) {
            Ok(payload) => match ae::selection_payload_to_sheet(
                &payload,
                self.editor_settings.ae_kara_cell_mode,
            ) {
                Ok(sheet) => {
                    self.sheet.replace_sheet(sheet);
                    self.sheet
                        .set_name(payload.sheet_name(self.editor_settings.ae_sheet_name_source));
                    self.file_state.set_current_source(None);
                    self.current_sheet_loaded_from_sts = false;
                    self.table_view.reset_for_new_sheet();
                    self.table_view
                        .sync_column_widths(&self.sheet, &self.table_settings);
                    self.reset_document_history();
                    self.status_message =
                        Some(strings::status_created_sheet_from_ae(self.locale()).to_owned());
                    ctx.request_repaint();
                }
                Err(error) => {
                    self.status_message = Some(strings::status_failed_convert_ae_data(
                        self.locale(),
                        error.localized_message(self.locale()),
                    ));
                }
            },
            Err(error) => {
                self.status_message = Some(strings::status_failed_receive_from_ae(
                    self.locale(),
                    error.localized_message(self.locale()),
                ));
            }
        }
    }

    pub(super) fn open_sheet_from_dialog(&mut self, frame: &eframe::Frame) {
        let locale = self.locale();
        if self.file_state.open_sheet_from_dialog(
            &mut self.sheet,
            &mut self.table_view,
            &mut self.current_sheet_loaded_from_sts,
            self.sheet_settings.default_fps,
            frame,
            locale,
        ) {
            self.reset_document_history();
            self.record_current_file_as_recent();
            self.set_opened_file_status();
        }
    }

    pub(super) fn open_sheet_path(&mut self, path: &std::path::Path) {
        let locale = self.locale();
        if self.file_state.open_sheet_path(
            &mut self.sheet,
            &mut self.table_view,
            &mut self.current_sheet_loaded_from_sts,
            self.sheet_settings.default_fps,
            path,
            locale,
        ) {
            self.reset_document_history();
            self.record_current_file_as_recent();
            self.set_opened_file_status();
        } else {
            self.app_settings.remove_recent_file(path);
            self.status_message = Some(strings::status_failed_open_path(self.locale(), path));
        }
    }
}

fn selection_has_editable_rows(
    selection: TableSelection,
    sheet: &neosts::DisplaySheetState,
) -> bool {
    (selection.start_row..=selection.end_row).any(|row| sheet.row_participates_in_timeline(row))
}

fn parse_ae_keyframe_paste(
    text: &str,
    selection: TableSelection,
    sheet_row_count: usize,
) -> Option<(usize, Vec<Vec<CellValue>>)> {
    let parsed = ae::parse_keyframe_data(text).ok()?;
    let start_row = selection
        .start_row
        .saturating_add(parsed.start_frame)
        .min(sheet_row_count);
    let selected_row_count = selection
        .end_row
        .saturating_add(1)
        .min(sheet_row_count)
        .saturating_sub(start_row);
    let max_row_count = sheet_row_count.saturating_sub(start_row);
    let column_count = selection
        .end_col
        .saturating_sub(selection.start_col)
        .saturating_add(1)
        .max(1);
    let mut parsed_values = parsed.values;
    if selected_row_count > parsed_values.len()
        && let Some(last_value) = parsed_values.last().cloned()
    {
        parsed_values.resize(selected_row_count, last_value);
    }
    let values = parsed_values
        .into_iter()
        .take(max_row_count)
        .map(|value| vec![value; column_count])
        .collect::<Vec<_>>();

    (!values.is_empty()).then_some((start_row, values))
}

fn parse_clipboard_cells(text: &str) -> Option<Vec<Vec<CellValue>>> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut rows = normalized
        .split('\n')
        .map(|line| {
            line.split('\t')
                .map(|cell| {
                    let trimmed = cell.trim();
                    if trimmed.is_empty() {
                        Some(CellValue::blank())
                    } else {
                        trimmed.parse::<i64>().ok().map(CellValue::Int)
                    }
                })
                .collect::<Option<Vec<_>>>()
        })
        .collect::<Option<Vec<_>>>()?;

    while rows.last().is_some_and(Vec::is_empty) {
        rows.pop();
    }
    while rows
        .last()
        .is_some_and(|row| row.len() == 1 && row[0].is_blank())
        && rows.len() > 1
    {
        rows.pop();
    }

    (!rows.is_empty()).then_some(rows)
}

#[cfg(test)]
mod tests {
    use super::parse_ae_keyframe_paste;
    use neosts::TableSelection;
    use sheet::CellValue;

    #[test]
    fn ae_keyframe_paste_extends_last_value_through_selected_rows() {
        let text = "Adobe After Effects 9.0 Keyframe Data\r\n\
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
             End of Keyframe Data\r\n";
        let selection = TableSelection {
            start_col: 0,
            end_col: 0,
            start_row: 0,
            end_row: 8,
        };

        let (start_row, values) = parse_ae_keyframe_paste(text, selection, 20).unwrap();

        assert_eq!(start_row, 0);
        assert_eq!(
            values,
            vec![
                vec![CellValue::Int(1)],
                vec![CellValue::Int(1)],
                vec![CellValue::Int(1)],
                vec![CellValue::Int(3)],
                vec![CellValue::Int(3)],
                vec![CellValue::Int(5)],
                vec![CellValue::Int(5)],
                vec![CellValue::Int(5)],
                vec![CellValue::Int(5)],
            ]
        );
    }

    #[test]
    fn ae_keyframe_paste_keeps_full_data_when_selection_is_shorter() {
        let text = "Adobe After Effects 9.0 Keyframe Data\r\n\
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
             \t6\t0.2500000\t\r\n\
             \r\n\
             End of Keyframe Data\r\n";
        let selection = TableSelection {
            start_col: 0,
            end_col: 0,
            start_row: 0,
            end_row: 6,
        };

        let (start_row, values) = parse_ae_keyframe_paste(text, selection, 20).unwrap();

        assert_eq!(start_row, 0);
        assert_eq!(
            values,
            vec![
                vec![CellValue::Int(1)],
                vec![CellValue::Int(1)],
                vec![CellValue::Int(1)],
                vec![CellValue::Int(3)],
                vec![CellValue::Int(3)],
                vec![CellValue::Int(3)],
                vec![CellValue::Int(7)],
            ]
        );
    }
}
