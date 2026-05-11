use crate::theme_files;
use eframe::egui::{self, Color32, Modal, style::ScrollStyle};
use neosts::settings::editor::{KeybindAction, keybind_action_id};
use neosts::settings::table::{ContinuationLineStyle, TableSettings};
use neosts::{
    AeKaraCellMode, AeKeyframeDataLocale, AeSheetNameSource, AppLocale, AppSettings,
    ClipboardExportFormat, EditorSettings, SheetSettings, TableViewState, strings,
};

const PREFERENCES_MODAL_WIDTH: f32 = 420.0;
const PREFERENCES_MODAL_HEIGHT: f32 = 640.0;
const PREFERENCES_HEADER_HEIGHT: f32 = 32.0;
const PREFERENCES_TAB_HEIGHT: f32 = 32.0;
const PREFERENCES_FOOTER_HEIGHT: f32 = 44.0;
const PREFERENCES_SECTION_GAP: f32 = 12.0;
const PREFERENCES_CONTENT_HEIGHT: f32 = PREFERENCES_MODAL_HEIGHT
    - PREFERENCES_HEADER_HEIGHT
    - PREFERENCES_TAB_HEIGHT
    - PREFERENCES_FOOTER_HEIGHT
    - PREFERENCES_SECTION_GAP * 3.0;
const DEFAULT_SHEET_FPS: u32 = 24;
const DEFAULT_SECONDS_PER_PAGE: u32 = 6;
const DEFAULT_INITIAL_FRAME_COUNT: u32 = 144;
const DEFAULT_INITIAL_COLUMN_COUNT: u32 = 6;
const MIN_SHEET_FPS: u32 = 1;
const MAX_SHEET_FPS: u32 = 120;
const MAX_SHEET_SECONDS: u32 = 100;
const MAX_SECONDS_PER_PAGE: u32 = 6;
const MIN_INITIAL_COLUMN_COUNT: u32 = 1;
const MAX_INITIAL_COLUMN_COUNT: u32 = 76;
const PREFERENCES_SCROLLBAR_WIDTH: f32 = 12.0;
const PREFERENCES_SCROLLBAR_HANDLE_MIN_LENGTH: f32 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferencesTab {
    Display,
    Ae,
    Color,
    Sheet,
    Keybind,
}

#[derive(Clone, Debug)]
pub struct PreferencesState {
    pub open: bool,
    pub tab: PreferencesTab,
    sheet_initial_duration_text: String,
    theme_file_message: Option<String>,
}

impl Default for PreferencesState {
    fn default() -> Self {
        Self {
            open: false,
            tab: PreferencesTab::Display,
            sheet_initial_duration_text: String::new(),
            theme_file_message: None,
        }
    }
}

fn show_preferences_grid(ui: &mut egui::Ui, id: &str, add_rows: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .striped(false)
        .spacing([12.0, 8.0])
        .min_col_width(120.0)
        .show(ui, add_rows);
}

pub fn show_preferences_dialog(
    state: &mut PreferencesState,
    table: &mut TableViewState,
    table_settings: &mut TableSettings,
    ctx: &egui::Context,
    app_settings: &mut AppSettings,
    sheet_settings: &mut SheetSettings,
    editor_settings: &mut EditorSettings,
) {
    if !state.open {
        state.sheet_initial_duration_text.clear();
        state.theme_file_message = None;
        return;
    }

    if state.sheet_initial_duration_text.is_empty() {
        state.sheet_initial_duration_text = format_sheet_duration_text(
            sheet_settings.initial_frame_count as usize,
            sheet_settings.default_fps,
        );
    }

    let mut cell_background_color = table_settings.cell_background_color;
    let mut zero_cell_background_color = table_settings.zero_cell_background_color;
    let mut use_zero_cell_background_color = table_settings.use_zero_cell_background_color();
    let mut show_zero_value_markers = table_settings.show_zero_value_markers();
    let mut show_header_ghosts = table_settings.show_header_ghosts();
    let mut selection_color = table_settings.selection_color;
    let mut hover_color = table_settings.hover_color;
    let mut column_header_background_color = table_settings.column_header_background_color;
    let mut row_header_background_color = table_settings.row_header_background_color;
    let mut color_theme_id = table_settings.color_theme_id();
    let mut up_scroll_trigger_percent = table_settings.up_scroll_trigger_ratio() * 100.0;
    let mut down_scroll_trigger_percent = table_settings.down_scroll_trigger_ratio() * 100.0;
    let locale = app_settings.locale();

    let modal = Modal::new(egui::Id::new("preferences_modal")).backdrop_color(Color32::TRANSPARENT);
    let response = modal.show(ctx, |ui| {
        ui.set_width(PREFERENCES_MODAL_WIDTH);
        ui.set_min_size(egui::vec2(
            PREFERENCES_MODAL_WIDTH,
            PREFERENCES_MODAL_HEIGHT,
        ));
        ui.set_max_size(egui::vec2(
            PREFERENCES_MODAL_WIDTH,
            PREFERENCES_MODAL_HEIGHT,
        ));

        ui.allocate_ui_with_layout(
            egui::vec2(PREFERENCES_MODAL_WIDTH, PREFERENCES_HEADER_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| ui.heading(strings::preferences_title(locale)),
        );
        ui.add_space(PREFERENCES_SECTION_GAP);

        ui.allocate_ui_with_layout(
            egui::vec2(PREFERENCES_MODAL_WIDTH, PREFERENCES_TAB_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.selectable_value(
                    &mut state.tab,
                    PreferencesTab::Display,
                    strings::display_tab(locale),
                );
                ui.selectable_value(&mut state.tab, PreferencesTab::Ae, strings::ae_tab(locale));
                ui.selectable_value(
                    &mut state.tab,
                    PreferencesTab::Color,
                    strings::colors_tab(locale),
                );
                ui.selectable_value(
                    &mut state.tab,
                    PreferencesTab::Sheet,
                    strings::sheet_tab(locale),
                );
                ui.selectable_value(
                    &mut state.tab,
                    PreferencesTab::Keybind,
                    strings::keys_tab(locale),
                );
            },
        );
        ui.add_space(PREFERENCES_SECTION_GAP);
        ui.separator();

        ui.allocate_ui_with_layout(
            egui::vec2(PREFERENCES_MODAL_WIDTH, PREFERENCES_CONTENT_HEIGHT),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.scope(|ui| {
                    let scroll = &mut ui.style_mut().spacing.scroll;
                    *scroll = ScrollStyle::solid();
                    scroll.bar_width = PREFERENCES_SCROLLBAR_WIDTH;
                    scroll.handle_min_length = PREFERENCES_SCROLLBAR_HANDLE_MIN_LENGTH;

                    egui::ScrollArea::vertical()
                        .id_salt("preferences_tab_content")
                        .auto_shrink([false, false])
                        .max_height(PREFERENCES_CONTENT_HEIGHT)
                        .show(ui, |ui| match state.tab {
                            PreferencesTab::Display => show_display_tab(
                                table,
                                table_settings,
                                ui,
                                app_settings,
                                sheet_settings,
                                editor_settings,
                                &mut show_zero_value_markers,
                                &mut show_header_ghosts,
                                &mut up_scroll_trigger_percent,
                                &mut down_scroll_trigger_percent,
                            ),
                            PreferencesTab::Ae => {
                                show_ae_tab(table, ui, editor_settings, app_settings.locale())
                            }
                            PreferencesTab::Color => show_color_tab(
                                state,
                                table,
                                table_settings,
                                ui,
                                ctx.global_style().visuals.dark_mode,
                                app_settings,
                                &mut cell_background_color,
                                &mut zero_cell_background_color,
                                &mut use_zero_cell_background_color,
                                &mut selection_color,
                                &mut hover_color,
                                &mut column_header_background_color,
                                &mut row_header_background_color,
                                &mut color_theme_id,
                            ),
                            PreferencesTab::Sheet => {
                                show_sheet_tab(state, ui, sheet_settings, app_settings.locale())
                            }
                            PreferencesTab::Keybind => {
                                show_keybind_tab(table, ui, editor_settings, app_settings.locale())
                            }
                        });
                });
            },
        );

        ui.add_space(PREFERENCES_SECTION_GAP);
        let remaining_height = (ui.available_height() - PREFERENCES_FOOTER_HEIGHT).max(0.0);
        ui.add_space(remaining_height);
        ui.separator();
        ui.add_space(8.0);
        ui.allocate_ui_with_layout(
            egui::vec2(PREFERENCES_MODAL_WIDTH, PREFERENCES_FOOTER_HEIGHT - 8.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.horizontal_centered(|ui| {
                    if ui
                        .add_sized(
                            egui::vec2(96.0, 28.0),
                            egui::Button::new(strings::close(locale)),
                        )
                        .clicked()
                    {
                        state.open = false;
                    }
                });
            },
        );
    });

    if response.should_close() {
        state.open = false;
    }
}

fn show_keybind_tab(
    table: &mut TableViewState,
    ui: &mut egui::Ui,
    editor_settings: &mut EditorSettings,
    locale: AppLocale,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(strings::key_bindings(locale));
            if ui.button(strings::reset(locale)).clicked() {
                table.reset_keybinds(&mut editor_settings.keybindings);
            }
        });
        ui.add_space(8.0);
        ui.label(strings::keybind_instruction(locale));
        ui.add_space(8.0);
        let grid_width = ui.available_width();
        let binding_width = 132.0;
        let button_width = 88.0;
        let capture_buttons_width = 184.0;
        let grid_spacing = 24.0;
        let label_width =
            (grid_width - binding_width - capture_buttons_width - grid_spacing).max(120.0);

        egui::Grid::new("keybind_grid")
            .num_columns(3)
            .striped(false)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                for action in KeybindAction::ALL {
                    let action_id = keybind_action_id(action);
                    let is_capturing = table.capture_keybind_action_id() == Some(action_id);
                    let row_height = if is_capturing {
                        ui.spacing().interact_size.y * 2.0 + ui.spacing().item_spacing.y
                    } else {
                        ui.spacing().interact_size.y
                    };

                    ui.allocate_ui_with_layout(
                        egui::vec2(label_width, row_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(action.localized_label(locale));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(binding_width, row_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.add_sized(
                                egui::vec2(binding_width, 0.0),
                                egui::Label::new(
                                    egui::RichText::new(table.keybind_binding_text(
                                        &editor_settings.keybindings,
                                        action_id,
                                    ))
                                    .monospace(),
                                )
                                .extend(),
                            );
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(
                            if is_capturing {
                                capture_buttons_width
                            } else {
                                button_width
                            },
                            row_height,
                        ),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if ui
                                .add_sized(
                                    egui::vec2(button_width, 0.0),
                                    egui::Button::new(if is_capturing {
                                        strings::waiting(locale)
                                    } else {
                                        strings::change(locale)
                                    }),
                                )
                                .clicked()
                            {
                                table.begin_keybind_capture(action_id);
                            }

                            if is_capturing
                                && ui
                                    .add_sized(
                                        egui::vec2(button_width, 0.0),
                                        egui::Button::new(strings::cancel(locale)),
                                    )
                                    .clicked()
                            {
                                table.cancel_keybind_capture();
                            }
                        },
                    );

                    ui.end_row();
                }
            });
    });
}

fn show_display_tab(
    _table: &mut TableViewState,
    table_settings: &mut TableSettings,
    ui: &mut egui::Ui,
    app_settings: &mut AppSettings,
    sheet_settings: &mut SheetSettings,
    _editor_settings: &mut EditorSettings,
    show_zero_value_markers: &mut bool,
    show_header_ghosts: &mut bool,
    up_scroll_trigger_percent: &mut f32,
    down_scroll_trigger_percent: &mut f32,
) {
    let locale = app_settings.locale();
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(strings::display_settings(locale));
            if ui.button(strings::reset(locale)).clicked() {
                table_settings.reset_display_settings();
                app_settings.always_on_top = false;
                app_settings.open_new_sheet_dialog_on_startup = true;
                *show_zero_value_markers = table_settings.show_zero_value_markers();
                *show_header_ghosts = table_settings.show_header_ghosts();
            }
        });
        ui.add_space(8.0);

        show_preferences_grid(ui, "display_general_grid", |ui| {
            ui.label(strings::language(locale));
            let mut selected_locale = locale;
            egui::ComboBox::from_id_salt("app_locale")
                .selected_text(strings::locale_native_label(selected_locale))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut selected_locale,
                        AppLocale::Japanese,
                        strings::locale_native_label(AppLocale::Japanese),
                    );
                    ui.selectable_value(
                        &mut selected_locale,
                        AppLocale::English,
                        strings::locale_native_label(AppLocale::English),
                    );
                });
            if selected_locale != locale {
                app_settings.set_locale(selected_locale);
            }
            ui.end_row();

            ui.label(strings::theme(locale));
            let imported_themes = theme_files::list_imported_themes(locale).unwrap_or_default();
            let imported_theme_base_id = 1000_i32;
            let mut theme_id = if app_settings.imported_theme_active() {
                app_settings
                    .imported_theme_path()
                    .and_then(|active_path| {
                        imported_themes
                            .iter()
                            .position(|theme| theme.path == active_path)
                            .map(|index| imported_theme_base_id + index as i32)
                    })
                    .unwrap_or(5)
            } else {
                table_settings.theme_id() as i32
            };
            egui::ComboBox::from_id_salt("app_theme")
                .selected_text(match theme_id {
                    1 => strings::light(locale).to_owned(),
                    2 => strings::dark(locale).to_owned(),
                    3 => strings::builtin_theme_label(3, locale).to_owned(),
                    4 => strings::builtin_theme_label(4, locale).to_owned(),
                    5 => strings::builtin_theme_label(5, locale).to_owned(),
                    6 => strings::builtin_theme_label(6, locale).to_owned(),
                    7 => strings::custom(locale).to_owned(),
                    _ if theme_id >= imported_theme_base_id => imported_themes
                        .get((theme_id - imported_theme_base_id) as usize)
                        .map(|theme| strings::imported_theme(locale, &theme.name))
                        .or_else(|| {
                            app_settings
                                .imported_theme_name()
                                .map(|name| strings::imported_theme(locale, name))
                        })
                        .unwrap_or_else(|| strings::custom(locale).to_owned()),
                    _ => strings::system(locale).to_owned(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut theme_id, 0, strings::system(locale));
                    ui.selectable_value(&mut theme_id, 1, strings::light(locale));
                    ui.selectable_value(&mut theme_id, 2, strings::dark(locale));
                    ui.selectable_value(&mut theme_id, 3, strings::builtin_theme_label(3, locale));
                    ui.selectable_value(&mut theme_id, 4, strings::builtin_theme_label(4, locale));
                    ui.selectable_value(&mut theme_id, 5, strings::builtin_theme_label(5, locale));
                    ui.selectable_value(&mut theme_id, 6, strings::builtin_theme_label(6, locale));
                    for (index, imported_theme) in imported_themes.iter().enumerate() {
                        ui.selectable_value(
                            &mut theme_id,
                            imported_theme_base_id + index as i32,
                            strings::imported_theme(locale, &imported_theme.name),
                        );
                    }
                });
            if theme_id >= imported_theme_base_id {
                if let Some(imported_theme) =
                    imported_themes.get((theme_id - imported_theme_base_id) as usize)
                    && (app_settings.imported_theme_path() != Some(imported_theme.path.as_path())
                        || !app_settings.imported_theme_active())
                    && theme_files::apply_theme_from_path(
                        table_settings,
                        &imported_theme.path,
                        locale,
                    )
                    .is_ok()
                {
                    app_settings.set_imported_theme(
                        imported_theme.path.clone(),
                        imported_theme.name.clone(),
                    );
                }
            } else if theme_id as u8 != table_settings.theme_id()
                || app_settings.imported_theme_active()
            {
                table_settings.set_theme_id(theme_id as u8);
                app_settings.set_imported_theme_active(false);
            }
            ui.end_row();

            ui.label(strings::prefs_scroll_up_boundary(locale));
            if ui
                .add(egui::Slider::new(up_scroll_trigger_percent, 0.0..=95.0).suffix("%"))
                .changed()
            {
                table_settings.set_up_scroll_trigger_ratio(*up_scroll_trigger_percent / 100.0);
            }
            ui.end_row();

            ui.label(strings::prefs_scroll_down_boundary(locale));
            if ui
                .add(egui::Slider::new(down_scroll_trigger_percent, 0.0..=95.0).suffix("%"))
                .changed()
            {
                table_settings.set_down_scroll_trigger_ratio(*down_scroll_trigger_percent / 100.0);
            }
            ui.end_row();

            ui.label(strings::prefs_continuation_min_frames(locale));
            let mut continuation_line_min_run_length =
                table_settings.continuation_line_min_run_length();
            if ui
                .add(egui::DragValue::new(&mut continuation_line_min_run_length).range(0..=999))
                .changed()
            {
                table_settings
                    .set_continuation_line_min_run_length(continuation_line_min_run_length);
            }
            ui.end_row();

            ui.label("");
            ui.small(strings::prefs_continuation_zero_note(locale));
            ui.end_row();

            ui.label(strings::prefs_continuation_type(locale));
            let mut continuation_line_style = table_settings.continuation_line_style();
            egui::ComboBox::from_id_salt("continuation_line_style")
                .selected_text(match continuation_line_style {
                    ContinuationLineStyle::Vertical => strings::prefs_continuation_vertical(locale),
                    ContinuationLineStyle::Horizontal => {
                        strings::prefs_continuation_horizontal(locale)
                    }
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut continuation_line_style,
                        ContinuationLineStyle::Vertical,
                        strings::prefs_continuation_vertical(locale),
                    );
                    ui.selectable_value(
                        &mut continuation_line_style,
                        ContinuationLineStyle::Horizontal,
                        strings::prefs_continuation_horizontal(locale),
                    );
                });
            if continuation_line_style != table_settings.continuation_line_style() {
                table_settings.set_continuation_line_style(continuation_line_style);
            }
            ui.end_row();
        });

        ui.checkbox(
            &mut app_settings.open_new_sheet_dialog_on_startup,
            strings::prefs_open_new_sheet_on_startup(locale),
        );
        if ui
            .checkbox(
                show_zero_value_markers,
                strings::prefs_show_blank_cel_markers(locale),
            )
            .changed()
        {
            table_settings.set_show_zero_value_markers(*show_zero_value_markers);
        }
        if ui
            .checkbox(
                show_header_ghosts,
                strings::prefs_show_header_ghosts(locale),
            )
            .changed()
        {
            table_settings.set_show_header_ghosts(*show_header_ghosts);
        }

        ui.add_space(8.0);
        ui.label(strings::prefs_frame_display(locale));
        show_preferences_grid(ui, "display_frame_grid", |ui| {
            ui.label(strings::prefs_frame_number(locale));
            let mut mode_id = table_settings.frame_header_mode_id();
            egui::ComboBox::from_id_salt("frame_header_mode")
                .selected_text(match mode_id {
                    1 => strings::prefs_page_per_seconds(
                        locale,
                        sheet_settings.default_seconds_per_page,
                    ),
                    2 => strings::prefs_frame_number_absolute(locale).to_owned(),
                    _ => strings::prefs_frame_number_frames(locale).to_owned(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut mode_id,
                        0,
                        strings::prefs_frame_number_frames(locale),
                    );
                    ui.selectable_value(
                        &mut mode_id,
                        1,
                        strings::prefs_page_per_seconds(
                            locale,
                            sheet_settings.default_seconds_per_page,
                        ),
                    );
                    ui.selectable_value(
                        &mut mode_id,
                        2,
                        strings::prefs_frame_number_absolute(locale),
                    );
                });
            if mode_id != table_settings.frame_header_mode_id() {
                table_settings.set_frame_header_mode_id(mode_id);
            }
            ui.end_row();

            ui.label(strings::prefs_display_density(locale));
            let mut density_id = table_settings.frame_header_density_id();
            egui::ComboBox::from_id_salt("frame_header_density")
                .selected_text(match density_id {
                    1 => strings::prefs_density_odd(locale),
                    2 => strings::prefs_density_even(locale),
                    _ => strings::prefs_density_all(locale),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut density_id, 0, strings::prefs_density_all(locale));
                    ui.selectable_value(&mut density_id, 1, strings::prefs_density_odd(locale));
                    ui.selectable_value(&mut density_id, 2, strings::prefs_density_even(locale));
                });
            if density_id != table_settings.frame_header_density_id() {
                table_settings.set_frame_header_density_id(density_id);
            }
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.label(strings::prefs_segment_display(locale));
        show_preferences_grid(ui, "display_segment_grid", |ui| {
            ui.label(strings::prefs_unit(locale));
            let mut mode_id = table_settings.segment_header_mode_id();
            egui::ComboBox::from_id_salt("segment_header_mode")
                .selected_text(match mode_id {
                    1 => strings::prefs_unit_pages(locale),
                    _ => strings::prefs_unit_seconds(locale),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut mode_id, 0, strings::prefs_unit_seconds(locale));
                    ui.selectable_value(&mut mode_id, 1, strings::prefs_unit_pages(locale));
                });
            if mode_id != table_settings.segment_header_mode_id() {
                table_settings.set_segment_header_mode_id(mode_id);
            }
            ui.end_row();

            ui.label(strings::prefs_display_density(locale));
            let mut density_id = table_settings.segment_header_density_id();
            egui::ComboBox::from_id_salt("segment_header_density")
                .selected_text(match density_id {
                    1 => strings::prefs_density_odd(locale),
                    2 => strings::prefs_density_even(locale),
                    _ => strings::prefs_density_all(locale),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut density_id, 0, strings::prefs_density_all(locale));
                    ui.selectable_value(&mut density_id, 1, strings::prefs_density_odd(locale));
                    ui.selectable_value(&mut density_id, 2, strings::prefs_density_even(locale));
                });
            if density_id != table_settings.segment_header_density_id() {
                table_settings.set_segment_header_density_id(density_id);
            }
            ui.end_row();
        });
    });
}

fn show_ae_tab(
    _table: &mut TableViewState,
    ui: &mut egui::Ui,
    editor_settings: &mut EditorSettings,
    locale: AppLocale,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(strings::after_effects_settings(locale));
            if ui.button(strings::reset(locale)).clicked() {
                editor_settings.reset_ae_settings();
            }
        });
        ui.add_space(8.0);

        show_preferences_grid(ui, "ae_settings_grid", |ui| {
            ui.label(strings::column_double_click_copy(locale));
            egui::ComboBox::from_id_salt("clipboard_export_format")
                .selected_text(
                    editor_settings
                        .clipboard_export_format
                        .localized_label(locale),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut editor_settings.clipboard_export_format,
                        ClipboardExportFormat::AfterEffects,
                        ClipboardExportFormat::AfterEffects.localized_label(locale),
                    );
                    ui.selectable_value(
                        &mut editor_settings.clipboard_export_format,
                        ClipboardExportFormat::Autograph,
                        ClipboardExportFormat::Autograph.localized_label(locale),
                    );
                });
            ui.end_row();

            ui.label(strings::ae_keyframe_data(locale));
            ui.add_enabled_ui(
                editor_settings.clipboard_export_format == ClipboardExportFormat::AfterEffects,
                |ui| {
                    egui::ComboBox::from_id_salt("ae_keyframe_data_locale")
                        .selected_text(
                            editor_settings
                                .ae_keyframe_data_locale
                                .localized_label(locale),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut editor_settings.ae_keyframe_data_locale,
                                AeKeyframeDataLocale::Japanese,
                                AeKeyframeDataLocale::Japanese.localized_label(locale),
                            );
                            ui.selectable_value(
                                &mut editor_settings.ae_keyframe_data_locale,
                                AeKeyframeDataLocale::English,
                                AeKeyframeDataLocale::English.localized_label(locale),
                            );
                        });
                },
            );
            ui.end_row();

            ui.label(strings::ae_keyframe_version(locale));
            ui.add_enabled_ui(
                editor_settings.clipboard_export_format == ClipboardExportFormat::AfterEffects,
                |ui| {
                    let field_height = ui.spacing().interact_size.y;
                    ui.add_sized(
                        egui::vec2(96.0, field_height),
                        egui::TextEdit::singleline(&mut editor_settings.ae_keyframe_version)
                            .hint_text("7.0"),
                    );
                },
            );
            ui.end_row();

            ui.label(strings::blank_cel_mode(locale));
            ui.add_enabled_ui(
                editor_settings.clipboard_export_format == ClipboardExportFormat::AfterEffects,
                |ui| {
                    egui::ComboBox::from_id_salt("ae_kara_cell_mode")
                        .selected_text(editor_settings.ae_kara_cell_mode.localized_label(locale))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut editor_settings.ae_kara_cell_mode,
                                AeKaraCellMode::Blinds,
                                AeKaraCellMode::Blinds.localized_label(locale),
                            );
                            ui.selectable_value(
                                &mut editor_settings.ae_kara_cell_mode,
                                AeKaraCellMode::MaxFrameCount,
                                AeKaraCellMode::MaxFrameCount.localized_label(locale),
                            );
                        });
                },
            );
            ui.end_row();

            ui.label(strings::ae_receive_sheet_name(locale));
            ui.add_enabled_ui(
                editor_settings.clipboard_export_format == ClipboardExportFormat::AfterEffects,
                |ui| {
                    egui::ComboBox::from_id_salt("ae_sheet_name_source")
                        .selected_text(editor_settings.ae_sheet_name_source.localized_label(locale))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut editor_settings.ae_sheet_name_source,
                                AeSheetNameSource::CompName,
                                AeSheetNameSource::CompName.localized_label(locale),
                            );
                            ui.selectable_value(
                                &mut editor_settings.ae_sheet_name_source,
                                AeSheetNameSource::ProjectName,
                                AeSheetNameSource::ProjectName.localized_label(locale),
                            );
                            ui.selectable_value(
                                &mut editor_settings.ae_sheet_name_source,
                                AeSheetNameSource::RenderQueueName,
                                AeSheetNameSource::RenderQueueName.localized_label(locale),
                            );
                        });
                },
            );
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.small(strings::ae_copy_note_1(locale));
        ui.small(strings::ae_copy_note_2(locale));
        ui.small(strings::ae_copy_note_3(locale));
    });
}

fn show_sheet_tab(
    state: &mut PreferencesState,
    ui: &mut egui::Ui,
    sheet_settings: &mut SheetSettings,
    locale: AppLocale,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(strings::sheet_settings(locale));
            if ui.button(strings::reset(locale)).clicked() {
                sheet_settings.default_fps = DEFAULT_SHEET_FPS;
                sheet_settings.default_seconds_per_page = DEFAULT_SECONDS_PER_PAGE;
                sheet_settings.initial_frame_count = DEFAULT_INITIAL_FRAME_COUNT;
                sheet_settings.initial_column_count = DEFAULT_INITIAL_COLUMN_COUNT;
                state.sheet_initial_duration_text = format_sheet_duration_text(
                    sheet_settings.initial_frame_count as usize,
                    sheet_settings.default_fps,
                );
            }
        });

        ui.add_space(8.0);
        show_preferences_grid(ui, "sheet_grid", |ui| {
            ui.label("fps");
            if ui
                .add(
                    egui::DragValue::new(&mut sheet_settings.default_fps)
                        .speed(1.0)
                        .range(MIN_SHEET_FPS..=MAX_SHEET_FPS),
                )
                .changed()
            {
                if let Ok(frame_count) = parse_sheet_duration_input(
                    &state.sheet_initial_duration_text,
                    sheet_settings.default_fps,
                ) {
                    sheet_settings.initial_frame_count = frame_count as u32;
                }
            }
            ui.end_row();

            ui.label(strings::prefs_seconds_per_page(locale));
            ui.add(
                egui::DragValue::new(&mut sheet_settings.default_seconds_per_page)
                    .speed(1.0)
                    .range(1..=MAX_SECONDS_PER_PAGE),
            );
            ui.end_row();
            let initial_duration_invalid = parse_sheet_duration_input(
                &state.sheet_initial_duration_text,
                sheet_settings.default_fps,
            )
            .is_err();
            ui.label(strings::prefs_initial_sheet_duration(locale));
            let field_height = ui.spacing().interact_size.y;
            let initial_duration_response = ui.add_sized(
                egui::vec2(96.0, field_height),
                egui::TextEdit::singleline(&mut state.sheet_initial_duration_text)
                    .id_salt("preferences_initial_sheet_duration"),
            );
            if initial_duration_response.changed() {
                if let Ok(frame_count) = parse_sheet_duration_input(
                    &state.sheet_initial_duration_text,
                    sheet_settings.default_fps,
                ) {
                    sheet_settings.initial_frame_count = frame_count as u32;
                }
            }
            ui.end_row();

            ui.label("");
            if initial_duration_invalid {
                ui.small(strings::prefs_invalid_format(locale));
            } else {
                ui.small("");
            }
            ui.end_row();

            ui.label(strings::prefs_initial_column_count(locale));
            if ui
                .add(
                    egui::DragValue::new(&mut sheet_settings.initial_column_count)
                        .speed(1.0)
                        .range(MIN_INITIAL_COLUMN_COUNT..=MAX_INITIAL_COLUMN_COUNT),
                )
                .changed()
            {}
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.label(strings::prefs_sheet_note_fps(locale));
        ui.label(strings::prefs_sheet_note_seconds_per_page(locale));
        ui.label(strings::prefs_sheet_note_duration_independent(locale));
        ui.label(strings::prefs_sheet_note_duration_limit(locale));
    });
}

fn parse_sheet_duration_input(input: &str, fps: u32) -> Result<usize, ()> {
    let trimmed = input.trim();
    let (seconds_str, frames_str) = trimmed.split_once('+').ok_or(())?;
    let seconds = seconds_str.parse::<usize>().map_err(|_| ())?;
    let frames = frames_str.parse::<usize>().map_err(|_| ())?;

    if seconds > MAX_SHEET_SECONDS as usize || frames >= fps as usize {
        return Err(());
    }

    let total_frames = seconds * fps as usize + frames;
    (1..=MAX_SHEET_SECONDS as usize * fps as usize)
        .contains(&total_frames)
        .then_some(total_frames)
        .ok_or(())
}

fn format_sheet_duration_text(frame_count: usize, fps: u32) -> String {
    let seconds = frame_count / fps as usize;
    let frames = frame_count % fps as usize;
    format!("{seconds}+{frames}")
}

#[allow(clippy::too_many_arguments)]
fn sync_color_controls_from_settings(
    table_settings: &TableSettings,
    cell_background_color: &mut Color32,
    zero_cell_background_color: &mut Color32,
    use_zero_cell_background_color: &mut bool,
    selection_color: &mut Color32,
    hover_color: &mut Color32,
    column_header_background_color: &mut Color32,
    row_header_background_color: &mut Color32,
    color_theme_id: &mut u8,
) {
    *cell_background_color = table_settings.cell_background_color;
    *zero_cell_background_color = table_settings.zero_cell_background_color;
    *use_zero_cell_background_color = table_settings.use_zero_cell_background_color();
    *selection_color = table_settings.selection_color;
    *hover_color = table_settings.hover_color;
    *column_header_background_color = table_settings.column_header_background_color;
    *row_header_background_color = table_settings.row_header_background_color;
    *color_theme_id = table_settings.color_theme_id();
}

#[allow(clippy::too_many_arguments)]
fn show_color_tab(
    state: &mut PreferencesState,
    _table: &mut TableViewState,
    table_settings: &mut TableSettings,
    ui: &mut egui::Ui,
    system_dark_mode: bool,
    app_settings: &mut AppSettings,
    cell_background_color: &mut Color32,
    zero_cell_background_color: &mut Color32,
    use_zero_cell_background_color: &mut bool,
    selection_color: &mut Color32,
    hover_color: &mut Color32,
    column_header_background_color: &mut Color32,
    row_header_background_color: &mut Color32,
    color_theme_id: &mut u8,
) {
    ui.vertical(|ui| {
        let locale = app_settings.locale();
        ui.horizontal(|ui| {
            ui.label(strings::prefs_color_settings(locale));
            if ui.button(strings::prefs_reset_colors(locale)).clicked() {
                table_settings.reset_color_settings(system_dark_mode);
                sync_color_controls_from_settings(
                    table_settings,
                    cell_background_color,
                    zero_cell_background_color,
                    use_zero_cell_background_color,
                    selection_color,
                    hover_color,
                    column_header_background_color,
                    row_header_background_color,
                    color_theme_id,
                );
                state.theme_file_message = Some(strings::theme_colors_reset(locale).to_owned());
            }
            if ui.button(strings::prefs_import(locale)).clicked() {
                match theme_files::import_theme_file(table_settings, locale) {
                    Ok((path, name)) => {
                        app_settings.set_imported_theme(path.clone(), name);
                        sync_color_controls_from_settings(
                            table_settings,
                            cell_background_color,
                            zero_cell_background_color,
                            use_zero_cell_background_color,
                            selection_color,
                            hover_color,
                            column_header_background_color,
                            row_header_background_color,
                            color_theme_id,
                        );
                        state.theme_file_message = Some(strings::theme_imported(locale, &path));
                    }
                    Err(message) => {
                        state.theme_file_message = Some(message);
                    }
                }
            }
            if ui.button(strings::prefs_save(locale)).clicked() {
                state.theme_file_message = Some(
                    match theme_files::export_theme_file(table_settings, locale) {
                        Ok(path) => strings::theme_saved(locale, &path),
                        Err(message) => message,
                    },
                );
            }
        });

        ui.add_space(8.0);
        ui.label(strings::prefs_colors(locale));
        ui.add_space(4.0);
        ui.small(strings::prefs_custom_theme_note(locale));
        if let Some(message) = &state.theme_file_message {
            ui.small(message);
        }
        ui.add_space(4.0);
        show_preferences_grid(ui, "color_palette_grid", |ui| {
            ui.label(strings::prefs_alternate_column_brightness(locale));
            if ui
                .add(egui::Slider::from_get_set(-1.0..=1.0, |value| {
                    if let Some(value) = value {
                        let display_amount = value as f32;
                        table_settings.alternate_column_mode =
                            if display_amount.abs() <= f32::EPSILON {
                                neosts::AlternateColumnMode::Off
                            } else {
                                neosts::AlternateColumnMode::Darken
                            };
                        table_settings.set_alternate_darken_amount(-display_amount);
                    }
                    if matches!(
                        table_settings.alternate_column_mode,
                        neosts::AlternateColumnMode::Darken
                    ) {
                        -table_settings.alternate_darken_amount() as f64
                    } else {
                        0.0
                    }
                }))
                .changed()
            {
                table_settings.set_alternate_saturation_amount(0.0);
                table_settings.mark_color_theme_custom();
                *color_theme_id = table_settings.color_theme_id();
            }
            ui.end_row();

            ui.label(strings::prefs_per_second_brightness(locale));
            if ui
                .add(egui::Slider::from_get_set(-1.0..=1.0, |value| {
                    if let Some(value) = value {
                        table_settings.set_alternate_second_darken_amount(-(value as f32));
                    }
                    -table_settings.alternate_second_darken_amount() as f64
                }))
                .changed()
            {
                table_settings.mark_color_theme_custom();
                *color_theme_id = table_settings.color_theme_id();
            }
            ui.end_row();

            ui.label(strings::prefs_background_color(locale));
            if egui::color_picker::color_edit_button_srgba(
                ui,
                cell_background_color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                table_settings.cell_background_color = *cell_background_color;
                table_settings.mark_color_theme_custom();
                *color_theme_id = table_settings.color_theme_id();
            }
            ui.end_row();

            ui.label(strings::prefs_blank_cel_background(locale));
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        use_zero_cell_background_color,
                        strings::prefs_enabled(locale),
                    )
                    .changed()
                {
                    table_settings
                        .set_use_zero_cell_background_color(*use_zero_cell_background_color);
                    table_settings.mark_color_theme_custom();
                    *color_theme_id = table_settings.color_theme_id();
                }
                ui.add_enabled_ui(*use_zero_cell_background_color, |ui| {
                    if egui::color_picker::color_edit_button_srgba(
                        ui,
                        zero_cell_background_color,
                        egui::color_picker::Alpha::Opaque,
                    )
                    .changed()
                    {
                        table_settings.zero_cell_background_color = *zero_cell_background_color;
                        table_settings.mark_color_theme_custom();
                        *color_theme_id = table_settings.color_theme_id();
                    }
                });
            });
            ui.end_row();

            ui.label(strings::prefs_selection_color(locale));
            if egui::color_picker::color_edit_button_srgba(
                ui,
                selection_color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                table_settings.selection_color = *selection_color;
                table_settings.mark_color_theme_custom();
                *color_theme_id = table_settings.color_theme_id();
            }
            ui.end_row();

            ui.label(strings::prefs_hover_color(locale));
            if egui::color_picker::color_edit_button_srgba(
                ui,
                hover_color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                table_settings.hover_color = *hover_color;
                table_settings.mark_color_theme_custom();
                *color_theme_id = table_settings.color_theme_id();
            }
            ui.end_row();

            ui.label(strings::prefs_header_color(locale));
            if egui::color_picker::color_edit_button_srgba(
                ui,
                column_header_background_color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed()
            {
                table_settings.column_header_background_color = *column_header_background_color;
                *row_header_background_color = *column_header_background_color;
                table_settings.row_header_background_color = *column_header_background_color;
                table_settings.mark_color_theme_custom();
                *color_theme_id = table_settings.color_theme_id();
            }
            ui.end_row();
        });
    });
}
