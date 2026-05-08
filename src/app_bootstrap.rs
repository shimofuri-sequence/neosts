use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::{fs, path::PathBuf};

#[derive(Debug)]
enum AppIconLoadError {
    Png(String),
    IcoTooSmall,
    IcoHeaderInvalid,
    IcoImageDataOverflow,
    IcoEntryOutsideFile,
    IcoEntryNotPng,
}

impl std::fmt::Display for AppIconLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Png(source) => write!(f, "PNG icon data is invalid: {source}"),
            Self::IcoTooSmall => write!(f, "ICO file is too small"),
            Self::IcoHeaderInvalid => write!(f, "ICO header is invalid"),
            Self::IcoImageDataOverflow => write!(f, "ICO image data overflowed"),
            Self::IcoEntryOutsideFile => write!(f, "ICO image entry points outside the file"),
            Self::IcoEntryNotPng => write!(f, "ICO entry is not PNG-compressed"),
        }
    }
}

pub fn load_embedded_app_icon() -> egui::IconData {
    load_ico_icon(include_bytes!("../assets/neosts.ico"))
        .or_else(|_| {
            eframe::icon_data::from_png_bytes(include_bytes!("../assets/NeoSTS_logo_512.png"))
                .map_err(|error| AppIconLoadError::Png(error.to_string()))
        })
        .expect("failed to load embedded app icon")
}

pub fn configure_japanese_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    if let Some(font_path) = find_japanese_font()
        && let Ok(font_bytes) = fs::read(&font_path)
    {
        fonts
            .font_data
            .insert("jp-ui".to_owned(), FontData::from_owned(font_bytes).into());

        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, "jp-ui".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
            family.insert(0, "jp-ui".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

fn load_ico_icon(ico_bytes: &[u8]) -> Result<egui::IconData, AppIconLoadError> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if ico_bytes.len() < 22 {
        return Err(AppIconLoadError::IcoTooSmall);
    }

    let reserved = u16::from_le_bytes([ico_bytes[0], ico_bytes[1]]);
    let image_type = u16::from_le_bytes([ico_bytes[2], ico_bytes[3]]);
    let image_count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]);

    if reserved != 0 || image_type != 1 || image_count == 0 {
        return Err(AppIconLoadError::IcoHeaderInvalid);
    }

    let image_size =
        u32::from_le_bytes([ico_bytes[14], ico_bytes[15], ico_bytes[16], ico_bytes[17]]) as usize;
    let image_offset =
        u32::from_le_bytes([ico_bytes[18], ico_bytes[19], ico_bytes[20], ico_bytes[21]]) as usize;
    let image_end = image_offset
        .checked_add(image_size)
        .ok_or(AppIconLoadError::IcoImageDataOverflow)?;
    let image_bytes = ico_bytes
        .get(image_offset..image_end)
        .ok_or(AppIconLoadError::IcoEntryOutsideFile)?;

    if !image_bytes.starts_with(PNG_SIGNATURE) {
        return Err(AppIconLoadError::IcoEntryNotPng);
    }

    eframe::icon_data::from_png_bytes(image_bytes)
        .map_err(|error| AppIconLoadError::Png(error.to_string()))
}

fn find_japanese_font() -> Option<PathBuf> {
    japanese_font_candidates()
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
}

fn japanese_font_candidates() -> Vec<&'static str> {
    #[cfg(target_os = "macos")]
    {
        vec![
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
            "/System/Library/Fonts/ヒラギノ丸ゴ ProN W4.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/Library/Fonts/NotoSansCJK-Regular.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ]
    }

    #[cfg(target_os = "windows")]
    {
        vec![
            r"C:\Windows\Fonts\meiryo.ttc",
            r"C:\Windows\Fonts\YuGothR.ttc",
            r"C:\Windows\Fonts\msgothic.ttc",
        ]
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        vec![
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJKJP-Regular.otf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansJP-Regular.otf",
        ]
    }
}
