use eframe::egui;
use neosts::TableSettings;

pub fn apply_app_theme_visuals(ctx: &egui::Context, table_settings: &TableSettings) {
    restore_default_theme_visuals(ctx);
    match table_settings.theme_id() {
        0 => ctx.set_theme(egui::ThemePreference::System),
        1 => ctx.set_theme(egui::ThemePreference::Light),
        2 => ctx.set_theme(egui::ThemePreference::Dark),
        3..=6 => {
            ctx.set_visuals_of(egui::Theme::Light, palette_visuals(table_settings, false));
            ctx.set_theme(egui::ThemePreference::Light);
        }
        _ => match table_settings.custom_theme_base_id() {
            0 => ctx.set_theme(egui::ThemePreference::System),
            1 => ctx.set_theme(egui::ThemePreference::Light),
            2 => ctx.set_theme(egui::ThemePreference::Dark),
            3..=6 => {
                ctx.set_visuals_of(egui::Theme::Light, palette_visuals(table_settings, false));
                ctx.set_theme(egui::ThemePreference::Light);
            }
            _ => ctx.set_theme(table_settings.theme_preference),
        },
    }
    apply_background_visuals(ctx, table_settings);
}

fn restore_default_theme_visuals(ctx: &egui::Context) {
    ctx.set_visuals_of(egui::Theme::Light, egui::Visuals::light());
    ctx.set_visuals_of(egui::Theme::Dark, egui::Visuals::dark());
}

fn apply_background_visuals(ctx: &egui::Context, table_settings: &TableSettings) {
    let background = table_settings.cell_background_color;
    let header_background = table_settings.column_header_background_color;
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.style_mut_of(theme, |style| {
            style.visuals.window_fill = background;
            style.visuals.panel_fill = background;
            style.visuals.widgets.noninteractive.bg_fill = background;
            style.visuals.widgets.noninteractive.weak_bg_fill = background;
            style.visuals.text_edit_bg_color = Some(header_background);
            style.visuals.extreme_bg_color = header_background;
        });
    }
}

fn palette_visuals(table_settings: &TableSettings, dark_mode: bool) -> egui::Visuals {
    let mut visuals = if dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let background = table_settings.cell_background_color;
    let header = table_settings.column_header_background_color;
    let selection = table_settings.selection_color;
    let hover = table_settings.hover_color;
    let text = if is_dark_color(background) {
        egui::Color32::from_rgb(232, 236, 242)
    } else {
        egui::Color32::from_rgb(58, 64, 72)
    };

    visuals.override_text_color = Some(text);
    visuals.panel_fill = background;
    visuals.window_fill = background;
    visuals.faint_bg_color = mix_colors(background, header, 0.35);
    visuals.extreme_bg_color = header;
    visuals.code_bg_color = mix_colors(background, header, 0.22);
    visuals.hyperlink_color = mix_colors(selection, egui::Color32::from_rgb(48, 88, 148), 0.35);
    visuals.selection.bg_fill = selection;
    visuals.selection.stroke.color = text;
    visuals.widgets.noninteractive.bg_fill = background;
    visuals.widgets.noninteractive.weak_bg_fill = background;
    visuals.widgets.noninteractive.fg_stroke.color = text;
    visuals.widgets.inactive.weak_bg_fill = header;
    visuals.widgets.inactive.bg_fill = header;
    visuals.widgets.inactive.bg_stroke.color = header;
    visuals.widgets.inactive.fg_stroke.color = text;
    visuals.widgets.hovered.weak_bg_fill = mix_colors(header, hover, 0.6);
    visuals.widgets.hovered.bg_fill = mix_colors(header, hover, 0.6);
    visuals.widgets.hovered.bg_stroke.color = text;
    visuals.widgets.hovered.fg_stroke.color = text;
    visuals.widgets.active.weak_bg_fill = mix_colors(header, selection, 0.5);
    visuals.widgets.active.bg_fill = mix_colors(header, selection, 0.5);
    visuals.widgets.active.bg_stroke.color = text;
    visuals.widgets.active.fg_stroke.color = text;
    visuals.widgets.open.weak_bg_fill = mix_colors(header, selection, 0.3);
    visuals.widgets.open.bg_fill = mix_colors(header, selection, 0.3);
    visuals.widgets.open.bg_stroke.color = text;
    visuals.widgets.open.fg_stroke.color = text;
    visuals
}

fn is_dark_color(color: egui::Color32) -> bool {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    luminance < 128.0
}

fn mix_colors(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let [ar, ag, ab, aa] = a.to_srgba_unmultiplied();
    let [br, bg, bb, ba] = b.to_srgba_unmultiplied();
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(lerp(ar, br), lerp(ag, bg), lerp(ab, bb), lerp(aa, ba))
}
