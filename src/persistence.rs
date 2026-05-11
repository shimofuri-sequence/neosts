use crate::theme_files;
use eframe::Storage;
use neosts::{
    AeKaraCellMode, AeKeyframeDataLocale, AeSheetNameSource, AppLocale, AppSettings,
    ClipboardExportFormat, DisplayMode, EditorSettings, SheetSettings, TableColorTheme,
    TableSettings, TableViewState,
};

const CELL_SCALE_KEY: &str = "table_view.cell_scale";
const DEFAULT_COLUMN_WIDTH_KEY: &str = "table_view.default_column_width";
const DEFAULT_ROW_HEIGHT_KEY: &str = "table_view.default_row_height";
const DEFAULT_HEADER_WIDTH_KEY: &str = "table_view.default_header_width";
const DEFAULT_HEADER_HEIGHT_KEY: &str = "table_view.default_header_height";
const ALTERNATE_COLUMN_MODE_KEY: &str = "table_view.alternate_column_mode";
const ALTERNATE_DARKEN_AMOUNT_KEY: &str = "table_view.alternate_darken_amount";
const ALTERNATE_SECOND_DARKEN_AMOUNT_KEY: &str = "table_view.alternate_second_darken_amount";
const ALTERNATE_SATURATION_AMOUNT_KEY: &str = "table_view.alternate_saturation_amount";
const ALTERNATE_COLUMN_COLOR_KEY: &str = "table_view.alternate_column_color";
const CELL_BACKGROUND_COLOR_KEY: &str = "table_view.cell_background_color";
const SELECTION_COLOR_KEY: &str = "table_view.selection_color";
const HOVER_COLOR_KEY: &str = "table_view.hover_color";
const COLUMN_HEADER_BACKGROUND_COLOR_KEY: &str = "table_view.column_header_background_color";
const ROW_HEADER_BACKGROUND_COLOR_KEY: &str = "table_view.row_header_background_color";
const MINIMAP_WIDTH_KEY: &str = "table_view.minimap_width";
const MINIMAP_HEIGHT_KEY: &str = "table_view.minimap_height";
const FRAME_HEADER_MODE_KEY: &str = "table_view.frame_header_mode";
const FRAME_HEADER_DENSITY_KEY: &str = "table_view.frame_header_density";
const SEGMENT_HEADER_MODE_KEY: &str = "table_view.segment_header_mode";
const SEGMENT_HEADER_DENSITY_KEY: &str = "table_view.segment_header_density";
const APP_THEME_KEY: &str = "table_view.app_theme";
const CUSTOM_THEME_BASE_KEY: &str = "table_view.custom_theme_base";
const THEME_PREFERENCE_KEY: &str = "table_view.theme_preference";
const UP_SCROLL_TRIGGER_RATIO_KEY: &str = "table_view.up_scroll_trigger_ratio";
const DOWN_SCROLL_TRIGGER_RATIO_KEY: &str = "table_view.down_scroll_trigger_ratio";
const CONTINUATION_LINE_MIN_RUN_LENGTH_KEY: &str = "table_view.continuation_line_min_run_length";
const CONTINUATION_LINE_STYLE_KEY: &str = "table_view.continuation_line_style";
const ZERO_CELL_BACKGROUND_COLOR_KEY: &str = "table_view.zero_cell_background_color";
const USE_ZERO_CELL_BACKGROUND_COLOR_KEY: &str = "table_view.use_zero_cell_background_color";
const SHOW_ZERO_VALUE_MARKERS_KEY: &str = "table_view.show_zero_value_markers";
const SHOW_HEADER_GHOSTS_KEY: &str = "table_view.show_header_ghosts";
const COLOR_THEME_KEY: &str = "table_view.color_theme";
const SPECIAL_INSERTED_ROW_BACKGROUND_COLOR_KEY: &str =
    "table_view.special_inserted_row_background_color";
const PUNCHED_ROW_BACKGROUND_COLOR_KEY: &str = "table_view.punched_row_background_color";
const KEYBIND_MOVE_UP_KEY: &str = "table_view.keybind.move_up";
const KEYBIND_MOVE_DOWN_KEY: &str = "table_view.keybind.move_down";
const KEYBIND_MOVE_LEFT_KEY: &str = "table_view.keybind.move_left";
const KEYBIND_MOVE_RIGHT_KEY: &str = "table_view.keybind.move_right";
const KEYBIND_JUMP_UP_KEY: &str = "table_view.keybind.jump_up";
const KEYBIND_JUMP_DOWN_KEY: &str = "table_view.keybind.jump_down";
const KEYBIND_DECREASE_SELECTION_KEY: &str = "table_view.keybind.decrease_selection";
const KEYBIND_INCREASE_SELECTION_KEY: &str = "table_view.keybind.increase_selection";
const KEYBIND_KARA_ZERO_INPUT_KEY: &str = "table_view.keybind.kara_zero_input";
const KEYBIND_TOGGLE_MINIMAP_KEY: &str = "table_view.keybind.toggle_minimap";
const KEYBIND_OPEN_PREFERENCES_KEY: &str = "table_view.keybind.open_preferences";
const EDITOR_DISPLAY_MODE_KEY: &str = "editor.display_mode";
const CLIPBOARD_EXPORT_FORMAT_KEY: &str = "editor.clipboard_export_format";
const AE_KEYFRAME_DATA_LOCALE_KEY: &str = "editor.ae_keyframe_data_locale";
const AE_KEYFRAME_VERSION_KEY: &str = "editor.ae_keyframe_version";
const AE_KARA_CELL_MODE_KEY: &str = "editor.ae_kara_cell_mode";
const AE_SHEET_NAME_SOURCE_KEY: &str = "editor.ae_sheet_name_source";
const DEFAULT_SHEET_FPS_KEY: &str = "sheet.default_fps";
const DEFAULT_SECONDS_PER_PAGE_KEY: &str = "sheet.default_seconds_per_page";
const DEFAULT_FRAMES_PER_PAGE_KEY: &str = "sheet.default_frames_per_page";
const INITIAL_FRAME_COUNT_KEY: &str = "sheet.initial_frame_count";
const INITIAL_COLUMN_COUNT_KEY: &str = "sheet.initial_column_count";
const WINDOW_ALWAYS_ON_TOP_KEY: &str = "app.window_always_on_top";
const OPEN_NEW_SHEET_DIALOG_ON_STARTUP_KEY: &str = "app.open_new_sheet_dialog_on_startup";
const APP_LOCALE_KEY: &str = "app.locale";
const IMPORTED_THEME_PATH_KEY: &str = "app.imported_theme_path";
const IMPORTED_THEME_NAME_KEY: &str = "app.imported_theme_name";
const IMPORTED_THEME_ACTIVE_KEY: &str = "app.imported_theme_active";
const RECENT_FILES_KEY: &str = "app.recent_files";

const MIN_SHEET_FPS: u32 = 1;
const MAX_SHEET_FPS: u32 = 120;
const MAX_SHEET_SECONDS: u32 = 100;
const MAX_SECONDS_PER_PAGE: u32 = 6;
const MIN_INITIAL_FRAME_COUNT: u32 = 1;
const MIN_INITIAL_COLUMN_COUNT: u32 = 1;
const MAX_INITIAL_COLUMN_COUNT: u32 = 76;
pub fn load_from_storage(
    storage: &dyn Storage,
    table_view: &mut TableViewState,
    table_settings: &mut TableSettings,
    app_settings: &mut AppSettings,
    sheet_settings: &mut SheetSettings,
    editor_settings: &mut EditorSettings,
) {
    if let Some(cell_scale) = eframe::get_value::<f32>(storage, CELL_SCALE_KEY) {
        table_settings.set_cell_scale(cell_scale);
    }
    if let Some(column_width) = eframe::get_value::<f32>(storage, DEFAULT_COLUMN_WIDTH_KEY) {
        table_settings.set_default_column_width(column_width);
    }
    if let Some(row_height) = eframe::get_value::<f32>(storage, DEFAULT_ROW_HEIGHT_KEY) {
        table_settings.set_default_row_height(row_height);
    }
    if let Some(header_width) = eframe::get_value::<f32>(storage, DEFAULT_HEADER_WIDTH_KEY) {
        table_settings.set_default_header_width(header_width);
    }
    if let Some(header_height) = eframe::get_value::<f32>(storage, DEFAULT_HEADER_HEIGHT_KEY) {
        table_settings.set_default_header_height(header_height);
    }
    if let Some(mode_id) = eframe::get_value::<u8>(storage, ALTERNATE_COLUMN_MODE_KEY) {
        table_settings.set_alternate_column_mode_id(mode_id);
    }
    if let Some(amount) = eframe::get_value::<f32>(storage, ALTERNATE_DARKEN_AMOUNT_KEY) {
        table_settings.set_alternate_darken_amount(amount);
    }
    if let Some(amount) = eframe::get_value::<f32>(storage, ALTERNATE_SECOND_DARKEN_AMOUNT_KEY) {
        table_settings.set_alternate_second_darken_amount(amount);
    }
    if let Some(amount) = eframe::get_value::<f32>(storage, ALTERNATE_SATURATION_AMOUNT_KEY) {
        table_settings.set_alternate_saturation_amount(amount);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, ALTERNATE_COLUMN_COLOR_KEY) {
        table_settings.set_alternate_column_rgba(rgba);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, CELL_BACKGROUND_COLOR_KEY) {
        table_settings.set_cell_background_rgba(rgba);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, SELECTION_COLOR_KEY) {
        table_settings.set_selection_rgba(rgba);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, HOVER_COLOR_KEY) {
        table_settings.set_hover_rgba(rgba);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, COLUMN_HEADER_BACKGROUND_COLOR_KEY) {
        table_settings.set_column_header_background_rgba(rgba);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, ROW_HEADER_BACKGROUND_COLOR_KEY) {
        table_settings.set_row_header_background_rgba(rgba);
    }
    if let Some(width) = eframe::get_value::<f32>(storage, MINIMAP_WIDTH_KEY) {
        table_settings.set_minimap_width(width);
    }
    if let Some(height) = eframe::get_value::<f32>(storage, MINIMAP_HEIGHT_KEY) {
        table_settings.set_minimap_height(height);
    }
    if let Some(mode_id) = eframe::get_value::<u8>(storage, FRAME_HEADER_MODE_KEY) {
        table_settings.set_frame_header_mode_id(mode_id);
    }
    if let Some(density_id) = eframe::get_value::<u8>(storage, FRAME_HEADER_DENSITY_KEY) {
        table_settings.set_frame_header_density_id(density_id);
    }
    if let Some(mode_id) = eframe::get_value::<u8>(storage, SEGMENT_HEADER_MODE_KEY) {
        table_settings.set_segment_header_mode_id(mode_id);
    }
    if let Some(density_id) = eframe::get_value::<u8>(storage, SEGMENT_HEADER_DENSITY_KEY) {
        table_settings.set_segment_header_density_id(density_id);
    }
    if let Some(theme_id) = eframe::get_value::<u8>(storage, THEME_PREFERENCE_KEY) {
        table_settings.set_theme_preference_id(theme_id);
    }
    if let Some(theme_id) = eframe::get_value::<u8>(storage, CUSTOM_THEME_BASE_KEY) {
        table_settings.set_custom_theme_base_id(theme_id);
    }
    if let Some(ratio) = eframe::get_value::<f32>(storage, UP_SCROLL_TRIGGER_RATIO_KEY) {
        table_settings.set_up_scroll_trigger_ratio(ratio);
    }
    if let Some(ratio) = eframe::get_value::<f32>(storage, DOWN_SCROLL_TRIGGER_RATIO_KEY) {
        table_settings.set_down_scroll_trigger_ratio(ratio);
    }
    if let Some(value) = eframe::get_value::<u32>(storage, CONTINUATION_LINE_MIN_RUN_LENGTH_KEY) {
        table_settings.set_continuation_line_min_run_length(value);
    }
    if let Some(value) = eframe::get_value::<u8>(storage, CONTINUATION_LINE_STYLE_KEY) {
        table_settings.set_continuation_line_style_id(value);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, ZERO_CELL_BACKGROUND_COLOR_KEY) {
        table_settings.set_zero_cell_background_rgba(rgba);
    }
    if let Some(enabled) = eframe::get_value::<bool>(storage, USE_ZERO_CELL_BACKGROUND_COLOR_KEY) {
        table_settings.set_use_zero_cell_background_color(enabled);
    }
    if let Some(enabled) = eframe::get_value::<bool>(storage, SHOW_ZERO_VALUE_MARKERS_KEY) {
        table_settings.set_show_zero_value_markers(enabled);
    }
    if let Some(enabled) = eframe::get_value::<bool>(storage, SHOW_HEADER_GHOSTS_KEY) {
        table_settings.set_show_header_ghosts(enabled);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, SPECIAL_INSERTED_ROW_BACKGROUND_COLOR_KEY)
    {
        table_settings.set_special_inserted_row_background_rgba(rgba);
    }
    if let Some(rgba) = eframe::get_value::<u32>(storage, PUNCHED_ROW_BACKGROUND_COLOR_KEY) {
        table_settings.set_punched_row_background_rgba(rgba);
    }
    if let Some(theme_id) = eframe::get_value::<u8>(storage, APP_THEME_KEY) {
        table_settings.set_theme_id(theme_id);
        if theme_id == 0 {
            table_settings.sync_default_color_theme(false);
        }
    } else if let Some(theme_id) = eframe::get_value::<u8>(storage, COLOR_THEME_KEY) {
        table_settings.set_color_theme_id(theme_id);
        match table_settings.color_theme {
            TableColorTheme::Default => table_settings.sync_default_color_theme(false),
            TableColorTheme::Custom => {}
            theme => table_settings.apply_color_theme(theme),
        }
    } else if has_legacy_color_settings(storage) {
        table_settings.mark_color_theme_custom();
    }

    for (index, key) in KEYBIND_KEYS.iter().enumerate() {
        if let Some(value) = eframe::get_value::<String>(storage, key) {
            table_view.set_keybind_value(&mut editor_settings.keybindings, index as u8, &value);
        }
    }

    if let Some(mode) = eframe::get_value::<u8>(storage, EDITOR_DISPLAY_MODE_KEY) {
        editor_settings.display_mode = if mode == 1 {
            DisplayMode::Keyframe
        } else {
            DisplayMode::FullFrame
        };
    }
    if let Some(format_id) = eframe::get_value::<u8>(storage, CLIPBOARD_EXPORT_FORMAT_KEY) {
        editor_settings.clipboard_export_format = ClipboardExportFormat::from_storage_id(format_id);
    }
    if let Some(locale_id) = eframe::get_value::<u8>(storage, AE_KEYFRAME_DATA_LOCALE_KEY) {
        editor_settings.ae_keyframe_data_locale = AeKeyframeDataLocale::from_storage_id(locale_id);
    }
    if let Some(version) = storage.get_string(AE_KEYFRAME_VERSION_KEY) {
        editor_settings.ae_keyframe_version = version;
    }
    if let Some(mode_id) = eframe::get_value::<u8>(storage, AE_KARA_CELL_MODE_KEY) {
        editor_settings.ae_kara_cell_mode = AeKaraCellMode::from_storage_id(mode_id);
    }
    if let Some(source_id) = eframe::get_value::<u8>(storage, AE_SHEET_NAME_SOURCE_KEY) {
        editor_settings.ae_sheet_name_source = AeSheetNameSource::from_storage_id(source_id);
    }
    if let Some(fps) = eframe::get_value::<u32>(storage, DEFAULT_SHEET_FPS_KEY) {
        sheet_settings.default_fps = clamp_sheet_fps(fps);
    }
    if let Some(seconds_per_page) = eframe::get_value::<u32>(storage, DEFAULT_SECONDS_PER_PAGE_KEY)
    {
        sheet_settings.default_seconds_per_page = clamp_seconds_per_page(seconds_per_page);
    } else if let Some(frames_per_page) =
        eframe::get_value::<u32>(storage, DEFAULT_FRAMES_PER_PAGE_KEY)
    {
        sheet_settings.default_seconds_per_page =
            frames_per_page_to_seconds(frames_per_page, sheet_settings.default_fps);
    }
    if let Some(frame_count) = eframe::get_value::<u32>(storage, INITIAL_FRAME_COUNT_KEY) {
        sheet_settings.initial_frame_count =
            clamp_initial_frame_count(frame_count, sheet_settings.default_fps);
    } else {
        sheet_settings.initial_frame_count = clamp_initial_frame_count(
            sheet_settings.initial_frame_count,
            sheet_settings.default_fps,
        );
    }
    if let Some(column_count) = eframe::get_value::<u32>(storage, INITIAL_COLUMN_COUNT_KEY) {
        sheet_settings.initial_column_count = clamp_initial_column_count(column_count);
    }
    if let Some(always_on_top) = eframe::get_value::<bool>(storage, WINDOW_ALWAYS_ON_TOP_KEY) {
        app_settings.always_on_top = always_on_top;
    }
    if let Some(open_on_startup) =
        eframe::get_value::<bool>(storage, OPEN_NEW_SHEET_DIALOG_ON_STARTUP_KEY)
    {
        app_settings.open_new_sheet_dialog_on_startup = open_on_startup;
    }
    if let Some(locale_id) = eframe::get_value::<u8>(storage, APP_LOCALE_KEY) {
        app_settings.set_locale(AppLocale::from_storage_id(locale_id));
    }
    if let Some(path) = storage.get_string(IMPORTED_THEME_PATH_KEY) {
        if !path.is_empty() {
            let name = storage
                .get_string(IMPORTED_THEME_NAME_KEY)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| theme_files::theme_name_from_path(std::path::Path::new(&path)));
            app_settings.set_imported_theme(std::path::PathBuf::from(&path), name);
            let active =
                eframe::get_value::<bool>(storage, IMPORTED_THEME_ACTIVE_KEY).unwrap_or(true);
            app_settings.set_imported_theme_active(active);
            if active {
                if let Some(theme_path) = app_settings.imported_theme_path() {
                    let _ = theme_files::apply_theme_from_path(
                        table_settings,
                        theme_path,
                        neosts::AppLocale::Japanese,
                    );
                }
            }
        }
    }
    if let Some(recent_files) = storage.get_string(RECENT_FILES_KEY) {
        app_settings.set_recent_files(
            recent_files
                .lines()
                .filter(|line| !line.is_empty())
                .map(std::path::PathBuf::from)
                .collect(),
        );
    }
}

pub fn save_to_storage(
    storage: &mut dyn Storage,
    table_view: &TableViewState,
    table_settings: &TableSettings,
    app_settings: &AppSettings,
    sheet_settings: &SheetSettings,
    editor_settings: &EditorSettings,
) {
    eframe::set_value(storage, CELL_SCALE_KEY, &table_settings.cell_scale());
    eframe::set_value(
        storage,
        DEFAULT_COLUMN_WIDTH_KEY,
        &table_settings.default_column_width(),
    );
    eframe::set_value(
        storage,
        DEFAULT_ROW_HEIGHT_KEY,
        &table_settings.default_row_height(),
    );
    eframe::set_value(
        storage,
        DEFAULT_HEADER_WIDTH_KEY,
        &table_settings.default_header_width(),
    );
    eframe::set_value(
        storage,
        DEFAULT_HEADER_HEIGHT_KEY,
        &table_settings.default_header_height(),
    );
    eframe::set_value(
        storage,
        ALTERNATE_COLUMN_MODE_KEY,
        &table_settings.alternate_column_mode_id(),
    );
    eframe::set_value(
        storage,
        ALTERNATE_DARKEN_AMOUNT_KEY,
        &table_settings.alternate_darken_amount(),
    );
    eframe::set_value(
        storage,
        ALTERNATE_SECOND_DARKEN_AMOUNT_KEY,
        &table_settings.alternate_second_darken_amount(),
    );
    eframe::set_value(
        storage,
        ALTERNATE_SATURATION_AMOUNT_KEY,
        &table_settings.alternate_saturation_amount(),
    );
    eframe::set_value(
        storage,
        ALTERNATE_COLUMN_COLOR_KEY,
        &table_settings.alternate_column_rgba(),
    );
    eframe::set_value(
        storage,
        CELL_BACKGROUND_COLOR_KEY,
        &table_settings.cell_background_rgba(),
    );
    eframe::set_value(
        storage,
        SELECTION_COLOR_KEY,
        &table_settings.selection_rgba(),
    );
    eframe::set_value(storage, HOVER_COLOR_KEY, &table_settings.hover_rgba());
    eframe::set_value(
        storage,
        COLUMN_HEADER_BACKGROUND_COLOR_KEY,
        &table_settings.column_header_background_rgba(),
    );
    eframe::set_value(
        storage,
        ROW_HEADER_BACKGROUND_COLOR_KEY,
        &table_settings.row_header_background_rgba(),
    );
    eframe::set_value(storage, MINIMAP_WIDTH_KEY, &table_settings.minimap_width());
    eframe::set_value(
        storage,
        MINIMAP_HEIGHT_KEY,
        &table_settings.minimap_height(),
    );
    eframe::set_value(
        storage,
        FRAME_HEADER_MODE_KEY,
        &table_settings.frame_header_mode_id(),
    );
    eframe::set_value(
        storage,
        FRAME_HEADER_DENSITY_KEY,
        &table_settings.frame_header_density_id(),
    );
    eframe::set_value(
        storage,
        SEGMENT_HEADER_MODE_KEY,
        &table_settings.segment_header_mode_id(),
    );
    eframe::set_value(
        storage,
        SEGMENT_HEADER_DENSITY_KEY,
        &table_settings.segment_header_density_id(),
    );
    eframe::set_value(storage, APP_THEME_KEY, &table_settings.theme_id());
    eframe::set_value(
        storage,
        CUSTOM_THEME_BASE_KEY,
        &table_settings.custom_theme_base_id(),
    );
    eframe::set_value(
        storage,
        THEME_PREFERENCE_KEY,
        &table_settings.theme_preference_id(),
    );
    eframe::set_value(
        storage,
        UP_SCROLL_TRIGGER_RATIO_KEY,
        &table_settings.up_scroll_trigger_ratio(),
    );
    eframe::set_value(
        storage,
        DOWN_SCROLL_TRIGGER_RATIO_KEY,
        &table_settings.down_scroll_trigger_ratio(),
    );
    eframe::set_value(
        storage,
        CONTINUATION_LINE_MIN_RUN_LENGTH_KEY,
        &table_settings.continuation_line_min_run_length(),
    );
    eframe::set_value(
        storage,
        CONTINUATION_LINE_STYLE_KEY,
        &table_settings.continuation_line_style_id(),
    );
    eframe::set_value(
        storage,
        ZERO_CELL_BACKGROUND_COLOR_KEY,
        &table_settings.zero_cell_background_rgba(),
    );
    eframe::set_value(
        storage,
        USE_ZERO_CELL_BACKGROUND_COLOR_KEY,
        &table_settings.use_zero_cell_background_color(),
    );
    eframe::set_value(
        storage,
        SHOW_ZERO_VALUE_MARKERS_KEY,
        &table_settings.show_zero_value_markers(),
    );
    eframe::set_value(
        storage,
        SHOW_HEADER_GHOSTS_KEY,
        &table_settings.show_header_ghosts(),
    );
    eframe::set_value(storage, COLOR_THEME_KEY, &table_settings.color_theme_id());
    eframe::set_value(
        storage,
        SPECIAL_INSERTED_ROW_BACKGROUND_COLOR_KEY,
        &table_settings.special_inserted_row_background_rgba(),
    );
    eframe::set_value(
        storage,
        PUNCHED_ROW_BACKGROUND_COLOR_KEY,
        &table_settings.punched_row_background_rgba(),
    );

    for (index, key) in KEYBIND_KEYS.iter().enumerate() {
        eframe::set_value(
            storage,
            key,
            &table_view.keybind_value(&editor_settings.keybindings, index as u8),
        );
    }

    let display_mode_id = match editor_settings.display_mode {
        DisplayMode::FullFrame => 0u8,
        DisplayMode::Keyframe => 1u8,
    };
    eframe::set_value(storage, EDITOR_DISPLAY_MODE_KEY, &display_mode_id);
    eframe::set_value(
        storage,
        CLIPBOARD_EXPORT_FORMAT_KEY,
        &editor_settings.clipboard_export_format.storage_id(),
    );
    eframe::set_value(
        storage,
        AE_KEYFRAME_DATA_LOCALE_KEY,
        &editor_settings.ae_keyframe_data_locale.storage_id(),
    );
    storage.set_string(
        AE_KEYFRAME_VERSION_KEY,
        editor_settings.ae_keyframe_version.clone(),
    );
    eframe::set_value(
        storage,
        AE_KARA_CELL_MODE_KEY,
        &editor_settings.ae_kara_cell_mode.storage_id(),
    );
    eframe::set_value(
        storage,
        AE_SHEET_NAME_SOURCE_KEY,
        &editor_settings.ae_sheet_name_source.storage_id(),
    );
    eframe::set_value(storage, DEFAULT_SHEET_FPS_KEY, &sheet_settings.default_fps);
    eframe::set_value(
        storage,
        DEFAULT_SECONDS_PER_PAGE_KEY,
        &sheet_settings.default_seconds_per_page,
    );
    eframe::set_value(
        storage,
        INITIAL_FRAME_COUNT_KEY,
        &sheet_settings.initial_frame_count,
    );
    eframe::set_value(
        storage,
        INITIAL_COLUMN_COUNT_KEY,
        &sheet_settings.initial_column_count,
    );
    eframe::set_value(
        storage,
        WINDOW_ALWAYS_ON_TOP_KEY,
        &app_settings.always_on_top,
    );
    storage.set_string(
        IMPORTED_THEME_PATH_KEY,
        app_settings
            .imported_theme_path()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
    );
    storage.set_string(
        IMPORTED_THEME_NAME_KEY,
        app_settings
            .imported_theme_name()
            .unwrap_or_default()
            .to_owned(),
    );
    eframe::set_value(
        storage,
        IMPORTED_THEME_ACTIVE_KEY,
        &app_settings.imported_theme_active(),
    );
    eframe::set_value(
        storage,
        OPEN_NEW_SHEET_DIALOG_ON_STARTUP_KEY,
        &app_settings.open_new_sheet_dialog_on_startup,
    );
    eframe::set_value(storage, APP_LOCALE_KEY, &app_settings.locale().storage_id());
    storage.set_string(
        RECENT_FILES_KEY,
        app_settings
            .recent_files()
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

const KEYBIND_KEYS: [&str; 11] = [
    KEYBIND_MOVE_UP_KEY,
    KEYBIND_MOVE_DOWN_KEY,
    KEYBIND_MOVE_LEFT_KEY,
    KEYBIND_MOVE_RIGHT_KEY,
    KEYBIND_JUMP_UP_KEY,
    KEYBIND_JUMP_DOWN_KEY,
    KEYBIND_DECREASE_SELECTION_KEY,
    KEYBIND_INCREASE_SELECTION_KEY,
    KEYBIND_KARA_ZERO_INPUT_KEY,
    KEYBIND_TOGGLE_MINIMAP_KEY,
    KEYBIND_OPEN_PREFERENCES_KEY,
];

fn has_legacy_color_settings(storage: &dyn Storage) -> bool {
    eframe::get_value::<u32>(storage, ALTERNATE_COLUMN_COLOR_KEY).is_some()
        || eframe::get_value::<u32>(storage, CELL_BACKGROUND_COLOR_KEY).is_some()
        || eframe::get_value::<u32>(storage, SELECTION_COLOR_KEY).is_some()
        || eframe::get_value::<u32>(storage, HOVER_COLOR_KEY).is_some()
        || eframe::get_value::<u32>(storage, COLUMN_HEADER_BACKGROUND_COLOR_KEY).is_some()
        || eframe::get_value::<u32>(storage, ROW_HEADER_BACKGROUND_COLOR_KEY).is_some()
        || eframe::get_value::<u32>(storage, ZERO_CELL_BACKGROUND_COLOR_KEY).is_some()
        || eframe::get_value::<bool>(storage, USE_ZERO_CELL_BACKGROUND_COLOR_KEY).is_some()
}

fn clamp_sheet_fps(fps: u32) -> u32 {
    fps.clamp(MIN_SHEET_FPS, MAX_SHEET_FPS)
}

fn clamp_seconds_per_page(seconds_per_page: u32) -> u32 {
    seconds_per_page.clamp(1, MAX_SECONDS_PER_PAGE)
}

fn frames_per_page_to_seconds(frames_per_page: u32, fps: u32) -> u32 {
    let fps = fps.max(MIN_SHEET_FPS);
    let seconds = frames_per_page.saturating_add(fps - 1) / fps;
    clamp_seconds_per_page(seconds)
}

fn clamp_initial_frame_count(frame_count: u32, fps: u32) -> u32 {
    let max_frame_count = fps.max(MIN_SHEET_FPS).saturating_mul(MAX_SHEET_SECONDS);
    frame_count.clamp(MIN_INITIAL_FRAME_COUNT, max_frame_count)
}

fn clamp_initial_column_count(column_count: u32) -> u32 {
    column_count.clamp(MIN_INITIAL_COLUMN_COUNT, MAX_INITIAL_COLUMN_COUNT)
}
