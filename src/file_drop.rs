use eframe::egui::{
    self, Align2, Color32, FontId, Id, LayerId, Order, Pos2, Rect, Stroke, StrokeKind,
};
use neosts::{AppLocale, strings};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDropAction {
    SetImageDirectory(PathBuf),
    OpenSheetFile(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropOverlay {
    pub target_rect: Rect,
    pub title: &'static str,
    pub message: &'static str,
    pub accepts_all: bool,
    pub overlay_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DropTarget {
    ImagePanel,
    SheetTable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragPayloadKind {
    DirectoryOnly,
    SheetFileOnly,
    UnsupportedOrMixed,
}

pub fn resolve_dropped_action(
    files: &[egui::DroppedFile],
    pointer_pos: Option<Pos2>,
    table_rect: Rect,
    image_panel_rect: Option<Rect>,
) -> Option<FileDropAction> {
    let payload_kind = classify_dropped_payload(files);
    let drop_target = resolve_drop_target(pointer_pos, table_rect, image_panel_rect, payload_kind)?;

    match drop_target {
        DropTarget::ImagePanel => files
            .iter()
            .filter_map(|file| file.path.as_ref())
            .find(|path| path.is_dir())
            .cloned()
            .map(FileDropAction::SetImageDirectory),
        DropTarget::SheetTable => files
            .iter()
            .filter_map(|file| file.path.as_ref())
            .find(|path| is_supported_sheet_file(path))
            .cloned()
            .map(FileDropAction::OpenSheetFile),
    }
}

pub fn resolve_hover_overlay(
    files: &[egui::HoveredFile],
    pointer_pos: Option<Pos2>,
    table_rect: Rect,
    image_panel_rect: Option<Rect>,
    locale: AppLocale,
) -> Option<DropOverlay> {
    let payload_kind = classify_hovered_payload(files);
    if payload_kind == DragPayloadKind::UnsupportedOrMixed {
        return None;
    }
    let drop_target = resolve_drop_target(pointer_pos, table_rect, image_panel_rect, payload_kind)?;

    match drop_target {
        DropTarget::ImagePanel => image_panel_rect.map(|target_rect| DropOverlay {
            target_rect,
            title: strings::overlay_drop_folder_title(locale),
            message: strings::overlay_set_image_folder(locale),
            accepts_all: true,
            overlay_id: "image_drop_overlay",
        }),
        DropTarget::SheetTable => Some(DropOverlay {
            target_rect: table_rect,
            title: strings::overlay_drop_to_open_title(locale),
            message: strings::overlay_open_sheet_files(locale),
            accepts_all: true,
            overlay_id: "sheet_drop_overlay",
        }),
    }
}

pub fn draw_drop_overlay(ctx: &egui::Context, overlay: DropOverlay) {
    let tint = if overlay.accepts_all {
        Color32::from_rgba_unmultiplied(70, 120, 140, 120)
    } else {
        Color32::from_rgba_unmultiplied(140, 90, 70, 130)
    };
    let border = if overlay.accepts_all {
        Color32::from_rgb(205, 232, 240)
    } else {
        Color32::from_rgb(250, 214, 190)
    };
    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new(overlay.overlay_id)));

    painter.rect_filled(overlay.target_rect, 0.0, tint);

    let panel_size = egui::vec2(
        (overlay.target_rect.width() - 24.0).clamp(180.0, 320.0),
        110.0,
    );
    let panel_rect = Rect::from_center_size(overlay.target_rect.center(), panel_size);
    painter.rect_filled(
        panel_rect,
        16.0,
        Color32::from_rgba_unmultiplied(20, 24, 28, 230),
    );
    painter.rect_stroke(
        panel_rect,
        16.0,
        Stroke::new(2.0, border),
        StrokeKind::Inside,
    );
    painter.text(
        panel_rect.center_top() + egui::vec2(0.0, 28.0),
        Align2::CENTER_CENTER,
        overlay.title,
        FontId::proportional(24.0),
        Color32::WHITE,
    );
    painter.text(
        panel_rect.center_top() + egui::vec2(0.0, 66.0),
        Align2::CENTER_CENTER,
        overlay.message,
        FontId::proportional(16.0),
        border,
    );
}

fn classify_hovered_payload(files: &[egui::HoveredFile]) -> DragPayloadKind {
    if files
        .iter()
        .all(|file| file.path.as_ref().is_some_and(|path| path.is_dir()))
    {
        DragPayloadKind::DirectoryOnly
    } else if files.iter().all(|file| {
        file.path
            .as_ref()
            .is_some_and(|path| is_supported_sheet_file(path))
    }) {
        DragPayloadKind::SheetFileOnly
    } else {
        DragPayloadKind::UnsupportedOrMixed
    }
}

fn classify_dropped_payload(files: &[egui::DroppedFile]) -> DragPayloadKind {
    if files
        .iter()
        .all(|file| file.path.as_ref().is_some_and(|path| path.is_dir()))
    {
        DragPayloadKind::DirectoryOnly
    } else if files.iter().all(|file| {
        file.path
            .as_ref()
            .is_some_and(|path| is_supported_sheet_file(path))
    }) {
        DragPayloadKind::SheetFileOnly
    } else {
        DragPayloadKind::UnsupportedOrMixed
    }
}

fn resolve_drop_target(
    pointer_pos: Option<Pos2>,
    table_rect: Rect,
    image_panel_rect: Option<Rect>,
    payload_kind: DragPayloadKind,
) -> Option<DropTarget> {
    if let Some(pos) = pointer_pos {
        if image_panel_rect.is_some_and(|rect| rect.contains(pos)) {
            return Some(DropTarget::ImagePanel);
        }
        if table_rect.contains(pos) {
            return Some(DropTarget::SheetTable);
        }
    }

    match payload_kind {
        DragPayloadKind::DirectoryOnly => None,
        DragPayloadKind::SheetFileOnly => Some(DropTarget::SheetTable),
        DragPayloadKind::UnsupportedOrMixed => pointer_pos.and_then(|pos| {
            if image_panel_rect.is_some_and(|rect| rect.contains(pos)) {
                Some(DropTarget::ImagePanel)
            } else if table_rect.contains(pos) {
                Some(DropTarget::SheetTable)
            } else {
                None
            }
        }),
    }
}

fn is_supported_sheet_file(path: &Path) -> bool {
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
