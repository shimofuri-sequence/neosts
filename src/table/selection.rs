use sheet::Sheet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellIndex {
    pub(crate) col: usize,
    pub(crate) row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SelectionRect {
    pub(crate) start: CellIndex,
    pub(crate) end: CellIndex,
}

impl SelectionRect {
    pub(crate) fn normalized(self) -> Self {
        Self {
            start: CellIndex {
                col: self.start.col.min(self.end.col),
                row: self.start.row.min(self.end.row),
            },
            end: CellIndex {
                col: self.start.col.max(self.end.col),
                row: self.start.row.max(self.end.row),
            },
        }
    }

    pub(crate) fn contains(self, cell: CellIndex) -> bool {
        let normalized = self.normalized();
        normalized.start.col <= cell.col
            && cell.col <= normalized.end.col
            && normalized.start.row <= cell.row
            && cell.row <= normalized.end.row
    }

    pub(crate) fn width(self) -> usize {
        let normalized = self.normalized();
        normalized.end.col - normalized.start.col + 1
    }

    pub(crate) fn height(self) -> usize {
        let normalized = self.normalized();
        normalized.end.row - normalized.start.row + 1
    }
}

pub(crate) fn next_timeline_row(current_row: usize, sheet: &Sheet) -> Option<usize> {
    sheet.next_timeline_row(current_row)
}

pub(crate) fn previous_timeline_row(
    current_row: usize,
    minimum_row: usize,
    sheet: &Sheet,
) -> Option<usize> {
    (minimum_row..current_row)
        .rev()
        .find(|&row| sheet.row_participates_in_timeline(row))
}

pub(crate) fn timeline_row_count_in_selection(selection: SelectionRect, sheet: &Sheet) -> usize {
    let normalized = selection.normalized();
    (normalized.start.row..=normalized.end.row)
        .filter(|&row| sheet.row_participates_in_timeline(row))
        .count()
        .max(1)
}

pub(crate) fn move_selection_rows(
    selection: SelectionRect,
    direction: isize,
    sheet: &Sheet,
) -> SelectionRect {
    move_selection_rows_with_visible_count(
        selection,
        direction,
        sheet,
        timeline_row_count_in_selection(selection, sheet),
    )
}

pub(crate) fn move_selection_rows_with_visible_count(
    selection: SelectionRect,
    direction: isize,
    sheet: &Sheet,
    visible_count: usize,
) -> SelectionRect {
    let normalized = selection.normalized();
    let row_count = sheet.row_count();
    let visible_count = visible_count.max(1);

    if direction > 0 {
        let mut collected = Vec::with_capacity(visible_count);
        for row in (normalized.end.row + 1)..row_count {
            if sheet.row_participates_in_timeline(row) {
                collected.push(row);
                if collected.len() == visible_count {
                    break;
                }
            }
        }
        if collected.is_empty() {
            return selection;
        }
        SelectionRect {
            start: CellIndex {
                col: normalized.start.col,
                row: collected[0],
            },
            end: CellIndex {
                col: normalized.end.col,
                row: *collected.last().unwrap(),
            },
        }
    } else {
        if normalized.start.row == 0 {
            return selection;
        }
        let mut collected: Vec<usize> = (0..normalized.start.row)
            .rev()
            .filter(|&row| sheet.row_participates_in_timeline(row))
            .take(visible_count)
            .collect();
        if collected.is_empty() {
            return selection;
        }
        collected.reverse();
        if collected.len() < visible_count {
            collected = (0..sheet.row_count())
                .filter(|&row| sheet.row_participates_in_timeline(row))
                .take(visible_count)
                .collect();
        }
        SelectionRect {
            start: CellIndex {
                col: normalized.start.col,
                row: collected[0],
            },
            end: CellIndex {
                col: normalized.end.col,
                row: *collected.last().unwrap(),
            },
        }
    }
}

pub(crate) fn move_selection_block(
    selection: SelectionRect,
    delta_cols: isize,
    delta_rows: isize,
    sheet: &Sheet,
) -> SelectionRect {
    let normalized = selection.normalized();
    let width = normalized.width();
    let height = normalized.height();
    let max_start_col = sheet.column_count().saturating_sub(width);
    let max_start_row = sheet.row_count().saturating_sub(height);

    let next_start_col = normalized
        .start
        .col
        .saturating_add_signed(delta_cols)
        .min(max_start_col);
    let next_start_row = normalized
        .start
        .row
        .saturating_add_signed(delta_rows)
        .min(max_start_row);

    SelectionRect {
        start: CellIndex {
            col: next_start_col,
            row: next_start_row,
        },
        end: CellIndex {
            col: next_start_col + width - 1,
            row: next_start_row + height - 1,
        },
    }
}

pub(crate) fn clamp_cell_to_sheet(cell: CellIndex, sheet: &Sheet) -> Option<CellIndex> {
    if sheet.column_count() == 0 || sheet.row_count() == 0 {
        return None;
    }

    Some(CellIndex {
        col: cell.col.min(sheet.column_count() - 1),
        row: cell.row.min(sheet.row_count() - 1),
    })
}

pub(crate) fn clamp_selection_to_sheet(
    selection: SelectionRect,
    sheet: &Sheet,
) -> Option<SelectionRect> {
    if sheet.column_count() == 0 || sheet.row_count() == 0 {
        return None;
    }

    let normalized = selection.normalized();
    Some(SelectionRect {
        start: CellIndex {
            col: normalized.start.col.min(sheet.column_count() - 1),
            row: normalized.start.row.min(sheet.row_count() - 1),
        },
        end: CellIndex {
            col: normalized.end.col.min(sheet.column_count() - 1),
            row: normalized.end.row.min(sheet.row_count() - 1),
        },
    })
}

pub(crate) fn row_selection(
    start_row: usize,
    end_row: usize,
    sheet: &Sheet,
) -> Option<SelectionRect> {
    let last_col = sheet.column_count().checked_sub(1)?;
    Some(SelectionRect {
        start: CellIndex {
            col: 0,
            row: start_row,
        },
        end: CellIndex {
            col: last_col,
            row: end_row,
        },
    })
}

pub(crate) fn column_selection(
    start_col: usize,
    end_col: usize,
    sheet: &Sheet,
) -> Option<SelectionRect> {
    let last_row = sheet.row_count().checked_sub(1)?;
    Some(SelectionRect {
        start: CellIndex {
            col: start_col,
            row: 0,
        },
        end: CellIndex {
            col: end_col,
            row: last_row,
        },
    })
}
