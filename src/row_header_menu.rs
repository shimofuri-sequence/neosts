use crate::commands::AppCommand;
use crate::row_actions::{RowAction, RowActionState};
use eframe::egui;

pub fn show_row_header_context_menu(
    ui: &mut egui::Ui,
    state: &RowActionState,
    has_punched_rows: bool,
) -> Option<RowAction> {
    if !state.has_context_row() {
        return None;
    }

    if ui.button(AppCommand::PunchRows.label()).clicked() {
        ui.close();
        return Some(RowAction::Punch);
    }
    if ui
        .add_enabled(
            has_punched_rows,
            egui::Button::new(AppCommand::UnpunchRows.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(RowAction::Unpunch);
    }
    if ui
        .add_enabled(
            state.has_target_rows(),
            egui::Button::new(AppCommand::AppendRowsAbove.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(RowAction::AppendAbove);
    }
    if ui
        .add_enabled(
            state.has_target_rows(),
            egui::Button::new(AppCommand::AppendRowsBelow.label()),
        )
        .clicked()
    {
        ui.close();
        return Some(RowAction::AppendBelow);
    }

    None
}
