use sheet::Sheet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnAction {
    Rename,
    Delete,
    InsertLeft,
    InsertRight,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColumnActionState {
    pub context_col: Option<usize>,
    pub target_col: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnActionEvent {
    OpenRenameColumnRequested { column: usize, current_name: String },
    SheetChanged { selected_column: Option<usize> },
}

impl ColumnActionState {
    pub fn has_context_col(&self) -> bool {
        self.context_col.is_some()
    }

    pub fn single_column_selected(&self) -> bool {
        self.target_col.is_some()
    }

    pub fn supports(&self, action: ColumnAction, sheet: &Sheet) -> bool {
        if !self.has_context_col() {
            return false;
        }

        match action {
            ColumnAction::Rename | ColumnAction::InsertLeft | ColumnAction::InsertRight => {
                self.single_column_selected()
            }
            ColumnAction::Delete => self.single_column_selected() && sheet.column_count() > 1,
        }
    }
}

pub fn execute_column_action(
    sheet: &mut Sheet,
    state: &ColumnActionState,
    action: ColumnAction,
) -> Option<ColumnActionEvent> {
    if !state.supports(action, sheet) {
        return None;
    }

    let target_col = state.target_col?;

    match action {
        ColumnAction::Rename => Some(ColumnActionEvent::OpenRenameColumnRequested {
            column: target_col,
            current_name: sheet.column_name(target_col).to_owned(),
        }),
        ColumnAction::Delete => {
            if !sheet.remove_column(target_col) {
                return None;
            }
            let selected_column = sheet
                .column_count()
                .checked_sub(1)
                .map(|last_col| target_col.min(last_col));
            Some(ColumnActionEvent::SheetChanged { selected_column })
        }
        ColumnAction::InsertLeft => {
            let base_name = sheet.column_name(target_col).to_owned();
            let selected_column = sheet.insert_blank_column(target_col);
            let _ = sheet.set_column_name(selected_column, inserted_column_name(&base_name, false));
            Some(ColumnActionEvent::SheetChanged {
                selected_column: Some(selected_column),
            })
        }
        ColumnAction::InsertRight => {
            let base_name = sheet.column_name(target_col).to_owned();
            let selected_column = sheet.insert_blank_column(target_col + 1);
            let _ = sheet.set_column_name(selected_column, inserted_column_name(&base_name, true));
            Some(ColumnActionEvent::SheetChanged {
                selected_column: Some(selected_column),
            })
        }
    }
}

fn inserted_column_name(base_name: &str, insert_right: bool) -> String {
    let suffix = if insert_right { "ue" } else { "shita" };
    format!("{} {}", base_name.trim(), suffix)
}
