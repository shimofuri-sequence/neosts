use super::selection::{CellIndex, SelectionRect, clamp_cell_to_sheet, clamp_selection_to_sheet};
use super::{
    TableViewEvent, TableViewState, column_action_state_for_context_col, column_selection,
    row_action_state_for_context_row,
};
use crate::column_actions::{
    ColumnAction, ColumnActionEvent, ColumnActionState, execute_column_action,
};
use crate::row_actions::{RowAction, RowActionEvent, RowActionState, execute_row_action};
use crate::settings::table::TableSettings;
use eframe::egui::Vec2;
use sheet::{CellValue, RowKind, Sheet};
use std::time::{SystemTime, UNIX_EPOCH};

impl TableViewState {
    fn sync_after_row_count_change(&mut self, sheet: &Sheet) {
        self.set_selection(
            self.selection
                .and_then(|selection| clamp_selection_to_sheet(selection, sheet)),
        );
        self.drag_anchor = self
            .drag_anchor
            .and_then(|anchor| clamp_cell_to_sheet(anchor, sheet));
        self.clear_pending_selection_drag();
        self.hovered_cell = self
            .hovered_cell
            .and_then(|cell| clamp_cell_to_sheet(cell, sheet));
        self.hovered_row = self.hovered_row.filter(|&row| row < sheet.row_count());
        self.pressed_body_cell = self
            .pressed_body_cell
            .and_then(|cell| clamp_cell_to_sheet(cell, sheet));
        self.pressed_row_header = self
            .pressed_row_header
            .filter(|&row| row < sheet.row_count());
        self.row_header_context_row = self
            .row_header_context_row
            .filter(|&row| row < sheet.row_count());
        self.scroll_selection_into_view = true;
        self.suppress_vertical_scroll_adjustment = false;
    }

    pub fn execute_row_action(
        &mut self,
        sheet: &mut Sheet,
        state: &RowActionState,
        action: RowAction,
    ) {
        let row_count_before = sheet.row_count();
        if let Some(event) = execute_row_action(sheet, state, action) {
            match event {
                RowActionEvent::OpenAppendRowsRequested { row, insert_above } => {
                    self.pending_events
                        .push(TableViewEvent::OpenAppendRowsRequested { row, insert_above });
                }
            }
        }
        if sheet.row_count() != row_count_before {
            self.sync_after_row_count_change(sheet);
        }
    }

    pub fn execute_context_row_action(&mut self, sheet: &mut Sheet, action: RowAction) {
        let action_state =
            row_action_state_for_context_row(self.selection, self.row_header_context_row);
        self.execute_row_action(sheet, &action_state, action);
    }

    pub fn execute_column_action(
        &mut self,
        sheet: &mut Sheet,
        settings: &TableSettings,
        state: &ColumnActionState,
        action: ColumnAction,
    ) {
        if let Some(event) = execute_column_action(sheet, state, action) {
            match event {
                ColumnActionEvent::OpenRenameColumnRequested {
                    column,
                    current_name,
                } => {
                    self.pending_events
                        .push(TableViewEvent::OpenRenameColumnRequested {
                            column,
                            current_name,
                        });
                }
                ColumnActionEvent::SheetChanged { selected_column } => {
                    self.sync_after_sheet_resize(sheet, settings);
                    self.set_selection(None);
                    self.active_column_headers = selected_column.map(|col| (col, col));
                    self.drag_anchor = None;
                    self.drag_mode = None;
                    self.pending_selection_drag = None;
                    self.edit_buffer.clear();
                    self.scroll_selection_into_view = false;
                    self.suppress_vertical_scroll_adjustment = false;
                }
            }
        }
    }

    pub fn execute_context_column_action(
        &mut self,
        sheet: &mut Sheet,
        settings: &TableSettings,
        action: ColumnAction,
    ) {
        let action_state =
            column_action_state_for_context_col(self.selection, self.column_header_context_col);
        self.execute_column_action(sheet, settings, &action_state, action);
    }

    pub fn rename_column(&mut self, sheet: &mut Sheet, column: usize, name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.is_empty() || column >= sheet.column_count() {
            return false;
        }

        sheet.set_column_name(column, trimmed);
        true
    }

    pub fn move_selected_columns(
        &mut self,
        sheet: &mut Sheet,
        settings: &TableSettings,
        direction: isize,
    ) -> bool {
        let Some(selection) = self.selection.map(SelectionRect::normalized) else {
            return false;
        };
        let start_col = selection.start.col;
        let end_col = selection.end.col;

        if !sheet.move_column_range(start_col, end_col, direction) {
            return false;
        }

        self.sync_column_widths(sheet, settings);
        if direction < 0 {
            self.column_widths[start_col - 1..=end_col].rotate_left(1);
        } else if direction > 0 {
            self.column_widths[start_col..=end_col + 1].rotate_right(1);
        }

        let next_start = start_col.saturating_add_signed(direction);
        let next_end = end_col.saturating_add_signed(direction);
        self.sync_after_sheet_resize(sheet, settings);
        self.set_selection(Some(SelectionRect {
            start: CellIndex {
                col: next_start,
                row: selection.start.row,
            },
            end: CellIndex {
                col: next_end,
                row: selection.end.row,
            },
        }));
        self.set_active_column_headers(Some((next_start, next_end)));
        if self.last_body_viewport_size.x > 0.0 {
            let selection_left = self.column_left(next_start, settings.cell_scale, settings);
            let selection_right = self.column_left(next_end, settings.cell_scale, settings)
                + self.column_width(next_end, settings.cell_scale, settings);
            let viewport_width = self.last_body_viewport_size.x;
            let max_offset_x = ((0..sheet.column_count())
                .map(|col| self.column_width(col, settings.cell_scale, settings))
                .sum::<f32>()
                - viewport_width)
                .max(0.0);

            if selection_left < self.horizontal_scroll {
                self.horizontal_scroll = selection_left;
            } else if selection_right > self.horizontal_scroll + viewport_width {
                self.horizontal_scroll = selection_right - viewport_width;
            }

            self.horizontal_scroll = self.horizontal_scroll.clamp(0.0, max_offset_x);
        }
        self.drag_anchor = None;
        self.drag_mode = None;
        self.pending_selection_drag = None;
        self.edit_buffer.clear();
        self.scroll_selection_into_view = true;
        self.pending_vertical_scroll_direction = None;
        self.preserve_horizontal_scroll_for_selection = false;
        self.suppress_vertical_scroll_adjustment = true;
        true
    }

    pub fn select_column(&mut self, sheet: &Sheet, column: usize) -> bool {
        let Some(selection) = column_selection(column, column, sheet) else {
            return false;
        };
        self.set_selection(Some(selection));
        self.active_column_headers = Some((column, column));
        self.drag_anchor = None;
        self.drag_mode = None;
        self.pending_selection_drag = None;
        self.edit_buffer.clear();
        self.scroll_selection_into_view = true;
        self.suppress_vertical_scroll_adjustment = false;
        true
    }

    pub fn reset_for_new_sheet(&mut self) {
        self.set_selection(None);
        self.drag_anchor = None;
        self.drag_mode = None;
        self.pending_selection_drag = None;
        self.middle_pan_last = None;
        self.hovered_cell = None;
        self.hovered_col = None;
        self.hovered_row = None;
        self.active_column_headers = None;
        self.edit_buffer.clear();
        self.horizontal_scroll = 0.0;
        self.vertical_scroll = 0.0;
        self.last_body_viewport_size = Vec2::ZERO;
        self.pending_vertical_scroll_direction = None;
        self.suppress_vertical_scroll_adjustment = false;
        self.pending_scroll_top_row = None;
        self.pending_scroll_to_cell = None;
        self.preserve_horizontal_scroll_for_pending_cell = false;
        self.preserve_horizontal_scroll_for_selection = false;
        self.minimap_hit_rect = None;
        self.scroll_id_salt = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        self.scroll_selection_into_view = false;
        self.minimap_drag_mode = None;
        self.column_header_context_col = None;
        self.row_header_context_row = None;
        self.pressed_body_cell = None;
        self.pressed_column_header = None;
        self.pressed_row_header = None;
        self.popup_was_open = false;
        self.column_widths.clear();
    }

    pub fn sync_after_sheet_resize(&mut self, sheet: &Sheet, settings: &TableSettings) {
        self.set_selection(
            self.selection
                .and_then(|selection| clamp_selection_to_sheet(selection, sheet)),
        );
        self.drag_anchor = self
            .drag_anchor
            .and_then(|anchor| clamp_cell_to_sheet(anchor, sheet));
        self.clear_pending_selection_drag();
        self.hovered_cell = self
            .hovered_cell
            .and_then(|cell| clamp_cell_to_sheet(cell, sheet));
        self.hovered_col = self.hovered_col.filter(|&col| col < sheet.column_count());
        self.hovered_row = self.hovered_row.filter(|&row| row < sheet.row_count());
        self.active_column_headers = self.active_column_headers.and_then(|(start, end)| {
            (sheet.column_count() > 0).then_some((
                start.min(sheet.column_count() - 1),
                end.min(sheet.column_count() - 1),
            ))
        });
        self.pressed_body_cell = self
            .pressed_body_cell
            .and_then(|cell| clamp_cell_to_sheet(cell, sheet));
        self.pressed_column_header = self
            .pressed_column_header
            .filter(|&col| col < sheet.column_count());
        self.column_header_context_col = self
            .column_header_context_col
            .filter(|&col| col < sheet.column_count());
        self.pressed_row_header = self
            .pressed_row_header
            .filter(|&row| row < sheet.row_count());
        self.row_header_context_row = self
            .row_header_context_row
            .filter(|&row| row < sheet.row_count());
        self.sync_column_widths(sheet, settings);
        self.scroll_selection_into_view = true;
        self.suppress_vertical_scroll_adjustment = false;
    }

    pub fn append_special_rows(
        &mut self,
        sheet: &mut Sheet,
        row: usize,
        count: usize,
        insert_above: bool,
    ) {
        let insert_at = if insert_above { row } else { row + 1 };
        let copy_from = if insert_above && row > 0 {
            row - 1
        } else {
            row
        };
        let values: Vec<CellValue> = (0..sheet.column_count())
            .map(|col| {
                sheet
                    .cell(col, copy_from)
                    .cloned()
                    .unwrap_or_else(CellValue::blank)
            })
            .collect();
        for i in 0..count {
            sheet.insert_row_with_kind(insert_at + i, values.clone(), RowKind::SpecialInserted);
        }
        if insert_above {
            self.set_selection(self.selection.map(|sel| SelectionRect {
                start: CellIndex {
                    col: sel.start.col,
                    row: sel.start.row + count,
                },
                end: CellIndex {
                    col: sel.end.col,
                    row: sel.end.row + count,
                },
            }));
        }
    }

    pub fn sync_column_widths(&mut self, sheet: &Sheet, settings: &TableSettings) {
        let target_len = sheet.column_count();
        let default_width = settings.default_column_width();
        if self.column_widths.len() < target_len {
            self.column_widths.resize(target_len, default_width);
        } else if self.column_widths.len() > target_len {
            self.column_widths.truncate(target_len);
        }
    }

    pub fn reset_column_widths_to_default(&mut self, sheet: &Sheet, settings: &TableSettings) {
        self.sync_column_widths(sheet, settings);
        self.column_widths.fill(settings.default_column_width());
    }
}

#[cfg(test)]
mod tests {
    use super::{CellIndex, SelectionRect, TableViewState};
    use crate::settings::table::TableSettings;
    use sheet::{CellValue, Sheet, SheetColumn};

    fn sample_sheet() -> Sheet {
        Sheet::new(vec![
            SheetColumn::new("A", vec![CellValue::from(1), CellValue::from(1)]),
            SheetColumn::new("B", vec![CellValue::from(2), CellValue::from(2)]),
            SheetColumn::new("C", vec![CellValue::from(3), CellValue::from(3)]),
            SheetColumn::new("D", vec![CellValue::from(4), CellValue::from(4)]),
        ])
    }

    #[test]
    fn moves_selected_columns_and_keeps_block_selected() {
        let mut sheet = sample_sheet();
        let settings = TableSettings::default();
        let mut table = TableViewState::default();
        table.sync_column_widths(&sheet, &settings);
        table.set_selection(Some(SelectionRect {
            start: CellIndex { col: 1, row: 0 },
            end: CellIndex { col: 2, row: 1 },
        }));

        assert!(table.move_selected_columns(&mut sheet, &settings, -1));
        assert_eq!(sheet.column_name(0), "B");
        assert_eq!(sheet.column_name(1), "C");
        assert_eq!(sheet.column_name(2), "A");
        assert_eq!(sheet.column_name(3), "D");
        let selection = table.selected_range().unwrap();
        assert_eq!(selection.start_col, 0);
        assert_eq!(selection.end_col, 1);
        assert_eq!(selection.start_row, 0);
        assert_eq!(selection.end_row, 1);

        assert!(table.move_selected_columns(&mut sheet, &settings, 1));
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(1), "B");
        assert_eq!(sheet.column_name(2), "C");
        assert_eq!(sheet.column_name(3), "D");
        let selection = table.selected_range().unwrap();
        assert_eq!(selection.start_col, 1);
        assert_eq!(selection.end_col, 2);
        assert_eq!(selection.start_row, 0);
        assert_eq!(selection.end_row, 1);
    }
}
