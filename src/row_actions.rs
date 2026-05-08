use sheet::{RowKind, Sheet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowAction {
    Punch,
    Unpunch,
    AppendAbove,
    AppendBelow,
    DeleteSpecial,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowActionState {
    pub context_row: Option<usize>,
    pub target_rows: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowActionEvent {
    OpenAppendRowsRequested { row: usize, insert_above: bool },
}

impl RowActionState {
    pub fn has_context_row(&self) -> bool {
        self.context_row.is_some()
    }

    pub fn has_punched_rows(&self, sheet: &Sheet) -> bool {
        self.target_rows
            .iter()
            .any(|&target_row| sheet.is_punched_row(target_row))
    }

    pub fn single_row(&self) -> bool {
        self.target_rows.len() == 1
    }

    pub fn has_target_rows(&self) -> bool {
        !self.target_rows.is_empty()
    }

    pub fn has_special_inserted_rows(&self, sheet: &Sheet) -> bool {
        self.target_rows
            .iter()
            .any(|&target_row| sheet.is_inserted_row(target_row))
    }

    fn first_target_row(&self) -> Option<usize> {
        self.target_rows.iter().copied().min()
    }

    fn last_target_row(&self) -> Option<usize> {
        self.target_rows.iter().copied().max()
    }

    pub fn supports(&self, action: RowAction, sheet: &Sheet) -> bool {
        if !self.has_context_row() {
            return false;
        }

        match action {
            RowAction::Punch => true,
            RowAction::Unpunch => self.has_punched_rows(sheet),
            RowAction::AppendAbove | RowAction::AppendBelow => self.has_target_rows(),
            RowAction::DeleteSpecial => self.has_special_inserted_rows(sheet),
        }
    }
}

pub fn execute_row_action(
    sheet: &mut Sheet,
    state: &RowActionState,
    action: RowAction,
) -> Option<RowActionEvent> {
    if !state.has_context_row() {
        return None;
    }

    match action {
        RowAction::Punch => {
            for target_row in &state.target_rows {
                let _ = sheet.set_row_kind(*target_row, RowKind::Punched);
            }
            None
        }
        RowAction::Unpunch => {
            for target_row in &state.target_rows {
                let _ = sheet.set_row_kind(*target_row, RowKind::Normal);
            }
            None
        }
        RowAction::AppendAbove => {
            state
                .first_target_row()
                .map(|row| RowActionEvent::OpenAppendRowsRequested {
                    row,
                    insert_above: true,
                })
        }
        RowAction::AppendBelow => {
            state
                .last_target_row()
                .map(|row| RowActionEvent::OpenAppendRowsRequested {
                    row,
                    insert_above: false,
                })
        }
        RowAction::DeleteSpecial => {
            let mut target_rows = state.target_rows.clone();
            target_rows.sort_unstable();
            target_rows.dedup();

            for &target_row in target_rows.iter().rev() {
                if sheet.is_inserted_row(target_row) {
                    let _ = sheet.remove_row(target_row);
                }
            }

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RowAction, RowActionEvent, RowActionState, execute_row_action};
    use sheet::Sheet;

    #[test]
    fn append_above_uses_first_selected_row_instead_of_context_row() {
        let mut sheet = Sheet::blank_with_fps(1, 10, 24);
        let state = RowActionState {
            context_row: Some(7),
            target_rows: vec![3, 4, 5],
        };

        let event = execute_row_action(&mut sheet, &state, RowAction::AppendAbove);

        assert_eq!(
            event,
            Some(RowActionEvent::OpenAppendRowsRequested {
                row: 3,
                insert_above: true,
            })
        );
    }

    #[test]
    fn append_below_uses_last_selected_row_instead_of_context_row() {
        let mut sheet = Sheet::blank_with_fps(1, 10, 24);
        let state = RowActionState {
            context_row: Some(2),
            target_rows: vec![3, 4, 5],
        };

        let event = execute_row_action(&mut sheet, &state, RowAction::AppendBelow);

        assert_eq!(
            event,
            Some(RowActionEvent::OpenAppendRowsRequested {
                row: 5,
                insert_above: false,
            })
        );
    }
}
