use super::{PendingDirtyAction, TableApp};
use eframe::egui;
use neosts::strings;
use std::path::PathBuf;

impl TableApp {
    pub(super) fn request_open_sheet(&mut self, frame: &eframe::Frame) {
        if self.has_unsaved_changes() {
            self.pending_dirty_action = Some(PendingDirtyAction::OpenSheet);
            return;
        }
        self.open_sheet_from_dialog(frame);
    }

    pub(super) fn request_open_dropped_sheet(&mut self, path: PathBuf) {
        if self.has_unsaved_changes() {
            self.pending_dirty_action = Some(PendingDirtyAction::OpenDroppedSheet(path));
            return;
        }
        self.open_sheet_path(&path);
    }

    pub(super) fn request_open_recent_sheet(&mut self, path: PathBuf) {
        if self.has_unsaved_changes() {
            self.pending_dirty_action = Some(PendingDirtyAction::OpenRecentSheet(path));
            return;
        }
        self.open_sheet_path(&path);
    }

    pub(super) fn request_new_sheet(&mut self) {
        if self.has_unsaved_changes() {
            self.pending_dirty_action = Some(PendingDirtyAction::NewSheet);
            return;
        }
        self.sheet_dialogs_state
            .open_new_sheet(&self.sheet_settings);
    }

    pub(super) fn request_new_sheet_from_after_effects_selection(
        &mut self,
        frame: &eframe::Frame,
        ctx: &egui::Context,
    ) {
        if self.has_unsaved_changes() {
            self.pending_dirty_action = Some(PendingDirtyAction::NewSheetFromAfterEffectsSelection);
            return;
        }
        self.new_sheet_from_after_effects_selection(frame, ctx);
    }

    pub(super) fn request_exit_app(&mut self, ctx: &egui::Context) {
        if self.has_unsaved_changes() {
            self.pending_dirty_action = Some(PendingDirtyAction::ExitApp);
            return;
        }
        self.allow_close_once = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    pub(super) fn perform_pending_dirty_action(
        &mut self,
        action: PendingDirtyAction,
        ctx: &egui::Context,
        frame: &eframe::Frame,
    ) {
        match action {
            PendingDirtyAction::OpenSheet => self.open_sheet_from_dialog(frame),
            PendingDirtyAction::OpenDroppedSheet(path) => self.open_sheet_path(&path),
            PendingDirtyAction::OpenRecentSheet(path) => self.open_sheet_path(&path),
            PendingDirtyAction::NewSheet => {
                self.sheet_dialogs_state
                    .open_new_sheet(&self.sheet_settings);
            }
            PendingDirtyAction::NewSheetFromAfterEffectsSelection => {
                self.new_sheet_from_after_effects_selection(frame, ctx);
            }
            PendingDirtyAction::ExitApp => {
                self.allow_close_once = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    pub(super) fn save_before_dirty_action(&mut self, frame: &eframe::Frame) -> bool {
        let suggested_name = self.sheet.name().to_owned();
        self.save_sheet_as_sts_dialog(&suggested_name, frame);
        !self.has_unsaved_changes()
    }

    pub(super) fn show_dirty_confirm_dialog(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(action) = self.pending_dirty_action.clone() else {
            return;
        };

        let mut save_and_continue = false;
        let mut discard_and_continue = false;
        let mut cancel = false;
        let locale = self.locale();
        let title = strings::dirty_title(locale);
        let continue_label = match &action {
            PendingDirtyAction::OpenSheet
            | PendingDirtyAction::OpenDroppedSheet(_)
            | PendingDirtyAction::OpenRecentSheet(_) => {
                strings::discard_open_without_saving(locale)
            }
            PendingDirtyAction::NewSheet => strings::discard_create_without_saving(locale),
            PendingDirtyAction::NewSheetFromAfterEffectsSelection => {
                strings::discard_continue_without_saving(locale)
            }
            PendingDirtyAction::ExitApp => strings::discard_quit_without_saving(locale),
        };

        egui::Modal::new(egui::Id::new("dirty_confirm_modal")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(title);
            ui.add_space(8.0);
            ui.label(strings::dirty_body(locale));
            ui.add_space(12.0);
            if ui.button(strings::save_and_continue(locale)).clicked() {
                save_and_continue = true;
            }
            if ui.button(continue_label).clicked() {
                discard_and_continue = true;
            }
            if ui.button(strings::cancel(locale)).clicked() {
                cancel = true;
            }
        });

        if save_and_continue {
            if self.save_before_dirty_action(frame) {
                self.pending_dirty_action = None;
                self.perform_pending_dirty_action(action, ctx, frame);
            } else {
                self.pending_dirty_action = None;
            }
        } else if discard_and_continue {
            self.pending_dirty_action = None;
            self.perform_pending_dirty_action(action, ctx, frame);
        } else if cancel {
            self.pending_dirty_action = None;
        }
    }
}
