use eframe::egui::{self, Color32};

pub const MIN_COLUMN_WIDTH: f32 = 48.0;
pub const MAX_COLUMN_WIDTH: f32 = 320.0;
pub const MIN_ROW_HEIGHT: f32 = 20.0;
pub const MAX_ROW_HEIGHT: f32 = 96.0;
pub const MIN_HEADER_WIDTH: f32 = 40.0;
pub const MAX_HEADER_WIDTH: f32 = 160.0;
pub const MIN_HEADER_HEIGHT: f32 = 24.0;
pub const MAX_HEADER_HEIGHT: f32 = 96.0;
pub const DEFAULT_CELL_BACKGROUND: Color32 = Color32::from_rgb(255, 255, 255);
pub const DEFAULT_SELECTION_COLOR: Color32 = Color32::from_rgb(172, 200, 255);
pub const DEFAULT_HOVER_COLOR: Color32 = Color32::from_rgb(236, 240, 250);
pub const DEFAULT_COLUMN_HEADER_BACKGROUND: Color32 = Color32::from_rgb(230, 233, 244);
pub const DEFAULT_ROW_HEADER_BACKGROUND: Color32 = Color32::from_rgb(230, 233, 244);
pub const DEFAULT_SCROLLBAR_BACKGROUND: Color32 = Color32::from_rgb(72, 72, 72);
pub const DEFAULT_SCROLLBAR_HANDLE: Color32 = Color32::from_rgb(172, 200, 255);
pub const DEFAULT_SCROLLBAR_HANDLE_HOVERED: Color32 = Color32::from_rgb(172, 200, 255);
pub const DEFAULT_SCROLLBAR_HANDLE_ACTIVE: Color32 = Color32::from_rgb(172, 200, 255);
pub const DEFAULT_MINIMAP_WIDTH: f32 = 180.0;
pub const DEFAULT_MINIMAP_HEIGHT: f32 = 180.0;
pub const MIN_MINIMAP_WIDTH: f32 = 96.0;
pub const MAX_MINIMAP_WIDTH: f32 = 360.0;
pub const MIN_MINIMAP_HEIGHT: f32 = 96.0;
pub const MAX_MINIMAP_HEIGHT: f32 = 360.0;
pub const DEFAULT_BODY_CELL_FONT_SIZE: f32 = 13.0;
pub const DEFAULT_COLUMN_HEADER_FONT_SIZE: f32 = 16.0;
pub const DEFAULT_ROW_HEADER_FONT_SIZE: f32 = 10.0;
pub const DEFAULT_COLUMN_WIDTH: f32 = 52.0;
pub const DEFAULT_ROW_HEIGHT: f32 = 20.0;
pub const DEFAULT_HEADER_WIDTH: f32 = 72.0;
pub const DEFAULT_HEADER_HEIGHT: f32 = 48.0;
pub const DEFAULT_SHOW_ZERO_VALUE_MARKERS: bool = true;
pub const DEFAULT_SHOW_HEADER_GHOSTS: bool = true;
pub const DEFAULT_THEME_PREFERENCE: egui::ThemePreference = egui::ThemePreference::System;
pub const DEFAULT_UP_SCROLL_TRIGGER_RATIO: f32 = 0.05;
pub const DEFAULT_DOWN_SCROLL_TRIGGER_RATIO: f32 = 0.55;
pub const DEFAULT_CONTINUATION_LINE_MIN_RUN_LENGTH: u32 = 4;
pub const DEFAULT_FRAME_HEADER_MODE: FrameHeaderMode = FrameHeaderMode::SecondFrame;
pub const DEFAULT_FRAME_HEADER_DENSITY: HeaderDisplayDensity = HeaderDisplayDensity::All;
pub const DEFAULT_SEGMENT_HEADER_MODE: SegmentHeaderMode = SegmentHeaderMode::Pages;
pub const DEFAULT_SEGMENT_HEADER_DENSITY: HeaderDisplayDensity = HeaderDisplayDensity::All;
pub const DEFAULT_SPECIAL_INSERTED_ROW_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(255, 235, 180);
pub const DEFAULT_PUNCHED_ROW_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(40, 40, 40);
pub const DEFAULT_CELL_BACKGROUND_DARK: Color32 = Color32::from_rgb(34, 39, 46);
pub const DEFAULT_SELECTION_COLOR_DARK: Color32 = Color32::from_rgb(84, 122, 184);
pub const DEFAULT_HOVER_COLOR_DARK: Color32 = Color32::from_rgb(54, 62, 74);
pub const DEFAULT_COLUMN_HEADER_BACKGROUND_DARK: Color32 = Color32::from_rgb(49, 56, 66);
pub const DEFAULT_ROW_HEADER_BACKGROUND_DARK: Color32 = Color32::from_rgb(49, 56, 66);
pub const DEFAULT_SCROLLBAR_BACKGROUND_DARK: Color32 = DEFAULT_SCROLLBAR_BACKGROUND;
pub const DEFAULT_SCROLLBAR_HANDLE_DARK: Color32 = DEFAULT_SCROLLBAR_HANDLE;
pub const DEFAULT_SCROLLBAR_HANDLE_HOVERED_DARK: Color32 = DEFAULT_SCROLLBAR_HANDLE_HOVERED;
pub const DEFAULT_SCROLLBAR_HANDLE_ACTIVE_DARK: Color32 = DEFAULT_SCROLLBAR_HANDLE_ACTIVE;
pub const DEFAULT_SPECIAL_INSERTED_ROW_BACKGROUND_DARK: Color32 = Color32::from_rgb(110, 90, 52);
pub const DEFAULT_PUNCHED_ROW_BACKGROUND_DARK: Color32 = Color32::from_rgb(18, 20, 24);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlternateColumnMode {
    Off,
    Darken,
    CustomColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameHeaderMode {
    SecondFrame,
    AbsoluteFrame,
    PageFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentHeaderMode {
    Seconds,
    Pages,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderDisplayDensity {
    All,
    Odd,
    Even,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuationLineStyle {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableColorTheme {
    Default,
    Asagi,
    Sakura,
    Lemon,
    Wakakusa,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TablePalette {
    pub alternate_column_mode: AlternateColumnMode,
    pub alternate_darken_amount: f32,
    pub alternate_second_darken_amount: f32,
    pub alternate_saturation_amount: f32,
    pub alternate_column_color: Color32,
    pub cell_background_color: Color32,
    pub zero_cell_background_color: Color32,
    pub use_zero_cell_background_color: bool,
    pub selection_color: Color32,
    pub hover_color: Color32,
    pub column_header_background_color: Color32,
    pub row_header_background_color: Color32,
    pub scrollbar_background_color: Color32,
    pub scrollbar_handle_color: Color32,
    pub scrollbar_handle_hovered_color: Color32,
    pub scrollbar_handle_active_color: Color32,
    pub special_inserted_row_background_color: Color32,
    pub punched_row_background_color: Color32,
}

#[derive(Clone, Debug)]
pub struct TableSettings {
    pub cell_scale: f32,
    pub default_column_width: f32,
    pub default_row_height: f32,
    pub default_header_width: f32,
    pub default_header_height: f32,
    pub show_zero_value_markers: bool,
    pub show_header_ghosts: bool,
    pub theme_preference: egui::ThemePreference,
    pub up_scroll_trigger_ratio: f32,
    pub down_scroll_trigger_ratio: f32,
    pub continuation_line_min_run_length: u32,
    pub continuation_line_style: ContinuationLineStyle,
    pub alternate_column_mode: AlternateColumnMode,
    pub alternate_darken_amount: f32,
    pub alternate_second_darken_amount: f32,
    pub alternate_saturation_amount: f32,
    pub alternate_column_color: Color32,
    pub cell_background_color: Color32,
    pub zero_cell_background_color: Color32,
    pub use_zero_cell_background_color: bool,
    pub selection_color: Color32,
    pub hover_color: Color32,
    pub column_header_background_color: Color32,
    pub row_header_background_color: Color32,
    pub scrollbar_background_color: Color32,
    pub scrollbar_handle_color: Color32,
    pub scrollbar_handle_hovered_color: Color32,
    pub scrollbar_handle_active_color: Color32,
    pub minimap_width: f32,
    pub minimap_height: f32,
    pub column_header_font_size: f32,
    pub body_cell_font_size: f32,
    pub row_header_font_size: f32,
    pub frame_header_mode: FrameHeaderMode,
    pub frame_header_density: HeaderDisplayDensity,
    pub segment_header_mode: SegmentHeaderMode,
    pub segment_header_density: HeaderDisplayDensity,
    pub color_theme: TableColorTheme,
    pub custom_theme_base_id: u8,
    pub special_inserted_row_background_color: Color32,
    pub punched_row_background_color: Color32,
}

impl Default for TableSettings {
    fn default() -> Self {
        Self {
            cell_scale: 1.0,
            default_column_width: DEFAULT_COLUMN_WIDTH,
            default_row_height: DEFAULT_ROW_HEIGHT,
            default_header_width: DEFAULT_HEADER_WIDTH,
            default_header_height: DEFAULT_HEADER_HEIGHT,
            show_zero_value_markers: DEFAULT_SHOW_ZERO_VALUE_MARKERS,
            show_header_ghosts: DEFAULT_SHOW_HEADER_GHOSTS,
            theme_preference: DEFAULT_THEME_PREFERENCE,
            up_scroll_trigger_ratio: DEFAULT_UP_SCROLL_TRIGGER_RATIO,
            down_scroll_trigger_ratio: DEFAULT_DOWN_SCROLL_TRIGGER_RATIO,
            continuation_line_min_run_length: DEFAULT_CONTINUATION_LINE_MIN_RUN_LENGTH,
            continuation_line_style: ContinuationLineStyle::Vertical,
            alternate_column_mode: AlternateColumnMode::Off,
            alternate_darken_amount: 0.0,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: DEFAULT_CELL_BACKGROUND,
            cell_background_color: DEFAULT_CELL_BACKGROUND,
            zero_cell_background_color: DEFAULT_CELL_BACKGROUND,
            use_zero_cell_background_color: false,
            selection_color: DEFAULT_SELECTION_COLOR,
            hover_color: DEFAULT_HOVER_COLOR,
            column_header_background_color: DEFAULT_COLUMN_HEADER_BACKGROUND,
            row_header_background_color: DEFAULT_ROW_HEADER_BACKGROUND,
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE,
            minimap_width: DEFAULT_MINIMAP_WIDTH,
            minimap_height: DEFAULT_MINIMAP_HEIGHT,
            column_header_font_size: DEFAULT_COLUMN_HEADER_FONT_SIZE,
            body_cell_font_size: DEFAULT_BODY_CELL_FONT_SIZE,
            row_header_font_size: DEFAULT_ROW_HEADER_FONT_SIZE,
            frame_header_mode: DEFAULT_FRAME_HEADER_MODE,
            frame_header_density: DEFAULT_FRAME_HEADER_DENSITY,
            segment_header_mode: DEFAULT_SEGMENT_HEADER_MODE,
            segment_header_density: DEFAULT_SEGMENT_HEADER_DENSITY,
            color_theme: TableColorTheme::Default,
            custom_theme_base_id: 0,
            special_inserted_row_background_color: DEFAULT_SPECIAL_INSERTED_ROW_BACKGROUND_LIGHT,
            punched_row_background_color: DEFAULT_PUNCHED_ROW_BACKGROUND_LIGHT,
        }
    }
}

impl TableSettings {
    pub fn color_theme_id(&self) -> u8 {
        match self.color_theme {
            TableColorTheme::Default => 0,
            TableColorTheme::Asagi => 1,
            TableColorTheme::Sakura => 2,
            TableColorTheme::Lemon => 3,
            TableColorTheme::Wakakusa => 4,
            TableColorTheme::Custom => 5,
        }
    }
    pub fn set_color_theme_id(&mut self, id: u8) {
        self.color_theme = match id {
            1 => TableColorTheme::Asagi,
            2 => TableColorTheme::Sakura,
            3 => TableColorTheme::Lemon,
            4 => TableColorTheme::Wakakusa,
            5 | 6 => TableColorTheme::Custom,
            _ => TableColorTheme::Default,
        };
    }
    pub fn theme_id(&self) -> u8 {
        match self.color_theme {
            TableColorTheme::Default => match self.theme_preference {
                egui::ThemePreference::System => 0,
                egui::ThemePreference::Light => 1,
                egui::ThemePreference::Dark => 2,
            },
            TableColorTheme::Asagi => 3,
            TableColorTheme::Sakura => 4,
            TableColorTheme::Lemon => 5,
            TableColorTheme::Wakakusa => 6,
            TableColorTheme::Custom => 7,
        }
    }
    pub fn custom_theme_base_id(&self) -> u8 {
        self.custom_theme_base_id
    }
    pub fn set_custom_theme_base_id(&mut self, id: u8) {
        self.custom_theme_base_id = id.min(6);
    }
    pub fn set_theme_id(&mut self, id: u8) {
        match id {
            0 => {
                self.custom_theme_base_id = 0;
                self.theme_preference = egui::ThemePreference::System;
                self.color_theme = TableColorTheme::Default;
            }
            1 => {
                self.custom_theme_base_id = 1;
                self.theme_preference = egui::ThemePreference::Light;
                self.color_theme = TableColorTheme::Default;
                self.apply_default_palette(false);
            }
            2 => {
                self.custom_theme_base_id = 2;
                self.theme_preference = egui::ThemePreference::Dark;
                self.color_theme = TableColorTheme::Default;
                self.apply_default_palette(true);
            }
            3 => {
                self.custom_theme_base_id = 3;
                self.theme_preference = egui::ThemePreference::Light;
                self.apply_color_theme(TableColorTheme::Asagi);
            }
            4 => {
                self.custom_theme_base_id = 4;
                self.theme_preference = egui::ThemePreference::Light;
                self.apply_color_theme(TableColorTheme::Sakura);
            }
            5 => {
                self.custom_theme_base_id = 5;
                self.theme_preference = egui::ThemePreference::Light;
                self.apply_color_theme(TableColorTheme::Lemon);
            }
            6 => {
                self.custom_theme_base_id = 6;
                self.theme_preference = egui::ThemePreference::Light;
                self.apply_color_theme(TableColorTheme::Wakakusa);
            }
            _ => self.mark_color_theme_custom(),
        }
    }
    pub fn apply_color_theme(&mut self, theme: TableColorTheme) {
        self.color_theme = theme;
        if let Some(palette) = TablePalette::for_theme(theme) {
            self.apply_palette(palette);
        }
    }
    pub fn mark_color_theme_custom(&mut self) {
        self.color_theme = TableColorTheme::Custom;
    }
    pub fn sync_default_color_theme(&mut self, dark_mode: bool) {
        if self.color_theme == TableColorTheme::Default {
            self.apply_default_palette(dark_mode);
        }
    }
    pub fn apply_palette(&mut self, palette: TablePalette) {
        self.alternate_column_mode = palette.alternate_column_mode;
        self.alternate_darken_amount = palette.alternate_darken_amount;
        self.alternate_second_darken_amount = palette.alternate_second_darken_amount;
        self.alternate_saturation_amount = palette.alternate_saturation_amount;
        self.alternate_column_color = palette.alternate_column_color;
        self.cell_background_color = palette.cell_background_color;
        self.zero_cell_background_color = palette.zero_cell_background_color;
        self.use_zero_cell_background_color = palette.use_zero_cell_background_color;
        self.selection_color = palette.selection_color;
        self.hover_color = palette.hover_color;
        self.column_header_background_color = palette.column_header_background_color;
        self.row_header_background_color = palette.row_header_background_color;
        self.scrollbar_background_color = palette.scrollbar_background_color;
        self.scrollbar_handle_color = palette.scrollbar_handle_color;
        self.scrollbar_handle_hovered_color = palette.scrollbar_handle_hovered_color;
        self.scrollbar_handle_active_color = palette.scrollbar_handle_active_color;
        self.special_inserted_row_background_color = palette.special_inserted_row_background_color;
        self.punched_row_background_color = palette.punched_row_background_color;
    }
    pub fn continuation_line_min_run_length(&self) -> u32 {
        self.continuation_line_min_run_length
    }
    pub fn set_continuation_line_min_run_length(&mut self, value: u32) {
        self.continuation_line_min_run_length = value;
    }
    pub fn continuation_line_style(&self) -> ContinuationLineStyle {
        self.continuation_line_style
    }
    pub fn continuation_line_style_id(&self) -> u8 {
        match self.continuation_line_style {
            ContinuationLineStyle::Vertical => 0,
            ContinuationLineStyle::Horizontal => 1,
        }
    }
    pub fn set_continuation_line_style(&mut self, style: ContinuationLineStyle) {
        self.continuation_line_style = style;
    }
    pub fn set_continuation_line_style_id(&mut self, id: u8) {
        self.continuation_line_style = match id {
            1 => ContinuationLineStyle::Horizontal,
            _ => ContinuationLineStyle::Vertical,
        };
    }
    pub fn punched_row_background_color(&self) -> Color32 {
        self.punched_row_background_color
    }
    pub fn set_punched_row_background_color(&mut self, color: Color32) {
        self.punched_row_background_color = color;
    }
    pub fn punched_row_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.punched_row_background_color)
    }
    pub fn set_punched_row_background_rgba(&mut self, rgba: u32) {
        self.punched_row_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn cell_scale(&self) -> f32 {
        self.cell_scale
    }
    pub fn set_cell_scale(&mut self, cell_scale: f32) {
        self.cell_scale = cell_scale.clamp(0.5, 2.0);
    }
    pub fn default_column_width(&self) -> f32 {
        self.default_column_width
    }
    pub fn set_default_column_width(&mut self, width: f32) {
        self.default_column_width = width.clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH);
    }
    pub fn default_row_height(&self) -> f32 {
        self.default_row_height
    }
    pub fn set_default_row_height(&mut self, height: f32) {
        self.default_row_height = height.clamp(MIN_ROW_HEIGHT, MAX_ROW_HEIGHT);
    }
    pub fn default_header_width(&self) -> f32 {
        self.default_header_width
    }
    pub fn set_default_header_width(&mut self, width: f32) {
        self.default_header_width = width.clamp(MIN_HEADER_WIDTH, MAX_HEADER_WIDTH);
    }
    pub fn default_header_height(&self) -> f32 {
        self.default_header_height
    }
    pub fn set_default_header_height(&mut self, height: f32) {
        self.default_header_height = height.clamp(MIN_HEADER_HEIGHT, MAX_HEADER_HEIGHT);
    }
    pub fn alternate_column_mode_id(&self) -> u8 {
        match self.alternate_column_mode {
            AlternateColumnMode::Off => 0,
            AlternateColumnMode::Darken => 1,
            AlternateColumnMode::CustomColor => 2,
        }
    }
    pub fn set_alternate_column_mode_id(&mut self, id: u8) {
        self.alternate_column_mode = match id {
            1 => AlternateColumnMode::Darken,
            2 => AlternateColumnMode::CustomColor,
            _ => AlternateColumnMode::Off,
        };
    }
    pub fn alternate_darken_amount(&self) -> f32 {
        self.alternate_darken_amount
    }
    pub fn set_alternate_darken_amount(&mut self, amount: f32) {
        self.alternate_darken_amount = amount.clamp(-1.0, 1.0);
    }
    pub fn alternate_second_darken_amount(&self) -> f32 {
        self.alternate_second_darken_amount
    }
    pub fn set_alternate_second_darken_amount(&mut self, amount: f32) {
        self.alternate_second_darken_amount = amount.clamp(-1.0, 1.0);
    }
    pub fn alternate_saturation_amount(&self) -> f32 {
        self.alternate_saturation_amount
    }
    pub fn set_alternate_saturation_amount(&mut self, amount: f32) {
        self.alternate_saturation_amount = amount.clamp(-1.0, 1.0);
    }
    pub fn alternate_column_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.alternate_column_color)
    }
    pub fn set_alternate_column_rgba(&mut self, rgba: u32) {
        self.alternate_column_color = color32_from_rgba_u32(rgba);
    }
    pub fn cell_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.cell_background_color)
    }
    pub fn set_cell_background_rgba(&mut self, rgba: u32) {
        self.cell_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn selection_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.selection_color)
    }
    pub fn set_selection_rgba(&mut self, rgba: u32) {
        self.selection_color = color32_from_rgba_u32(rgba);
    }
    pub fn hover_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.hover_color)
    }
    pub fn set_hover_rgba(&mut self, rgba: u32) {
        self.hover_color = color32_from_rgba_u32(rgba);
    }
    pub fn column_header_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.column_header_background_color)
    }
    pub fn set_column_header_background_rgba(&mut self, rgba: u32) {
        self.column_header_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn row_header_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.row_header_background_color)
    }
    pub fn set_row_header_background_rgba(&mut self, rgba: u32) {
        self.row_header_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn scrollbar_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.scrollbar_background_color)
    }
    pub fn set_scrollbar_background_rgba(&mut self, rgba: u32) {
        self.scrollbar_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn scrollbar_handle_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.scrollbar_handle_color)
    }
    pub fn set_scrollbar_handle_rgba(&mut self, rgba: u32) {
        self.scrollbar_handle_color = color32_from_rgba_u32(rgba);
    }
    pub fn scrollbar_handle_hovered_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.scrollbar_handle_hovered_color)
    }
    pub fn set_scrollbar_handle_hovered_rgba(&mut self, rgba: u32) {
        self.scrollbar_handle_hovered_color = color32_from_rgba_u32(rgba);
    }
    pub fn scrollbar_handle_active_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.scrollbar_handle_active_color)
    }
    pub fn set_scrollbar_handle_active_rgba(&mut self, rgba: u32) {
        self.scrollbar_handle_active_color = color32_from_rgba_u32(rgba);
    }
    pub fn minimap_width(&self) -> f32 {
        self.minimap_width
    }
    pub fn set_minimap_width(&mut self, width: f32) {
        self.minimap_width = width.clamp(MIN_MINIMAP_WIDTH, MAX_MINIMAP_WIDTH);
    }
    pub fn minimap_height(&self) -> f32 {
        self.minimap_height
    }
    pub fn set_minimap_height(&mut self, height: f32) {
        self.minimap_height = height.clamp(MIN_MINIMAP_HEIGHT, MAX_MINIMAP_HEIGHT);
    }
    pub fn column_header_font_size(&self) -> f32 {
        self.column_header_font_size
    }
    pub fn set_column_header_font_size(&mut self, size: f32) {
        self.column_header_font_size = size.clamp(6.0, 72.0);
    }
    pub fn body_cell_font_size(&self) -> f32 {
        self.body_cell_font_size
    }
    pub fn set_body_cell_font_size(&mut self, size: f32) {
        self.body_cell_font_size = size.clamp(6.0, 72.0);
    }
    pub fn row_header_font_size(&self) -> f32 {
        self.row_header_font_size
    }
    pub fn set_row_header_font_size(&mut self, size: f32) {
        self.row_header_font_size = size.clamp(6.0, 72.0);
    }
    pub fn zero_cell_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.zero_cell_background_color)
    }
    pub fn set_zero_cell_background_rgba(&mut self, rgba: u32) {
        self.zero_cell_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn use_zero_cell_background_color(&self) -> bool {
        self.use_zero_cell_background_color
    }
    pub fn set_use_zero_cell_background_color(&mut self, enabled: bool) {
        self.use_zero_cell_background_color = enabled;
    }
    pub fn show_zero_value_markers(&self) -> bool {
        self.show_zero_value_markers
    }
    pub fn set_show_zero_value_markers(&mut self, enabled: bool) {
        self.show_zero_value_markers = enabled;
    }
    pub fn show_header_ghosts(&self) -> bool {
        self.show_header_ghosts
    }
    pub fn set_show_header_ghosts(&mut self, enabled: bool) {
        self.show_header_ghosts = enabled;
    }
    pub fn frame_header_mode_id(&self) -> u8 {
        match self.frame_header_mode {
            FrameHeaderMode::SecondFrame => 0,
            FrameHeaderMode::PageFrame => 1,
            FrameHeaderMode::AbsoluteFrame => 2,
        }
    }
    pub fn set_frame_header_mode_id(&mut self, mode_id: u8) {
        self.frame_header_mode = match mode_id {
            1 => FrameHeaderMode::PageFrame,
            2 => FrameHeaderMode::AbsoluteFrame,
            _ => FrameHeaderMode::SecondFrame,
        };
    }
    pub fn frame_header_density_id(&self) -> u8 {
        match self.frame_header_density {
            HeaderDisplayDensity::All => 0,
            HeaderDisplayDensity::Odd => 1,
            HeaderDisplayDensity::Even => 2,
        }
    }
    pub fn set_frame_header_density_id(&mut self, density_id: u8) {
        self.frame_header_density = match density_id {
            1 => HeaderDisplayDensity::Odd,
            2 => HeaderDisplayDensity::Even,
            _ => HeaderDisplayDensity::All,
        };
    }
    pub fn segment_header_mode_id(&self) -> u8 {
        match self.segment_header_mode {
            SegmentHeaderMode::Seconds => 0,
            SegmentHeaderMode::Pages => 1,
        }
    }
    pub fn set_segment_header_mode_id(&mut self, mode_id: u8) {
        self.segment_header_mode = match mode_id {
            1 => SegmentHeaderMode::Pages,
            _ => SegmentHeaderMode::Seconds,
        };
    }
    pub fn segment_header_density_id(&self) -> u8 {
        match self.segment_header_density {
            HeaderDisplayDensity::All => 0,
            HeaderDisplayDensity::Odd => 1,
            HeaderDisplayDensity::Even => 2,
        }
    }
    pub fn set_segment_header_density_id(&mut self, density_id: u8) {
        self.segment_header_density = match density_id {
            1 => HeaderDisplayDensity::Odd,
            2 => HeaderDisplayDensity::Even,
            _ => HeaderDisplayDensity::All,
        };
    }
    pub fn theme_preference_id(&self) -> u8 {
        match self.theme_preference {
            egui::ThemePreference::Dark => 0,
            egui::ThemePreference::Light => 1,
            egui::ThemePreference::System => 2,
        }
    }
    pub fn set_theme_preference_id(&mut self, id: u8) {
        self.theme_preference = match id {
            0 => egui::ThemePreference::Dark,
            1 => egui::ThemePreference::Light,
            _ => egui::ThemePreference::System,
        };
    }
    pub fn up_scroll_trigger_ratio(&self) -> f32 {
        self.up_scroll_trigger_ratio
    }
    pub fn set_up_scroll_trigger_ratio(&mut self, ratio: f32) {
        self.up_scroll_trigger_ratio = ratio.clamp(0.0, 0.95);
    }
    pub fn down_scroll_trigger_ratio(&self) -> f32 {
        self.down_scroll_trigger_ratio
    }
    pub fn set_down_scroll_trigger_ratio(&mut self, ratio: f32) {
        self.down_scroll_trigger_ratio = ratio.clamp(0.0, 0.95);
    }
    pub fn special_inserted_row_background_rgba(&self) -> u32 {
        color32_to_rgba_u32(self.special_inserted_row_background_color)
    }
    pub fn set_special_inserted_row_background_rgba(&mut self, rgba: u32) {
        self.special_inserted_row_background_color = color32_from_rgba_u32(rgba);
    }
    pub fn reset_color_settings(&mut self, system_dark_mode: bool) {
        match self.theme_id() {
            0 => {
                self.color_theme = TableColorTheme::Default;
                self.apply_default_palette(system_dark_mode);
            }
            1 => {
                self.theme_preference = egui::ThemePreference::Light;
                self.color_theme = TableColorTheme::Default;
                self.apply_default_palette(false);
            }
            2 => {
                self.theme_preference = egui::ThemePreference::Dark;
                self.color_theme = TableColorTheme::Default;
                self.apply_default_palette(true);
            }
            3 => self.apply_color_theme(TableColorTheme::Asagi),
            4 => self.apply_color_theme(TableColorTheme::Sakura),
            5 => self.apply_color_theme(TableColorTheme::Lemon),
            6 => self.apply_color_theme(TableColorTheme::Wakakusa),
            _ => match self.custom_theme_base_id {
                0 => {
                    self.color_theme = TableColorTheme::Default;
                    self.theme_preference = egui::ThemePreference::System;
                    self.apply_default_palette(system_dark_mode);
                }
                1 => {
                    self.color_theme = TableColorTheme::Default;
                    self.theme_preference = egui::ThemePreference::Light;
                    self.apply_default_palette(false);
                }
                2 => {
                    self.color_theme = TableColorTheme::Default;
                    self.theme_preference = egui::ThemePreference::Dark;
                    self.apply_default_palette(true);
                }
                3 => {
                    self.theme_preference = egui::ThemePreference::Light;
                    self.apply_color_theme(TableColorTheme::Asagi);
                }
                4 => {
                    self.theme_preference = egui::ThemePreference::Light;
                    self.apply_color_theme(TableColorTheme::Sakura);
                }
                5 => {
                    self.theme_preference = egui::ThemePreference::Light;
                    self.apply_color_theme(TableColorTheme::Lemon);
                }
                6 => {
                    self.theme_preference = egui::ThemePreference::Light;
                    self.apply_color_theme(TableColorTheme::Wakakusa);
                }
                _ => {
                    self.color_theme = TableColorTheme::Default;
                    self.apply_default_palette(matches!(
                        self.theme_preference,
                        egui::ThemePreference::Dark
                    ));
                }
            },
        }
    }

    pub fn reset_size_settings(&mut self) {
        self.default_column_width = DEFAULT_COLUMN_WIDTH;
        self.default_row_height = DEFAULT_ROW_HEIGHT;
        self.default_header_width = DEFAULT_HEADER_WIDTH;
        self.default_header_height = DEFAULT_HEADER_HEIGHT;
        self.minimap_width = DEFAULT_MINIMAP_WIDTH;
        self.minimap_height = DEFAULT_MINIMAP_HEIGHT;
        self.column_header_font_size = DEFAULT_COLUMN_HEADER_FONT_SIZE;
        self.body_cell_font_size = DEFAULT_BODY_CELL_FONT_SIZE;
        self.row_header_font_size = DEFAULT_ROW_HEADER_FONT_SIZE;
    }

    pub fn reset_display_settings(&mut self) {
        self.reset_size_settings();
        self.show_zero_value_markers = DEFAULT_SHOW_ZERO_VALUE_MARKERS;
        self.show_header_ghosts = DEFAULT_SHOW_HEADER_GHOSTS;
        self.theme_preference = DEFAULT_THEME_PREFERENCE;
        self.color_theme = TableColorTheme::Default;
        self.custom_theme_base_id = 0;
        self.up_scroll_trigger_ratio = DEFAULT_UP_SCROLL_TRIGGER_RATIO;
        self.down_scroll_trigger_ratio = DEFAULT_DOWN_SCROLL_TRIGGER_RATIO;
        self.continuation_line_min_run_length = DEFAULT_CONTINUATION_LINE_MIN_RUN_LENGTH;
        self.continuation_line_style = ContinuationLineStyle::Vertical;
        self.frame_header_mode = DEFAULT_FRAME_HEADER_MODE;
        self.frame_header_density = DEFAULT_FRAME_HEADER_DENSITY;
        self.segment_header_mode = DEFAULT_SEGMENT_HEADER_MODE;
        self.segment_header_density = DEFAULT_SEGMENT_HEADER_DENSITY;
    }
}

impl TablePalette {
    pub fn default(dark_mode: bool) -> Self {
        if dark_mode {
            Self::dark_default()
        } else {
            Self::light_default()
        }
    }

    pub fn light_default() -> Self {
        Self {
            alternate_column_mode: AlternateColumnMode::Off,
            alternate_darken_amount: 0.0,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: DEFAULT_CELL_BACKGROUND,
            cell_background_color: DEFAULT_CELL_BACKGROUND,
            zero_cell_background_color: DEFAULT_CELL_BACKGROUND,
            use_zero_cell_background_color: false,
            selection_color: DEFAULT_SELECTION_COLOR,
            hover_color: DEFAULT_HOVER_COLOR,
            column_header_background_color: DEFAULT_COLUMN_HEADER_BACKGROUND,
            row_header_background_color: DEFAULT_ROW_HEADER_BACKGROUND,
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE,
            special_inserted_row_background_color: DEFAULT_SPECIAL_INSERTED_ROW_BACKGROUND_LIGHT,
            punched_row_background_color: DEFAULT_PUNCHED_ROW_BACKGROUND_LIGHT,
        }
    }

    pub fn asagi() -> Self {
        Self {
            alternate_column_mode: AlternateColumnMode::Darken,
            alternate_darken_amount: 0.05,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: color32_from_rgba_u32(4294967295),
            cell_background_color: color32_from_rgba_u32(4293521095),
            zero_cell_background_color: color32_from_rgba_u32(4292272298),
            use_zero_cell_background_color: false,
            selection_color: color32_from_rgba_u32(4293966998),
            hover_color: color32_from_rgba_u32(4292997028),
            column_header_background_color: color32_from_rgba_u32(4292271275),
            row_header_background_color: color32_from_rgba_u32(4292271275),
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE,
            special_inserted_row_background_color: color32_from_rgba_u32(4290046975),
            punched_row_background_color: color32_from_rgba_u32(4280821800),
        }
    }

    pub fn sakura() -> Self {
        Self {
            alternate_column_mode: AlternateColumnMode::Darken,
            alternate_darken_amount: 0.05,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: color32_from_rgba_u32(4294967295),
            cell_background_color: color32_from_rgba_u32(4294633468),
            zero_cell_background_color: color32_from_rgba_u32(4293579519),
            use_zero_cell_background_color: false,
            selection_color: color32_from_rgba_u32(4293966998),
            hover_color: color32_from_rgba_u32(4294026492),
            column_header_background_color: color32_from_rgba_u32(4294565625),
            row_header_background_color: color32_from_rgba_u32(4294565625),
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE,
            special_inserted_row_background_color: color32_from_rgba_u32(4290046975),
            punched_row_background_color: color32_from_rgba_u32(4280821800),
        }
    }

    pub fn lemon() -> Self {
        Self {
            alternate_column_mode: AlternateColumnMode::Darken,
            alternate_darken_amount: 0.05,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: color32_from_rgba_u32(4294967295),
            cell_background_color: color32_from_rgba_u32(4291489776),
            zero_cell_background_color: color32_from_rgba_u32(4290046186),
            use_zero_cell_background_color: false,
            selection_color: color32_from_rgba_u32(4292071305),
            hover_color: color32_from_rgba_u32(4289854968),
            column_header_background_color: color32_from_rgba_u32(4290372325),
            row_header_background_color: color32_from_rgba_u32(4290372325),
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE,
            special_inserted_row_background_color: color32_from_rgba_u32(4290046975),
            punched_row_background_color: color32_from_rgba_u32(4280821800),
        }
    }

    pub fn wakakusa() -> Self {
        Self {
            alternate_column_mode: AlternateColumnMode::Darken,
            alternate_darken_amount: 0.05,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: color32_from_rgba_u32(4294967295),
            cell_background_color: color32_from_rgba_u32(4291358676),
            zero_cell_background_color: color32_from_rgba_u32(4289714622),
            use_zero_cell_background_color: false,
            selection_color: color32_from_rgba_u32(4292071305),
            hover_color: color32_from_rgba_u32(4288348095),
            column_header_background_color: color32_from_rgba_u32(4289783746),
            row_header_background_color: color32_from_rgba_u32(4289783746),
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE,
            special_inserted_row_background_color: color32_from_rgba_u32(4290046975),
            punched_row_background_color: color32_from_rgba_u32(4280821800),
        }
    }

    pub fn dark_default() -> Self {
        Self {
            alternate_column_mode: AlternateColumnMode::Off,
            alternate_darken_amount: 0.0,
            alternate_second_darken_amount: 0.0,
            alternate_saturation_amount: 0.0,
            alternate_column_color: DEFAULT_CELL_BACKGROUND_DARK,
            cell_background_color: DEFAULT_CELL_BACKGROUND_DARK,
            zero_cell_background_color: DEFAULT_CELL_BACKGROUND_DARK,
            use_zero_cell_background_color: false,
            selection_color: DEFAULT_SELECTION_COLOR_DARK,
            hover_color: DEFAULT_HOVER_COLOR_DARK,
            column_header_background_color: DEFAULT_COLUMN_HEADER_BACKGROUND_DARK,
            row_header_background_color: DEFAULT_ROW_HEADER_BACKGROUND_DARK,
            scrollbar_background_color: DEFAULT_SCROLLBAR_BACKGROUND_DARK,
            scrollbar_handle_color: DEFAULT_SCROLLBAR_HANDLE_DARK,
            scrollbar_handle_hovered_color: DEFAULT_SCROLLBAR_HANDLE_HOVERED_DARK,
            scrollbar_handle_active_color: DEFAULT_SCROLLBAR_HANDLE_ACTIVE_DARK,
            special_inserted_row_background_color: DEFAULT_SPECIAL_INSERTED_ROW_BACKGROUND_DARK,
            punched_row_background_color: DEFAULT_PUNCHED_ROW_BACKGROUND_DARK,
        }
    }

    pub fn for_theme(theme: TableColorTheme) -> Option<Self> {
        match theme {
            TableColorTheme::Default => Some(Self::light_default()),
            TableColorTheme::Asagi => Some(Self::asagi()),
            TableColorTheme::Sakura => Some(Self::sakura()),
            TableColorTheme::Lemon => Some(Self::lemon()),
            TableColorTheme::Wakakusa => Some(Self::wakakusa()),
            TableColorTheme::Custom => None,
        }
    }
}

impl TableSettings {
    fn apply_default_palette(&mut self, dark_mode: bool) {
        self.apply_palette(TablePalette::default(dark_mode));
    }
}

fn color32_to_rgba_u32(color: Color32) -> u32 {
    color
        .to_srgba_unmultiplied()
        .map(u32::from)
        .into_iter()
        .enumerate()
        .fold(0, |acc, (index, value)| acc | (value << (index * 8)))
}

fn color32_from_rgba_u32(rgba: u32) -> Color32 {
    let r = (rgba & 0xFF) as u8;
    let g = ((rgba >> 8) & 0xFF) as u8;
    let b = ((rgba >> 16) & 0xFF) as u8;
    let a = ((rgba >> 24) & 0xFF) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}
