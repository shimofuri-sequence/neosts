use std::path::{Path, PathBuf};

const MAX_RECENT_FILES: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppLocale {
    Japanese,
    English,
}

impl AppLocale {
    pub const fn storage_id(self) -> u8 {
        match self {
            Self::Japanese => 0,
            Self::English => 1,
        }
    }

    pub const fn from_storage_id(id: u8) -> Self {
        match id {
            1 => Self::English,
            _ => Self::Japanese,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppSettings {
    pub always_on_top: bool,
    pub open_new_sheet_dialog_on_startup: bool,
    locale: AppLocale,
    recent_files: Vec<PathBuf>,
    imported_theme_path: Option<PathBuf>,
    imported_theme_name: Option<String>,
    imported_theme_active: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            always_on_top: false,
            open_new_sheet_dialog_on_startup: true,
            locale: AppLocale::Japanese,
            recent_files: Vec::new(),
            imported_theme_path: None,
            imported_theme_name: None,
            imported_theme_active: false,
        }
    }
}

impl AppSettings {
    pub fn locale(&self) -> AppLocale {
        self.locale
    }

    pub fn set_locale(&mut self, locale: AppLocale) {
        self.locale = locale;
    }

    pub fn recent_files(&self) -> &[PathBuf] {
        &self.recent_files
    }

    pub fn set_recent_files(&mut self, recent_files: Vec<PathBuf>) {
        self.recent_files = recent_files.into_iter().take(MAX_RECENT_FILES).collect();
    }

    pub fn record_recent_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        self.recent_files.retain(|entry| entry != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(MAX_RECENT_FILES);
    }

    pub fn remove_recent_file(&mut self, path: &Path) {
        self.recent_files.retain(|entry| entry != path);
    }

    pub fn imported_theme_path(&self) -> Option<&Path> {
        self.imported_theme_path.as_deref()
    }

    pub fn imported_theme_name(&self) -> Option<&str> {
        self.imported_theme_name.as_deref()
    }

    pub fn imported_theme_active(&self) -> bool {
        self.imported_theme_active
    }

    pub fn set_imported_theme(&mut self, path: PathBuf, name: String) {
        self.imported_theme_path = Some(path);
        self.imported_theme_name = Some(name);
        self.imported_theme_active = true;
    }

    pub fn set_imported_theme_active(&mut self, active: bool) {
        self.imported_theme_active = active;
    }
}

#[derive(Clone, Debug)]
pub struct SheetSettings {
    pub default_fps: u32,
    pub default_seconds_per_page: u32,
    pub initial_frame_count: u32,
    pub initial_column_count: u32,
}

impl Default for SheetSettings {
    fn default() -> Self {
        Self {
            default_fps: 24,
            default_seconds_per_page: 6,
            initial_frame_count: 144,
            initial_column_count: 6,
        }
    }
}

impl SheetSettings {
    pub fn frames_per_page(&self, fps: u32) -> u32 {
        self.default_seconds_per_page
            .max(1)
            .saturating_mul(fps.max(1))
    }

    pub fn initial_frame_count(&self) -> u32 {
        self.initial_frame_count
    }
}
