use super::{DocumentSnapshot, MAX_HISTORY_ENTRIES, TableApp};
use eframe::egui;

impl TableApp {
    pub(super) fn capture_document_snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            sheet: self.sheet.clone(),
            current_source: self.file_state.current_source_owned(),
            loaded_from_sts: self.current_sheet_loaded_from_sts,
        }
    }

    pub(super) fn reset_document_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        let snapshot = self.capture_document_snapshot();
        self.committed_snapshot = snapshot.clone();
        self.clean_snapshot = snapshot;
    }

    pub(super) fn commit_document_history_if_changed(&mut self) {
        let snapshot = self.capture_document_snapshot();
        if snapshot == self.committed_snapshot {
            return;
        }

        self.undo_stack.push(self.committed_snapshot.clone());
        if self.undo_stack.len() > MAX_HISTORY_ENTRIES {
            let overflow = self.undo_stack.len() - MAX_HISTORY_ENTRIES;
            self.undo_stack.drain(0..overflow);
        }
        self.redo_stack.clear();
        self.committed_snapshot = snapshot;
    }

    pub(super) fn restore_document_snapshot(&mut self, snapshot: DocumentSnapshot) {
        self.sheet = snapshot.sheet;
        self.file_state.set_current_source(snapshot.current_source);
        self.current_sheet_loaded_from_sts = snapshot.loaded_from_sts;
        self.table_view
            .sync_after_sheet_resize(&self.sheet, &self.table_settings);
    }

    pub(super) fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub(super) fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub(super) fn has_unsaved_changes(&self) -> bool {
        self.capture_document_snapshot() != self.clean_snapshot
    }

    pub(super) fn undo(&mut self, ctx: &egui::Context) {
        let Some(snapshot) = self.undo_stack.pop() else {
            return;
        };
        self.redo_stack.push(self.capture_document_snapshot());
        self.restore_document_snapshot(snapshot.clone());
        self.committed_snapshot = snapshot;
        ctx.request_repaint();
    }

    pub(super) fn redo(&mut self, ctx: &egui::Context) {
        let Some(snapshot) = self.redo_stack.pop() else {
            return;
        };
        self.undo_stack.push(self.capture_document_snapshot());
        self.restore_document_snapshot(snapshot.clone());
        self.committed_snapshot = snapshot;
        ctx.request_repaint();
    }
}
