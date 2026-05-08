use crate::{TableApp, show_edit_menu, theme_files};
use eframe::egui::{self, Margin};
use neosts::{AppCommand, AppMenu, ColumnAction, ColumnActionState, strings};

impl TableApp {
    pub fn show_app_header(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        let ctx = ui.ctx().clone();
        let locale = self.locale();
        egui::Panel::top("app_header")
            .resizable(false)
            .exact_size(58.0)
            .frame(egui::Frame::default().inner_margin(Margin::same(0)))
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let has_sheet_data =
                        self.sheet.column_count() > 0 && self.sheet.row_count() > 0;

                    ui.menu_button(AppMenu::File.localized_label(locale), |ui| {
                        ui.set_min_width(220.0);
                        if ui
                            .add(Self::menu_button(
                                AppCommand::NewSheet.localized_label(locale),
                                &Self::format_shortcut(
                                    &ctx,
                                    egui::Modifiers::COMMAND,
                                    egui::Key::N,
                                ),
                            ))
                            .clicked()
                        {
                            self.request_new_sheet();
                            ui.close();
                        }
                        if ui
                            .add(Self::menu_button(
                                AppCommand::OpenSheet.localized_label(locale),
                                &Self::format_shortcut(
                                    &ctx,
                                    egui::Modifiers::COMMAND,
                                    egui::Key::O,
                                ),
                            ))
                            .clicked()
                        {
                            self.request_open_sheet(frame);
                            ui.close();
                        }
                        ui.menu_button(self.recent_files_label(), |ui| {
                            self.show_recent_files_menu(ui);
                        });
                        ui.separator();
                        ui.add_enabled_ui(has_sheet_data, |ui| {
                            if ui
                                .add(Self::menu_button(
                                    AppCommand::ResizeSheet.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::K,
                                    ),
                                ))
                                .clicked()
                            {
                                self.sheet_dialogs_state.open_resize_sheet(&self.sheet);
                                ui.close();
                            }
                        });
                        ui.separator();
                        if ui
                            .add_enabled(
                                self.after_effects_is_available(frame),
                                egui::Button::new(
                                    AppCommand::NewSheetFromAfterEffectsSelection
                                        .localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.request_new_sheet_from_after_effects_selection(frame, &ctx);
                            ui.close();
                        }
                        ui.separator();
                        ui.add_enabled_ui(has_sheet_data, |ui| {
                            if ui
                                .add(Self::menu_button(
                                    AppCommand::SaveSts.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::S,
                                    ),
                                ))
                                .clicked()
                            {
                                self.save_sheet(frame);
                                ui.close();
                            }
                            if ui
                                .add(Self::menu_button(
                                    AppCommand::SaveStsAs.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
                                        egui::Key::S,
                                    ),
                                ))
                                .clicked()
                            {
                                self.save_sheet_as(frame);
                                ui.close();
                            }
                        });
                        ui.separator();
                        if ui
                            .add(Self::menu_button(
                                AppCommand::ExitApp.localized_label(locale),
                                &Self::format_shortcut(
                                    &ctx,
                                    egui::Modifiers::COMMAND,
                                    egui::Key::Q,
                                ),
                            ))
                            .clicked()
                        {
                            self.request_exit_app(&ctx);
                            ui.close();
                        }
                    });

                    ui.menu_button(AppMenu::Edit.localized_label(locale), |ui| {
                        ui.set_min_width(180.0);
                        let has_selection = self.table_view.selected_cell().is_some();
                        if ui
                            .add_enabled(
                                self.can_undo(),
                                Self::menu_button(
                                    AppCommand::Undo.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::Z,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.undo(&ctx);
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                self.can_redo(),
                                Self::menu_button(
                                    AppCommand::Redo.localized_label(locale),
                                    &Self::format_shortcuts(
                                        &ctx,
                                        &[
                                            (
                                                egui::Modifiers::COMMAND
                                                    .plus(egui::Modifiers::SHIFT),
                                                egui::Key::Z,
                                            ),
                                            (egui::Modifiers::COMMAND, egui::Key::Y),
                                        ],
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.redo(&ctx);
                            ui.close();
                        }
                        ui.separator();
                        if ui
                            .add_enabled(
                                has_selection,
                                Self::menu_button(
                                    AppCommand::Cut.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::X,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.cut_selection_to_clipboard(&ctx);
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                has_selection,
                                Self::menu_button(
                                    AppCommand::Copy.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::C,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.copy_selection_to_clipboard(&ctx);
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                has_selection,
                                Self::menu_button(
                                    AppCommand::Paste.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::V,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.paste_from_clipboard(&ctx);
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                has_selection,
                                egui::Button::new(
                                    AppCommand::RepeatSelectionDown.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.repeat_selection_down(&ctx);
                            ui.close();
                            return;
                        }
                        ui.separator();
                        if ui
                            .add(Self::menu_button(
                                AppCommand::OpenPreferences.localized_label(locale),
                                &self
                                    .editor_settings
                                    .keybindings
                                    .open_preferences
                                    .display_text(),
                            ))
                            .clicked()
                        {
                            self.preferences_state.open = true;
                            ui.close();
                        }
                    });

                    ui.menu_button(AppMenu::Row.localized_label(locale), |ui| {
                        ui.set_min_width(180.0);
                        let row_menu_state = self.table_view.edit_menu_state(&self.sheet);
                        show_edit_menu(
                            ui,
                            &mut self.table_view,
                            &mut self.sheet,
                            row_menu_state,
                            locale,
                        );
                    });

                    ui.menu_button(AppMenu::Column.localized_label(locale), |ui| {
                        ui.set_min_width(240.0);
                        let after_effects_available = self.after_effects_is_available(frame);
                        if ui
                            .add_enabled(
                                self.table_view.selected_header_column().is_some()
                                    && after_effects_available,
                                egui::Button::new(
                                    AppCommand::SendColumnToAfterEffects.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.send_selected_column_to_after_effects(frame);
                            ui.close();
                        }
                        ui.separator();
                        self.show_column_edit_menu(ui, &ctx, frame);
                    });

                    ui.menu_button(AppMenu::View.localized_label(locale), |ui| {
                        ui.set_min_width(180.0);
                        let can_zoom_in = self.can_zoom_in();
                        let can_zoom_out = self.can_zoom_out();
                        let can_reset_zoom = self.can_reset_zoom();
                        if ui
                            .add_enabled(
                                can_zoom_in,
                                Self::menu_button(
                                    AppCommand::ZoomIn.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::Equals,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.zoom_in();
                            ctx.request_repaint();
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                can_zoom_out,
                                Self::menu_button(
                                    AppCommand::ZoomOut.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::Minus,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.zoom_out();
                            ctx.request_repaint();
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                can_reset_zoom,
                                Self::menu_button(
                                    AppCommand::ResetZoom.localized_label(locale),
                                    &Self::format_shortcut(
                                        &ctx,
                                        egui::Modifiers::COMMAND,
                                        egui::Key::Num0,
                                    ),
                                ),
                            )
                            .clicked()
                        {
                            self.reset_zoom();
                            ctx.request_repaint();
                            ui.close();
                        }
                        ui.separator();
                        if ui
                            .checkbox(
                                &mut self.app_settings.always_on_top,
                                AppCommand::ToggleAlwaysOnTop.localized_label(locale),
                            )
                            .changed()
                        {
                            ctx.request_repaint();
                        }
                        ui.separator();

                        let mut show_minimap = self.table_view.show_minimap();
                        if ui
                            .scope(|ui| {
                                Self::menu_checkbox_with_shortcut(
                                    ui,
                                    &mut show_minimap,
                                    AppCommand::ToggleMinimap.localized_label(locale),
                                    &self
                                        .editor_settings
                                        .keybindings
                                        .toggle_minimap
                                        .display_text(),
                                )
                            })
                            .inner
                            .changed()
                        {
                            self.table_view.set_show_minimap(show_minimap);
                            ctx.request_repaint();
                        }

                        ui.separator();
                        ui.menu_button(self.theme_menu_label(), |ui| {
                            self.show_theme_menu(ui, &ctx);
                        });
                    });

                    ui.menu_button(AppMenu::Help.localized_label(locale), |ui| {
                        ui.set_min_width(180.0);
                        if ui
                            .button(AppCommand::ShowAbout.localized_label(locale))
                            .clicked()
                        {
                            self.show_about_dialog = true;
                            ui.close();
                        }
                    });

                    ui.separator();
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    let header_text = |text: String| egui::RichText::new(text).size(15.0).strong();
                    let duration_text = egui::RichText::new(self.sheet.duration_text())
                        .size(18.0)
                        .strong();

                    ui.label(header_text(self.duration_header_label().to_owned()));
                    ui.label(duration_text);
                    ui.label(header_text(format!(
                        " ({})",
                        self.sheet.effective_frame_count()
                    )));
                    ui.separator();
                    ui.label(header_text(format!("fps: {}", self.sheet.fps())));
                    ui.separator();
                });
            });
    }

    pub fn show_app_footer(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("app_footer")
            .resizable(false)
            .exact_size(30.0)
            .frame(egui::Frame::default().inner_margin(Margin {
                left: 8,
                right: 8,
                top: 3,
                bottom: 3,
            }))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let drag_status = self.table_view.drag_status_message(
                        ui.ctx().input(|i| i.modifiers.command && i.modifiers.shift),
                    );

                    if let Some(message) = drag_status {
                        ui.label(message);
                    } else if let Some(message) = &self.status_message {
                        ui.label(message);
                        ui.separator();
                    }

                    if let Some(height) = self.table_view.selected_timeline_row_count(&self.sheet)
                        && height > 0
                    {
                        ui.label(format!("{height}コマ"));
                    }
                });
            });
    }

    fn show_theme_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let current_theme_id = self.table_settings.theme_id();

        for (theme_id, label) in [
            (0, strings::builtin_theme_label(0, self.locale())),
            (1, strings::builtin_theme_label(1, self.locale())),
            (2, strings::builtin_theme_label(2, self.locale())),
            (3, strings::builtin_theme_label(3, self.locale())),
            (4, strings::builtin_theme_label(4, self.locale())),
            (5, strings::builtin_theme_label(5, self.locale())),
            (6, strings::builtin_theme_label(6, self.locale())),
        ] {
            let label = if theme_id == 2 {
                strings::builtin_theme_label(2, self.locale())
            } else {
                label
            };
            let mut selected = current_theme_id == theme_id;
            let response = ui.checkbox(&mut selected, label);
            if response.clicked() {
                self.table_settings.set_theme_id(theme_id);
                self.app_settings.set_imported_theme_active(false);
                ctx.request_repaint();
                ui.close();
                return;
            }
        }

        if let Ok(imported_themes) = theme_files::list_imported_themes(self.locale())
            && !imported_themes.is_empty()
        {
            ui.separator();
            for imported_theme in imported_themes {
                let locale = self.locale();
                let active = self.app_settings.imported_theme_active()
                    && self.app_settings.imported_theme_path()
                        == Some(imported_theme.path.as_path());
                let mut selected = active;
                let response = ui.checkbox(
                    &mut selected,
                    strings::imported_theme(locale, &imported_theme.name),
                );
                let response = response.on_hover_text(imported_theme.path.display().to_string());
                if response.clicked() {
                    if theme_files::apply_theme_from_path(
                        &mut self.table_settings,
                        &imported_theme.path,
                        locale,
                    )
                    .is_ok()
                    {
                        self.app_settings
                            .set_imported_theme(imported_theme.path, imported_theme.name);
                        ctx.request_repaint();
                        ui.close();
                        return;
                    }
                }
            }
        }

        if current_theme_id == 7 {
            ui.separator();
            ui.label(strings::builtin_theme_label(7, self.locale()));
        }
    }

    fn theme_menu_label(&self) -> String {
        if self.app_settings.imported_theme_active()
            && let Some(name) = self.app_settings.imported_theme_name()
        {
            return format!("{}: {name}", self.theme_label());
        }
        let label = strings::builtin_theme_label(self.table_settings.theme_id(), self.locale());
        format!("{}: {label}", self.theme_label())
    }

    fn show_recent_files_menu(&mut self, ui: &mut egui::Ui) {
        let recent_files = self.app_settings.recent_files().to_vec();
        if recent_files.is_empty() {
            ui.add_enabled(false, egui::Button::new(self.no_recent_files_label()));
            return;
        }

        for path in recent_files {
            let label = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
                .unwrap_or_else(|| path.as_os_str().to_string_lossy().into_owned());

            if ui
                .button(label)
                .on_hover_text(path.display().to_string())
                .clicked()
            {
                self.request_open_recent_sheet(path);
                ui.close();
                return;
            }
        }
    }

    fn show_column_edit_menu(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        _frame: &eframe::Frame,
    ) {
        let selected_col = self.selected_column_index();
        let column_state = ColumnActionState {
            context_col: selected_col,
            target_col: selected_col,
        };

        if ui
            .add_enabled(
                selected_col.is_some(),
                egui::Button::new("列の値をクリップボードにコピー"),
            )
            .clicked()
        {
            self.copy_selected_column_to_clipboard(ctx);
            ui.close();
            return;
        }
        if ui
            .add_enabled(selected_col.is_some(), egui::Button::new("列の行を全選択"))
            .clicked()
        {
            if let Some(col) = selected_col {
                let _ = self.table_view.select_column(&self.sheet, col);
                ctx.request_repaint();
            }
            ui.close();
            return;
        }
        ui.separator();
        if ui
            .add_enabled(
                column_state.supports(ColumnAction::Rename, &self.sheet),
                egui::Button::new(AppCommand::RenameColumn.label()),
            )
            .clicked()
        {
            if let Some(col) = selected_col {
                self.sheet_dialogs_state
                    .open_rename_column(col, self.sheet.column_name(col).to_owned());
            }
            ui.close();
            return;
        }
        if ui
            .add_enabled(
                column_state.supports(ColumnAction::InsertLeft, &self.sheet),
                egui::Button::new("左に列を挿入"),
            )
            .clicked()
        {
            self.table_view.execute_column_action(
                &mut self.sheet,
                &self.table_settings,
                &column_state,
                ColumnAction::InsertLeft,
            );
            ui.close();
            return;
        }
        if ui
            .add_enabled(
                column_state.supports(ColumnAction::InsertRight, &self.sheet),
                egui::Button::new("右に列を挿入"),
            )
            .clicked()
        {
            self.table_view.execute_column_action(
                &mut self.sheet,
                &self.table_settings,
                &column_state,
                ColumnAction::InsertRight,
            );
            ui.close();
            return;
        }
        ui.separator();
        if ui
            .add_enabled(selected_col.is_some(), egui::Button::new("列の値を削除"))
            .clicked()
        {
            self.delete_selected_column_values();
            ui.close();
            return;
        }
        if ui
            .add_enabled(
                column_state.supports(ColumnAction::Delete, &self.sheet),
                egui::Button::new(AppCommand::DeleteColumn.label()),
            )
            .clicked()
        {
            self.table_view.execute_column_action(
                &mut self.sheet,
                &self.table_settings,
                &column_state,
                ColumnAction::Delete,
            );
            ui.close();
            return;
        }
    }

    fn recent_files_label(&self) -> &'static str {
        strings::recent_files(self.locale())
    }

    fn no_recent_files_label(&self) -> &'static str {
        strings::no_recent_files(self.locale())
    }
    fn duration_header_label(&self) -> &'static str {
        strings::duration_with_colon(self.locale())
    }
}
