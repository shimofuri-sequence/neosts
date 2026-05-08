use neosts::{AppLocale, TableColorTheme, TableSettings, strings};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const APP_ID: &str = "neosts";
const BUILTIN_THEME_NAMES: &[&str] = &["あさぎ", "さくら", "レモン", "若草"];

#[derive(Clone, Debug)]
pub struct ImportedThemeEntry {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ThemeFileData {
    version: u32,
    app_theme_id: u8,
    custom_theme_base_id: u8,
    theme_preference_id: u8,
    color_theme_id: u8,
    alternate_column_mode_id: u8,
    alternate_darken_amount: f32,
    #[serde(default)]
    alternate_second_darken_amount: f32,
    alternate_saturation_amount: f32,
    alternate_column_rgba: u32,
    cell_background_rgba: u32,
    zero_cell_background_rgba: u32,
    use_zero_cell_background_color: bool,
    selection_rgba: u32,
    hover_rgba: u32,
    column_header_background_rgba: u32,
    row_header_background_rgba: u32,
    special_inserted_row_background_rgba: u32,
    punched_row_background_rgba: u32,
}

pub fn export_theme_file(
    table_settings: &TableSettings,
    locale: AppLocale,
) -> Result<PathBuf, String> {
    let Some(path) = FileDialog::new()
        .add_filter("NeoSTS theme", &["theme"])
        .set_file_name("NeoSTS.theme")
        .save_file()
    else {
        return Err(strings::theme_export_cancelled(locale).to_owned());
    };

    let path = ensure_theme_extension(path);
    let yaml = serde_yaml::to_string(&ThemeFileData::from_settings(table_settings))
        .map_err(|error| strings::theme_export_serialize_failed(locale, error))?;
    fs::write(&path, yaml)
        .map_err(|error| strings::theme_export_write_failed(locale, &path, error))?;
    Ok(path)
}

pub fn import_theme_file(
    table_settings: &mut TableSettings,
    locale: AppLocale,
) -> Result<(PathBuf, String), String> {
    let Some(source_path) = FileDialog::new()
        .add_filter("NeoSTS theme", &["theme"])
        .pick_file()
    else {
        return Err(strings::theme_import_cancelled(locale).to_owned());
    };

    let imported_path = copy_theme_to_library(&source_path, locale)?;
    apply_theme_from_path(table_settings, &imported_path, locale)?;
    let name = theme_name_from_path(&imported_path);
    Ok((imported_path, name))
}

pub fn apply_theme_from_path(
    table_settings: &mut TableSettings,
    path: &Path,
    locale: AppLocale,
) -> Result<(), String> {
    let yaml = fs::read_to_string(path)
        .map_err(|error| strings::theme_read_failed(locale, path, error))?;
    let data = serde_yaml::from_str::<ThemeFileData>(&yaml)
        .map_err(|error| strings::theme_parse_failed(locale, error))?;
    apply_theme_file(table_settings, &data);
    Ok(())
}

pub fn list_imported_themes(locale: AppLocale) -> Result<Vec<ImportedThemeEntry>, String> {
    let themes_dir = theme_library_dir(locale)?;
    if !themes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut themes = Vec::new();
    let entries = fs::read_dir(&themes_dir)
        .map_err(|error| strings::theme_library_read_failed(locale, &themes_dir, error))?;

    for entry in entries {
        let entry = entry
            .map_err(|error| strings::theme_library_list_failed(locale, &themes_dir, error))?;
        let path = entry.path();
        let is_theme = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("theme"));
        if !is_theme || !path.is_file() {
            continue;
        }
        let name = theme_name_from_path(&path);
        if BUILTIN_THEME_NAMES.iter().any(|builtin| *builtin == name) {
            continue;
        }
        themes.push(ImportedThemeEntry { name, path });
    }

    themes.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(themes)
}

pub fn theme_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("カスタムテーマ")
        .to_owned()
}

fn copy_theme_to_library(source_path: &Path, locale: AppLocale) -> Result<PathBuf, String> {
    let themes_dir = theme_library_dir(locale)?;
    fs::create_dir_all(&themes_dir)
        .map_err(|error| strings::theme_library_dir_create_failed(locale, &themes_dir, error))?;
    let file_name = source_path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Imported.theme"));
    let destination = ensure_theme_extension(themes_dir.join(file_name));
    fs::copy(source_path, &destination).map_err(|error| {
        strings::theme_import_copy_failed(locale, source_path, &destination, error)
    })?;
    Ok(destination)
}

fn theme_library_dir(locale: AppLocale) -> Result<PathBuf, String> {
    eframe::storage_dir(APP_ID)
        .map(|dir| dir.join("themes"))
        .ok_or_else(|| strings::theme_library_dir_unavailable(locale).to_owned())
}

fn ensure_theme_extension(path: PathBuf) -> PathBuf {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("theme") => path,
        _ => path.with_extension("theme"),
    }
}

impl ThemeFileData {
    fn from_settings(table_settings: &TableSettings) -> Self {
        Self {
            version: 1,
            app_theme_id: table_settings.theme_id(),
            custom_theme_base_id: table_settings.custom_theme_base_id(),
            theme_preference_id: table_settings.theme_preference_id(),
            color_theme_id: table_settings.color_theme_id(),
            alternate_column_mode_id: table_settings.alternate_column_mode_id(),
            alternate_darken_amount: table_settings.alternate_darken_amount(),
            alternate_second_darken_amount: table_settings.alternate_second_darken_amount(),
            alternate_saturation_amount: table_settings.alternate_saturation_amount(),
            alternate_column_rgba: table_settings.alternate_column_rgba(),
            cell_background_rgba: table_settings.cell_background_rgba(),
            zero_cell_background_rgba: table_settings.zero_cell_background_rgba(),
            use_zero_cell_background_color: table_settings.use_zero_cell_background_color(),
            selection_rgba: table_settings.selection_rgba(),
            hover_rgba: table_settings.hover_rgba(),
            column_header_background_rgba: table_settings.column_header_background_rgba(),
            row_header_background_rgba: table_settings.row_header_background_rgba(),
            special_inserted_row_background_rgba: table_settings
                .special_inserted_row_background_rgba(),
            punched_row_background_rgba: table_settings.punched_row_background_rgba(),
        }
    }
}

fn apply_theme_file(table_settings: &mut TableSettings, data: &ThemeFileData) {
    table_settings.set_theme_preference_id(data.theme_preference_id);
    table_settings.set_custom_theme_base_id(data.custom_theme_base_id);
    if data.app_theme_id <= 6 {
        table_settings.set_theme_id(data.app_theme_id);
    } else {
        table_settings.mark_color_theme_custom();
    }

    table_settings.set_alternate_column_mode_id(data.alternate_column_mode_id);
    table_settings.set_alternate_darken_amount(data.alternate_darken_amount);
    table_settings.set_alternate_second_darken_amount(data.alternate_second_darken_amount);
    table_settings.set_alternate_saturation_amount(data.alternate_saturation_amount);
    table_settings.set_alternate_column_rgba(data.alternate_column_rgba);
    table_settings.set_cell_background_rgba(data.cell_background_rgba);
    table_settings.set_zero_cell_background_rgba(data.zero_cell_background_rgba);
    table_settings.set_use_zero_cell_background_color(data.use_zero_cell_background_color);
    table_settings.set_selection_rgba(data.selection_rgba);
    table_settings.set_hover_rgba(data.hover_rgba);
    table_settings.set_column_header_background_rgba(data.column_header_background_rgba);
    table_settings.set_row_header_background_rgba(data.row_header_background_rgba);
    table_settings
        .set_special_inserted_row_background_rgba(data.special_inserted_row_background_rgba);
    table_settings.set_punched_row_background_rgba(data.punched_row_background_rgba);

    if data.app_theme_id == 7 || data.color_theme_id >= 5 {
        table_settings.mark_color_theme_custom();
    } else {
        table_settings.color_theme = match data.color_theme_id {
            1 => TableColorTheme::Asagi,
            2 => TableColorTheme::Sakura,
            3 => TableColorTheme::Lemon,
            4 => TableColorTheme::Wakakusa,
            _ => TableColorTheme::Default,
        };
    }
}
