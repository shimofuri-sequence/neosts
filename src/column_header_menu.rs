use crate::column_actions::{ColumnAction, ColumnActionState};
use crate::commands::AppCommand;
use eframe::egui;

pub fn show_column_header_context_menu(
    ui: &mut egui::Ui,
    state: &ColumnActionState,
    can_delete: bool,
) -> Option<ColumnAction> {
    if !state.has_context_col() {
        return None;
    }

    if ui
        .add_enabled(
            state.single_column_selected(),
            egui::Button::new(AppCommand::RenameColumn.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(ColumnAction::Rename);
    }
    if ui
        .add_enabled(
            can_delete,
            egui::Button::new(AppCommand::DeleteColumn.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(ColumnAction::Delete);
    }
    if ui
        .add_enabled(
            state.single_column_selected(),
            egui::Button::new(AppCommand::InsertColumnLeft.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(ColumnAction::InsertLeft);
    }
    if ui
        .add_enabled(
            state.single_column_selected(),
            egui::Button::new(AppCommand::InsertColumnRight.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(ColumnAction::InsertRight);
    }

    None
}
