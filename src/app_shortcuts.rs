use super::TableApp;
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use neosts::TableShortcutResult;

impl TableApp {
    pub(super) fn handle_file_shortcuts(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        const EXIT_APP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);

        if ctx.input_mut(|input| input.consume_shortcut(&EXIT_APP)) {
            self.request_exit_app(ctx);
            return;
        }

        if self.preferences_state.open
            || self.sheet_dialogs_state.is_any_open()
            || self.show_about_dialog
            || self.pending_dirty_action.is_some()
            || self.pending_column_menu.is_some()
            || self.pending_row_menu.is_some()
        {
            return;
        }

        const NEW_SHEET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
        const OPEN_SHEET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
        const SAVE_SHEET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
        const SAVE_SHEET_AS: KeyboardShortcut =
            KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S);
        const RESIZE_SHEET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::K);
        let has_sheet_data = self.sheet.column_count() > 0 && self.sheet.row_count() > 0;

        if ctx.input_mut(|input| input.consume_shortcut(&SAVE_SHEET_AS)) {
            self.save_sheet_as(frame);
            return;
        }

        if ctx.input_mut(|input| input.consume_shortcut(&SAVE_SHEET)) {
            self.save_sheet(frame);
            return;
        }

        if ctx.input_mut(|input| input.consume_shortcut(&OPEN_SHEET)) {
            self.request_open_sheet(frame);
            return;
        }

        if has_sheet_data && ctx.input_mut(|input| input.consume_shortcut(&RESIZE_SHEET)) {
            self.sheet_dialogs_state.open_resize_sheet(&self.sheet);
            return;
        }

        if ctx.input_mut(|input| input.consume_shortcut(&NEW_SHEET)) {
            self.request_new_sheet();
        }
    }

    pub(super) fn handle_cell_scale_shortcuts(&mut self, ctx: &egui::Context) {
        const ZOOM_IN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Equals);
        const ZOOM_OUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Minus);
        const ZOOM_RESET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Num0);

        if ctx.input_mut(|input| input.consume_shortcut(&ZOOM_RESET)) {
            self.reset_zoom();
            return;
        }

        if ctx.input_mut(|input| input.consume_shortcut(&ZOOM_IN)) {
            self.zoom_in();
        }

        if ctx.input_mut(|input| input.consume_shortcut(&ZOOM_OUT)) {
            self.zoom_out();
        }
    }

    pub(super) fn handle_table_keybind_shortcuts(&mut self, ctx: &egui::Context) {
        if self.pending_dirty_action.is_some() {
            return;
        }

        if self.preferences_state.open
            || self.sheet_dialogs_state.is_any_open()
            || self.show_about_dialog
            || self.pending_column_menu.is_some()
            || self.pending_row_menu.is_some()
        {
            return;
        }

        let home_pressed = ctx.input(|input| input.key_pressed(Key::Home));
        let end_pressed = ctx.input(|input| input.key_pressed(Key::End));
        let page_up_pressed = ctx.input(|input| input.key_pressed(Key::PageUp));
        let page_down_pressed = ctx.input(|input| input.key_pressed(Key::PageDown));

        if home_pressed {
            if self.table_view.scroll_to_top(&self.sheet) {
                ctx.request_repaint();
            }
            return;
        }

        if end_pressed {
            if self.table_view.scroll_to_bottom(&self.sheet) {
                ctx.request_repaint();
            }
            return;
        }

        if page_up_pressed {
            if self.table_view.scroll_by_rows_preserving_horizontal(
                -(self.sheet.fps() as isize),
                &self.sheet,
                &self.table_settings,
            ) {
                ctx.request_repaint();
            }
            return;
        }

        if page_down_pressed {
            if self.table_view.scroll_by_rows_preserving_horizontal(
                self.sheet.fps() as isize,
                &self.sheet,
                &self.table_settings,
            ) {
                ctx.request_repaint();
            }
            return;
        }

        const MOVE_SELECTED_COLUMNS_LEFT: KeyboardShortcut =
            KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::ALT), Key::ArrowLeft);
        const MOVE_SELECTED_COLUMNS_RIGHT: KeyboardShortcut =
            KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::ALT), Key::ArrowRight);

        if ctx.input_mut(|input| input.consume_shortcut(&MOVE_SELECTED_COLUMNS_LEFT)) {
            if self
                .table_view
                .move_selected_columns(&mut self.sheet, &self.table_settings, -1)
            {
                ctx.request_repaint();
            }
            return;
        }

        if ctx.input_mut(|input| input.consume_shortcut(&MOVE_SELECTED_COLUMNS_RIGHT)) {
            if self
                .table_view
                .move_selected_columns(&mut self.sheet, &self.table_settings, 1)
            {
                ctx.request_repaint();
            }
            return;
        }

        let TableShortcutResult {
            changed,
            open_preferences,
        } = self.table_view.handle_global_shortcuts(
            ctx,
            &self.sheet,
            &self.editor_settings.keybindings,
        );

        if open_preferences {
            self.preferences_state.open = true;
        }

        if changed {
            ctx.request_repaint();
        }
    }

    pub(super) fn handle_undo_redo_shortcuts(&mut self, ctx: &egui::Context) {
        if self.preferences_state.open
            || self.sheet_dialogs_state.is_any_open()
            || self.pending_dirty_action.is_some()
        {
            return;
        }

        const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
        const REDO: KeyboardShortcut =
            KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);
        const REDO_FALLBACK: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);

        if ctx.input_mut(|input| input.consume_shortcut(&REDO))
            || ctx.input_mut(|input| input.consume_shortcut(&REDO_FALLBACK))
        {
            self.redo(ctx);
            return;
        }

        if ctx.input_mut(|input| input.consume_shortcut(&UNDO)) {
            self.undo(ctx);
        }
    }
}
