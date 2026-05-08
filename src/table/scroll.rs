use super::selection::{CellIndex, SelectionRect, clamp_cell_to_sheet};
use super::{TableViewState, VerticalScrollDirection};
use crate::settings::table::TableSettings;
use eframe::egui::{Pos2, Rect, Vec2, scroll_area::State as ScrollState, style::ScrollStyle};
use sheet::Sheet;

impl TableViewState {
    pub fn scroll_to_cell(&mut self, col: usize, row: usize, sheet: &Sheet) {
        if let Some(cell) = clamp_cell_to_sheet(CellIndex { col, row }, sheet) {
            self.pending_scroll_top_row = None;
            self.pending_scroll_to_cell = Some(cell);
            self.preserve_horizontal_scroll_for_pending_cell = false;
            self.scroll_selection_into_view = false;
            self.pending_vertical_scroll_direction = None;
            self.suppress_vertical_scroll_adjustment = false;
        }
    }

    pub fn scroll_to_row(&mut self, row: usize, sheet: &Sheet) {
        let col = self
            .selection
            .map(SelectionRect::normalized)
            .map(|selection| selection.start.col)
            .unwrap_or(0);
        self.scroll_to_cell(col, row, sheet);
    }

    pub fn scroll_to_row_preserving_horizontal(&mut self, row: usize, sheet: &Sheet) {
        let col = self
            .selection
            .map(SelectionRect::normalized)
            .map(|selection| selection.start.col)
            .unwrap_or(0);
        if let Some(cell) = clamp_cell_to_sheet(CellIndex { col, row }, sheet) {
            self.pending_scroll_top_row = None;
            self.pending_scroll_to_cell = Some(cell);
            self.preserve_horizontal_scroll_for_pending_cell = true;
            self.scroll_selection_into_view = false;
            self.pending_vertical_scroll_direction = None;
            self.suppress_vertical_scroll_adjustment = false;
        }
    }

    pub fn scroll_to_top_row_preserving_horizontal(&mut self, row: usize, sheet: &Sheet) {
        if sheet.row_count() == 0 {
            return;
        }

        self.pending_scroll_top_row = Some(row.min(sheet.row_count() - 1));
        self.pending_scroll_to_cell = None;
        self.scroll_selection_into_view = false;
        self.pending_vertical_scroll_direction = None;
        self.suppress_vertical_scroll_adjustment = false;
    }

    pub fn scroll_to_top(&mut self, sheet: &Sheet) -> bool {
        if sheet.row_count() == 0 {
            return false;
        }

        self.scroll_to_row_preserving_horizontal(0, sheet);
        true
    }

    pub fn scroll_to_bottom(&mut self, sheet: &Sheet) -> bool {
        let Some(last_row) = sheet.row_count().checked_sub(1) else {
            return false;
        };

        self.scroll_to_row_preserving_horizontal(last_row, sheet);
        true
    }

    pub fn scroll_by_rows_preserving_horizontal(
        &mut self,
        delta_rows: isize,
        sheet: &Sheet,
        settings: &TableSettings,
    ) -> bool {
        if sheet.row_count() == 0 || delta_rows == 0 {
            return false;
        }

        let cell_height = settings.default_row_height * settings.cell_scale();
        let current_top_row = current_top_row_from_scroll(self.vertical_scroll, cell_height);
        let last_row = sheet.row_count().saturating_sub(1);
        let next_row = current_top_row
            .saturating_add_signed(delta_rows)
            .min(last_row);

        self.scroll_to_top_row_preserving_horizontal(next_row, sheet);
        true
    }
}

fn current_top_row_from_scroll(vertical_scroll: f32, cell_height: f32) -> usize {
    if cell_height <= 0.0 {
        return 0;
    }

    (vertical_scroll / cell_height).floor().max(0.0) as usize
}

pub(super) fn apply_pending_scroll_request(
    state: &mut ScrollState,
    viewport_rect: Rect,
    content_size: Vec2,
    vertical_scroll_direction: Option<VerticalScrollDirection>,
    suppress_vertical_scroll_adjustment: bool,
    previous_selection: Option<SelectionRect>,
    selection: Option<SelectionRect>,
    pending_scroll_top_row: Option<usize>,
    pending_scroll_to_cell: Option<CellIndex>,
    preserve_horizontal_scroll_for_pending_cell: bool,
    preserve_horizontal_scroll_for_selection: bool,
    table: &TableViewState,
    settings: &TableSettings,
    cell_scale: f32,
    cell_height: f32,
) -> bool {
    if let Some(row) = pending_scroll_top_row {
        let viewport_height = viewport_rect.height();
        let max_offset_y = (content_size.y - viewport_height).max(0.0);
        state.offset.y = snap_vertical_offset_to_row_boundary(
            row as f32 * cell_height,
            max_offset_y,
            cell_height,
        );
        return true;
    }

    if let Some(cell) = pending_scroll_to_cell {
        ensure_cell_visible(
            state,
            viewport_rect,
            content_size,
            cell,
            preserve_horizontal_scroll_for_pending_cell,
            table,
            settings,
            cell_scale,
            cell_height,
        );
        return true;
    }

    let Some(selection) = selection.map(SelectionRect::normalized) else {
        return false;
    };

    let selection_left = table.column_left(selection.start.col, cell_scale, settings);
    let selection_right = table.column_left(selection.end.col, cell_scale, settings)
        + table.column_width(selection.end.col, cell_scale, settings);
    let selection_top = selection.start.row as f32 * cell_height;
    let selection_bottom = (selection.end.row + 1) as f32 * cell_height;

    let viewport_width = viewport_rect.width();
    let viewport_height = viewport_rect.height();
    let max_offset_x = (content_size.x - viewport_width).max(0.0);
    let max_offset_y = (content_size.y - viewport_height).max(0.0);

    if !preserve_horizontal_scroll_for_selection {
        if selection_left < state.offset.x {
            state.offset.x = selection_left;
        } else if selection_right > state.offset.x + viewport_width {
            state.offset.x = selection_right - viewport_width;
        }
    }

    if !suppress_vertical_scroll_adjustment {
        let max_anchor_y = (viewport_height - cell_height).max(0.0);
        let up_anchor_y = (viewport_height * settings.up_scroll_trigger_ratio()).min(max_anchor_y);
        let down_anchor_y =
            (viewport_height * settings.down_scroll_trigger_ratio()).min(max_anchor_y);
        let up_trigger_y = state.offset.y + up_anchor_y;
        let down_trigger_y = state.offset.y + down_anchor_y;
        let inferred_direction = previous_selection
            .map(|selection| selection.normalized().start.row)
            .and_then(|previous_start_row| {
                if previous_start_row < selection.start.row {
                    Some(VerticalScrollDirection::Down)
                } else if previous_start_row > selection.start.row {
                    Some(VerticalScrollDirection::Up)
                } else {
                    None
                }
            });

        match vertical_scroll_direction.or(inferred_direction) {
            Some(VerticalScrollDirection::Down) => {
                if selection_top > down_trigger_y {
                    state.offset.y = selection_top - down_anchor_y;
                }
            }
            Some(VerticalScrollDirection::Up) => {
                if selection_top < up_trigger_y {
                    state.offset.y = selection_top - up_anchor_y;
                }
            }
            None => {
                if selection_top < state.offset.y {
                    state.offset.y = selection_top;
                } else if selection_bottom > state.offset.y + viewport_height {
                    state.offset.y = selection_bottom - viewport_height;
                }
            }
        }
    }

    state.offset.x = state.offset.x.clamp(0.0, max_offset_x);
    state.offset.y =
        snap_vertical_offset_to_row_boundary(state.offset.y, max_offset_y, cell_height);
    true
}

pub(super) fn auto_scroll_delta(
    pointer_pos: Pos2,
    viewport_rect: Rect,
    content_size: Vec2,
    state: ScrollState,
) -> Vec2 {
    const EDGE_MARGIN: f32 = 24.0;
    const SPEED: f32 = 0.35;

    let max_offset_x = (content_size.x - viewport_rect.width()).max(0.0);
    let max_offset_y = (content_size.y - viewport_rect.height()).max(0.0);

    let horizontal_push = if pointer_pos.x < viewport_rect.min.x {
        pointer_pos.x - viewport_rect.min.x
    } else if pointer_pos.x > viewport_rect.max.x {
        pointer_pos.x - viewport_rect.max.x
    } else if pointer_pos.x < viewport_rect.min.x + EDGE_MARGIN {
        pointer_pos.x - (viewport_rect.min.x + EDGE_MARGIN)
    } else if pointer_pos.x > viewport_rect.max.x - EDGE_MARGIN {
        pointer_pos.x - (viewport_rect.max.x - EDGE_MARGIN)
    } else {
        0.0
    };

    let vertical_push = if pointer_pos.y < viewport_rect.min.y {
        pointer_pos.y - viewport_rect.min.y
    } else if pointer_pos.y > viewport_rect.max.y {
        pointer_pos.y - viewport_rect.max.y
    } else if pointer_pos.y < viewport_rect.min.y + EDGE_MARGIN {
        pointer_pos.y - (viewport_rect.min.y + EDGE_MARGIN)
    } else if pointer_pos.y > viewport_rect.max.y - EDGE_MARGIN {
        pointer_pos.y - (viewport_rect.max.y - EDGE_MARGIN)
    } else {
        0.0
    };

    let next_x = (state.offset.x + horizontal_push * SPEED).clamp(0.0, max_offset_x);
    let next_y = (state.offset.y + vertical_push * SPEED).clamp(0.0, max_offset_y);

    Vec2::new(next_x - state.offset.x, next_y - state.offset.y)
}

pub(super) fn clamp_pos(pos: Pos2, rect: Rect) -> Pos2 {
    Pos2::new(
        pos.x.clamp(rect.min.x, rect.max.x - 1.0),
        pos.y.clamp(rect.min.y, rect.max.y - 1.0),
    )
}

pub(super) fn is_pointer_over_scrollbar(
    pointer_pos: Pos2,
    viewport_rect: Rect,
    content_size: Vec2,
    scroll_style: ScrollStyle,
) -> bool {
    if !viewport_rect.contains(pointer_pos) {
        return false;
    }

    let scrollbar_band =
        scroll_style.bar_width + scroll_style.bar_inner_margin + scroll_style.bar_outer_margin;
    let has_horizontal_scrollbar = content_size.x > viewport_rect.width();
    let has_vertical_scrollbar = content_size.y > viewport_rect.height();

    (has_vertical_scrollbar && pointer_pos.x >= viewport_rect.max.x - scrollbar_band)
        || (has_horizontal_scrollbar && pointer_pos.y >= viewport_rect.max.y - scrollbar_band)
}

fn ensure_cell_visible(
    state: &mut ScrollState,
    viewport_rect: Rect,
    content_size: Vec2,
    cell: CellIndex,
    preserve_horizontal_scroll: bool,
    table: &TableViewState,
    settings: &TableSettings,
    cell_scale: f32,
    cell_height: f32,
) {
    let cell_left = table.column_left(cell.col, cell_scale, settings);
    let cell_right = cell_left + table.column_width(cell.col, cell_scale, settings);
    let cell_top = cell.row as f32 * cell_height;
    let cell_bottom = (cell.row + 1) as f32 * cell_height;

    let viewport_width = viewport_rect.width();
    let viewport_height = viewport_rect.height();
    let max_offset_x = (content_size.x - viewport_width).max(0.0);
    let max_offset_y = (content_size.y - viewport_height).max(0.0);

    if !preserve_horizontal_scroll {
        if cell_left < state.offset.x {
            state.offset.x = cell_left;
        } else if cell_right > state.offset.x + viewport_width {
            state.offset.x = cell_right - viewport_width;
        }
    }

    if cell_top < state.offset.y {
        state.offset.y = cell_top;
    } else if cell_bottom > state.offset.y + viewport_height {
        state.offset.y = cell_bottom - viewport_height;
    }

    state.offset.x = state.offset.x.clamp(0.0, max_offset_x);
    state.offset.y =
        snap_vertical_offset_to_row_boundary(state.offset.y, max_offset_y, cell_height);
}

fn snap_vertical_offset_to_row_boundary(offset_y: f32, max_offset_y: f32, cell_height: f32) -> f32 {
    if cell_height <= 0.0 {
        return offset_y.clamp(0.0, max_offset_y);
    }

    let clamped = offset_y.clamp(0.0, max_offset_y);
    if (max_offset_y - clamped).abs() <= 0.5 {
        return max_offset_y;
    }

    let snapped = (clamped / cell_height).round() * cell_height;
    snapped.clamp(0.0, max_offset_y)
}
