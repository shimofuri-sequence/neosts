use eframe::egui;
use neosts::display::DEFAULT_SHEET_NAME;
use neosts::{AppLocale, DisplaySheetState, SheetSettings, strings};

const MAX_SHEET_SECONDS: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetDialogAction {
    AppendRows {
        row: usize,
        count: usize,
        insert_above: bool,
    },
    ResizeSheet {
        sheet_name: String,
        total_frames: usize,
        columns: usize,
        fps: u32,
    },
    NewSheet {
        sheet_name: String,
        total_frames: u32,
        columns: u32,
        fps: u32,
    },
    RenameColumn {
        column: usize,
        name: String,
    },
}

#[derive(Default)]
pub struct SheetDialogsState {
    append_rows: Option<AppendRowsDialogState>,
    new_sheet: Option<SheetResizeDialogState>,
    resize_sheet: Option<SheetResizeDialogState>,
    rename_column: Option<RenameColumnDialogState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AppendRowsDialogState {
    row: usize,
    insert_above: bool,
    max_count: usize,
    count_text: String,
    select_on_open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SheetResizeDialogState {
    sheet_name: String,
    duration_text: String,
    columns: u32,
    fps_mode: SheetResizeFpsMode,
    custom_fps: u32,
    select_duration_on_open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RenameColumnDialogState {
    column: usize,
    name: String,
    select_on_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SheetResizeFpsMode {
    Fps24,
    Fps30,
    Custom,
}

impl SheetDialogsState {
    pub fn is_any_open(&self) -> bool {
        self.append_rows.is_some()
            || self.new_sheet.is_some()
            || self.resize_sheet.is_some()
            || self.rename_column.is_some()
    }

    pub fn open_append_rows(&mut self, row: usize, insert_above: bool, max_count: u32) {
        self.append_rows = Some(AppendRowsDialogState {
            row,
            insert_above,
            max_count: max_count.max(1) as usize,
            count_text: "12".to_owned(),
            select_on_open: true,
        });
    }

    pub fn open_new_sheet(&mut self, sheet_settings: &SheetSettings) {
        let (fps_mode, custom_fps) = match sheet_settings.default_fps {
            24 => (SheetResizeFpsMode::Fps24, 24),
            30 => (SheetResizeFpsMode::Fps30, 30),
            fps => (SheetResizeFpsMode::Custom, fps),
        };
        self.new_sheet = Some(SheetResizeDialogState {
            sheet_name: DEFAULT_SHEET_NAME.to_owned(),
            duration_text: format_sheet_duration_text(
                sheet_settings.initial_frame_count() as usize,
                sheet_settings.default_fps,
            ),
            columns: sheet_settings.initial_column_count,
            fps_mode,
            custom_fps,
            select_duration_on_open: true,
        });
    }

    pub fn open_resize_sheet(&mut self, sheet: &DisplaySheetState) {
        let (fps_mode, custom_fps) = match sheet.fps() {
            24 => (SheetResizeFpsMode::Fps24, 24),
            30 => (SheetResizeFpsMode::Fps30, 30),
            fps => (SheetResizeFpsMode::Custom, fps),
        };
        self.resize_sheet = Some(SheetResizeDialogState {
            sheet_name: sheet.name().to_owned(),
            duration_text: sheet.duration_text(),
            columns: sheet.column_count() as u32,
            fps_mode,
            custom_fps,
            select_duration_on_open: true,
        });
    }

    pub fn open_rename_column(&mut self, column: usize, current_name: String) {
        self.rename_column = Some(RenameColumnDialogState {
            column,
            name: current_name,
            select_on_open: true,
        });
    }
}

pub fn show_sheet_dialogs(
    state: &mut SheetDialogsState,
    ctx: &egui::Context,
    sheet: &mut DisplaySheetState,
    locale: AppLocale,
) -> Vec<SheetDialogAction> {
    let mut actions = Vec::new();

    if let Some(action) = show_append_rows_dialog(state, ctx, locale) {
        actions.push(action);
    }
    if let Some(action) = show_resize_sheet_dialog(state, ctx, sheet, locale) {
        actions.push(action);
    }
    if let Some(action) = show_new_sheet_dialog(state, ctx, locale) {
        actions.push(action);
    }
    if let Some(action) = show_rename_column_dialog(state, ctx, sheet, locale) {
        actions.push(action);
    }

    actions
}

fn show_append_rows_dialog(
    state: &mut SheetDialogsState,
    ctx: &egui::Context,
    locale: AppLocale,
) -> Option<SheetDialogAction> {
    let Some(mut pending) = state.append_rows.take() else {
        return None;
    };

    let title = if pending.insert_above {
        strings::append_rows_above(locale)
    } else {
        strings::append_rows_below(locale)
    };
    let mut confirmed = false;
    let mut cancelled = false;
    let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    egui::Modal::new(egui::Id::new("append_modal")).show(ctx, |ui| {
        ui.set_min_width(220.0);
        ui.heading(title);
        ui.add_space(8.0);
        ui.label(strings::rows_to_append(locale, pending.max_count));
        let enter_pressed = dialog_submit_enter_pressed(ui);
        let response = ui.add(egui::TextEdit::singleline(&mut pending.count_text).return_key(None));
        if pending.select_on_open {
            response.request_focus();
            select_all_text_edit(ui, response.id, pending.count_text.chars().count());
            pending.select_on_open = false;
        } else if response.gained_focus() {
            select_all_text_edit(ui, response.id, pending.count_text.chars().count());
        }
        let ok_enabled = pending
            .count_text
            .trim()
            .parse::<usize>()
            .is_ok_and(|n| (1..=pending.max_count).contains(&n));
        if !ok_enabled {
            ui.small(strings::append_rows_help(locale, pending.max_count));
        }
        if enter_pressed && ok_enabled {
            confirmed = true;
        }
        ui.add_space(8.0);
        show_dialog_action_buttons(
            ui,
            ok_enabled,
            &mut confirmed,
            &mut cancelled,
            locale,
            |ui, width| {
                ui.add_enabled(
                    ok_enabled,
                    egui::Button::new(strings::ok(locale)).min_size(egui::vec2(width, 0.0)),
                )
                .clicked()
            },
        );
    });

    if escape_pressed {
        cancelled = true;
    }

    if confirmed {
        if let Ok(count) = pending.count_text.trim().parse::<usize>() {
            return Some(SheetDialogAction::AppendRows {
                row: pending.row,
                count,
                insert_above: pending.insert_above,
            });
        }
        return None;
    }

    if cancelled {
        return None;
    }

    state.append_rows = Some(pending);
    None
}

fn show_resize_sheet_dialog(
    state: &mut SheetDialogsState,
    ctx: &egui::Context,
    _sheet: &mut DisplaySheetState,
    locale: AppLocale,
) -> Option<SheetDialogAction> {
    let Some(mut pending) = state.resize_sheet.take() else {
        return None;
    };

    let previous_fps = match pending.fps_mode {
        SheetResizeFpsMode::Fps24 => 24,
        SheetResizeFpsMode::Fps30 => 30,
        SheetResizeFpsMode::Custom => pending.custom_fps.max(1),
    };
    let valid_name = !pending.sheet_name.trim().is_empty();
    let mut confirmed = false;
    let mut cancelled = false;
    let mut duration_has_focus = false;
    let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    let mut parsed_duration = parse_sheet_duration_input(&pending.duration_text, previous_fps);

    egui::Modal::new(egui::Id::new("sheet_resize_modal")).show(ctx, |ui| {
        ui.set_min_width(260.0);
        ui.heading(strings::sheet_settings(locale));
        ui.add_space(8.0);
        ui.label(strings::sheet_name(locale));
        ui.add(egui::TextEdit::singleline(&mut pending.sheet_name).return_key(None));
        if !valid_name {
            ui.small(strings::sheet_name_empty(locale));
        }
        ui.add_space(6.0);
        ui.label(strings::duration(locale));
        let enter_pressed = dialog_submit_enter_pressed(ui);
        let duration_response = ui.add(
            egui::TextEdit::singleline(&mut pending.duration_text).id_salt("sheet_resize_duration"),
        );
        if pending.select_duration_on_open {
            duration_response.request_focus();
            select_all_text_edit(
                ui,
                duration_response.id,
                pending.duration_text.chars().count(),
            );
            pending.select_duration_on_open = false;
        }
        duration_has_focus = duration_response.has_focus();
        if parsed_duration.is_err() {
            ui.small(strings::duration_input_help(locale));
        }
        ui.add_space(6.0);
        ui.label("fps");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut pending.fps_mode, SheetResizeFpsMode::Fps24, "24");
            ui.selectable_value(&mut pending.fps_mode, SheetResizeFpsMode::Fps30, "30");
            ui.selectable_value(
                &mut pending.fps_mode,
                SheetResizeFpsMode::Custom,
                strings::custom(locale),
            );
        });
        ui.add_enabled_ui(
            matches!(pending.fps_mode, SheetResizeFpsMode::Custom),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut pending.custom_fps)
                        .speed(1.0)
                        .range(1..=120),
                );
            },
        );
        let fps = match pending.fps_mode {
            SheetResizeFpsMode::Fps24 => 24,
            SheetResizeFpsMode::Fps30 => 30,
            SheetResizeFpsMode::Custom => pending.custom_fps.max(1),
        };
        if fps != previous_fps {
            if let Ok(total_frames) =
                parse_sheet_duration_input(&pending.duration_text, previous_fps)
            {
                pending.duration_text = format_sheet_duration_text(total_frames, fps);
            }
        }
        parsed_duration = parse_sheet_duration_input(&pending.duration_text, fps);
        ui.add_space(6.0);
        ui.label(strings::columns(locale));
        ui.add(
            egui::DragValue::new(&mut pending.columns)
                .speed(1.0)
                .range(1..=76),
        );
        ui.add_space(10.0);
        show_dialog_action_buttons(
            ui,
            parsed_duration.is_ok() && valid_name,
            &mut confirmed,
            &mut cancelled,
            locale,
            |ui, width| {
                ui.add_enabled(
                    parsed_duration.is_ok() && valid_name,
                    egui::Button::new(strings::ok(locale)).min_size(egui::vec2(width, 0.0)),
                )
                .clicked()
            },
        );
        if enter_pressed && parsed_duration.is_ok() && valid_name {
            confirmed = true;
        }
    });

    sync_duration_text_ime(ctx, duration_has_focus);

    if escape_pressed {
        cancelled = true;
    }

    if confirmed {
        let fps = match pending.fps_mode {
            SheetResizeFpsMode::Fps24 => 24,
            SheetResizeFpsMode::Fps30 => 30,
            SheetResizeFpsMode::Custom => pending.custom_fps.max(1),
        };
        return Some(SheetDialogAction::ResizeSheet {
            sheet_name: pending.sheet_name,
            total_frames: parsed_duration.unwrap_or(1),
            columns: (pending.columns as usize).max(1),
            fps,
        });
    }

    if cancelled {
        return None;
    }

    state.resize_sheet = Some(pending);
    None
}

fn show_new_sheet_dialog(
    state: &mut SheetDialogsState,
    ctx: &egui::Context,
    locale: AppLocale,
) -> Option<SheetDialogAction> {
    let Some(mut pending) = state.new_sheet.take() else {
        return None;
    };

    let fps = match pending.fps_mode {
        SheetResizeFpsMode::Fps24 => 24,
        SheetResizeFpsMode::Fps30 => 30,
        SheetResizeFpsMode::Custom => pending.custom_fps.max(1),
    };
    let parsed_duration = parse_sheet_duration_input(&pending.duration_text, fps);
    let valid_name = !pending.sheet_name.trim().is_empty();
    let mut confirmed = false;
    let mut cancelled = false;
    let mut duration_has_focus = false;
    let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    egui::Modal::new(egui::Id::new("new_sheet_modal")).show(ctx, |ui| {
        ui.set_min_width(260.0);
        ui.heading(strings::new_sheet(locale));
        ui.add_space(8.0);
        ui.label(strings::sheet_name(locale));
        let name_response =
            ui.add(egui::TextEdit::singleline(&mut pending.sheet_name).return_key(None));
        if name_response.gained_focus() {
            select_all_text_edit(ui, name_response.id, pending.sheet_name.chars().count());
        }
        if !valid_name {
            ui.small(strings::sheet_name_empty(locale));
        }
        ui.add_space(6.0);
        ui.label(strings::duration(locale));
        let enter_pressed = dialog_submit_enter_pressed(ui);
        let duration_response = ui.add(
            egui::TextEdit::singleline(&mut pending.duration_text).id_salt("new_sheet_duration"),
        );
        if pending.select_duration_on_open {
            duration_response.request_focus();
            select_all_text_edit(
                ui,
                duration_response.id,
                pending.duration_text.chars().count(),
            );
            pending.select_duration_on_open = false;
        } else if duration_response.gained_focus() {
            select_all_text_edit(
                ui,
                duration_response.id,
                pending.duration_text.chars().count(),
            );
        }
        duration_has_focus = duration_response.has_focus();
        if parsed_duration.is_err() {
            ui.small(strings::duration_input_help(locale));
        }
        ui.add_space(6.0);
        ui.label("fps");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut pending.fps_mode, SheetResizeFpsMode::Fps24, "24");
            ui.selectable_value(&mut pending.fps_mode, SheetResizeFpsMode::Fps30, "30");
            ui.selectable_value(
                &mut pending.fps_mode,
                SheetResizeFpsMode::Custom,
                strings::custom(locale),
            );
        });
        ui.add_enabled_ui(
            matches!(pending.fps_mode, SheetResizeFpsMode::Custom),
            |ui| {
                ui.add(
                    egui::DragValue::new(&mut pending.custom_fps)
                        .speed(1.0)
                        .range(1..=120),
                );
            },
        );
        ui.add_space(6.0);
        ui.label(strings::columns(locale));
        ui.add(
            egui::DragValue::new(&mut pending.columns)
                .speed(1.0)
                .range(1..=76),
        );
        ui.add_space(10.0);
        show_dialog_action_buttons(
            ui,
            parsed_duration.is_ok() && valid_name,
            &mut confirmed,
            &mut cancelled,
            locale,
            |ui, width| {
                ui.add_enabled(
                    parsed_duration.is_ok() && valid_name,
                    egui::Button::new(strings::ok(locale)).min_size(egui::vec2(width, 0.0)),
                )
                .clicked()
            },
        );
        if enter_pressed && parsed_duration.is_ok() && valid_name {
            confirmed = true;
        }
    });

    sync_duration_text_ime(ctx, duration_has_focus);

    if escape_pressed {
        cancelled = true;
    }

    if confirmed {
        return Some(SheetDialogAction::NewSheet {
            sheet_name: pending.sheet_name,
            total_frames: parsed_duration.unwrap_or(1) as u32,
            columns: pending.columns.max(1),
            fps,
        });
    }

    if cancelled {
        return None;
    }

    state.new_sheet = Some(pending);
    None
}

fn show_rename_column_dialog(
    state: &mut SheetDialogsState,
    ctx: &egui::Context,
    sheet: &mut DisplaySheetState,
    locale: AppLocale,
) -> Option<SheetDialogAction> {
    let Some(mut pending) = state.rename_column.take() else {
        return None;
    };

    if pending.column >= sheet.column_count() {
        return None;
    }

    let mut confirmed = false;
    let mut cancelled = false;
    let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    egui::Modal::new(egui::Id::new("rename_column_modal")).show(ctx, |ui| {
        ui.set_min_width(320.0);
        ui.heading(strings::rename_column(locale));
        ui.add_space(8.0);
        ui.label(strings::new_column_name(locale));
        let enter_pressed = dialog_submit_enter_pressed(ui);
        let response = ui.add(
            egui::TextEdit::singleline(&mut pending.name)
                .id_salt("rename_column_name")
                .return_key(None),
        );
        if pending.select_on_open {
            response.request_focus();
            select_all_text_edit(ui, response.id, pending.name.chars().count());
            pending.select_on_open = false;
        } else if response.gained_focus() {
            select_all_text_edit(ui, response.id, pending.name.chars().count());
        }

        let trimmed = pending.name.trim().to_owned();
        let can_submit = !trimmed.is_empty();

        if enter_pressed && can_submit {
            confirmed = true;
        }

        ui.add_space(8.0);
        show_dialog_action_buttons(
            ui,
            can_submit,
            &mut confirmed,
            &mut cancelled,
            locale,
            |ui, width| {
                if ui
                    .add_enabled(
                        can_submit,
                        egui::Button::new(strings::rename(locale)).min_size(egui::vec2(width, 0.0)),
                    )
                    .clicked()
                {
                    return true;
                }
                false
            },
        );
    });

    if escape_pressed {
        cancelled = true;
    }

    if confirmed {
        return Some(SheetDialogAction::RenameColumn {
            column: pending.column,
            name: pending.name.trim().to_owned(),
        });
    }

    if cancelled {
        return None;
    }

    state.rename_column = Some(pending);
    None
}

fn select_all_text_edit(ui: &egui::Ui, id: egui::Id, text_len: usize) {
    let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(0),
            egui::text::CCursor::new(text_len),
        )));
    egui::TextEdit::store_state(ui.ctx(), id, state);
}

fn sync_duration_text_ime(ctx: &egui::Context, duration_has_focus: bool) {
    ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(!duration_has_focus));
}

fn dialog_submit_enter_pressed(ui: &egui::Ui) -> bool {
    ui.input(|i| {
        i.key_pressed(egui::Key::Enter)
            && !i
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(_)))
    })
}

fn show_dialog_action_buttons(
    ui: &mut egui::Ui,
    apply_enabled: bool,
    confirmed: &mut bool,
    cancelled: &mut bool,
    locale: AppLocale,
    mut show_apply_button: impl FnMut(&mut egui::Ui, f32) -> bool,
) {
    let available_width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(available_width, 0.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let show_cancel_first = cfg!(target_os = "macos");
            let button_width = 92.0;
            let cancel_clicked = |ui: &mut egui::Ui| {
                ui.add(
                    egui::Button::new(strings::cancel(locale))
                        .min_size(egui::vec2(button_width, 0.0)),
                )
                .clicked()
            };

            if show_cancel_first {
                if cancel_clicked(ui) {
                    *cancelled = true;
                }
                if show_apply_button(ui, button_width) && apply_enabled {
                    *confirmed = true;
                }
            } else {
                if show_apply_button(ui, button_width) && apply_enabled {
                    *confirmed = true;
                }
                if cancel_clicked(ui) {
                    *cancelled = true;
                }
            }
        },
    );
}

fn parse_sheet_duration_input(input: &str, fps: u32) -> Result<usize, ()> {
    let trimmed = input.trim();
    let (seconds_str, frames_str) = trimmed.split_once('+').ok_or(())?;
    let seconds = seconds_str.parse::<usize>().map_err(|_| ())?;
    let frames = frames_str.parse::<usize>().map_err(|_| ())?;

    if seconds > MAX_SHEET_SECONDS || frames >= fps as usize {
        return Err(());
    }

    let total_frames = seconds * fps as usize + frames;
    (1..=MAX_SHEET_SECONDS * fps as usize)
        .contains(&total_frames)
        .then_some(total_frames)
        .ok_or(())
}

fn format_sheet_duration_text(frame_count: usize, fps: u32) -> String {
    let seconds = frame_count / fps as usize;
    let frames = frame_count % fps as usize;
    format!("{seconds}+{frames}")
}
