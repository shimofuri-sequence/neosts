use super::selection::{SelectionRect, move_selection_rows};
use super::{TableViewEvent, TableViewState, VerticalScrollDirection};
use crate::display::{
    apply_decrement_from_above_to_sheet, apply_delete_to_sheet, apply_enter_to_sheet,
    apply_increment_from_above_to_sheet,
};
use crate::settings::editor::{DisplayMode, KeyBindings};
use eframe::egui;
use sheet::Sheet;

impl TableViewState {
    const STANDARD_EDIT_MODE: DisplayMode = DisplayMode::FullFrame;

    pub fn advance_after_write(&mut self, sheet: &Sheet) -> bool {
        let Some(selection) = self.selection.map(SelectionRect::normalized) else {
            return false;
        };

        self.advance_selection_after_write(sheet, selection);
        true
    }

    pub(super) fn handle_numeric_input(
        &mut self,
        ctx: &egui::Context,
        sheet: &mut Sheet,
        _display_mode: DisplayMode,
        keybindings: &KeyBindings,
    ) {
        if self.external_modal_open {
            return;
        }

        if self.selection.is_none() {
            self.edit_buffer.clear();
            return;
        }

        let mut commit_requested = false;
        let mut clear_requested = false;
        let mut increment_requested = false;
        let mut decrement_requested = false;
        let mut force_zero_requested = false;
        let mut delete_requested = false;
        let mut copy_requested = false;
        let mut cut_requested = false;
        let mut pasted_text = None;
        let mut changed = false;

        ctx.input(|input| {
            for event in &input.events {
                match event {
                    egui::Event::Copy => {
                        copy_requested = true;
                    }
                    egui::Event::Cut => {
                        cut_requested = true;
                    }
                    egui::Event::Text(text) => {
                        for ch in text.chars() {
                            if ch.is_ascii_digit() && self.edit_buffer.len() < 4 {
                                self.edit_buffer.push(ch);
                                changed = true;
                            } else if ch == '+' {
                                if self.edit_buffer.is_empty() {
                                    increment_requested = true;
                                } else {
                                    commit_requested = true;
                                }
                            } else if ch == '-' {
                                if self.edit_buffer.is_empty() {
                                    decrement_requested = true;
                                } else {
                                    commit_requested = true;
                                }
                            }
                        }
                    }
                    egui::Event::Key {
                        key: egui::Key::Plus,
                        pressed: true,
                        ..
                    } => {
                        if self.edit_buffer.is_empty() {
                            increment_requested = true;
                        } else {
                            commit_requested = true;
                        }
                    }
                    egui::Event::Key {
                        key: egui::Key::Minus,
                        pressed: true,
                        ..
                    } => {
                        if self.edit_buffer.is_empty() {
                            decrement_requested = true;
                        } else {
                            commit_requested = true;
                        }
                    }
                    egui::Event::Paste(text) => {
                        pasted_text = Some(text.clone());
                    }
                    egui::Event::Key {
                        key: egui::Key::Delete,
                        pressed: true,
                        ..
                    } => {
                        delete_requested = true;
                    }
                    egui::Event::Key {
                        key: egui::Key::Backspace,
                        pressed: true,
                        ..
                    } => {
                        if self.edit_buffer.pop().is_some() {
                            changed = true;
                        } else {
                            delete_requested = true;
                        }
                    }
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        ..
                    } => {
                        commit_requested = true;
                    }
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed: true,
                        ..
                    } => {
                        clear_requested = true;
                    }
                    _ => {}
                }
            }

            if self.edit_buffer.is_empty() && !input.modifiers.command && !input.modifiers.alt {
                increment_requested |= input.key_pressed(egui::Key::Plus);
                decrement_requested |= input.key_pressed(egui::Key::Minus);
            }

            force_zero_requested |= keybindings.kara_zero_input.matches(input);
        });

        if clear_requested {
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if copy_requested || cut_requested {
            if cut_requested {
                self.pending_events.push(TableViewEvent::CutRequested);
            } else {
                self.pending_events.push(TableViewEvent::CopyRequested);
            }
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if let Some(text) = pasted_text {
            self.pending_events
                .push(TableViewEvent::PasteRequested { text });
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if commit_requested {
            if let Some(selection) = self.selection {
                let normalized = selection.normalized();
                let input_value = self.edit_buffer.parse::<i64>().ok();

                if self.edit_buffer.is_empty() || input_value.is_some() {
                    apply_enter_to_sheet(
                        sheet,
                        Self::STANDARD_EDIT_MODE,
                        normalized.start.col,
                        normalized.end.col,
                        normalized.start.row,
                        normalized.end.row,
                        input_value,
                    );
                    self.pending_vertical_scroll_direction = Some(VerticalScrollDirection::Down);
                    self.advance_selection_after_write(sheet, normalized);
                }
            }
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if force_zero_requested {
            if let Some(selection) = self.selection {
                let normalized = selection.normalized();
                apply_enter_to_sheet(
                    sheet,
                    Self::STANDARD_EDIT_MODE,
                    normalized.start.col,
                    normalized.end.col,
                    normalized.start.row,
                    normalized.end.row,
                    Some(0),
                );
                self.pending_vertical_scroll_direction = Some(VerticalScrollDirection::Down);
                self.advance_selection_after_write(sheet, normalized);
            }
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if increment_requested || decrement_requested {
            if let Some(selection) = self.selection {
                let normalized = selection.normalized();
                if let Some(input_value) = self.edit_buffer.parse::<i64>().ok() {
                    apply_enter_to_sheet(
                        sheet,
                        Self::STANDARD_EDIT_MODE,
                        normalized.start.col,
                        normalized.end.col,
                        normalized.start.row,
                        normalized.end.row,
                        Some(input_value),
                    );
                } else if increment_requested {
                    apply_increment_from_above_to_sheet(
                        sheet,
                        Self::STANDARD_EDIT_MODE,
                        normalized.start.col,
                        normalized.end.col,
                        normalized.start.row,
                        normalized.end.row,
                    );
                } else {
                    apply_decrement_from_above_to_sheet(
                        sheet,
                        Self::STANDARD_EDIT_MODE,
                        normalized.start.col,
                        normalized.end.col,
                        normalized.start.row,
                        normalized.end.row,
                    );
                }
                self.pending_vertical_scroll_direction = Some(VerticalScrollDirection::Down);
                self.advance_selection_after_write(sheet, normalized);
            }
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if delete_requested {
            if let Some(selection) = self.selection {
                let normalized = selection.normalized();
                apply_delete_to_sheet(
                    sheet,
                    Self::STANDARD_EDIT_MODE,
                    normalized.start.col,
                    normalized.end.col,
                    normalized.start.row,
                    normalized.end.row,
                );
                self.pending_vertical_scroll_direction = Some(VerticalScrollDirection::Down);
                self.advance_selection_after_write(sheet, normalized);
            }
            self.edit_buffer.clear();
            ctx.request_repaint();
            return;
        }

        if changed {
            ctx.request_repaint();
        }
    }

    fn advance_selection_after_write(&mut self, sheet: &Sheet, selection: SelectionRect) {
        self.set_selection(Some(move_selection_rows(selection, 1, sheet)));
        self.scroll_selection_into_view = true;
        self.pending_vertical_scroll_direction = Some(VerticalScrollDirection::Down);
        self.suppress_vertical_scroll_adjustment = false;
        self.preserve_horizontal_scroll_for_selection = true;
    }
}
