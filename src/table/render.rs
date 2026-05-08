use super::selection::{CellIndex, SelectionRect};
use super::{TableViewState, VisibleRange};
use crate::display::{
    SecondDividerKind, is_odd_second_band_for_sheet, row_header_labels_for_sheet,
    second_divider_kind_for_sheet,
};
use crate::settings::table::{
    AlternateColumnMode, ContinuationLineStyle, FrameHeaderMode, HeaderDisplayDensity,
    SegmentHeaderMode, TableSettings,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use sheet::{CellValue, Sheet};
use std::ops::Range;

pub(super) fn draw_corner_header(painter: &egui::Painter, corner_rect: Rect, header_fill: Color32) {
    let border = Stroke::new(0.5, Color32::from_gray(150));

    painter.rect_filled(corner_rect, 0.0, header_fill);
    painter.rect_stroke(corner_rect, 0.0, border, StrokeKind::Inside);
}

pub(super) fn draw_column_headers(
    painter: &egui::Painter,
    header_area: Rect,
    cell_scale: f32,
    font_size: f32,
    header_height: f32,
    sheet: &Sheet,
    selection: Option<SelectionRect>,
    hovered_col: Option<usize>,
    active_cols: Option<(usize, usize)>,
    header_fill: Color32,
    selected_fill: Color32,
    hover_fill: Color32,
    flashing_col: Option<(usize, f64, Color32)>,
    current_time: f64,
    table: &TableViewState,
    settings: &TableSettings,
    visible_cols: Range<usize>,
) {
    let painter = painter.with_clip_rect(header_area);
    let border = Stroke::new(0.5, Color32::from_gray(150));
    let selected_cols = selection
        .map(SelectionRect::normalized)
        .filter(|selection| {
            sheet.row_count() > 0
                && selection.start.row == 0
                && selection.end.row + 1 == sheet.row_count()
        });

    for col in visible_cols {
        let width = table.column_width(col, cell_scale, settings);
        let rect = Rect::from_min_size(
            Pos2::new(
                header_area.min.x + table.column_left(col, cell_scale, settings)
                    - table.horizontal_scroll,
                header_area.min.y,
            ),
            Vec2::new(width, header_height),
        );
        if rect.intersects(header_area) {
            let base_fill = if selected_cols
                .is_some_and(|selection| selection.start.col <= col && col <= selection.end.col)
            {
                selected_fill
            } else if active_cols.is_some_and(|(start, end)| start <= col && col <= end) {
                selected_fill
            } else if hovered_col == Some(col) {
                hover_fill
            } else {
                header_fill
            };
            let fill = if let Some((flash_col, flash_start, flash_color)) = flashing_col {
                if flash_col == col {
                    const FLASH_HOLD_DURATION: f64 = 0.1;
                    const FLASH_FADE_DURATION: f64 = 0.4;
                    let elapsed = (current_time - flash_start).max(0.0);
                    let t = if elapsed <= FLASH_HOLD_DURATION {
                        0.0
                    } else {
                        ((elapsed - FLASH_HOLD_DURATION) / FLASH_FADE_DURATION).clamp(0.0, 1.0)
                            as f32
                    };
                    Color32::from_rgba_unmultiplied(
                        lerp_u8(flash_color.r(), base_fill.r(), t),
                        lerp_u8(flash_color.g(), base_fill.g(), t),
                        lerp_u8(flash_color.b(), base_fill.b(), t),
                        255,
                    )
                } else {
                    base_fill
                }
            } else {
                base_fill
            };
            let text_color = foreground_for_background(fill);
            painter.rect_filled(rect, 0.0, fill);
            painter.rect_stroke(rect, 0.0, border, StrokeKind::Inside);
            let text_clip_rect = rect
                .shrink2(Vec2::new(4.0 * cell_scale, 0.0))
                .intersect(header_area);
            let title_center = Pos2::new(rect.center().x, rect.min.y + header_height * 0.34);
            let title_font = FontId::proportional(scaled_font_size(font_size, cell_scale));
            let (header_text, _) = ellipsize_text_to_width(
                &painter,
                sheet.column_name(col),
                &title_font,
                text_clip_rect.width(),
            );
            painter.with_clip_rect(text_clip_rect).text(
                title_center,
                Align2::CENTER_CENTER,
                header_text,
                title_font,
                text_color,
            );
            if settings.show_header_ghosts() {
                if let Some(ghost_text) = header_ghost_text(sheet, col, table, settings) {
                    let ghost_pos = Pos2::new(rect.center().x, rect.min.y + header_height * 0.93);
                    painter.with_clip_rect(text_clip_rect).text(
                        ghost_pos,
                        Align2::CENTER_BOTTOM,
                        ghost_text,
                        FontId::proportional(scaled_font_size(font_size * 0.58, cell_scale)),
                        Color32::from_rgba_unmultiplied(
                            text_color.r(),
                            text_color.g(),
                            text_color.b(),
                            150,
                        ),
                    );
                }
            }
        }
    }
}

pub(super) fn ellipsize_text_to_width(
    painter: &egui::Painter,
    text: &str,
    font_id: &FontId,
    max_width: f32,
) -> (String, bool) {
    if text.is_empty() || max_width <= 0.0 {
        return (String::new(), !text.is_empty());
    }

    let text_width = painter
        .layout_no_wrap(text.to_owned(), font_id.clone(), Color32::WHITE)
        .size()
        .x;
    if text_width <= max_width {
        return (text.to_owned(), false);
    }

    let ellipsis = "...";
    let ellipsis_width = painter
        .layout_no_wrap(ellipsis.to_owned(), font_id.clone(), Color32::WHITE)
        .size()
        .x;
    if ellipsis_width > max_width {
        return (String::new(), true);
    }

    let mut truncated = text.to_owned();
    while !truncated.is_empty() {
        truncated.pop();
        let candidate = format!("{truncated}{ellipsis}");
        let candidate_width = painter
            .layout_no_wrap(candidate.clone(), font_id.clone(), Color32::WHITE)
            .size()
            .x;
        if candidate_width <= max_width {
            return (candidate, true);
        }
    }

    (ellipsis.to_owned(), true)
}

pub(super) fn draw_body_cells(
    painter: &egui::Painter,
    origin: Pos2,
    selection: Option<SelectionRect>,
    hovered_cell: Option<CellIndex>,
    editing_cell: Option<CellIndex>,
    edit_buffer: Option<&str>,
    cell_scale: f32,
    cell_height: f32,
    font_size: f32,
    alternate_column_mode: &AlternateColumnMode,
    alternate_darken_amount: f32,
    alternate_second_darken_amount: f32,
    alternate_saturation_amount: f32,
    _alternate_column_color: Color32,
    cell_background_color: Color32,
    punched_row_background_color: Color32,
    zero_cell_background_color: Color32,
    use_zero_cell_background_color: bool,
    show_zero_value_markers: bool,
    special_inserted_row_background_color: Color32,
    selection_color: Color32,
    hover_color: Color32,
    copy_on_drop: bool,
    col_limit: usize,
    table: &TableViewState,
    settings: &TableSettings,
    sheet: &Sheet,
    visible: &VisibleRange,
    kara_cell_x_value: Option<i64>,
) {
    let border = Stroke::new(0.5, Color32::from_gray(165));
    let editing_fill = Color32::from_rgb(118, 204, 96);
    let visible_cols = visible.cols.start.min(col_limit)..visible.cols.end.min(col_limit);
    for row in visible.rows.clone() {
        for col in visible_cols.clone() {
            let cell = CellIndex { col, row };
            let rect = Rect::from_min_size(
                Pos2::new(
                    origin.x + table.column_left(col, cell_scale, settings),
                    origin.y + row as f32 * cell_height,
                ),
                Vec2::new(table.column_width(col, cell_scale, settings), cell_height),
            );
            let is_editing = editing_cell == Some(cell);
            let is_selected = selection.is_some_and(|selected| selected.contains(cell));
            let is_hovered = hovered_cell == Some(cell);
            let is_punched = sheet.is_punched_row(row);
            let is_special_inserted = sheet.is_inserted_row(row);
            let is_zero = use_zero_cell_background_color
                && sheet
                    .cell(col, row)
                    .is_some_and(|v| matches!(v, CellValue::Int(0)));
            let base_color = if is_punched {
                punched_row_background_color
            } else if is_special_inserted {
                special_inserted_row_background_color
            } else if is_zero {
                zero_cell_background_color
            } else {
                cell_background_color
            };
            let base_fill = if is_punched {
                base_color
            } else if col % 2 == 1 {
                match alternate_column_mode {
                    AlternateColumnMode::Off => base_color,
                    AlternateColumnMode::Darken | AlternateColumnMode::CustomColor => {
                        adjust_contrast(
                            base_color,
                            alternate_darken_amount,
                            alternate_saturation_amount,
                        )
                    }
                }
            } else {
                base_color
            };
            let base_fill = if !is_punched
                && alternate_second_darken_amount.abs() > f32::EPSILON
                && is_odd_second_band_for_sheet(sheet, row)
            {
                adjust_contrast(base_fill, alternate_second_darken_amount, 0.0)
            } else {
                base_fill
            };
            let fill = if is_punched {
                base_fill
            } else if is_editing {
                editing_fill
            } else if is_selected {
                blend_color(base_fill, selection_color, 0.5)
            } else if is_hovered {
                hover_color
            } else {
                base_fill
            };

            painter.rect_filled(rect, 0.0, fill);
            painter.rect_stroke(rect, 0.0, border, StrokeKind::Inside);
            let is_cross_cell = sheet.is_cross_cell(row, col);
            if is_editing {
                // Keep the editing cell visually clean so the live input is not obscured.
            } else if show_zero_value_markers
                && is_cross_cell
                && sheet.should_display_cell(row, col)
            {
                draw_cross_mark(painter, rect);
            } else if sheet.should_draw_continuation_line(
                row,
                col,
                settings.continuation_line_min_run_length() as usize,
            ) {
                let is_last_row = row + 1 == sheet.row_count();
                if is_cross_cell {
                    if show_zero_value_markers {
                        draw_wavy_continuation_line(painter, rect, row, cell_height, cell_scale);
                        if is_last_row {
                            draw_arrowhead(painter, rect, cell_scale);
                        }
                    }
                } else {
                    let style = settings.continuation_line_style();
                    if is_last_row && style == ContinuationLineStyle::Vertical {
                        draw_arrow_down(painter, rect, cell_scale);
                    } else {
                        draw_continuation_line(painter, rect, style);
                    }
                }
            }
            let text = if editing_cell == Some(cell) {
                edit_buffer
                    .map(str::to_owned)
                    .unwrap_or_else(|| display_cell_text(sheet, row, col, kara_cell_x_value))
            } else {
                display_cell_text(sheet, row, col, kara_cell_x_value)
            };
            let text_clip_rect = rect
                .shrink2(Vec2::new(2.0 * cell_scale, 1.0 * cell_scale))
                .intersect(rect);
            let text_painter = painter.with_clip_rect(text_clip_rect);
            draw_emphasized_cell_text(
                &text_painter,
                rect.center(),
                text,
                FontId::proportional(scaled_font_size(font_size, cell_scale)),
                foreground_for_background(fill),
                cell_scale,
            );
        }
    }

    if let Some((source_selection, target_selection, copy_on_drop)) =
        table.keyframe_drag_preview(copy_on_drop)
    {
        draw_selection_move_preview(
            painter,
            origin,
            cell_scale,
            cell_height,
            font_size,
            cell_background_color,
            selection_color,
            show_zero_value_markers,
            target_selection,
            source_selection,
            copy_on_drop,
            table,
            settings,
            sheet,
            visible,
            col_limit,
        );
    }
}

fn header_ghost_text(
    sheet: &Sheet,
    col: usize,
    table: &TableViewState,
    settings: &TableSettings,
) -> Option<String> {
    let (top_row, partially_clipped) = header_ghost_top_row(table, sheet, settings)?;
    if partially_clipped {
        let visible_text = sheet.display_cell_text(top_row, col);
        if !visible_text.is_empty() {
            return Some(visible_text);
        }
    }
    let current = sheet.cell(col, top_row)?.as_i64();
    if current == 0 || sheet.should_display_cell(top_row, col) {
        return None;
    }
    let source_row = ghost_source_row(sheet, top_row, col)?;
    let text = sheet.display_cell_text(source_row, col);
    (!text.is_empty()).then_some(text)
}

fn header_ghost_top_row(
    table: &TableViewState,
    sheet: &Sheet,
    settings: &TableSettings,
) -> Option<(usize, bool)> {
    if sheet.row_count() == 0 {
        return None;
    }
    let cell_height = settings.default_row_height * settings.cell_scale;
    if cell_height <= 0.0 {
        return None;
    }
    let top_row_float = (table.vertical_scroll / cell_height).max(0.0);
    let top_row = top_row_float.floor() as usize;
    let partially_clipped = (top_row_float.fract()).abs() > f32::EPSILON;
    (top_row < sheet.row_count()).then_some((top_row, partially_clipped))
}

fn ghost_source_row(sheet: &Sheet, top_row: usize, col: usize) -> Option<usize> {
    let current = sheet.cell(col, top_row)?.as_i64();
    (0..top_row).rev().find(|&candidate| {
        sheet.should_display_cell(candidate, col)
            && sheet
                .cell(col, candidate)
                .is_some_and(|value| value.as_i64() == current)
    })
}

fn display_cell_text(
    sheet: &Sheet,
    row: usize,
    col: usize,
    kara_cell_x_value: Option<i64>,
) -> String {
    if kara_cell_x_value.is_some_and(|value| {
        sheet
            .cell(col, row)
            .is_some_and(|cell| cell.as_i64() == value)
            && sheet.should_display_cell(row, col)
    }) {
        "X".to_owned()
    } else {
        sheet.display_cell_text(row, col)
    }
}

pub(super) fn draw_row_headers(
    painter: &egui::Painter,
    area: Rect,
    vertical_scroll: f32,
    cell_height: f32,
    cell_scale: f32,
    font_size: f32,
    header_width: f32,
    sheet: &Sheet,
    selection: Option<SelectionRect>,
    hovered_row: Option<usize>,
    frame_mode: FrameHeaderMode,
    segment_mode: SegmentHeaderMode,
    frames_per_page: usize,
    frame_density: HeaderDisplayDensity,
    segment_density: HeaderDisplayDensity,
    header_fill: Color32,
    selected_fill: Color32,
    hover_fill: Color32,
    visible_rows: Range<usize>,
) {
    let border = Stroke::new(0.5, Color32::from_gray(150));
    let selected_rows = selection
        .map(SelectionRect::normalized)
        .filter(|selection| {
            sheet.column_count() > 0
                && selection.start.col == 0
                && selection.end.col + 1 == sheet.column_count()
        });

    for row in visible_rows {
        let rect = Rect::from_min_size(
            Pos2::new(
                area.min.x,
                area.min.y + row as f32 * cell_height - vertical_scroll,
            ),
            Vec2::new(header_width, cell_height),
        );
        if rect.intersects(area) {
            let fill = if selected_rows
                .is_some_and(|selection| selection.start.row <= row && row <= selection.end.row)
            {
                selected_fill
            } else if hovered_row == Some(row) {
                hover_fill
            } else {
                header_fill
            };
            let text_color = foreground_for_background(fill);
            painter.rect_filled(rect, 0.0, fill);
            painter.rect_stroke(rect, 0.0, border, StrokeKind::Inside);
            let labels = row_header_labels_for_sheet(
                sheet,
                row,
                frames_per_page,
                frame_mode,
                segment_mode,
                frame_density,
                segment_density,
            );
            let text_clip_rect = rect
                .shrink2(Vec2::new(4.0 * cell_scale, 1.0 * cell_scale))
                .intersect(area);
            let text_painter = painter.with_clip_rect(text_clip_rect);
            if let Some(label) = labels.frame_label {
                let frame_text_color = if labels.frame_is_inserted {
                    Color32::from_rgb(48, 104, 220)
                } else if labels.frame_is_alert {
                    Color32::from_rgb(220, 72, 72)
                } else {
                    text_color
                };
                let text_pos =
                    Pos2::new(rect.max.x - 4.0 * cell_scale, rect.max.y - 0.5 * cell_scale);
                text_painter.text(
                    text_pos,
                    Align2::RIGHT_BOTTOM,
                    label,
                    FontId::proportional(scaled_font_size(font_size, cell_scale)),
                    frame_text_color,
                );
            }
            if let Some(label) = labels.segment_label {
                let segment_text_color = if labels.frame_is_inserted {
                    Color32::from_rgb(48, 104, 220)
                } else if labels.frame_is_alert {
                    Color32::from_rgb(220, 72, 72)
                } else {
                    text_color
                };
                let text_pos =
                    Pos2::new(rect.min.x + 4.0 * cell_scale, rect.max.y - 0.5 * cell_scale);
                text_painter.text(
                    text_pos,
                    Align2::LEFT_BOTTOM,
                    label,
                    FontId::proportional(scaled_font_size(font_size, cell_scale)),
                    segment_text_color,
                );
            }
        }
    }
}

pub(super) fn draw_second_dividers(
    painter: &egui::Painter,
    area: Rect,
    vertical_scroll: f32,
    cell_height: f32,
    sheet: &Sheet,
    visible_rows: Range<usize>,
    dark_mode: bool,
) {
    let fps = sheet.fps() as usize;
    if fps == 0 {
        return;
    }

    let (quarter_second_stroke, half_second_stroke, one_second_stroke) = if dark_mode {
        (
            Stroke::new(1.0, Color32::from_gray(170)),
            Stroke::new(2.0, Color32::from_gray(205)),
            Stroke::new(3.0, Color32::from_gray(235)),
        )
    } else {
        (
            Stroke::new(1.0, Color32::from_gray(135)),
            Stroke::new(2.0, Color32::from_gray(110)),
            Stroke::new(3.0, Color32::from_gray(90)),
        )
    };
    for row in visible_rows {
        if let Some(divider_kind) = second_divider_kind_for_sheet(sheet, row) {
            let y = area.min.y + row as f32 * cell_height - vertical_scroll + cell_height;
            if y >= area.min.y && y <= area.max.y {
                let stroke = match divider_kind {
                    SecondDividerKind::Full => one_second_stroke,
                    SecondDividerKind::Half => half_second_stroke,
                    SecondDividerKind::Quarter => quarter_second_stroke,
                };
                painter.line_segment([Pos2::new(area.min.x, y), Pos2::new(area.max.x, y)], stroke);
            }
        }
    }
}

fn draw_cross_mark(painter: &egui::Painter, rect: Rect) {
    let stroke = Stroke::new(1.4, Color32::from_rgb(172, 178, 186));
    let inset = 0.8;

    painter.line_segment(
        [
            Pos2::new(rect.min.x + inset, rect.min.y + inset),
            Pos2::new(rect.max.x - inset, rect.max.y - inset),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(rect.min.x + inset, rect.max.y - inset),
            Pos2::new(rect.max.x - inset, rect.min.y + inset),
        ],
        stroke,
    );
}

fn draw_continuation_line(painter: &egui::Painter, rect: Rect, style: ContinuationLineStyle) {
    let stroke = Stroke::new(1.4, Color32::from_rgb(172, 178, 186));
    match style {
        ContinuationLineStyle::Vertical => {
            let inset = 1.0;
            let x = rect.center().x;

            painter.line_segment(
                [
                    Pos2::new(x, rect.min.y + inset),
                    Pos2::new(x, rect.max.y - inset),
                ],
                stroke,
            );
        }
        ContinuationLineStyle::Horizontal => {
            let inset = (rect.width() * 0.44).clamp(14.0, rect.width() * 0.495);
            let y = rect.center().y;

            painter.line_segment(
                [
                    Pos2::new(rect.min.x + inset, y),
                    Pos2::new(rect.max.x - inset, y),
                ],
                stroke,
            );
        }
    }
}

fn draw_arrowhead(painter: &egui::Painter, rect: Rect, cell_scale: f32) {
    let color = Color32::from_rgb(172, 178, 186);
    let inset = 1.0;
    let x = rect.center().x;
    let bottom = rect.max.y - inset;
    let head_half = (7.0 * cell_scale).clamp(5.0, 13.0);
    let head_height = head_half * 1.2;

    let triangle = egui::Shape::convex_polygon(
        vec![
            Pos2::new(x - head_half, bottom - head_height),
            Pos2::new(x + head_half, bottom - head_height),
            Pos2::new(x, bottom),
        ],
        color,
        Stroke::NONE,
    );
    painter.add(triangle);
}

fn draw_arrow_down(painter: &egui::Painter, rect: Rect, cell_scale: f32) {
    let stroke = Stroke::new(1.4, Color32::from_rgb(172, 178, 186));
    let inset = 1.0;
    let x = rect.center().x;
    let top = rect.min.y + inset;
    let bottom = rect.max.y - inset;
    let head_half = (7.0 * cell_scale).clamp(5.0, 13.0);
    let head_height = head_half * 1.2;

    painter.line_segment(
        [Pos2::new(x, top), Pos2::new(x, bottom - head_height)],
        stroke,
    );
    draw_arrowhead(painter, rect, cell_scale);
}

fn draw_wavy_continuation_line(
    painter: &egui::Painter,
    rect: Rect,
    row: usize,
    cell_height: f32,
    _cell_scale: f32,
) {
    let stroke = Stroke::new(1.4, Color32::from_rgb(172, 178, 186));
    // Keep the wave proportions tied to the cell height so zooming preserves the pattern.
    let inset = (cell_height * 0.05).max(0.8);
    let amplitude = (cell_height * 0.12).max(1.8);
    let wavelength = (cell_height * 0.9).max(8.0);
    let sample_step = (wavelength / 10.0).max(0.8);
    let center_x = rect.center().x;
    let top = rect.min.y + inset;
    let bottom = rect.max.y - inset;
    let global_top = row as f32 * cell_height + inset;

    let mut points = Vec::new();
    let mut y = top;
    while y <= bottom {
        let global_y = global_top + (y - top);
        let phase = (global_y / wavelength) * std::f32::consts::TAU;
        let x = center_x + amplitude * phase.sin();
        points.push(Pos2::new(x, y));
        y += sample_step;
    }
    if points.last().is_none_or(|point| point.y < bottom) {
        let global_y = global_top + (bottom - top);
        let phase = (global_y / wavelength) * std::f32::consts::TAU;
        let x = center_x + amplitude * phase.sin();
        points.push(Pos2::new(x, bottom));
    }

    painter.add(egui::Shape::line(points, stroke));
}

fn draw_selection_move_preview(
    painter: &egui::Painter,
    origin: Pos2,
    cell_scale: f32,
    cell_height: f32,
    font_size: f32,
    cell_background_color: Color32,
    selection_color: Color32,
    show_zero_value_markers: bool,
    target_selection: SelectionRect,
    source_selection: SelectionRect,
    copy_on_drop: bool,
    table: &TableViewState,
    settings: &TableSettings,
    sheet: &Sheet,
    visible: &VisibleRange,
    col_limit: usize,
) {
    let target_selection = target_selection.normalized();
    let source_selection = source_selection.normalized();
    let move_green = Color32::from_rgb(28, 138, 68);
    let copy_red = Color32::from_rgb(156, 24, 24);
    let preview_tint = if copy_on_drop { copy_red } else { move_green };
    let border = Stroke::new(1.0, Color32::from_gray(120));
    let visible_cols = visible.cols.start.min(col_limit)..visible.cols.end.min(col_limit);

    for row in visible.rows.clone() {
        if row < target_selection.start.row || row > target_selection.end.row {
            continue;
        }
        for col in visible_cols.clone() {
            if col < target_selection.start.col || col > target_selection.end.col {
                continue;
            }

            let source_row = source_selection.start.row + (row - target_selection.start.row);
            let source_col = source_selection.start.col + (col - target_selection.start.col);
            let rect = Rect::from_min_size(
                Pos2::new(
                    origin.x + table.column_left(col, cell_scale, settings),
                    origin.y + row as f32 * cell_height,
                ),
                Vec2::new(table.column_width(col, cell_scale, settings), cell_height),
            );
            let selection_fill = blend_color(cell_background_color, selection_color, 0.8);
            let fill = blend_color(selection_fill, preview_tint, 0.35);

            painter.rect_filled(rect, 0.0, fill);

            let is_cross_cell = sheet.is_cross_cell(source_row, source_col);
            if show_zero_value_markers
                && is_cross_cell
                && sheet.should_display_cell(source_row, source_col)
            {
                draw_cross_mark(painter, rect);
            } else if sheet.should_draw_continuation_line(
                source_row,
                source_col,
                settings.continuation_line_min_run_length() as usize,
            ) {
                let is_last_row = source_row + 1 == sheet.row_count();
                if is_cross_cell {
                    if show_zero_value_markers {
                        draw_wavy_continuation_line(painter, rect, row, cell_height, cell_scale);
                        if is_last_row {
                            draw_arrowhead(painter, rect, cell_scale);
                        }
                    }
                } else {
                    let style = settings.continuation_line_style();
                    if is_last_row && style == ContinuationLineStyle::Vertical {
                        draw_arrow_down(painter, rect, cell_scale);
                    } else {
                        draw_continuation_line(painter, rect, style);
                    }
                }
            }

            let text = sheet.display_cell_text(source_row, source_col);
            let text_clip_rect = rect
                .shrink2(Vec2::new(2.0 * cell_scale, 1.0 * cell_scale))
                .intersect(rect);
            let text_painter = painter.with_clip_rect(text_clip_rect);
            draw_emphasized_cell_text(
                &text_painter,
                rect.center(),
                text,
                FontId::proportional(scaled_font_size(font_size, cell_scale)),
                foreground_for_background(fill),
                cell_scale,
            );
        }
    }

    let outer_rect = Rect::from_min_max(
        Pos2::new(
            origin.x + table.column_left(target_selection.start.col, cell_scale, settings),
            origin.y + target_selection.start.row as f32 * cell_height,
        ),
        Pos2::new(
            origin.x
                + table.column_left(target_selection.end.col, cell_scale, settings)
                + table.column_width(target_selection.end.col, cell_scale, settings),
            origin.y + (target_selection.end.row + 1) as f32 * cell_height,
        ),
    );
    painter.rect_stroke(outer_rect, 0.0, border, StrokeKind::Outside);
}

fn draw_emphasized_cell_text(
    painter: &egui::Painter,
    center: Pos2,
    text: String,
    font_id: FontId,
    color: Color32,
    cell_scale: f32,
) {
    let offset = (0.35 * cell_scale).clamp(0.2, 0.7);
    let vertical_offset = (1.2 * cell_scale).clamp(1.0, 2.0);
    painter.text(
        Pos2::new(center.x - offset, center.y + vertical_offset),
        Align2::CENTER_CENTER,
        text.clone(),
        font_id.clone(),
        color,
    );
    painter.text(
        Pos2::new(center.x + offset, center.y + vertical_offset),
        Align2::CENTER_CENTER,
        text,
        font_id,
        color,
    );
}

pub(super) fn adjust_contrast(color: Color32, brightness: f32, saturation: f32) -> Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let mut hsv = egui::ecolor::Hsva::from_srgb([r, g, b]);
    if brightness >= 0.0 {
        hsv.v = (hsv.v * (1.0 - brightness)).clamp(0.0, 1.0);
    } else {
        let t = -brightness;
        hsv.v = (hsv.v + (1.0 - hsv.v) * t).clamp(0.0, 1.0);
    }
    if saturation >= 0.0 {
        hsv.s = (hsv.s * (1.0 + saturation)).clamp(0.0, 1.0);
    } else {
        let t = -saturation;
        hsv.s = (hsv.s * (1.0 - t)).clamp(0.0, 1.0);
    }
    let [nr, ng, nb] = hsv.to_srgb();
    Color32::from_rgba_unmultiplied(nr, ng, nb, a)
}

fn blend_color(base: Color32, overlay: Color32, amount: f32) -> Color32 {
    let t = amount.clamp(0.0, 1.0);
    let [br, bg, bb, ba] = base.to_srgba_unmultiplied();
    let [or, og, ob, oa] = overlay.to_srgba_unmultiplied();
    let lerp = |from: u8, to: u8| -> u8 {
        ((from as f32) + ((to as f32) - (from as f32)) * t).round() as u8
    };

    Color32::from_rgba_unmultiplied(lerp(br, or), lerp(bg, og), lerp(bb, ob), lerp(ba, oa))
}

fn foreground_for_background(bg: Color32) -> Color32 {
    let [r, g, b, _] = bg.to_srgba_unmultiplied();
    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luminance < 128.0 {
        Color32::from_gray(230)
    } else {
        Color32::from_gray(30)
    }
}

fn scaled_font_size(base_size: f32, cell_scale: f32) -> f32 {
    base_size * cell_scale
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}
