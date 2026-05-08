use neosts::sts;
use neosts::{
    AppLocale, ArdFile, DitisFile, DisplaySheetState, StsFile, SxfFile, TableViewState,
    TdtsFile, XdtsFile, strings,
};
use rfd::FileDialog;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct SheetFileState {
    current_source: Option<PathBuf>,
    last_error: Option<String>,
}

impl SheetFileState {
    pub fn current_source(&self) -> Option<&Path> {
        self.current_source.as_deref()
    }

    pub fn current_source_owned(&self) -> Option<PathBuf> {
        self.current_source.clone()
    }

    pub fn set_current_source(&mut self, current_source: Option<PathBuf>) {
        self.current_source = current_source;
    }

    pub fn save_sheet_to_current_path(
        &mut self,
        sheet: &DisplaySheetState,
        loaded_from_sts: &mut bool,
        locale: AppLocale,
    ) -> Result<PathBuf, String> {
        let Some(path) = self.current_source.clone() else {
            return Err(strings::file_path_missing_for_save(locale).to_owned());
        };

        match sts::write_sheet_to_path(sheet.sheet(), &path) {
            Ok(()) => {
                *loaded_from_sts = true;
                self.last_error = None;
                Ok(path)
            }
            Err(error) => {
                let message = strings::file_save_failed(locale, &path, error);
                self.last_error = Some(message.clone());
                Err(message)
            }
        }
    }

    pub fn load_startup_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
        locale: AppLocale,
    ) -> bool {
        if let Err(error) = self.load_path(
            sheet,
            table_view,
            loaded_from_sts,
            default_fps,
            path,
            locale,
        ) {
            self.last_error = Some(strings::startup_load_failed(locale, path, error));
            return false;
        }

        true
    }

    pub fn open_sheet_from_dialog(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        parent_window: &(impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle),
        locale: AppLocale,
    ) -> bool {
        let mut dialog = FileDialog::new()
            .set_parent(parent_window)
            .add_filter(
                "STS / SXF / ARD / XDTS / TDTS / DITIS sheet",
                &["sts", "sxf", "ard", "xdts", "tdts", "ditis"],
            )
            .add_filter("STS sheet", &["sts"])
            .add_filter("SXF sheet", &["sxf"])
            .add_filter("ARD sheet", &["ard"])
            .add_filter("XDTS sheet", &["xdts"])
            .add_filter("TDTS sheet", &["tdts"])
            .add_filter("DITIS sheet", &["ditis"]);
        if let Some(parent) = self.current_source().and_then(Path::parent) {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.pick_file() else {
            return false;
        };

        if let Err(error) = self.load_path(
            sheet,
            table_view,
            loaded_from_sts,
            default_fps,
            &path,
            locale,
        ) {
            self.last_error = Some(strings::file_open_failed(locale, &path, error));
            return false;
        }

        true
    }

    pub fn save_sheet_as_sts_dialog(
        &mut self,
        sheet: &DisplaySheetState,
        loaded_from_sts: &mut bool,
        suggested_name: &str,
        parent_window: &(impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle),
        locale: AppLocale,
    ) -> bool {
        let mut dialog = FileDialog::new()
            .set_parent(parent_window)
            .add_filter("STS sheet", &["sts"])
            .set_file_name(format!("{suggested_name}.sts"));
        if let Some(parent) = self.current_source().and_then(Path::parent) {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.save_file() else {
            return false;
        };

        let path = ensure_sts_extension(path);

        match sts::write_sheet_to_path(sheet.sheet(), &path) {
            Ok(()) => {
                self.current_source = Some(path);
                *loaded_from_sts = true;
                self.last_error = None;
                true
            }
            Err(error) => {
                self.last_error = Some(strings::file_save_failed(locale, &path, error));
                false
            }
        }
    }

    pub fn open_sheet_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
        locale: AppLocale,
    ) -> bool {
        if !is_supported_extension(path) {
            self.last_error = Some(strings::unsupported_file_format_with_path(locale, path));
            return false;
        }

        if let Err(error) = self.load_path(
            sheet,
            table_view,
            loaded_from_sts,
            default_fps,
            path,
            locale,
        ) {
            self.last_error = Some(strings::file_open_failed(locale, path, error));
            return false;
        }

        true
    }

    fn load_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
        locale: AppLocale,
    ) -> Result<(), String> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("sts") => self
                .load_sts_path(sheet, table_view, loaded_from_sts, default_fps, path)
                .map_err(|e| e.localized_message(locale)),
            Some("sxf") => self
                .load_sxf_path(sheet, table_view, loaded_from_sts, default_fps, path)
                .map_err(|e| e.localized_message(locale)),
            Some("ard") => self
                .load_ard_path(sheet, table_view, loaded_from_sts, default_fps, path)
                .map_err(|e| e.localized_message(locale)),
            Some("xdts") => self
                .load_xdts_path(sheet, table_view, loaded_from_sts, default_fps, path)
                .map_err(|e| e.localized_message(locale)),
            Some("tdts") => self
                .load_tdts_path(sheet, table_view, loaded_from_sts, default_fps, path)
                .map_err(|e| e.localized_message(locale)),
            Some("ditis") => self
                .load_ditis_path(sheet, table_view, loaded_from_sts, default_fps, path)
                .map_err(|e| e.localized_message(locale)),
            _ => Err(strings::unsupported_file_format(locale).to_owned()),
        }
    }

    fn load_sts_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
    ) -> Result<(), neosts::StsError> {
        let loaded_sheet =
            StsFile::from_path(path).and_then(|sts| sts.to_sheet_with_fps(default_fps))?;
        sheet.replace_sheet(loaded_sheet);
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            sheet.set_name(stem);
        }
        table_view.reset_for_new_sheet();
        *loaded_from_sts = true;
        self.current_source = Some(path.to_path_buf());
        self.last_error = None;
        Ok(())
    }

    fn load_sxf_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
    ) -> Result<(), neosts::SxfError> {
        let loaded_sheet =
            SxfFile::from_path(path).and_then(|sxf| sxf.to_sheet_with_fps(default_fps))?;
        sheet.replace_sheet(loaded_sheet);
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            sheet.set_name(stem);
        }
        table_view.reset_for_new_sheet();
        *loaded_from_sts = false;
        self.current_source = Some(path.to_path_buf());
        self.last_error = None;
        Ok(())
    }

    fn load_ard_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
    ) -> Result<(), neosts::ArdError> {
        let loaded_sheet =
            ArdFile::from_path(path).and_then(|ard| ard.to_sheet_with_fps(default_fps))?;
        sheet.replace_sheet(loaded_sheet);
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            sheet.set_name(stem);
        }
        table_view.reset_for_new_sheet();
        *loaded_from_sts = false;
        self.current_source = Some(path.to_path_buf());
        self.last_error = None;
        Ok(())
    }

    fn load_xdts_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
    ) -> Result<(), neosts::XdtsError> {
        let loaded_sheet =
            XdtsFile::from_path(path).and_then(|xdts| xdts.to_sheet_with_fps(default_fps))?;
        sheet.replace_sheet(loaded_sheet);
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            sheet.set_name(stem);
        }
        table_view.reset_for_new_sheet();
        *loaded_from_sts = false;
        self.current_source = Some(path.to_path_buf());
        self.last_error = None;
        Ok(())
    }

    fn load_ditis_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
    ) -> Result<(), neosts::DitisError> {
        let ditis = DitisFile::from_path(path)?;
        let internal_name = ditis.first_sheet_name().map(str::to_owned);
        let loaded_sheet = ditis.to_sheet_with_fps(default_fps)?;
        sheet.replace_sheet(loaded_sheet);
        if let Some(name) = internal_name {
            sheet.set_name(name);
        } else if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            sheet.set_name(stem);
        }
        table_view.reset_for_new_sheet();
        *loaded_from_sts = false;
        self.current_source = Some(path.to_path_buf());
        self.last_error = None;
        Ok(())
    }

    fn load_tdts_path(
        &mut self,
        sheet: &mut DisplaySheetState,
        table_view: &mut TableViewState,
        loaded_from_sts: &mut bool,
        default_fps: u32,
        path: &Path,
    ) -> Result<(), neosts::TdtsError> {
        let tdts = TdtsFile::from_path(path)?;
        let internal_name = tdts.first_sheet_name().map(str::to_owned);
        let loaded_sheet = tdts.to_sheet_with_fps(default_fps)?;
        sheet.replace_sheet(loaded_sheet);
        if let Some(name) = internal_name {
            sheet.set_name(name);
        } else if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            sheet.set_name(stem);
        }
        table_view.reset_for_new_sheet();
        *loaded_from_sts = false;
        self.current_source = Some(path.to_path_buf());
        self.last_error = None;
        Ok(())
    }
}

fn ensure_sts_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("sts"))
    {
        path
    } else {
        path.with_extension("sts")
    }
}

fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("sts")
                || ext.eq_ignore_ascii_case("sxf")
                || ext.eq_ignore_ascii_case("ard")
                || ext.eq_ignore_ascii_case("xdts")
                || ext.eq_ignore_ascii_case("tdts")
                || ext.eq_ignore_ascii_case("ditis")
        })
}
