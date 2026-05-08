#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app_bootstrap;
mod app_chrome;
mod app_dirty;
mod app_document;
mod app_history;
mod app_shortcuts;
mod app_theme;
mod file_drop;
mod file_operations;
mod persistence;
mod preferences;
mod sheet_dialogs;
mod theme_files;

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, Rect};
use file_drop::FileDropAction;
use neosts::{
    AppCommand, AppLocale, AppSettings, ClipboardExportFormat, ColumnAction, DisplayMode,
    DisplaySheetState, EditorSettings, RowAction, Sheet, SheetSettings, TableColumnMenuState,
    TableEditMenuState, TableRowMenuState, TableSettings, TableViewEvent, TableViewProps,
    TableViewState, ae, autograph, strings,
};
#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{env, path::PathBuf};
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;
const CELL_SCALE_STEP: f32 = 0.1;
const MIN_CELL_SCALE: f32 = 0.5;
const MAX_CELL_SCALE: f32 = 2.0;
const MAX_HISTORY_ENTRIES: usize = 100;
const IMAGE_PANEL_DEFAULT_WIDTH: f32 = 260.0;
const GITHUB_REPOSITORY_URL: &str = "https://github.com/shimofuri-sequence/neosts";

#[derive(Clone, Debug, PartialEq, Eq)]
struct DocumentSnapshot {
    sheet: DisplaySheetState,
    current_source: Option<PathBuf>,
    loaded_from_sts: bool,
}

#[derive(Clone, Debug)]
struct ColumnMenuPopupState {
    position: egui::Pos2,
    state: TableColumnMenuState,
}

#[derive(Clone, Debug)]
struct DeferredColumnMenuPopupState {
    popup: ColumnMenuPopupState,
    open_at: f64,
}

#[derive(Clone, Debug)]
struct RowMenuPopupState {
    position: egui::Pos2,
    state: TableRowMenuState,
}

fn main() -> eframe::Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let startup_path = args
        .iter()
        .find(|arg| !arg.to_string_lossy().starts_with("--"))
        .map(PathBuf::from);
    let app_icon = app_bootstrap::load_embedded_app_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("NeoSTS")
            .with_inner_size([420.0, 920.0])
            .with_icon(app_icon),
        event_loop_builder: Some(Box::new(|builder| {
            #[cfg(target_os = "macos")]
            builder.with_default_menu(false);
            #[cfg(not(target_os = "macos"))]
            let _ = builder;
        })),
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        "NeoSTS",
        options,
        Box::new(|cc| {
            app_bootstrap::configure_japanese_fonts(&cc.egui_ctx);
            Ok(Box::new(TableApp::new(cc, startup_path.clone())))
        }),
    )
}

struct TableApp {
    sheet: DisplaySheetState,
    current_sheet_loaded_from_sts: bool,
    table_view: TableViewState,
    file_state: file_operations::SheetFileState,
    undo_stack: Vec<DocumentSnapshot>,
    redo_stack: Vec<DocumentSnapshot>,
    committed_snapshot: DocumentSnapshot,
    clean_snapshot: DocumentSnapshot,
    app_settings: AppSettings,
    last_applied_always_on_top: Option<bool>,
    sheet_settings: SheetSettings,
    editor_settings: EditorSettings,
    table_settings: TableSettings,
    preferences_state: preferences::PreferencesState,
    sheet_dialogs_state: sheet_dialogs::SheetDialogsState,
    pending_column_menu: Option<ColumnMenuPopupState>,
    deferred_column_menu: Option<DeferredColumnMenuPopupState>,
    pending_row_menu: Option<RowMenuPopupState>,
    show_about_dialog: bool,
    about_icon_texture: Option<egui::TextureHandle>,
    show_image_panel: bool,
    image_directory: Option<PathBuf>,
    status_message: Option<String>,
    pending_dirty_action: Option<PendingDirtyAction>,
    allow_close_once: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PendingDirtyAction {
    OpenSheet,
    OpenDroppedSheet(PathBuf),
    OpenRecentSheet(PathBuf),
    NewSheet,
    NewSheetFromAfterEffectsSelection,
    ExitApp,
}

impl TableApp {
    pub(crate) fn locale(&self) -> AppLocale {
        self.app_settings.locale()
    }

    fn new(cc: &eframe::CreationContext<'_>, startup_path: Option<PathBuf>) -> Self {
        let mut app = Self {
            sheet: DisplaySheetState::new(Sheet::new(Vec::new())),
            current_sheet_loaded_from_sts: false,
            table_view: TableViewState::default(),
            file_state: file_operations::SheetFileState::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            committed_snapshot: DocumentSnapshot {
                sheet: DisplaySheetState::new(Sheet::new(Vec::new())),
                current_source: None,
                loaded_from_sts: false,
            },
            clean_snapshot: DocumentSnapshot {
                sheet: DisplaySheetState::new(Sheet::new(Vec::new())),
                current_source: None,
                loaded_from_sts: false,
            },
            app_settings: AppSettings::default(),
            last_applied_always_on_top: None,
            sheet_settings: SheetSettings::default(),
            editor_settings: EditorSettings::default(),
            table_settings: TableSettings::default(),
            preferences_state: preferences::PreferencesState::default(),
            sheet_dialogs_state: sheet_dialogs::SheetDialogsState::default(),
            pending_column_menu: None,
            deferred_column_menu: None,
            pending_row_menu: None,
            show_about_dialog: false,
            about_icon_texture: None,
            show_image_panel: false,
            image_directory: None,
            status_message: None,
            pending_dirty_action: None,
            allow_close_once: false,
        };

        ae::cleanup_temp_jsx_files();

        if let Some(storage) = cc.storage {
            persistence::load_from_storage(
                storage,
                &mut app.table_view,
                &mut app.table_settings,
                &mut app.app_settings,
                &mut app.sheet_settings,
                &mut app.editor_settings,
            );
        }

        cc.egui_ctx.options_mut(|options| {
            options.zoom_with_keyboard = false;
        });

        if let Some(path) = startup_path {
            let locale = app.locale();
            if app.file_state.load_startup_path(
                &mut app.sheet,
                &mut app.table_view,
                &mut app.current_sheet_loaded_from_sts,
                app.sheet_settings.default_fps,
                &path,
                locale,
            ) {
                app.reset_document_history();
                app.record_current_file_as_recent();
                app.set_opened_file_status();
            }
        }

        if app.file_state.current_source().is_none()
            && app.app_settings.open_new_sheet_dialog_on_startup
        {
            app.sheet_dialogs_state.open_new_sheet(&app.sheet_settings);
        }
        app.reset_document_history();
        app
    }

    fn show_image_panel(&mut self, ui: &mut egui::Ui) -> Option<Rect> {
        if !self.show_image_panel {
            return None;
        }

        let response = egui::Panel::right("image_panel")
            .resizable(true)
            .default_size(IMAGE_PANEL_DEFAULT_WIDTH)
            .min_size(180.0)
            .show_inside(ui, |ui| {
                ui.heading("画像リスト");
                ui.add_space(8.0);
                if let Some(path) = &self.image_directory {
                    ui.label(format!("画像フォルダ: {}", path.display()));
                } else {
                    ui.label("ここに画像リストや選択行に対応する画像を表示します。");
                }
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label("準備中");
                ui.small("フォルダをこのペインへドロップすると受け取ります。");
            });
        Some(response.response.rect)
    }

    fn handle_file_drop(
        &mut self,
        ctx: &egui::Context,
        table_rect: Rect,
        image_panel_rect: Option<Rect>,
    ) {
        if self.file_drop_blocked_by_modal() {
            return;
        }

        let dropped_files = ctx.input(|input| input.raw.dropped_files.clone());
        if dropped_files.is_empty() {
            return;
        }

        let drop_pos = ctx.input(|input| input.pointer.latest_pos().or(input.pointer.hover_pos()));
        if let Some(action) = file_drop::resolve_dropped_action(
            &dropped_files,
            drop_pos,
            table_rect,
            image_panel_rect,
        ) {
            match action {
                FileDropAction::SetImageDirectory(path) => {
                    self.image_directory = Some(path);
                }
                FileDropAction::OpenSheetFile(path) => {
                    self.request_open_dropped_sheet(path);
                }
            }
        }
    }

    fn draw_drop_overlay(
        &self,
        ctx: &egui::Context,
        table_rect: Rect,
        image_panel_rect: Option<Rect>,
    ) {
        if self.file_drop_blocked_by_modal() {
            return;
        }

        let hovered_files = ctx.input(|input| input.raw.hovered_files.clone());
        if hovered_files.is_empty() {
            return;
        }

        ctx.request_repaint();

        let pointer_pos =
            ctx.input(|input| input.pointer.hover_pos().or(input.pointer.latest_pos()));
        if let Some(overlay) = file_drop::resolve_hover_overlay(
            &hovered_files,
            pointer_pos,
            table_rect,
            image_panel_rect,
            self.locale(),
        ) {
            ctx.set_cursor_icon(if overlay.accepts_all {
                egui::CursorIcon::Copy
            } else {
                egui::CursorIcon::NotAllowed
            });
            file_drop::draw_drop_overlay(ctx, overlay);
        }
    }

    fn file_drop_blocked_by_modal(&self) -> bool {
        self.preferences_state.open
            || self.sheet_dialogs_state.is_any_open()
            || self.show_about_dialog
            || self.pending_dirty_action.is_some()
    }

    fn show_column_menu_popup(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(popup) = self.pending_column_menu.clone() else {
            return;
        };
        let locale = self.locale();

        let mut close_popup = false;
        let area = egui::Area::new(egui::Id::new("column_header_context_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(popup.position)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    Self::apply_context_menu_visuals(ui);
                    ui.set_min_width(180.0);
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        if popup.state.target_col.is_none() {
                            ui.label(strings::right_click_column_header(locale));
                            return;
                        }

                        if ui
                            .add_enabled(
                                popup.state.target_col.is_some()
                                    && self.after_effects_is_available(frame),
                                context_menu_item(
                                    AppCommand::SendColumnToAfterEffects.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            if let Some(target_col) = popup.state.target_col {
                                self.send_column_to_after_effects(target_col, frame);
                            }
                            close_popup = true;
                            ui.close();
                            return;
                        }

                        ui.separator();

                        if ui
                            .add_enabled(
                                popup.state.can_rename,
                                context_menu_item(AppCommand::RenameColumn.localized_label(locale)),
                            )
                            .clicked()
                        {
                            if let (Some(column), Some(current_name)) =
                                (popup.state.target_col, popup.state.current_name.clone())
                            {
                                self.sheet_dialogs_state
                                    .open_rename_column(column, current_name);
                            }
                            close_popup = true;
                            ui.close();
                            return;
                        }

                        if ui
                            .add_enabled(
                                popup.state.can_delete,
                                context_menu_item(AppCommand::DeleteColumn.localized_label(locale)),
                            )
                            .clicked()
                        {
                            self.table_view.execute_context_column_action(
                                &mut self.sheet,
                                &self.table_settings,
                                ColumnAction::Delete,
                            );
                            close_popup = true;
                            ui.close();
                            return;
                        }

                        ui.separator();

                        if ui
                            .add_enabled(
                                popup.state.can_insert_left,
                                context_menu_item(
                                    AppCommand::InsertColumnLeft.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.table_view.execute_context_column_action(
                                &mut self.sheet,
                                &self.table_settings,
                                ColumnAction::InsertLeft,
                            );
                            close_popup = true;
                            ui.close();
                            return;
                        }

                        if ui
                            .add_enabled(
                                popup.state.can_insert_right,
                                context_menu_item(
                                    AppCommand::InsertColumnRight.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.table_view.execute_context_column_action(
                                &mut self.sheet,
                                &self.table_settings,
                                ColumnAction::InsertRight,
                            );
                            close_popup = true;
                            ui.close();
                        }
                    });
                });
            });

        let popup_rect = area.response.rect;
        let escape_pressed = ctx.input(|input| input.key_pressed(Key::Escape));
        let clicked_outside = ctx.input(|input| {
            input.pointer.any_pressed()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|pos| !popup_rect.contains(pos))
        });

        if close_popup || escape_pressed || clicked_outside {
            self.pending_column_menu = None;
        }
    }

    fn open_deferred_column_menu_if_ready(&mut self, ctx: &egui::Context) {
        let Some(deferred) = self.deferred_column_menu.clone() else {
            return;
        };

        let now = ctx.input(|i| i.time);
        if now >= deferred.open_at {
            self.pending_column_menu = Some(deferred.popup);
            self.deferred_column_menu = None;
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after_secs((deferred.open_at - now) as f32);
        }
    }

    fn show_row_menu_popup(&mut self, ctx: &egui::Context) {
        let Some(popup) = self.pending_row_menu.clone() else {
            return;
        };
        let locale = self.locale();

        let mut close_popup = false;
        let area = egui::Area::new(egui::Id::new("row_header_context_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(popup.position)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    Self::apply_context_menu_visuals(ui);
                    ui.set_min_width(200.0);
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        if popup.state.context_row.is_none() {
                            ui.label(strings::right_click_row_header(locale));
                            return;
                        }

                        if ui
                            .add_enabled(
                                popup.state.can_punch,
                                context_menu_item(AppCommand::PunchRows.localized_label(locale)),
                            )
                            .clicked()
                        {
                            self.table_view
                                .execute_context_row_action(&mut self.sheet, RowAction::Punch);
                            close_popup = true;
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                popup.state.can_unpunch,
                                context_menu_item(AppCommand::UnpunchRows.localized_label(locale)),
                            )
                            .clicked()
                        {
                            self.table_view
                                .execute_context_row_action(&mut self.sheet, RowAction::Unpunch);
                            close_popup = true;
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                popup.state.can_append_above,
                                context_menu_item(
                                    AppCommand::AppendRowsAbove.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.table_view.execute_context_row_action(
                                &mut self.sheet,
                                RowAction::AppendAbove,
                            );
                            close_popup = true;
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                popup.state.can_append_below,
                                context_menu_item(
                                    AppCommand::AppendRowsBelow.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.table_view.execute_context_row_action(
                                &mut self.sheet,
                                RowAction::AppendBelow,
                            );
                            close_popup = true;
                            ui.close();
                            return;
                        }
                        if ui
                            .add_enabled(
                                popup.state.can_delete_special,
                                context_menu_item(
                                    AppCommand::DeleteSpecialRows.localized_label(locale),
                                ),
                            )
                            .clicked()
                        {
                            self.table_view.execute_context_row_action(
                                &mut self.sheet,
                                RowAction::DeleteSpecial,
                            );
                            close_popup = true;
                            ui.close();
                        }
                    });
                });
            });

        let popup_rect = area.response.rect;
        let escape_pressed = ctx.input(|input| input.key_pressed(Key::Escape));
        let clicked_outside = ctx.input(|input| {
            input.pointer.any_pressed()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|pos| !popup_rect.contains(pos))
        });

        if close_popup || escape_pressed || clicked_outside {
            self.pending_row_menu = None;
        }
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_about_dialog {
            return;
        }

        let mut open = self.show_about_dialog;
        egui::Modal::new(egui::Id::new("about_neosts_modal")).show(ctx, |ui| {
            ui.set_width(320.0);
            if let Some(texture) = self.about_icon_texture(ctx) {
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(egui::vec2(96.0, 96.0))
                            .maintain_aspect_ratio(true),
                    );
                });
                ui.add_space(10.0);
            }
            ui.heading("NeoSTS");
            ui.add_space(8.0);
            ui.label(self.about_description_label());
            ui.add_space(8.0);
            ui.monospace(format!("Version {}", env!("CARGO_PKG_VERSION")));
            ui.label(format!("Authors: {}", env!("CARGO_PKG_AUTHORS")));
            ui.add_space(12.0);
            if ui.button(self.close_label()).clicked() {
                open = false;
            }
        });
        self.show_about_dialog = open;
    }

    fn about_icon_texture(&mut self, ctx: &egui::Context) -> Option<&egui::TextureHandle> {
        if self.about_icon_texture.is_none() {
            let icon = eframe::icon_data::from_png_bytes(include_bytes!(
                "../assets/NeoSTS_icon_about.png"
            ))
            .ok()?;
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [icon.width as usize, icon.height as usize],
                &icon.rgba,
            );
            let texture =
                ctx.load_texture("about_neosts_icon", image, egui::TextureOptions::LINEAR);
            self.about_icon_texture = Some(texture);
        }

        self.about_icon_texture.as_ref()
    }
}

fn context_menu_item(label: &'static str) -> egui::Button<'static> {
    egui::Button::new(label)
        .frame(true)
        .frame_when_inactive(false)
        .corner_radius(4)
        .min_size(egui::vec2(160.0, 24.0))
}

fn show_edit_menu(
    ui: &mut egui::Ui,
    table_view: &mut TableViewState,
    sheet: &mut Sheet,
    state: TableEditMenuState,
    locale: AppLocale,
) {
    ui.set_min_width(180.0);

    if ui
        .add_enabled(
            state.can_punch,
            egui::Button::new(AppCommand::PunchRows.localized_label(locale)),
        )
        .clicked()
    {
        table_view.execute_edit_menu_action(sheet, RowAction::Punch);
        ui.close();
        return;
    }
    if ui
        .add_enabled(
            state.can_unpunch,
            egui::Button::new(AppCommand::UnpunchRows.localized_label(locale)),
        )
        .clicked()
    {
        table_view.execute_edit_menu_action(sheet, RowAction::Unpunch);
        ui.close();
        return;
    }
    if ui
        .add_enabled(
            state.can_append_above,
            egui::Button::new(AppCommand::AppendRowsAbove.localized_label(locale)),
        )
        .clicked()
    {
        table_view.execute_edit_menu_action(sheet, RowAction::AppendAbove);
        ui.close();
        return;
    }
    if ui
        .add_enabled(
            state.can_append_below,
            egui::Button::new(AppCommand::AppendRowsBelow.localized_label(locale)),
        )
        .clicked()
    {
        table_view.execute_edit_menu_action(sheet, RowAction::AppendBelow);
        ui.close();
        return;
    }
    if ui
        .add_enabled(
            state.can_delete_special,
            egui::Button::new(AppCommand::DeleteSpecialRows.localized_label(locale)),
        )
        .clicked()
    {
        table_view.execute_edit_menu_action(sheet, RowAction::DeleteSpecial);
        ui.close();
    }
}

impl eframe::App for TableApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.render_root_viewport(ui, frame);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        persistence::save_to_storage(
            storage,
            &self.table_view,
            &self.table_settings,
            &self.app_settings,
            &self.sheet_settings,
            &self.editor_settings,
        );
    }
}

impl TableApp {
    const DEFAULT_CELL_SCALE: f32 = 1.0;
    const CELL_SCALE_EPSILON: f32 = 0.001;

    fn render_root_viewport(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|input| input.viewport().close_requested()) {
            if self.allow_close_once {
                self.allow_close_once = false;
            } else if self.has_unsaved_changes() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if self.pending_dirty_action.is_none() {
                    self.pending_dirty_action = Some(PendingDirtyAction::ExitApp);
                }
            }
        }

        app_theme::apply_app_theme_visuals(&ctx, &self.table_settings);
        self.apply_window_title(&ctx);
        self.table_settings
            .sync_default_color_theme(ctx.global_style().visuals.dark_mode);
        self.handle_file_shortcuts(&ctx, frame);
        self.handle_cell_scale_shortcuts(&ctx);
        self.handle_undo_redo_shortcuts(&ctx);
        self.handle_table_keybind_shortcuts(&ctx);
        self.show_app_header(ui, frame);
        self.show_app_footer(ui);
        let image_panel_rect = self.show_image_panel(ui);
        let table_rect = ui.available_rect_before_wrap();
        self.handle_file_drop(&ctx, table_rect, image_panel_rect);
        let kara_cell_x_value = self.ae_kara_cell_x_value();
        let sheet_fps = self.sheet.fps();
        let table_events = self.table_view.show(
            ui,
            TableViewProps {
                sheet: self.sheet.sheet_mut(),
                default_frames_per_page: self.sheet_settings.frames_per_page(sheet_fps) as usize,
                display_mode: DisplayMode::FullFrame,
                keybindings: &mut self.editor_settings.keybindings,
                settings: &mut self.table_settings,
                modal_open: self.preferences_state.open
                    || self.sheet_dialogs_state.is_any_open()
                    || self.show_about_dialog
                    || self.pending_dirty_action.is_some()
                    || self.pending_column_menu.is_some()
                    || self.pending_row_menu.is_some(),
                context_menu_open: self.pending_column_menu.is_some()
                    || self.pending_row_menu.is_some(),
                kara_cell_x_value,
            },
        );
        self.handle_table_events(&ctx, table_events);
        self.open_deferred_column_menu_if_ready(&ctx);
        self.show_column_menu_popup(&ctx, frame);
        self.show_row_menu_popup(&ctx);
        let locale = self.locale();
        let dialog_actions = sheet_dialogs::show_sheet_dialogs(
            &mut self.sheet_dialogs_state,
            &ctx,
            &mut self.sheet,
            locale,
        );
        for action in dialog_actions {
            self.apply_sheet_dialog_action(&ctx, action);
        }
        preferences::show_preferences_dialog(
            &mut self.preferences_state,
            &mut self.table_view,
            &mut self.table_settings,
            &ctx,
            &mut self.app_settings,
            &mut self.sheet_settings,
            &mut self.editor_settings,
        );
        self.show_about_dialog(&ctx);
        self.show_dirty_confirm_dialog(&ctx, frame);
        self.draw_drop_overlay(&ctx, table_rect, image_panel_rect);
        self.apply_window_level(&ctx);
        self.commit_document_history_if_changed();
    }

    fn menu_button<'a>(label: &'a str, shortcut: &'a str) -> egui::Button<'a> {
        egui::Button::new(label).shortcut_text(shortcut)
    }

    fn menu_checkbox_with_shortcut(
        ui: &mut egui::Ui,
        checked: &mut bool,
        label: &str,
        shortcut: &str,
    ) -> egui::Response {
        ui.horizontal(|ui| {
            let response = ui.checkbox(checked, label);
            ui.add_space(8.0);
            ui.label(egui::RichText::new(shortcut).weak());
            response
        })
        .inner
    }

    fn format_shortcut(ctx: &egui::Context, modifiers: Modifiers, key: Key) -> String {
        ctx.format_shortcut(&KeyboardShortcut::new(modifiers, key))
    }

    fn format_shortcuts(ctx: &egui::Context, shortcuts: &[(Modifiers, Key)]) -> String {
        shortcuts
            .iter()
            .map(|(modifiers, key)| Self::format_shortcut(ctx, *modifiers, *key))
            .collect::<Vec<_>>()
            .join(" / ")
    }

    fn apply_context_menu_visuals(ui: &mut egui::Ui) {
        let visuals = &mut ui.style_mut().visuals.widgets;
        visuals.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.active.bg_stroke = egui::Stroke::NONE;
        visuals.open.bg_stroke = egui::Stroke::NONE;
    }

    fn zoom_in(&mut self) {
        let next_scale = (self.table_settings.cell_scale() + CELL_SCALE_STEP)
            .clamp(MIN_CELL_SCALE, MAX_CELL_SCALE);
        self.table_settings
            .set_cell_scale((next_scale * 10.0).round() / 10.0);
    }

    fn zoom_out(&mut self) {
        let next_scale = (self.table_settings.cell_scale() - CELL_SCALE_STEP)
            .clamp(MIN_CELL_SCALE, MAX_CELL_SCALE);
        self.table_settings
            .set_cell_scale((next_scale * 10.0).round() / 10.0);
    }

    fn reset_zoom(&mut self) {
        self.table_settings.set_cell_scale(Self::DEFAULT_CELL_SCALE);
    }

    fn can_zoom_in(&self) -> bool {
        self.table_settings.cell_scale() < MAX_CELL_SCALE - Self::CELL_SCALE_EPSILON
    }

    fn can_zoom_out(&self) -> bool {
        self.table_settings.cell_scale() > MIN_CELL_SCALE + Self::CELL_SCALE_EPSILON
    }

    fn can_reset_zoom(&self) -> bool {
        (self.table_settings.cell_scale() - Self::DEFAULT_CELL_SCALE).abs()
            > Self::CELL_SCALE_EPSILON
    }

    fn displayed_sheet_name(&self) -> String {
        let mut name = self.sheet.name().to_owned();
        if self.has_unsaved_changes() {
            name.push('*');
        }
        name
    }

    fn ae_kara_cell_x_value(&self) -> Option<i64> {
        match self.editor_settings.ae_kara_cell_mode {
            neosts::AeKaraCellMode::Blinds => None,
            neosts::AeKaraCellMode::MaxFrameCount => Some(100),
        }
    }

    fn apply_window_level(&mut self, ctx: &egui::Context) {
        if self.last_applied_always_on_top == Some(self.app_settings.always_on_top) {
            return;
        }

        let level = if self.app_settings.always_on_top {
            egui::viewport::WindowLevel::AlwaysOnTop
        } else {
            egui::viewport::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
        self.last_applied_always_on_top = Some(self.app_settings.always_on_top);
        ctx.request_repaint();
    }

    fn apply_window_title(&self, ctx: &egui::Context) {
        let title = if self.app_settings.always_on_top {
            format!("{} - NeoSTS [✔]", self.displayed_sheet_name())
        } else {
            format!("{} - NeoSTS", self.displayed_sheet_name())
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    fn open_github_repository(&self, ctx: &egui::Context) {
        ctx.open_url(egui::OpenUrl {
            url: GITHUB_REPOSITORY_URL.to_owned(),
            new_tab: true,
        });
    }

    fn handle_table_events(&mut self, ctx: &egui::Context, events: Vec<TableViewEvent>) {
        for event in events {
            match event {
                TableViewEvent::CopyRequested => {
                    self.copy_selection_to_clipboard(ctx);
                }
                TableViewEvent::CutRequested => {
                    self.cut_selection_to_clipboard(ctx);
                }
                TableViewEvent::PasteRequested { text } => {
                    self.paste_text_into_selection(ctx, &text);
                }
                TableViewEvent::OpenAppendRowsRequested { row, insert_above } => {
                    self.sheet_dialogs_state.open_append_rows(
                        row,
                        insert_above,
                        self.sheet
                            .fps()
                            .saturating_mul(self.sheet_settings.default_seconds_per_page),
                    );
                }
                TableViewEvent::OpenRenameColumnRequested {
                    column,
                    current_name,
                } => {
                    self.pending_column_menu = None;
                    self.deferred_column_menu = None;
                    self.sheet_dialogs_state
                        .open_rename_column(column, current_name);
                }
                TableViewEvent::ColumnHeaderContextMenuRequested { position, state } => {
                    self.pending_row_menu = None;
                    self.pending_column_menu = None;
                    self.deferred_column_menu = Some(DeferredColumnMenuPopupState {
                        popup: ColumnMenuPopupState { position, state },
                        open_at: ctx.input(|i| i.time) + 0.3,
                    });
                }
                TableViewEvent::RowHeaderContextMenuRequested { position, state } => {
                    self.pending_column_menu = None;
                    self.deferred_column_menu = None;
                    self.pending_row_menu = Some(RowMenuPopupState { position, state });
                }
                TableViewEvent::ColumnHeaderSecondaryDoubleClicked {
                    column_index,
                    column_name,
                    values,
                } => {
                    self.pending_column_menu = None;
                    self.deferred_column_menu = None;
                    let flash_color = match self.editor_settings.clipboard_export_format {
                        ClipboardExportFormat::AfterEffects => egui::Color32::from_rgb(220, 50, 50),
                        ClipboardExportFormat::Autograph => egui::Color32::from_rgb(28, 138, 68),
                    };
                    self.table_view.flash_column_header(
                        column_index,
                        ctx.input(|i| i.time),
                        flash_color,
                    );
                    let ae_keyframe_version = self.editor_settings.normalized_ae_keyframe_version();
                    let text = match self.editor_settings.clipboard_export_format {
                        ClipboardExportFormat::AfterEffects => ae::keyframe_data(
                            &column_name,
                            &values,
                            self.sheet.fps(),
                            &ae_keyframe_version,
                            self.editor_settings.ae_keyframe_data_locale,
                            neosts::AeKaraCellMode::Blinds,
                            0,
                        ),
                        ClipboardExportFormat::Autograph => {
                            autograph::keyframe_data(&column_name, &values, self.sheet.fps())
                        }
                    };
                    ctx.copy_text(text);
                }
            }
        }
    }

    fn apply_sheet_dialog_action(
        &mut self,
        ctx: &egui::Context,
        action: sheet_dialogs::SheetDialogAction,
    ) {
        match action {
            sheet_dialogs::SheetDialogAction::AppendRows {
                row,
                count,
                insert_above,
            } => {
                self.table_view
                    .append_special_rows(&mut self.sheet, row, count, insert_above);
            }
            sheet_dialogs::SheetDialogAction::ResizeSheet {
                sheet_name,
                total_frames,
                columns,
                fps,
            } => {
                self.sheet.set_name(sheet_name);
                self.sheet.set_fps(fps);
                self.sheet.resize_sheet(columns, total_frames);
                self.table_view
                    .sync_after_sheet_resize(&self.sheet, &self.table_settings);
            }
            sheet_dialogs::SheetDialogAction::NewSheet {
                sheet_name,
                total_frames,
                columns,
                fps,
            } => {
                self.sheet
                    .replace_with_blank_sheet(columns.max(1), total_frames, fps);
                self.sheet.set_name(sheet_name);
                self.current_sheet_loaded_from_sts = false;
                self.table_view.reset_for_new_sheet();
                self.table_view
                    .sync_column_widths(&self.sheet, &self.table_settings);
                self.reset_document_history();
            }
            sheet_dialogs::SheetDialogAction::RenameColumn { column, name } => {
                let _ = self
                    .table_view
                    .rename_column(&mut self.sheet, column, &name);
            }
        }

        ctx.request_repaint();
    }
}

#[cfg(target_os = "windows")]
fn current_window_hwnd(frame: &eframe::Frame) -> Option<isize> {
    let raw = frame.window_handle().ok()?.as_raw();
    match raw {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

#[cfg(not(target_os = "windows"))]
fn current_window_hwnd(_frame: &eframe::Frame) -> Option<isize> {
    None
}

impl TableApp {
    fn about_description_label(&self) -> &'static str {
        strings::about_description(self.locale())
    }

    fn close_label(&self) -> &'static str {
        strings::close(self.locale())
    }

    fn theme_label(&self) -> &'static str {
        strings::theme(self.locale())
    }
}
