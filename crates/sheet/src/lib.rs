use std::collections::HashSet;
use std::fmt;
use thiserror::Error;

const DEFAULT_SHEET_FPS: u32 = 24;
const MIN_SHEET_FPS: u32 = 1;
pub const BLANK_CELL_VALUE: i64 = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellValue {
    Int(i64),
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
        }
    }
}

impl From<i64> for CellValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<i32> for CellValue {
    fn from(value: i32) -> Self {
        Self::Int(value as i64)
    }
}

impl CellValue {
    pub fn blank() -> Self {
        Self::Int(BLANK_CELL_VALUE)
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Self::Int(value) => *value,
        }
    }

    pub fn is_blank(&self) -> bool {
        self.as_i64() == BLANK_CELL_VALUE
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetColumn {
    name: String,
    values: Vec<CellValue>,
}

impl SheetColumn {
    pub fn new(name: impl Into<String>, values: Vec<CellValue>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowKind {
    #[default]
    Normal,
    Punched,
    // A specially marked inserted row. It may appear at the beginning,
    // end, or between existing rows.
    SpecialInserted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sheet {
    columns: Vec<SheetColumn>,
    row_kinds: Vec<RowKind>,
    row_count: usize,
    fps: u32,
}

impl Sheet {
    pub fn new(columns: Vec<SheetColumn>) -> Self {
        Self::with_fps(columns, DEFAULT_SHEET_FPS)
    }

    pub fn blank_with_fps(column_count: usize, row_count: usize, fps: u32) -> Self {
        let columns = (0..column_count)
            .map(|col| {
                SheetColumn::new(
                    default_column_name(col),
                    vec![CellValue::blank(); row_count],
                )
            })
            .collect();

        Self {
            columns,
            row_kinds: vec![RowKind::Normal; row_count],
            row_count,
            fps,
        }
    }

    pub fn with_fps(columns: Vec<SheetColumn>, fps: u32) -> Self {
        let row_count = columns
            .first()
            .map(|column| column.values.len())
            .unwrap_or(0);
        Self {
            columns,
            row_kinds: vec![RowKind::Normal; row_count],
            row_count,
            fps,
        }
    }

    pub fn try_new(columns: Vec<SheetColumn>) -> Result<Self, SheetError> {
        Self::try_with_fps(columns, DEFAULT_SHEET_FPS)
    }

    pub fn try_with_fps(columns: Vec<SheetColumn>, fps: u32) -> Result<Self, SheetError> {
        let row_count = columns
            .first()
            .map(|column| column.values.len())
            .unwrap_or(0);
        let invalid_column = columns
            .iter()
            .find(|column| column.values.len() != row_count)
            .map(|column| (column.name.clone(), column.values.len()));

        if let Some((name, value_count)) = invalid_column {
            return Err(SheetError::InconsistentColumnLength {
                column_name: name,
                expected: row_count,
                actual: value_count,
            });
        }

        Ok(Self {
            columns,
            row_kinds: vec![RowKind::Normal; row_count],
            row_count,
            fps,
        })
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn row_kind(&self, row: usize) -> Option<RowKind> {
        self.row_kinds.get(row).copied()
    }

    pub fn set_row_kind(&mut self, row: usize, kind: RowKind) -> bool {
        let Some(row_kind) = self.row_kinds.get_mut(row) else {
            return false;
        };
        *row_kind = kind;
        true
    }

    pub fn is_punched_row(&self, row: usize) -> bool {
        self.row_kind(row) == Some(RowKind::Punched)
    }

    pub fn is_visual_only_row(&self, row: usize) -> bool {
        self.is_punched_row(row)
    }

    pub fn is_inserted_row(&self, row: usize) -> bool {
        self.row_kind(row) == Some(RowKind::SpecialInserted)
    }

    pub fn is_special_inserted_row(&self, row: usize) -> bool {
        self.is_inserted_row(row)
    }

    pub fn row_participates_in_timeline(&self, row: usize) -> bool {
        !self.is_visual_only_row(row)
    }

    pub fn effective_frame_count(&self) -> usize {
        (0..self.row_count)
            .filter(|&row| self.row_participates_in_timeline(row))
            .count()
    }

    pub fn real_frame_number_at(&self, row: usize) -> usize {
        (0..=row)
            .filter(|&candidate| !self.is_inserted_row(candidate))
            .count()
            .max(1)
    }

    pub fn absolute_frame_number_at(&self, row: usize) -> usize {
        (0..=row)
            .filter(|&candidate| self.row_participates_in_timeline(candidate))
            .count()
            .max(1)
    }

    pub fn inserted_row_group_position(&self, row: usize) -> Option<usize> {
        self.is_inserted_row(row).then(|| {
            (0..row)
                .rev()
                .take_while(|&candidate| self.is_inserted_row(candidate))
                .count()
                + 1
        })
    }

    pub fn inserted_absolute_frame_number_at(&self, row: usize) -> Option<usize> {
        let group_pos = self.inserted_row_group_position(row)?;
        let base = (0..row)
            .rev()
            .find(|&candidate| !self.is_inserted_row(candidate))
            .map(|candidate| self.absolute_frame_number_at(candidate))
            .unwrap_or(0);
        Some(base + group_pos)
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps.max(MIN_SHEET_FPS);
    }

    pub fn cell(&self, col: usize, row: usize) -> Option<&CellValue> {
        self.columns.get(col)?.values.get(row)
    }

    pub fn previous_timeline_row(&self, row: usize) -> Option<usize> {
        if row == 0 {
            return None;
        }

        (0..row)
            .rev()
            .find(|&candidate| self.row_participates_in_timeline(candidate))
    }

    pub fn next_timeline_row(&self, row: usize) -> Option<usize> {
        ((row + 1)..self.row_count).find(|&candidate| self.row_participates_in_timeline(candidate))
    }

    pub fn timeline_rows_in_range(&self, start_row: usize, end_row: usize) -> Vec<usize> {
        (start_row..=end_row)
            .filter(|&row| self.row_participates_in_timeline(row))
            .collect()
    }

    pub fn column_values_skipping_visual_only(&self, col: usize) -> Vec<CellValue> {
        (0..self.row_count)
            .filter(|&row| self.row_participates_in_timeline(row))
            .filter_map(|row| self.cell(col, row).cloned())
            .collect()
    }

    pub fn insert_row(&mut self, at: usize, values: Vec<CellValue>) {
        for (col_idx, column) in self.columns.iter_mut().enumerate() {
            let value = values
                .get(col_idx)
                .cloned()
                .unwrap_or_else(CellValue::blank);
            column.values.insert(at.min(column.values.len()), value);
        }
        self.row_kinds
            .insert(at.min(self.row_kinds.len()), RowKind::Normal);
        self.row_count += 1;
    }

    pub fn insert_row_with_kind(&mut self, at: usize, values: Vec<CellValue>, kind: RowKind) {
        self.insert_row(at, values);
        let _ = self.set_row_kind(at.min(self.row_count.saturating_sub(1)), kind);
    }

    pub fn remove_row(&mut self, row: usize) -> bool {
        if row >= self.row_count {
            return false;
        }

        for column in &mut self.columns {
            column.values.remove(row);
        }
        self.row_kinds.remove(row);
        self.row_count -= 1;
        true
    }

    pub fn resize_rows(&mut self, new_row_count: usize) {
        for column in &mut self.columns {
            column.values.resize(new_row_count, CellValue::blank());
        }
        self.row_kinds.resize(new_row_count, RowKind::Normal);
        self.row_count = new_row_count;
    }

    pub fn resize_columns(&mut self, new_column_count: usize) {
        let current = self.columns.len();
        if new_column_count >= current {
            for col in current..new_column_count {
                self.columns.push(SheetColumn::new(
                    default_column_name(col),
                    vec![CellValue::blank(); self.row_count],
                ));
            }
        } else {
            self.columns.truncate(new_column_count);
        }
    }

    pub fn set_column_name(&mut self, col: usize, name: impl Into<String>) -> bool {
        let Some(column) = self.columns.get_mut(col) else {
            return false;
        };
        let trimmed = name.into().trim().to_owned();
        if trimmed.is_empty() {
            return false;
        }
        column.name = trimmed;
        true
    }

    pub fn insert_blank_column(&mut self, at: usize) -> usize {
        let insert_at = at.min(self.columns.len());
        let name = self.unique_default_column_name(insert_at);
        self.columns.insert(
            insert_at,
            SheetColumn::new(name, vec![CellValue::blank(); self.row_count]),
        );
        insert_at
    }

    pub fn remove_column(&mut self, col: usize) -> bool {
        if col >= self.columns.len() {
            return false;
        }
        self.columns.remove(col);
        true
    }

    pub fn move_column_range(&mut self, start_col: usize, end_col: usize, direction: isize) -> bool {
        if self.columns.is_empty() || start_col > end_col || end_col >= self.columns.len() {
            return false;
        }

        if direction < 0 {
            if start_col == 0 {
                return false;
            }
            self.columns[start_col - 1..=end_col].rotate_left(1);
            true
        } else if direction > 0 {
            if end_col + 1 >= self.columns.len() {
                return false;
            }
            self.columns[start_col..=end_col + 1].rotate_right(1);
            true
        } else {
            false
        }
    }

    pub fn set_cell(&mut self, col: usize, row: usize, value: CellValue) -> bool {
        let Some(cell) = self
            .columns
            .get_mut(col)
            .and_then(|column| column.values.get_mut(row))
        else {
            return false;
        };
        *cell = value;
        true
    }

    pub fn is_cross_cell(&self, row: usize, col: usize) -> bool {
        matches!(self.cell(col, row), Some(CellValue::Int(0)))
    }

    pub fn is_blank_cell(&self, row: usize, col: usize) -> bool {
        self.cell(col, row).is_some_and(CellValue::is_blank)
    }

    pub fn should_display_cell(&self, row: usize, col: usize) -> bool {
        let Some(current) = self.cell(col, row) else {
            return false;
        };

        let previous_row = if self.is_visual_only_row(row) {
            row.checked_sub(1)
        } else {
            self.previous_timeline_row(row)
        };
        let Some(previous_row) = previous_row else {
            return true;
        };

        !self
            .cell(col, previous_row)
            .is_some_and(|previous| previous == current)
    }

    pub fn should_draw_continuation_line(
        &self,
        row: usize,
        col: usize,
        min_run_length: usize,
    ) -> bool {
        if min_run_length == 0 {
            return false;
        }

        let Some(current) = self.cell(col, row) else {
            return false;
        };

        let visual_only = self.is_visual_only_row(row);
        let previous_row = if visual_only {
            self.previous_row(row)
        } else {
            self.previous_timeline_row(row)
        };

        if self.should_display_cell(row, col) || previous_row.is_none() {
            return false;
        }

        let mut start_row = row;
        while let Some(previous_row) = if visual_only {
            self.previous_row(start_row)
        } else {
            self.previous_timeline_row(start_row)
        } && self
            .cell(col, previous_row)
            .is_some_and(|previous| previous == current)
        {
            start_row = previous_row;
        }

        let mut end_row = row;
        while let Some(next_row) = if visual_only {
            self.next_row(end_row)
        } else {
            self.next_timeline_row(end_row)
        } {
            if self.cell(col, next_row).is_some_and(|next| next == current) {
                end_row = next_row;
            } else {
                break;
            }
        }

        let run_length = end_row.saturating_sub(start_row) + 1;
        run_length >= min_run_length
    }

    pub fn total_height(&self, cell_height: f32) -> f32 {
        self.row_count() as f32 * cell_height
    }

    pub fn column_name(&self, col: usize) -> &str {
        self.columns
            .get(col)
            .map(|column| column.name.as_str())
            .unwrap_or("")
    }

    pub fn display_cell_text(&self, row: usize, col: usize) -> String {
        if self.is_cross_cell(row, col) || !self.should_display_cell(row, col) {
            return String::new();
        }

        let Some(current) = self.cell(col, row) else {
            return String::new();
        };
        if current.is_blank() {
            return String::new();
        }
        current.to_string()
    }

    fn unique_default_column_name(&self, index: usize) -> String {
        let base = default_column_name(index);
        let existing = self
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect::<HashSet<_>>();
        if !existing.contains(&base) {
            return base;
        }

        let mut suffix = 2usize;
        loop {
            let candidate = format!("{base}{suffix}");
            if !existing.contains(&candidate) {
                return candidate;
            }
            suffix += 1;
        }
    }

    fn previous_row(&self, row: usize) -> Option<usize> {
        row.checked_sub(1)
    }

    fn next_row(&self, row: usize) -> Option<usize> {
        (row + 1 < self.row_count).then_some(row + 1)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SheetError {
    #[error("column `{column_name}` has {actual} values, expected {expected}")]
    InconsistentColumnLength {
        column_name: String,
        expected: usize,
        actual: usize,
    },
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

#[cfg(test)]
mod tests {
    use super::{CellValue, RowKind, Sheet};

    #[test]
    fn rows_are_normal_by_default() {
        let sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![CellValue::from(1), CellValue::from(2), CellValue::from(3)],
        )]);

        assert_eq!(sheet.row_kind(0), Some(RowKind::Normal));
        assert_eq!(sheet.row_kind(1), Some(RowKind::Normal));
        assert_eq!(sheet.row_kind(2), Some(RowKind::Normal));
    }

    #[test]
    fn can_update_row_kind_and_count_effective_frames() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
            ],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::Punched));
        assert!(sheet.set_row_kind(2, RowKind::SpecialInserted));
        assert_eq!(sheet.row_kind(1), Some(RowKind::Punched));
        assert_eq!(sheet.row_kind(2), Some(RowKind::SpecialInserted));
        assert_eq!(sheet.effective_frame_count(), 3);
        assert!(sheet.is_visual_only_row(1));
        assert!(!sheet.is_visual_only_row(2));
        assert!(!sheet.is_inserted_row(1));
        assert!(sheet.is_inserted_row(2));
        assert!(!sheet.row_participates_in_timeline(1));
        assert!(sheet.row_participates_in_timeline(2));
    }

    #[test]
    fn punched_rows_do_not_display_and_do_not_block_next_visible_value() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(6),
                CellValue::from(6),
                CellValue::from(6),
                CellValue::from(5),
                CellValue::from(5),
                CellValue::from(5),
            ],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::Punched));
        assert!(sheet.set_row_kind(2, RowKind::Punched));
        assert!(sheet.set_row_kind(3, RowKind::Punched));

        assert_eq!(sheet.display_cell_text(0, 0), "6");
        assert_eq!(sheet.display_cell_text(1, 0), "");
        assert_eq!(sheet.display_cell_text(2, 0), "");
        assert_eq!(sheet.display_cell_text(3, 0), "5");
        assert_eq!(sheet.display_cell_text(4, 0), "5");
        assert_eq!(sheet.display_cell_text(5, 0), "");
    }

    #[test]
    fn inserted_rows_start_as_normal() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![CellValue::from(1), CellValue::from(2)],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::Punched));
        sheet.insert_row(1, vec![CellValue::from(99)]);

        assert_eq!(sheet.row_kind(0), Some(RowKind::Normal));
        assert_eq!(sheet.row_kind(1), Some(RowKind::Normal));
        assert_eq!(sheet.row_kind(2), Some(RowKind::Punched));
    }

    #[test]
    fn insert_row_with_kind_marks_inserted_rows() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![CellValue::from(1), CellValue::from(2)],
        )]);

        sheet.insert_row_with_kind(1, vec![CellValue::from(9)], RowKind::SpecialInserted);

        assert_eq!(sheet.row_kind(1), Some(RowKind::SpecialInserted));
        assert!(sheet.is_inserted_row(1));
        assert_eq!(sheet.cell(0, 1), Some(&CellValue::from(9)));
    }

    #[test]
    fn removing_row_updates_values_and_row_kinds() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
            ],
        )]);
        assert!(sheet.set_row_kind(2, RowKind::SpecialInserted));

        assert!(sheet.remove_row(1));

        assert_eq!(sheet.row_count(), 3);
        assert_eq!(sheet.cell(0, 0), Some(&CellValue::from(1)));
        assert_eq!(sheet.cell(0, 1), Some(&CellValue::from(3)));
        assert_eq!(sheet.cell(0, 2), Some(&CellValue::from(4)));
        assert_eq!(sheet.row_kind(1), Some(RowKind::SpecialInserted));
        assert!(!sheet.remove_row(3));
    }

    #[test]
    fn can_rename_insert_and_remove_columns() {
        let mut sheet = Sheet::blank_with_fps(2, 3, 24);

        assert!(sheet.set_column_name(0, "動画"));
        assert_eq!(sheet.column_name(0), "動画");

        let inserted = sheet.insert_blank_column(1);
        assert_eq!(inserted, 1);
        assert_eq!(sheet.column_count(), 3);
        assert_eq!(sheet.column_name(1), "B2");
        assert!(sheet.cell(1, 0).is_some_and(CellValue::is_blank));

        assert!(sheet.remove_column(1));
        assert_eq!(sheet.column_count(), 2);
        assert_eq!(sheet.column_name(0), "動画");
    }

    #[test]
    fn can_move_selected_column_range_left_and_right() {
        let mut sheet = Sheet::new(vec![
            super::SheetColumn::new("A", vec![CellValue::from(1)]),
            super::SheetColumn::new("B", vec![CellValue::from(2)]),
            super::SheetColumn::new("C", vec![CellValue::from(3)]),
            super::SheetColumn::new("D", vec![CellValue::from(4)]),
        ]);

        assert!(sheet.move_column_range(1, 2, -1));
        assert_eq!(sheet.column_name(0), "B");
        assert_eq!(sheet.column_name(1), "C");
        assert_eq!(sheet.column_name(2), "A");
        assert_eq!(sheet.column_name(3), "D");

        assert!(sheet.move_column_range(0, 1, 1));
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(1), "B");
        assert_eq!(sheet.column_name(2), "C");
        assert_eq!(sheet.column_name(3), "D");

        assert!(sheet.move_column_range(1, 2, 1));
        assert_eq!(sheet.column_name(0), "A");
        assert_eq!(sheet.column_name(1), "D");
        assert_eq!(sheet.column_name(2), "B");
        assert_eq!(sheet.column_name(3), "C");
    }

    #[test]
    fn column_values_skipping_visual_only_omits_visual_only_rows() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
            ],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::Punched));
        assert!(sheet.set_row_kind(3, RowKind::Punched));

        assert_eq!(
            sheet.column_values_skipping_visual_only(0),
            vec![CellValue::from(1), CellValue::from(3)]
        );
    }

    #[test]
    fn punched_rows_keep_continuation_lines_visible() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(3),
                CellValue::from(3),
                CellValue::from(4),
            ],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::Punched));
        assert!(sheet.set_row_kind(2, RowKind::Punched));

        assert!(sheet.should_draw_continuation_line(2, 0, 2));
        assert!(!sheet.should_draw_continuation_line(3, 0, 2));
    }

    #[test]
    fn timeline_row_navigation_skips_visual_only_rows() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
                CellValue::from(5),
            ],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::Punched));
        assert!(sheet.set_row_kind(3, RowKind::Punched));

        assert_eq!(sheet.previous_timeline_row(2), Some(0));
        assert_eq!(sheet.next_timeline_row(0), Some(2));
        assert_eq!(sheet.next_timeline_row(2), Some(4));
        assert_eq!(sheet.timeline_rows_in_range(0, 4), vec![0, 2, 4]);
    }

    #[test]
    fn frame_number_helpers_respect_inserted_and_visual_only_rows() {
        let mut sheet = Sheet::new(vec![super::SheetColumn::new(
            "A",
            vec![
                CellValue::from(1),
                CellValue::from(2),
                CellValue::from(3),
                CellValue::from(4),
                CellValue::from(5),
            ],
        )]);

        assert!(sheet.set_row_kind(1, RowKind::SpecialInserted));
        assert!(sheet.set_row_kind(2, RowKind::SpecialInserted));
        assert!(sheet.set_row_kind(4, RowKind::Punched));

        assert_eq!(sheet.real_frame_number_at(3), 2);
        assert_eq!(sheet.absolute_frame_number_at(3), 4);
        assert_eq!(sheet.inserted_row_group_position(1), Some(1));
        assert_eq!(sheet.inserted_row_group_position(2), Some(2));
        assert_eq!(sheet.inserted_absolute_frame_number_at(1), Some(2));
        assert_eq!(sheet.inserted_absolute_frame_number_at(2), Some(3));
        assert_eq!(sheet.inserted_absolute_frame_number_at(3), None);
        assert_eq!(sheet.absolute_frame_number_at(4), 4);
    }

    #[test]
    fn inserted_column_name_avoids_duplicates() {
        let mut sheet = Sheet::new(vec![
            super::SheetColumn::new("A", vec![CellValue::from(1)]),
            super::SheetColumn::new("B", vec![CellValue::from(2)]),
        ]);

        let inserted = sheet.insert_blank_column(1);

        assert_eq!(inserted, 1);
        assert_eq!(sheet.column_name(1), "B2");
    }
}
