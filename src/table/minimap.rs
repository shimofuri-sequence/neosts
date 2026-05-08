#![allow(dead_code)]

use super::render::adjust_contrast;
use super::{
    AlternateColumnMode, MAX_MINIMAP_HEIGHT, MAX_MINIMAP_WIDTH, MIN_MINIMAP_HEIGHT,
    MIN_MINIMAP_WIDTH, TableViewState,
};
use crate::display::is_odd_second_band_for_sheet;
use crate::settings::table::TableSettings;
use eframe::egui::scroll_area::State as ScrollState;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use sheet::{CellValue, Sheet};

#[derive(Clone, Copy)]
pub(super) struct MinimapOverlayLayout {
    pub(super) outer_rect: Rect,
    pub(super) minimap_rect: Rect,
    pub(super) viewport_rect: Rect,
    pub(super) scale_x: f32,
    pub(super) scale_y: f32,
}

pub(super) fn minimap_resize_handle_rect(minimap: MinimapOverlayLayout) -> Rect {
    let size = 18.0;
    Rect::from_min_size(minimap.outer_rect.min, Vec2::splat(size)).intersect(minimap.outer_rect)
}

pub(super) fn compute_minimap_overlay_layout(
    viewport_rect: Rect,
    content_size: Vec2,
    state: ScrollState,
    target_width: f32,
    target_height: f32,
) -> Option<MinimapOverlayLayout> {
    if viewport_rect.width() < 160.0
        || viewport_rect.height() < 160.0
        || content_size.x <= 0.0
        || content_size.y <= 0.0
    {
        return None;
    }

    let margin = 12.0;
    let padding = 8.0;
    let target_width = target_width.clamp(MIN_MINIMAP_WIDTH, MAX_MINIMAP_WIDTH);
    let target_height = target_height.clamp(MIN_MINIMAP_HEIGHT, MAX_MINIMAP_HEIGHT);
    let outer_size = Vec2::new(target_width, target_height);
    let content_frame_size = Vec2::new(
        (outer_size.x - padding * 2.0).max(1.0),
        (outer_size.y - padding * 2.0).max(1.0),
    );
    let scale_x = content_frame_size.x / content_size.x;
    let scale_y = content_frame_size.y / content_size.y;
    let minimap_size = Vec2::new(content_size.x * scale_x, content_size.y * scale_y);
    let outer_rect = Rect::from_min_size(
        Pos2::new(
            viewport_rect.max.x - outer_size.x - margin,
            viewport_rect.max.y - outer_size.y - margin,
        ),
        outer_size,
    );
    let content_frame = outer_rect.shrink(padding);
    let minimap_rect = Rect::from_min_size(
        Pos2::new(
            content_frame.center().x - minimap_size.x * 0.5,
            content_frame.center().y - minimap_size.y * 0.5,
        ),
        minimap_size,
    );
    let viewport_rect = Rect::from_min_size(
        Pos2::new(
            minimap_rect.min.x + state.offset.x * scale_x,
            minimap_rect.min.y + state.offset.y * scale_y,
        ),
        Vec2::new(
            (viewport_rect.width() * scale_x).min(minimap_rect.width()),
            (viewport_rect.height() * scale_y).min(minimap_rect.height()),
        ),
    )
    .intersect(minimap_rect);

    Some(MinimapOverlayLayout {
        outer_rect,
        minimap_rect,
        viewport_rect,
        scale_x,
        scale_y,
    })
}

pub(super) fn minimap_scroll_offset_for_pointer(
    pointer_pos: Pos2,
    drag_offset: Vec2,
    minimap: MinimapOverlayLayout,
    viewport_rect: Rect,
    content_size: Vec2,
) -> Vec2 {
    let desired_min = Pos2::new(pointer_pos.x - drag_offset.x, pointer_pos.y - drag_offset.y);
    let max_offset_x = (content_size.x - viewport_rect.width()).max(0.0);
    let max_offset_y = (content_size.y - viewport_rect.height()).max(0.0);
    let max_min_x = (minimap.minimap_rect.max.x - minimap.viewport_rect.width())
        .max(minimap.minimap_rect.min.x);
    let max_min_y = (minimap.minimap_rect.max.y - minimap.viewport_rect.height())
        .max(minimap.minimap_rect.min.y);
    let clamped_min = Pos2::new(
        desired_min.x.clamp(minimap.minimap_rect.min.x, max_min_x),
        desired_min.y.clamp(minimap.minimap_rect.min.y, max_min_y),
    );

    Vec2::new(
        ((clamped_min.x - minimap.minimap_rect.min.x) / minimap.scale_x).clamp(0.0, max_offset_x),
        ((clamped_min.y - minimap.minimap_rect.min.y) / minimap.scale_y).clamp(0.0, max_offset_y),
    )
}

pub(super) fn draw_minimap_overlay(
    painter: &egui::Painter,
    minimap: MinimapOverlayLayout,
    cell_scale: f32,
    cell_height: f32,
    alternate_column_mode: &AlternateColumnMode,
    alternate_darken_amount: f32,
    alternate_second_darken_amount: f32,
    alternate_saturation_amount: f32,
    _alternate_column_color: Color32,
    cell_background_color: Color32,
    zero_cell_background_color: Color32,
    use_zero_cell_background_color: bool,
    col_limit: usize,
    table: &TableViewState,
    settings: &TableSettings,
    sheet: &Sheet,
) {
    if col_limit == 0 || sheet.row_count() == 0 {
        return;
    }

    painter.rect_filled(
        minimap.outer_rect,
        6.0,
        Color32::from_rgba_unmultiplied(20, 24, 30, 160),
    );
    painter.rect_filled(
        minimap.minimap_rect,
        4.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 160),
    );

    for col in 0..col_limit {
        let cell_left = minimap.minimap_rect.min.x
            + table.column_left(col, cell_scale, settings) * minimap.scale_x;
        let cell_width = (table.column_width(col, cell_scale, settings) * minimap.scale_x).max(1.0);
        let is_zero_column = use_zero_cell_background_color
            && (0..sheet.row_count()).all(|row| {
                sheet
                    .cell(col, row)
                    .is_some_and(|value| matches!(value, CellValue::Int(n) if *n == 0))
            });
        let base_color = if is_zero_column {
            zero_cell_background_color
        } else {
            cell_background_color
        };
        let base_fill = if col % 2 == 1 {
            match alternate_column_mode {
                AlternateColumnMode::Off => base_color,
                AlternateColumnMode::Darken | AlternateColumnMode::CustomColor => adjust_contrast(
                    base_color,
                    alternate_darken_amount,
                    alternate_saturation_amount,
                ),
            }
        } else {
            base_color
        };

        let mut run_start: Option<usize> = None;
        for row in 0..=sheet.row_count() {
            let is_visible_in_minimap = row < sheet.row_count()
                && sheet
                    .cell(col, row)
                    .is_some_and(|value| matches!(value, CellValue::Int(n) if *n >= 1));
            let row_fill = if row < sheet.row_count()
                && alternate_second_darken_amount.abs() > f32::EPSILON
                && is_odd_second_band_for_sheet(sheet, row)
            {
                adjust_contrast(base_fill, alternate_second_darken_amount, 0.0)
            } else {
                base_fill
            }
            .gamma_multiply(0.8)
            .linear_multiply(0.7);

            match (run_start, is_visible_in_minimap) {
                (None, true) => run_start = Some(row),
                (Some(start), false) => {
                    let block_top =
                        minimap.minimap_rect.min.y + start as f32 * cell_height * minimap.scale_y;
                    let block_height =
                        ((row - start) as f32 * cell_height * minimap.scale_y).max(1.0);
                    let block_rect = Rect::from_min_size(
                        Pos2::new(cell_left, block_top),
                        Vec2::new(cell_width, block_height),
                    )
                    .intersect(minimap.minimap_rect);

                    if block_rect.is_positive() {
                        let start_fill = if alternate_second_darken_amount.abs() > f32::EPSILON
                            && is_odd_second_band_for_sheet(sheet, start)
                        {
                            adjust_contrast(base_fill, alternate_second_darken_amount, 0.0)
                        } else {
                            base_fill
                        }
                        .gamma_multiply(0.8)
                        .linear_multiply(0.7);
                        painter.rect_filled(block_rect, 0.0, start_fill);
                    }

                    run_start = None;
                }
                (Some(start), true)
                    if row < sheet.row_count()
                        && row_fill
                            != if alternate_second_darken_amount.abs() > f32::EPSILON
                                && is_odd_second_band_for_sheet(sheet, start)
                            {
                                adjust_contrast(base_fill, alternate_second_darken_amount, 0.0)
                            } else {
                                base_fill
                            }
                            .gamma_multiply(0.8)
                            .linear_multiply(0.7) =>
                {
                    let block_top =
                        minimap.minimap_rect.min.y + start as f32 * cell_height * minimap.scale_y;
                    let block_height =
                        ((row - start) as f32 * cell_height * minimap.scale_y).max(1.0);
                    let block_rect = Rect::from_min_size(
                        Pos2::new(cell_left, block_top),
                        Vec2::new(cell_width, block_height),
                    )
                    .intersect(minimap.minimap_rect);

                    if block_rect.is_positive() {
                        let start_fill = if alternate_second_darken_amount.abs() > f32::EPSILON
                            && is_odd_second_band_for_sheet(sheet, start)
                        {
                            adjust_contrast(base_fill, alternate_second_darken_amount, 0.0)
                        } else {
                            base_fill
                        }
                        .gamma_multiply(0.8)
                        .linear_multiply(0.7);
                        painter.rect_filled(block_rect, 0.0, start_fill);
                    }

                    run_start = Some(row);
                }
                _ => {}
            }
        }
    }

    if minimap.viewport_rect.is_positive() {
        painter.rect_stroke(
            minimap.viewport_rect,
            3.0,
            Stroke::new(2.0, Color32::from_rgb(220, 48, 48)),
            StrokeKind::Inside,
        );
    }

    let handle_rect = minimap_resize_handle_rect(minimap);
    let handle_color = Color32::from_rgba_unmultiplied(230, 230, 230, 180);
    painter.line_segment(
        [
            Pos2::new(handle_rect.max.x - 12.0, handle_rect.min.y + 4.0),
            Pos2::new(handle_rect.min.x + 4.0, handle_rect.max.y - 12.0),
        ],
        Stroke::new(1.5, handle_color),
    );
    painter.line_segment(
        [
            Pos2::new(handle_rect.max.x - 8.0, handle_rect.min.y + 4.0),
            Pos2::new(handle_rect.min.x + 4.0, handle_rect.max.y - 8.0),
        ],
        Stroke::new(1.5, handle_color),
    );
    painter.line_segment(
        [
            Pos2::new(handle_rect.max.x - 4.0, handle_rect.min.y + 4.0),
            Pos2::new(handle_rect.min.x + 4.0, handle_rect.max.y - 4.0),
        ],
        Stroke::new(1.5, handle_color),
    );
}
