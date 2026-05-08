use crate::settings::editor::DisplayMode;
use sheet::{BLANK_CELL_VALUE, CellValue, Sheet};

pub(crate) fn apply_enter_to_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
    input_value: Option<i64>,
) {
    match display_mode {
        DisplayMode::FullFrame => {
            if let Some(value) = input_value {
                let fill_values = (start_col..=end_col)
                    .map(|_| CellValue::Int(value))
                    .collect::<Vec<_>>();
                overwrite_range(sheet, start_col, end_col, start_row, end_row, fill_values);
            } else {
                let fill_values = (start_col..=end_col)
                    .map(|col| value_above_or_blank(sheet, start_row, col))
                    .collect::<Vec<_>>();
                overwrite_range(sheet, start_col, end_col, start_row, end_row, fill_values);
            }
        }
        DisplayMode::Keyframe => {
            if let Some(value) = input_value {
                for col in start_col..=end_col {
                    autofill_column_range(sheet, col, start_row, end_row, CellValue::Int(value));
                }
            } else {
                for col in start_col..=end_col {
                    autofill_column_range(
                        sheet,
                        col,
                        start_row,
                        end_row,
                        value_above_or_blank(sheet, start_row, col),
                    );
                }
            }
        }
    }
}

pub(crate) fn apply_increment_from_above_to_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) {
    match display_mode {
        DisplayMode::FullFrame => {
            let fill_values = (start_col..=end_col)
                .map(|col| {
                    let above = numeric_value_above(sheet, start_row, col);
                    let next = if above >= 1 {
                        above.saturating_add(1)
                    } else {
                        above
                    };
                    CellValue::Int(next)
                })
                .collect::<Vec<_>>();
            overwrite_range(sheet, start_col, end_col, start_row, end_row, fill_values);
        }
        DisplayMode::Keyframe => {
            for col in start_col..=end_col {
                let above = numeric_value_above(sheet, start_row, col);
                let next = if above >= 1 {
                    above.saturating_add(1)
                } else {
                    above
                };
                autofill_column_range(sheet, col, start_row, end_row, CellValue::Int(next));
            }
        }
    }
}

pub(crate) fn apply_decrement_from_above_to_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) {
    match display_mode {
        DisplayMode::FullFrame => {
            let fill_values = (start_col..=end_col)
                .map(|col| {
                    let above = numeric_value_above(sheet, start_row, col);
                    let next = if above >= 2 {
                        above.saturating_sub(1)
                    } else {
                        above
                    };
                    CellValue::Int(next)
                })
                .collect::<Vec<_>>();
            overwrite_range(sheet, start_col, end_col, start_row, end_row, fill_values);
        }
        DisplayMode::Keyframe => {
            for col in start_col..=end_col {
                let above = numeric_value_above(sheet, start_row, col);
                let next = if above >= 2 {
                    above.saturating_sub(1)
                } else {
                    above
                };
                autofill_column_range(sheet, col, start_row, end_row, CellValue::Int(next));
            }
        }
    }
}

pub(crate) fn apply_delete_to_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) {
    match display_mode {
        DisplayMode::FullFrame => {
            let fill_values = (start_col..=end_col)
                .map(|_| CellValue::Int(0))
                .collect::<Vec<_>>();
            overwrite_range(sheet, start_col, end_col, start_row, end_row, fill_values);
        }
        DisplayMode::Keyframe => {
            for col in start_col..=end_col {
                autofill_delete_column_range(sheet, col, start_row, end_row);
            }
        }
    }
}

pub(crate) fn copy_range_as_tsv_for_sheet(
    sheet: &Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) -> String {
    if display_mode == DisplayMode::Keyframe
        && !keyframe_range_has_visible_keys(sheet, start_col, end_col, start_row, end_row)
    {
        return String::new();
    }
    let punched_aware_fullframe = display_mode == DisplayMode::FullFrame
        && range_contains_punched_rows(sheet, start_row, end_row);

    let mut lines = Vec::new();
    for row in editable_rows_in_range(sheet, start_row, end_row) {
        let mut cells = Vec::new();
        for col in start_col..=end_col {
            let value = match display_mode {
                DisplayMode::FullFrame if punched_aware_fullframe => {
                    fullframe_effective_value(sheet, col, row)
                }
                DisplayMode::FullFrame => sheet
                    .cell(col, row)
                    .cloned()
                    .unwrap_or_else(CellValue::blank),
                DisplayMode::Keyframe => keyframe_visible_value(sheet, col, row),
            };
            let text = if display_mode == DisplayMode::Keyframe && value.is_blank() {
                String::new()
            } else {
                value.as_i64().to_string()
            };
            cells.push(text);
        }
        lines.push(cells.join("\t"));
    }
    lines.join("\n")
}

pub(crate) fn apply_paste_to_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    start_row: usize,
    values: &[Vec<CellValue>],
) {
    match display_mode {
        DisplayMode::FullFrame => {
            let target_rows = paste_target_rows(sheet, start_row);
            for (row, row_values) in target_rows.into_iter().zip(values.iter()) {
                for (col_offset, value) in row_values.iter().enumerate() {
                    let col = start_col + col_offset;
                    if col >= sheet.column_count() {
                        break;
                    }
                    sheet.set_cell(col, row, value.clone());
                }
            }
        }
        DisplayMode::Keyframe => {
            if values.iter().flatten().all(CellValue::is_blank) {
                let target_rows = paste_target_rows(sheet, start_row);
                for (row, row_values) in target_rows.into_iter().zip(values.iter()) {
                    for (col_offset, value) in row_values.iter().enumerate() {
                        let col = start_col + col_offset;
                        if col >= sheet.column_count() {
                            break;
                        }
                        autofill_column_range(sheet, col, row, row, value.clone());
                    }
                }
                return;
            }

            let mut key_updates = Vec::new();
            let target_rows = paste_target_rows(sheet, start_row);
            for (row, row_values) in target_rows.into_iter().zip(values.iter()) {
                for (col_offset, value) in row_values.iter().enumerate() {
                    let col = start_col + col_offset;
                    if col >= sheet.column_count() {
                        break;
                    }
                    if !value.is_blank() {
                        key_updates.push((row, col, value.clone()));
                    }
                }
            }

            for (row, col, value) in key_updates.into_iter().rev() {
                autofill_column_range(sheet, col, row, row, value);
            }
        }
    }
}

pub(crate) fn transfer_range_on_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
    target_col: usize,
    target_row: usize,
    delete_source: bool,
) {
    if start_col == target_col && start_row == target_row {
        return;
    }

    let values = match display_mode {
        DisplayMode::FullFrame => collect_raw_range(sheet, start_col, end_col, start_row, end_row),
        DisplayMode::Keyframe => {
            collect_keyframe_drag_range(sheet, start_col, end_col, start_row, end_row)
        }
    };

    if display_mode == DisplayMode::Keyframe && values.iter().flatten().all(CellValue::is_blank) {
        return;
    }

    if delete_source {
        apply_delete_to_sheet(sheet, display_mode, start_col, end_col, start_row, end_row);
    }
    apply_paste_to_sheet(sheet, display_mode, target_col, target_row, &values);
}

pub(crate) fn repeat_range_down_on_sheet(
    sheet: &mut Sheet,
    display_mode: DisplayMode,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) {
    let values = match display_mode {
        DisplayMode::FullFrame => collect_raw_range(sheet, start_col, end_col, start_row, end_row),
        DisplayMode::Keyframe => {
            collect_keyframe_drag_range(sheet, start_col, end_col, start_row, end_row)
        }
    };

    if values.is_empty() {
        return;
    }

    let target_rows = editable_rows_in_range(
        sheet,
        end_row.saturating_add(1),
        sheet.row_count().saturating_sub(1),
    );
    if target_rows.is_empty() {
        return;
    }

    for (index, row) in target_rows.into_iter().enumerate() {
        let pattern_row = &values[index % values.len()];
        for (col_offset, value) in pattern_row.iter().enumerate() {
            let col = start_col + col_offset;
            if col > end_col || col >= sheet.column_count() {
                break;
            }
            sheet.set_cell(col, row, value.clone());
        }
    }
}

fn overwrite_range(
    sheet: &mut Sheet,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
    fill_values: Vec<CellValue>,
) {
    let target_rows = editable_rows_in_range(sheet, start_row, end_row);
    for (col_offset, col) in (start_col..=end_col).enumerate() {
        let value = fill_values[col_offset].clone();
        for &row in &target_rows {
            sheet.set_cell(col, row, value.clone());
        }
    }
}

fn collect_raw_range(
    sheet: &Sheet,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) -> Vec<Vec<CellValue>> {
    let punched_aware_fullframe = range_contains_punched_rows(sheet, start_row, end_row);
    editable_rows_in_range(sheet, start_row, end_row)
        .into_iter()
        .map(|row| {
            (start_col..=end_col)
                .map(|col| {
                    if punched_aware_fullframe {
                        fullframe_effective_value(sheet, col, row)
                    } else {
                        sheet
                            .cell(col, row)
                            .cloned()
                            .unwrap_or_else(CellValue::blank)
                    }
                })
                .collect()
        })
        .collect()
}

fn collect_keyframe_drag_range(
    sheet: &Sheet,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) -> Vec<Vec<CellValue>> {
    editable_rows_in_range(sheet, start_row, end_row)
        .into_iter()
        .map(|row| {
            (start_col..=end_col)
                .map(|col| keyframe_visible_value(sheet, col, row))
                .collect()
        })
        .collect()
}

fn keyframe_visible_value(sheet: &Sheet, col: usize, row: usize) -> CellValue {
    if sheet.should_display_cell(row, col) && !sheet.is_cross_cell(row, col) {
        sheet
            .cell(col, row)
            .cloned()
            .unwrap_or_else(CellValue::blank)
    } else {
        CellValue::blank()
    }
}

fn keyframe_range_has_visible_keys(
    sheet: &Sheet,
    start_col: usize,
    end_col: usize,
    start_row: usize,
    end_row: usize,
) -> bool {
    editable_rows_in_range(sheet, start_row, end_row)
        .into_iter()
        .any(|row| {
            (start_col..=end_col)
                .any(|col| sheet.should_display_cell(row, col) && !sheet.is_cross_cell(row, col))
        })
}

fn autofill_column_range(
    sheet: &mut Sheet,
    col: usize,
    start_row: usize,
    end_row: usize,
    value: CellValue,
) {
    let target_end = next_display_row_in_column(sheet, col, end_row)
        .map(|row| row.saturating_sub(1))
        .unwrap_or_else(|| sheet.row_count().saturating_sub(1))
        .max(end_row);
    for row in editable_rows_in_range(sheet, start_row, target_end) {
        sheet.set_cell(col, row, value.clone());
    }
}

fn fullframe_effective_value(sheet: &Sheet, col: usize, row: usize) -> CellValue {
    if sheet.row_participates_in_timeline(row) {
        if let Some(value) = effective_visible_value_at_or_before(sheet, col, row) {
            return value;
        }
    }

    sheet
        .cell(col, row)
        .cloned()
        .unwrap_or_else(CellValue::blank)
}

fn effective_visible_value_at_or_before(
    sheet: &Sheet,
    col: usize,
    row: usize,
) -> Option<CellValue> {
    (0..=row).rev().find_map(|candidate| {
        if !sheet.row_participates_in_timeline(candidate) {
            return None;
        }
        let value = keyframe_visible_value(sheet, col, candidate);
        (!value.is_blank()).then_some(value)
    })
}

fn range_contains_punched_rows(sheet: &Sheet, start_row: usize, end_row: usize) -> bool {
    (start_row..=end_row).any(|row| !sheet.row_participates_in_timeline(row))
}

fn autofill_delete_column_range(sheet: &mut Sheet, col: usize, start_row: usize, end_row: usize) {
    let key_rows = (0..sheet.row_count())
        .filter(|&row| sheet.should_display_cell(row, col))
        .collect::<Vec<_>>();
    let selected_key_rows = key_rows
        .iter()
        .copied()
        .filter(|&row| {
            start_row <= row && row <= end_row && sheet.row_participates_in_timeline(row)
        })
        .collect::<Vec<_>>();

    for key_row in selected_key_rows {
        let next_key_row = key_rows.iter().copied().find(|&row| row > key_row);
        let replacement = if key_row == 0 {
            CellValue::blank()
        } else {
            sheet
                .cell(col, key_row - 1)
                .cloned()
                .unwrap_or_else(CellValue::blank)
        };
        let fill_end = next_key_row
            .map(|row| row.saturating_sub(1))
            .unwrap_or_else(|| sheet.row_count().saturating_sub(1));
        for row in editable_rows_in_range(sheet, key_row, fill_end) {
            sheet.set_cell(col, row, replacement.clone());
        }
    }
}

fn editable_rows_in_range(sheet: &Sheet, start_row: usize, end_row: usize) -> Vec<usize> {
    sheet.timeline_rows_in_range(start_row, end_row)
}

fn paste_target_rows(sheet: &Sheet, start_row: usize) -> Vec<usize> {
    (start_row..sheet.row_count())
        .filter(|&row| sheet.row_participates_in_timeline(row))
        .collect()
}

fn next_display_row_in_column(sheet: &Sheet, col: usize, row: usize) -> Option<usize> {
    ((row + 1)..sheet.row_count()).find(|&next_row| sheet.should_display_cell(next_row, col))
}

fn value_above_or_blank(sheet: &Sheet, row: usize, col: usize) -> CellValue {
    previous_editable_row(sheet, row)
        .and_then(|previous_row| sheet.cell(col, previous_row).cloned())
        .unwrap_or_else(CellValue::blank)
}

fn numeric_value_above(sheet: &Sheet, row: usize, col: usize) -> i64 {
    match previous_editable_row(sheet, row).and_then(|previous_row| sheet.cell(col, previous_row)) {
        Some(CellValue::Int(value)) => *value,
        None => BLANK_CELL_VALUE,
    }
}

fn previous_editable_row(sheet: &Sheet, row: usize) -> Option<usize> {
    sheet.previous_timeline_row(row)
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayMode, apply_decrement_from_above_to_sheet, apply_delete_to_sheet,
        apply_enter_to_sheet, apply_increment_from_above_to_sheet, apply_paste_to_sheet,
        copy_range_as_tsv_for_sheet, repeat_range_down_on_sheet, transfer_range_on_sheet,
    };
    use sheet::{CellValue, RowKind, Sheet, SheetColumn};

    #[test]
    fn fullframe_updates_only_selected_row() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_enter_to_sheet(&mut sheet, DisplayMode::FullFrame, 0, 0, 1, 1, Some(3));

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.display_cell_text(1, 0), "3");
        assert_eq!(sheet.display_cell_text(2, 0), "1");
    }

    #[test]
    fn keyframe_updates_until_next_display_value() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_enter_to_sheet(&mut sheet, DisplayMode::Keyframe, 0, 0, 1, 1, Some(3));

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.display_cell_text(1, 0), "3");
        assert_eq!(sheet.display_cell_text(2, 0), "");
        assert_eq!(sheet.display_cell_text(3, 0), "2");
    }

    #[test]
    fn enter_and_step_from_above_skip_punched_rows() {
        let mut enter_sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![6.into(), 6.into(), 6.into(), 5.into(), 5.into(), 5.into()],
            )],
            24,
        );
        enter_sheet.set_row_kind(1, RowKind::Punched);
        enter_sheet.set_row_kind(2, RowKind::Punched);
        enter_sheet.set_row_kind(3, RowKind::Punched);
        apply_enter_to_sheet(&mut enter_sheet, DisplayMode::FullFrame, 0, 0, 4, 4, None);
        assert_eq!(enter_sheet.cell(0, 4).map(CellValue::as_i64), Some(6));

        let mut inc_sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![6.into(), 6.into(), 6.into(), 5.into(), 5.into(), 5.into()],
            )],
            24,
        );
        inc_sheet.set_row_kind(1, RowKind::Punched);
        inc_sheet.set_row_kind(2, RowKind::Punched);
        inc_sheet.set_row_kind(3, RowKind::Punched);
        apply_increment_from_above_to_sheet(&mut inc_sheet, DisplayMode::FullFrame, 0, 0, 4, 4);
        assert_eq!(inc_sheet.cell(0, 4).map(CellValue::as_i64), Some(7));

        let mut dec_sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![6.into(), 6.into(), 6.into(), 5.into(), 5.into(), 5.into()],
            )],
            24,
        );
        dec_sheet.set_row_kind(1, RowKind::Punched);
        dec_sheet.set_row_kind(2, RowKind::Punched);
        dec_sheet.set_row_kind(3, RowKind::Punched);
        apply_decrement_from_above_to_sheet(&mut dec_sheet, DisplayMode::FullFrame, 0, 0, 4, 4);
        assert_eq!(dec_sheet.cell(0, 4).map(CellValue::as_i64), Some(5));
    }

    #[test]
    fn delete_in_fullframe_fills_selection_with_zero() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_delete_to_sheet(&mut sheet, DisplayMode::FullFrame, 0, 0, 1, 2);

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.display_cell_text(1, 0), "");
        assert_eq!(sheet.display_cell_text(2, 0), "");
    }

    #[test]
    fn delete_in_keyframe_removes_selected_key_and_merges_runs() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![
                    0.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    1.into(),
                    1.into(),
                    0.into(),
                    0.into(),
                    0.into(),
                ],
            )],
            24,
        );

        apply_delete_to_sheet(&mut sheet, DisplayMode::Keyframe, 0, 0, 1, 3);

        for row in 0..sheet.row_count() {
            assert_eq!(sheet.cell(0, row).map(CellValue::as_i64), Some(0));
        }
    }

    #[test]
    fn copy_range_as_tsv_uses_underlying_cell_values() {
        let sheet = Sheet::with_fps(
            vec![
                SheetColumn::new("A", vec![1.into(), 1.into(), CellValue::blank()]),
                SheetColumn::new("B", vec![0.into(), 2.into(), 3.into()]),
            ],
            24,
        );

        assert_eq!(
            copy_range_as_tsv_for_sheet(&sheet, DisplayMode::FullFrame, 0, 1, 0, 2),
            "1\t0\n1\t2\n0\t3"
        );
    }

    #[test]
    fn copy_range_as_tsv_skips_punched_rows() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![
                    2.into(),
                    2.into(),
                    2.into(),
                    3.into(),
                    3.into(),
                    4.into(),
                    4.into(),
                    5.into(),
                    5.into(),
                    6.into(),
                ],
            )],
            24,
        );
        for row in 3..=7 {
            sheet.set_row_kind(row, RowKind::Punched);
        }

        assert_eq!(
            copy_range_as_tsv_for_sheet(&sheet, DisplayMode::FullFrame, 0, 0, 0, 9),
            "2\n2\n2\n5\n6"
        );
    }

    #[test]
    fn copy_range_as_tsv_in_keyframe_uses_only_visible_keys() {
        let sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 4.into(), 4.into()],
            )],
            24,
        );

        assert_eq!(
            copy_range_as_tsv_for_sheet(&sheet, DisplayMode::Keyframe, 0, 0, 0, 4),
            "1\n\n\n4\n"
        );
    }

    #[test]
    fn copy_range_as_tsv_in_keyframe_skips_cross_cells_and_empty_ranges() {
        let sheet = Sheet::with_fps(
            vec![
                SheetColumn::new("A", vec![0.into(), 0.into(), 2.into(), 2.into()]),
                SheetColumn::new(
                    "B",
                    vec![
                        CellValue::blank(),
                        CellValue::blank(),
                        CellValue::blank(),
                        CellValue::blank(),
                    ],
                ),
            ],
            24,
        );

        assert_eq!(
            copy_range_as_tsv_for_sheet(&sheet, DisplayMode::Keyframe, 0, 0, 0, 3),
            "\n\n2\n"
        );
        assert_eq!(
            copy_range_as_tsv_for_sheet(&sheet, DisplayMode::Keyframe, 1, 1, 0, 3),
            ""
        );
    }

    #[test]
    fn paste_in_fullframe_overwrites_only_target_cells() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_paste_to_sheet(
            &mut sheet,
            DisplayMode::FullFrame,
            0,
            1,
            &[vec![CellValue::Int(3)], vec![CellValue::Int(4)]],
        );

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(4));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(2));
    }

    #[test]
    fn paste_in_fullframe_skips_punched_rows() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 9.into(), 2.into(), 2.into()],
            )],
            24,
        );
        sheet.set_row_kind(2, RowKind::Punched);

        apply_paste_to_sheet(
            &mut sheet,
            DisplayMode::FullFrame,
            0,
            1,
            &[vec![CellValue::Int(3)], vec![CellValue::Int(4)]],
        );

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(9));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(4));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(2));
    }

    #[test]
    fn delete_in_fullframe_preserves_punched_rows() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 9.into(), 2.into(), 2.into()],
            )],
            24,
        );
        sheet.set_row_kind(2, RowKind::Punched);

        apply_delete_to_sheet(&mut sheet, DisplayMode::FullFrame, 0, 0, 1, 3);

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(9));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(2));
    }

    #[test]
    fn paste_in_keyframe_extends_each_pasted_value_until_next_key() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_paste_to_sheet(
            &mut sheet,
            DisplayMode::Keyframe,
            0,
            1,
            &[vec![CellValue::Int(3)], vec![CellValue::Int(4)]],
        );

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(4));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(2));
    }

    #[test]
    fn paste_in_keyframe_ignores_blank_rows_when_values_are_mixed() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_paste_to_sheet(
            &mut sheet,
            DisplayMode::Keyframe,
            0,
            1,
            &[
                vec![CellValue::Int(3)],
                vec![CellValue::blank()],
                vec![CellValue::Int(4)],
            ],
        );

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(4));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(4));
        assert_eq!(sheet.cell(0, 5).map(CellValue::as_i64), Some(4));
    }

    #[test]
    fn paste_in_keyframe_preserves_existing_later_keys() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![
                    CellValue::blank(),
                    CellValue::blank(),
                    CellValue::blank(),
                    CellValue::blank(),
                    CellValue::blank(),
                    CellValue::blank(),
                    2.into(),
                    2.into(),
                    2.into(),
                    3.into(),
                    3.into(),
                    3.into(),
                ],
            )],
            24,
        );

        apply_paste_to_sheet(
            &mut sheet,
            DisplayMode::Keyframe,
            0,
            0,
            &[
                vec![CellValue::Int(2)],
                vec![CellValue::blank()],
                vec![CellValue::blank()],
                vec![CellValue::Int(3)],
                vec![CellValue::blank()],
                vec![CellValue::blank()],
            ],
        );

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 5).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 6).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(0, 9).map(CellValue::as_i64), Some(3));
    }

    #[test]
    fn paste_blank_in_keyframe_creates_blank_run() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![1.into(), 1.into(), 1.into(), 2.into(), 2.into()],
            )],
            24,
        );

        apply_paste_to_sheet(
            &mut sheet,
            DisplayMode::Keyframe,
            0,
            1,
            &[vec![CellValue::blank()]],
        );

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.display_cell_text(1, 0), "");
        assert_eq!(sheet.display_cell_text(2, 0), "");
        assert_eq!(sheet.display_cell_text(3, 0), "2");
    }

    #[test]
    fn transfer_range_in_keyframe_is_noop_when_selection_has_no_visible_keys() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![0.into(), 0.into(), 0.into(), 3.into()],
            )],
            24,
        );

        transfer_range_on_sheet(&mut sheet, DisplayMode::Keyframe, 0, 0, 0, 1, 0, 2, true);

        assert_eq!(sheet.cell(0, 0).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 1).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(3));
    }

    #[test]
    fn repeat_range_down_repeats_selection_pattern_until_bottom() {
        let mut sheet = Sheet::with_fps(
            vec![
                SheetColumn::new(
                    "A",
                    vec![1.into(), 2.into(), 0.into(), 0.into(), 0.into(), 0.into()],
                ),
                SheetColumn::new(
                    "B",
                    vec![3.into(), 4.into(), 0.into(), 0.into(), 0.into(), 0.into()],
                ),
            ],
            24,
        );

        repeat_range_down_on_sheet(&mut sheet, DisplayMode::FullFrame, 0, 1, 0, 1);

        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(1, 2).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(1, 3).map(CellValue::as_i64), Some(4));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(1));
        assert_eq!(sheet.cell(1, 4).map(CellValue::as_i64), Some(3));
        assert_eq!(sheet.cell(0, 5).map(CellValue::as_i64), Some(2));
        assert_eq!(sheet.cell(1, 5).map(CellValue::as_i64), Some(4));
    }

    #[test]
    fn repeat_range_down_skips_punched_rows() {
        let mut sheet = Sheet::with_fps(
            vec![SheetColumn::new(
                "A",
                vec![7.into(), 8.into(), 9.into(), 0.into(), 0.into(), 0.into()],
            )],
            24,
        );
        sheet.set_row_kind(3, RowKind::Punched);

        repeat_range_down_on_sheet(&mut sheet, DisplayMode::FullFrame, 0, 0, 0, 1);

        assert_eq!(sheet.cell(0, 2).map(CellValue::as_i64), Some(7));
        assert_eq!(sheet.cell(0, 3).map(CellValue::as_i64), Some(0));
        assert_eq!(sheet.cell(0, 4).map(CellValue::as_i64), Some(8));
        assert_eq!(sheet.cell(0, 5).map(CellValue::as_i64), Some(7));
    }
}
