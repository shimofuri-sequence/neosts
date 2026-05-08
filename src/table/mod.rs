mod actions;
mod editing;
mod minimap;
mod render;
mod scroll;
mod selection;

use self::minimap::{
    compute_minimap_overlay_layout, draw_minimap_overlay, minimap_resize_handle_rect,
    minimap_scroll_offset_for_pointer,
};
use self::render::{
    draw_body_cells, draw_column_headers, draw_corner_header, draw_row_headers,
    draw_second_dividers, ellipsize_text_to_width,
};
use self::scroll::{
    apply_pending_scroll_request, auto_scroll_delta, clamp_pos, is_pointer_over_scrollbar,
};
use self::selection::{
    CellIndex, SelectionRect, column_selection, move_selection_block,
    move_selection_rows_with_visible_count, next_timeline_row, previous_timeline_row,
    row_selection, timeline_row_count_in_selection,
};
use crate::column_actions::{ColumnAction, ColumnActionState};
use crate::display::transfer_range_on_sheet;
use crate::row_actions::{RowAction, RowActionState};
use crate::settings::editor::{
    DisplayMode, KeyBinding, KeyBindings, KeybindAction, keybind_action_from_id, keybind_action_id,
};
use crate::settings::table::{
    AlternateColumnMode, MAX_MINIMAP_HEIGHT, MAX_MINIMAP_WIDTH, MIN_MINIMAP_HEIGHT,
    MIN_MINIMAP_WIDTH, TableSettings,
};
use eframe::egui::{
    self, CentralPanel, Margin, Pos2, Rect, ScrollArea, Sense, Vec2,
    scroll_area::{ScrollBarVisibility, State as ScrollState},
};
use sheet::{CellValue, Sheet};
use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragMode {
    Cell,
    SelectionMove,
    RowHeader,
    ColumnHeader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerticalScrollDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MinimapDragMode {
    Scroll(Vec2),
    Resize,
}

#[derive(Clone, Debug)]
struct VisibleRange {
    rows: Range<usize>,
    cols: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingSelectionDrag {
    origin_cell: CellIndex,
    original_selection: SelectionRect,
    moved: bool,
}

#[derive(Clone, Debug)]
pub enum TableViewEvent {
    CopyRequested,
    CutRequested,
    PasteRequested {
        text: String,
    },
    OpenAppendRowsRequested {
        row: usize,
        insert_above: bool,
    },
    OpenRenameColumnRequested {
        column: usize,
        current_name: String,
    },
    ColumnHeaderContextMenuRequested {
        position: Pos2,
        state: TableColumnMenuState,
    },
    RowHeaderContextMenuRequested {
        position: Pos2,
        state: TableRowMenuState,
    },
    ColumnHeaderSecondaryDoubleClicked {
        column_index: usize,
        column_name: String,
        values: Vec<CellValue>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableShortcutResult {
    pub changed: bool,
    pub open_preferences: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableSelection {
    pub start_col: usize,
    pub end_col: usize,
    pub start_row: usize,
    pub end_row: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableEditMenuState {
    pub has_context_row: bool,
    pub selected_row_count: usize,
    pub can_punch: bool,
    pub can_unpunch: bool,
    pub can_append_above: bool,
    pub can_append_below: bool,
    pub can_delete_special: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableColumnMenuState {
    pub context_col: Option<usize>,
    pub target_col: Option<usize>,
    pub current_name: Option<String>,
    pub can_rename: bool,
    pub can_delete: bool,
    pub can_insert_left: bool,
    pub can_insert_right: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableRowMenuState {
    pub context_row: Option<usize>,
    pub target_rows: Vec<usize>,
    pub selected_row_count: usize,
    pub can_punch: bool,
    pub can_unpunch: bool,
    pub can_append_above: bool,
    pub can_append_below: bool,
    pub can_delete_special: bool,
}

pub struct TableViewProps<'a> {
    pub sheet: &'a mut Sheet,
    pub default_frames_per_page: usize,
    pub display_mode: DisplayMode,
    pub keybindings: &'a mut KeyBindings,
    pub settings: &'a mut TableSettings,
    pub modal_open: bool,
    pub context_menu_open: bool,
    pub kara_cell_x_value: Option<i64>,
}

pub struct TableViewState {
    selection: Option<SelectionRect>,
    preserved_selection_row_count: Option<usize>,
    previous_selection: Option<SelectionRect>,
    drag_anchor: Option<CellIndex>,
    drag_mode: Option<DragMode>,
    pending_selection_drag: Option<PendingSelectionDrag>,
    middle_pan_last: Option<Pos2>,
    hovered_cell: Option<CellIndex>,
    hovered_col: Option<usize>,
    hovered_row: Option<usize>,
    active_column_headers: Option<(usize, usize)>,
    edit_buffer: String,
    horizontal_scroll: f32,
    vertical_scroll: f32,
    last_body_viewport_size: Vec2,
    pending_vertical_scroll_direction: Option<VerticalScrollDirection>,
    suppress_vertical_scroll_adjustment: bool,
    pending_scroll_top_row: Option<usize>,
    pending_scroll_to_cell: Option<CellIndex>,
    preserve_horizontal_scroll_for_pending_cell: bool,
    preserve_horizontal_scroll_for_selection: bool,
    external_modal_open: bool,
    show_minimap: bool,
    minimap_hit_rect: Option<Rect>,
    scroll_id_salt: u128,
    scroll_selection_into_view: bool,
    minimap_drag_mode: Option<MinimapDragMode>,
    column_header_context_col: Option<usize>,
    row_header_context_row: Option<usize>,
    pressed_body_cell: Option<CellIndex>,
    pressed_column_header: Option<usize>,
    pressed_row_header: Option<usize>,
    popup_was_open: bool,
    pending_events: Vec<TableViewEvent>,
    capturing_keybind: Option<KeybindAction>,
    column_widths: Vec<f32>,
    flashing_column_header: Option<(usize, f64, egui::Color32)>,
}

impl Default for TableViewState {
    fn default() -> Self {
        Self {
            selection: None,
            preserved_selection_row_count: None,
            previous_selection: None,
            drag_anchor: None,
            drag_mode: None,
            pending_selection_drag: None,
            middle_pan_last: None,
            hovered_cell: None,
            hovered_col: None,
            hovered_row: None,
            active_column_headers: None,
            edit_buffer: String::new(),
            horizontal_scroll: 0.0,
            vertical_scroll: 0.0,
            last_body_viewport_size: Vec2::ZERO,
            pending_vertical_scroll_direction: None,
            suppress_vertical_scroll_adjustment: false,
            pending_scroll_top_row: None,
            pending_scroll_to_cell: None,
            preserve_horizontal_scroll_for_pending_cell: false,
            preserve_horizontal_scroll_for_selection: false,
            external_modal_open: false,
            show_minimap: false,
            minimap_hit_rect: None,
            scroll_id_salt: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0),
            scroll_selection_into_view: false,
            minimap_drag_mode: None,
            column_header_context_col: None,
            row_header_context_row: None,
            pressed_body_cell: None,
            pressed_column_header: None,
            pressed_row_header: None,
            popup_was_open: false,
            pending_events: Vec::new(),
            capturing_keybind: None,
            column_widths: Vec::new(),
            flashing_column_header: None,
        }
    }
}

impl TableViewState {
    pub fn keybind_binding_text(&self, keybindings: &KeyBindings, action: u8) -> String {
        keybindings
            .binding(keybind_action_from_id(action))
            .display_text()
    }

    pub fn begin_keybind_capture(&mut self, action: u8) {
        self.capturing_keybind = Some(keybind_action_from_id(action));
    }

    pub fn cancel_keybind_capture(&mut self) {
        self.capturing_keybind = None;
    }

    pub fn capture_keybind_action_id(&self) -> Option<u8> {
        self.capturing_keybind.map(keybind_action_id)
    }

    pub fn apply_captured_keybind(&mut self, ctx: &egui::Context, keybindings: &mut KeyBindings) {
        let Some(action) = self.capturing_keybind else {
            return;
        };

        let mut captured = None;
        ctx.input(|input| {
            let mut captured_text = None;
            let mut captured_key = None;
            for event in &input.events {
                match event {
                    egui::Event::Text(text) => {
                        if captured_text.is_none() {
                            captured_text = KeyBinding::from_text(text);
                        }
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        if captured_key.is_none() {
                            captured_key = KeyBinding::from_modifiers(*key, *modifiers);
                        }
                    }
                    _ => {}
                }
            }
            captured = captured_text.or(captured_key);
        });

        if let Some(binding) = captured {
            keybindings.set_binding(action, binding);
            self.capturing_keybind = None;
        }
    }

    pub fn reset_keybinds(&mut self, keybindings: &mut KeyBindings) {
        *keybindings = Default::default();
        self.capturing_keybind = None;
    }

    pub fn keybind_value(&self, keybindings: &KeyBindings, action: u8) -> String {
        keybindings
            .binding(keybind_action_from_id(action))
            .storage_value()
    }

    pub fn set_keybind_value(&mut self, keybindings: &mut KeyBindings, action: u8, value: &str) {
        let action = keybind_action_from_id(action);
        if let Some(binding) = KeyBinding::from_storage_value(value) {
            keybindings.set_binding(
                action,
                crate::settings::editor::KeyBindings::migrated_binding(action, binding),
            );
        }
    }

    pub fn take_events(&mut self) -> Vec<TableViewEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn flash_column_header(&mut self, column: usize, started_at: f64, color: egui::Color32) {
        self.flashing_column_header = Some((column, started_at, color));
    }

    fn selected_row_action_state(&self) -> RowActionState {
        row_action_state_from_selection(self.selection)
    }

    pub fn edit_menu_state(&self, sheet: &Sheet) -> TableEditMenuState {
        let action_state = self.selected_row_action_state();
        TableEditMenuState {
            has_context_row: action_state.has_context_row(),
            selected_row_count: action_state.target_rows.len(),
            can_punch: action_state.supports(RowAction::Punch, sheet),
            can_unpunch: action_state.has_punched_rows(sheet),
            can_append_above: action_state.supports(RowAction::AppendAbove, sheet),
            can_append_below: action_state.supports(RowAction::AppendBelow, sheet),
            can_delete_special: action_state.has_special_inserted_rows(sheet),
        }
    }

    pub fn execute_edit_menu_action(&mut self, sheet: &mut Sheet, action: RowAction) {
        let action_state = self.selected_row_action_state();
        self.execute_row_action(sheet, &action_state, action);
    }

    pub fn column_menu_state(&self, sheet: &Sheet) -> TableColumnMenuState {
        let action_state =
            column_action_state_for_context_col(self.selection, self.column_header_context_col);
        let target_col = action_state.target_col;
        TableColumnMenuState {
            context_col: action_state.context_col,
            target_col,
            current_name: target_col.map(|col| sheet.column_name(col).to_owned()),
            can_rename: action_state.supports(ColumnAction::Rename, sheet),
            can_delete: action_state.supports(ColumnAction::Delete, sheet),
            can_insert_left: action_state.supports(ColumnAction::InsertLeft, sheet),
            can_insert_right: action_state.supports(ColumnAction::InsertRight, sheet),
        }
    }

    pub fn row_menu_state(&self, sheet: &Sheet) -> TableRowMenuState {
        let action_state =
            row_action_state_for_context_row(self.selection, self.row_header_context_row);
        TableRowMenuState {
            context_row: action_state.context_row,
            selected_row_count: action_state.target_rows.len(),
            target_rows: action_state.target_rows.clone(),
            can_punch: action_state.supports(RowAction::Punch, sheet),
            can_unpunch: action_state.supports(RowAction::Unpunch, sheet),
            can_append_above: action_state.supports(RowAction::AppendAbove, sheet),
            can_append_below: action_state.supports(RowAction::AppendBelow, sheet),
            can_delete_special: action_state.supports(RowAction::DeleteSpecial, sheet),
        }
    }

    pub fn handle_global_shortcuts(
        &mut self,
        ctx: &egui::Context,
        sheet: &Sheet,
        keybindings: &KeyBindings,
    ) -> TableShortcutResult {
        let mut move_up = false;
        let mut move_down = false;
        let mut move_left = false;
        let mut move_right = false;
        let mut jump_up = false;
        let mut jump_down = false;
        let mut decrease_selection = false;
        let mut increase_selection = false;
        let mut toggle_minimap = false;
        let mut open_preferences = false;

        ctx.input(|input| {
            let decrease_binding = keybindings.decrease_selection;
            let increase_binding = keybindings.increase_selection;
            let fixed_decrease_binding = KeyBinding::new(egui::Key::ArrowUp).with_shift();
            let fixed_increase_binding = KeyBinding::new(egui::Key::ArrowDown).with_shift();
            move_up = keybindings.move_up.matches(input);
            move_down = keybindings.move_down.matches(input);
            move_left = keybindings.move_left.matches(input);
            move_right = keybindings.move_right.matches(input);
            jump_up = keybindings.jump_up.matches(input);
            jump_down = keybindings.jump_down.matches(input);
            decrease_selection =
                decrease_binding.matches(input) || fixed_decrease_binding.matches(input);
            increase_selection =
                increase_binding.matches(input) || fixed_increase_binding.matches(input);
            toggle_minimap = keybindings.toggle_minimap.matches(input);
            open_preferences = keybindings.open_preferences.matches(input);
        });

        let mut changed = false;
        if move_up {
            changed |= self.move_selection_up(sheet);
        } else if move_down {
            changed |= self.move_selection_down(sheet);
        } else if move_left {
            changed |= self.move_selection_left(sheet);
        } else if move_right {
            changed |= self.move_selection_right(sheet);
        } else if jump_up {
            changed |= self.jump_selection_up(sheet);
        } else if jump_down {
            changed |= self.jump_selection_down(sheet);
        } else if decrease_selection {
            changed |= self.decrease_selection(sheet);
        } else if increase_selection {
            changed |= self.increase_selection(sheet);
        } else if toggle_minimap {
            self.toggle_minimap();
            changed = true;
        } else if open_preferences {
            changed = true;
        }

        TableShortcutResult {
            changed,
            open_preferences,
        }
    }

    fn visible_col_count(&self, sheet: &Sheet, _settings: &TableSettings) -> usize {
        sheet.column_count()
    }

    fn column_width(&self, col: usize, cell_scale: f32, settings: &TableSettings) -> f32 {
        self.column_widths
            .get(col)
            .copied()
            .unwrap_or(settings.default_column_width())
            * cell_scale
    }

    fn column_left(&self, col: usize, cell_scale: f32, _settings: &TableSettings) -> f32 {
        self.column_widths
            .iter()
            .take(col)
            .map(|width| width * cell_scale)
            .sum()
    }

    fn col_at_x(
        &self,
        sheet: &Sheet,
        x: f32,
        cell_scale: f32,
        settings: &TableSettings,
    ) -> Option<usize> {
        if x < 0.0 {
            return None;
        }

        let mut left = 0.0;
        for index in 0..sheet.column_count() {
            let right = left + self.column_width(index, cell_scale, settings);
            if x < right {
                return Some(index);
            }
            left = right;
        }

        None
    }

    fn set_selection(&mut self, selection: Option<SelectionRect>) {
        self.previous_selection = self.selection.map(SelectionRect::normalized);
        self.selection = selection;
        if selection.is_none() {
            self.preserved_selection_row_count = None;
        }
        self.active_column_headers = None;
    }

    fn remember_selection_row_count(&mut self, sheet: &Sheet) {
        self.preserved_selection_row_count = self
            .selection
            .map(SelectionRect::normalized)
            .map(|selection| timeline_row_count_in_selection(selection, sheet));
    }

    fn clear_pending_selection_drag(&mut self) {
        self.pending_selection_drag = None;
        if self.drag_mode == Some(DragMode::SelectionMove) {
            self.drag_mode = None;
        }
        if self.drag_anchor.is_some() && self.drag_mode.is_none() {
            self.drag_anchor = None;
        }
    }

    fn keyframe_drag_preview(
        &self,
        copy_on_drop: bool,
    ) -> Option<(SelectionRect, SelectionRect, bool)> {
        let pending_drag = self.pending_selection_drag?;
        let target = self.selection?.normalized();
        (self.drag_mode == Some(DragMode::SelectionMove)
            && target != pending_drag.original_selection)
            .then_some((pending_drag.original_selection, target, copy_on_drop))
    }

    pub fn drag_status_message(&self, copy_on_drop: bool) -> Option<&'static str> {
        self.keyframe_drag_preview(copy_on_drop)
            .map(|(_, _, copy)| {
                if copy {
                    "ドラッグコピー"
                } else {
                    "ドラッグ移動"
                }
            })
    }

    fn set_active_column_headers(&mut self, columns: Option<(usize, usize)>) {
        self.active_column_headers = columns.map(|(start, end)| (start.min(end), start.max(end)));
    }

    fn selection_commands_blocked(&self) -> bool {
        self.external_modal_open
    }

    fn global_shortcuts_blocked(&self) -> bool {
        self.selection_commands_blocked() || self.capturing_keybind.is_some()
    }

    fn can_run_selection_command(&self, sheet: &Sheet) -> bool {
        !self.selection_commands_blocked() && sheet.column_count() > 0 && sheet.row_count() > 0
    }

    fn apply_selection_change(
        &mut self,
        selection: Option<SelectionRect>,
        vertical_scroll_direction: Option<VerticalScrollDirection>,
        suppress_vertical_scroll_adjustment: bool,
        preserve_horizontal_scroll: bool,
        preserve_edit_buffer: bool,
    ) {
        if selection != self.selection {
            self.set_selection(selection);
            self.drag_anchor = None;
            if !preserve_edit_buffer {
                self.edit_buffer.clear();
            }
            self.scroll_selection_into_view = true;
            self.pending_vertical_scroll_direction = vertical_scroll_direction;
            self.suppress_vertical_scroll_adjustment = suppress_vertical_scroll_adjustment;
            self.preserve_horizontal_scroll_for_selection = preserve_horizontal_scroll;
        }
    }

    fn move_selection_vertically(
        &mut self,
        sheet: &Sheet,
        direction: isize,
        vertical_scroll_direction: Option<VerticalScrollDirection>,
    ) -> bool {
        if !self.can_run_selection_command(sheet) {
            return false;
        }

        let next_selection = self.selection.map(|selection| {
            move_selection_rows_with_visible_count(
                selection,
                direction,
                sheet,
                self.preserved_selection_row_count
                    .unwrap_or_else(|| timeline_row_count_in_selection(selection, sheet)),
            )
        });
        if next_selection == self.selection {
            return false;
        }

        let preserve_horizontal_scroll = self
            .selection
            .map(SelectionRect::normalized)
            .is_some_and(|selection| selection.width() == sheet.column_count());
        self.apply_selection_change(
            next_selection,
            vertical_scroll_direction,
            false,
            preserve_horizontal_scroll,
            true,
        );
        true
    }

    fn move_selection_horizontally(&mut self, sheet: &Sheet, direction: isize) -> bool {
        if !self.can_run_selection_command(sheet) {
            return false;
        }

        let next_selection = self.selection.map(|selection| {
            move_selection_block(selection, direction * selection.width() as isize, 0, sheet)
        });
        if next_selection == self.selection {
            return false;
        }

        self.apply_selection_change(next_selection, None, true, false, true);
        true
    }

    pub fn show(&mut self, ui: &mut egui::Ui, props: TableViewProps<'_>) -> Vec<TableViewEvent> {
        let ctx = ui.ctx().clone();
        let TableViewProps {
            sheet,
            default_frames_per_page,
            display_mode,
            keybindings,
            settings,
            modal_open,
            context_menu_open,
            kara_cell_x_value,
        } = props;
        self.external_modal_open = modal_open;
        self.sync_column_widths(sheet, settings);
        self.apply_captured_keybind(&ctx, keybindings);
        self.handle_numeric_input(&ctx, sheet, display_mode, keybindings);
        let cell_height = settings.default_row_height * settings.cell_scale;
        let header_width = settings.default_header_width * settings.cell_scale;
        let header_height = settings.default_header_height * settings.cell_scale;
        let editing_cell = self
            .selection
            .filter(|_| !self.edit_buffer.is_empty())
            .map(|selection| selection.normalized().start);
        let scroll_area_salt = ("table_body", self.scroll_id_salt);
        let current_horizontal_scroll = ScrollState::load(&ctx, egui::Id::new(scroll_area_salt))
            .map(|state| state.offset.x)
            .unwrap_or(self.horizontal_scroll);

        let vcols = self.visible_col_count(sheet, settings);
        let table_interaction_enabled = !self.external_modal_open;
        let context_menu_switch_enabled = table_interaction_enabled || context_menu_open;
        let mut column_header_rect_for_paint = None;
        let mut corner_rect_for_paint = None;
        let mut header_painter = None;
        let mut row_header_hovered = false;
        let mut row_header_rect_for_paint = None;

        egui::Panel::top("table_header")
            .resizable(false)
            .exact_size(header_height)
            .frame(egui::Frame::default().inner_margin(Margin::same(0)))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                header_painter = Some(ui.painter().clone());
                let full_rect = ui.max_rect();
                let corner_rect =
                    Rect::from_min_size(full_rect.min, Vec2::new(header_width, header_height));
                let header_strip_rect = Rect::from_min_max(
                    Pos2::new(full_rect.min.x + header_width, full_rect.min.y),
                    full_rect.max,
                );
                column_header_rect_for_paint = Some(header_strip_rect);
                corner_rect_for_paint = Some(corner_rect);
                let header_response = ui.interact(
                    header_strip_rect,
                    ui.id().with("column_header_drag"),
                    if context_menu_switch_enabled {
                        Sense::click_and_drag()
                    } else {
                        Sense::hover()
                    },
                );

                if table_interaction_enabled
                    && !self.popup_was_open
                    && ctx.input(|i| i.pointer.primary_pressed())
                {
                    self.pressed_column_header = ctx
                        .input(|i| i.pointer.press_origin())
                        .filter(|pos| header_strip_rect.contains(*pos))
                        .and_then(|pointer_pos| {
                            column_header_at(
                                self,
                                sheet,
                                pointer_pos,
                                header_strip_rect,
                                current_horizontal_scroll,
                                settings.cell_scale,
                                settings,
                            )
                        })
                        .filter(|&col| col < vcols);

                    if let Some(col) = self.pressed_column_header {
                        self.set_active_column_headers(Some((col, col)));
                        self.drag_anchor = Some(CellIndex { col, row: 0 });
                        self.drag_mode = Some(DragMode::ColumnHeader);
                        self.edit_buffer.clear();
                        ctx.request_repaint();
                    }
                }

                if table_interaction_enabled
                    && !self.popup_was_open
                    && header_response.double_clicked()
                {
                    if let Some(pointer_pos) = header_response.interact_pointer_pos() {
                        if let Some(col) = column_header_at(
                            self,
                            sheet,
                            pointer_pos,
                            header_strip_rect,
                            current_horizontal_scroll,
                            settings.cell_scale,
                            settings,
                        ) {
                            self.pending_events
                                .push(TableViewEvent::OpenRenameColumnRequested {
                                    column: col,
                                    current_name: sheet.column_name(col).to_owned(),
                                });
                            ctx.request_repaint();
                        }
                    }
                }

                let secondary_header_multi_clicked = header_response
                    .double_clicked_by(egui::PointerButton::Secondary)
                    || header_response.triple_clicked_by(egui::PointerButton::Secondary);

                if table_interaction_enabled
                    && !self.popup_was_open
                    && secondary_header_multi_clicked
                {
                    if let Some(pointer_pos) = header_response.interact_pointer_pos() {
                        if let Some(col) = column_header_at(
                            self,
                            sheet,
                            pointer_pos,
                            header_strip_rect,
                            current_horizontal_scroll,
                            settings.cell_scale,
                            settings,
                        ) {
                            let values = sheet.column_values_skipping_visual_only(col);
                            self.pending_events.push(
                                TableViewEvent::ColumnHeaderSecondaryDoubleClicked {
                                    column_index: col,
                                    column_name: sheet.column_name(col).to_owned(),
                                    values,
                                },
                            );
                            ctx.request_repaint();
                        }
                    }
                }

                if context_menu_switch_enabled
                    && header_response.secondary_clicked()
                    && !secondary_header_multi_clicked
                {
                    if let Some(pointer_pos) = header_response.interact_pointer_pos() {
                        self.column_header_context_col = column_header_at(
                            self,
                            sheet,
                            pointer_pos,
                            header_strip_rect,
                            current_horizontal_scroll,
                            settings.cell_scale,
                            settings,
                        )
                        .filter(|&col| col < vcols);
                        if let Some(col) = self.column_header_context_col {
                            self.set_active_column_headers(Some((col, col)));
                            self.pending_events.push(
                                TableViewEvent::ColumnHeaderContextMenuRequested {
                                    position: pointer_pos,
                                    state: self.column_menu_state(sheet),
                                },
                            );
                        }
                    }
                }

                self.hovered_col = header_response
                    .hover_pos()
                    .and_then(|pos| {
                        column_header_at(
                            self,
                            sheet,
                            pos,
                            header_strip_rect,
                            current_horizontal_scroll,
                            settings.cell_scale,
                            settings,
                        )
                    })
                    .filter(|&col| col < vcols);
                if let Some(col) = self.hovered_col {
                    let header_font = egui::FontId::proportional(
                        settings.column_header_font_size * settings.cell_scale,
                    );
                    let available_width = (self.column_width(col, settings.cell_scale, settings)
                        - 8.0 * settings.cell_scale)
                        .max(0.0);
                    let (_, truncated) = ellipsize_text_to_width(
                        ui.painter(),
                        sheet.column_name(col),
                        &header_font,
                        available_width,
                    );
                    if truncated {
                        let tooltip_response = ui.interact(
                            column_header_rect(
                                self,
                                col,
                                header_strip_rect,
                                settings.cell_scale,
                                header_height,
                                settings,
                            ),
                            ui.id().with("column_header_tooltip").with(col),
                            Sense::hover(),
                        );
                        tooltip_response.on_hover_text(sheet.column_name(col));
                    }
                }
                const FLASH_DURATION: f64 = 0.4;
                let current_time = ctx.input(|i| i.time);
                if let Some((_, start, _)) = self.flashing_column_header {
                    if current_time - start < FLASH_DURATION {
                        ctx.request_repaint();
                    } else {
                        self.flashing_column_header = None;
                    }
                }
            });

        CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(Margin::same(0)))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                egui::Panel::left("row_headers")
                    .resizable(false)
                    .exact_size(header_width)
                    .frame(egui::Frame::default().inner_margin(Margin::same(0)))
                    .show_inside(ui, |ui| {
                        let row_header_rect = ui.max_rect();
                        row_header_rect_for_paint = Some(row_header_rect);
                        let row_header_response = ui.interact(
                            row_header_rect,
                            ui.id().with("row_header_drag"),
                            if context_menu_switch_enabled {
                                Sense::click_and_drag()
                            } else {
                                Sense::hover()
                            },
                        );

                        let popup_now_open = egui::Popup::is_any_open(&ctx);
                        if table_interaction_enabled
                            && !self.popup_was_open
                            && ctx.input(|i| i.pointer.primary_pressed())
                        {
                            self.pressed_row_header = ctx
                                .input(|i| i.pointer.press_origin())
                                .filter(|pos| row_header_rect.contains(*pos))
                                .and_then(|pointer_pos| {
                                    row_header_at(
                                        pointer_pos,
                                        row_header_rect,
                                        self.vertical_scroll,
                                        cell_height,
                                        sheet,
                                    )
                                });

                            if let Some(row) = self.pressed_row_header {
                                self.active_column_headers = None;
                                self.set_selection(row_selection(row, row, sheet));
                                self.remember_selection_row_count(sheet);
                                self.drag_anchor = Some(CellIndex { col: 0, row });
                                self.drag_mode = Some(DragMode::RowHeader);
                                self.edit_buffer.clear();
                                ctx.request_repaint();
                            }
                        }

                        if context_menu_switch_enabled && row_header_response.secondary_clicked() {
                            if let Some(pointer_pos) = row_header_response.interact_pointer_pos() {
                                if let Some(row) = row_header_at(
                                    pointer_pos,
                                    row_header_rect,
                                    self.vertical_scroll,
                                    cell_height,
                                    sheet,
                                ) {
                                    self.row_header_context_row = Some(row);
                                    self.pending_events.push(
                                        TableViewEvent::RowHeaderContextMenuRequested {
                                            position: pointer_pos,
                                            state: self.row_menu_state(sheet),
                                        },
                                    );
                                } else {
                                    self.row_header_context_row = None;
                                }
                            }
                        }

                        self.hovered_row = row_header_response.hover_pos().and_then(|pos| {
                            row_header_at(
                                pos,
                                row_header_rect,
                                self.vertical_scroll,
                                cell_height,
                                sheet,
                            )
                        });
                        row_header_hovered = row_header_response.hovered();
                        let visible_rows = visible_row_range(
                            sheet,
                            row_header_rect.height(),
                            self.vertical_scroll,
                            cell_height,
                        );
                        draw_row_headers(
                            ui.painter(),
                            row_header_rect,
                            self.vertical_scroll,
                            cell_height,
                            settings.cell_scale,
                            settings.row_header_font_size,
                            header_width,
                            sheet,
                            self.selection,
                            self.hovered_row,
                            settings.frame_header_mode,
                            settings.segment_header_mode,
                            default_frames_per_page,
                            settings.frame_header_density,
                            settings.segment_header_density,
                            settings.row_header_background_color,
                            settings.selection_color,
                            settings.hover_color,
                            visible_rows.clone(),
                        );
                        self.popup_was_open = popup_now_open;
                    });

                CentralPanel::default()
                    .frame(egui::Frame::default().inner_margin(Margin::same(0)))
                    .show_inside(ui, |ui| {
                        let body_size = Vec2::new(
                            (0..vcols)
                                .map(|col| self.column_width(col, settings.cell_scale, settings))
                                .sum::<f32>(),
                            sheet.total_height(cell_height),
                        );
                        let scroll_area_salt = ("table_body", self.scroll_id_salt);
                        let inner = ui.scope(|ui| {
                            let requested_scroll_offset = if (self.scroll_selection_into_view
                                || self.pending_scroll_top_row.is_some()
                                || self.pending_scroll_to_cell.is_some())
                                && self.last_body_viewport_size.x > 0.0
                                && self.last_body_viewport_size.y > 0.0
                            {
                                let mut preview_state = ScrollState::default();
                                preview_state.offset =
                                    Vec2::new(self.horizontal_scroll, self.vertical_scroll);
                                apply_pending_scroll_request(
                                    &mut preview_state,
                                    Rect::from_min_size(Pos2::ZERO, self.last_body_viewport_size),
                                    body_size,
                                    self.pending_vertical_scroll_direction,
                                    self.suppress_vertical_scroll_adjustment,
                                    self.previous_selection,
                                    self.selection,
                                    self.pending_scroll_top_row,
                                    self.pending_scroll_to_cell,
                                    self.preserve_horizontal_scroll_for_pending_cell,
                                    self.preserve_horizontal_scroll_for_selection,
                                    self,
                                    settings,
                                    settings.cell_scale,
                                    cell_height,
                                );
                                Some(preview_state.offset)
                            } else {
                                None
                            };
                            self.apply_scrollbar_style(ui, settings);
                            let mut scroll_area = ScrollArea::both()
                                .id_salt(scroll_area_salt)
                                .auto_shrink([false, false])
                                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded);
                            if let Some(offset) = requested_scroll_offset {
                                scroll_area = scroll_area.scroll_offset(offset);
                            }
                            scroll_area.show(ui, |ui| {
                                let clip_rect = ui.clip_rect();
                                let scroll_style = ui.style().spacing.scroll;
                                let (response, painter) = ui.allocate_painter(
                                    body_size,
                                    if table_interaction_enabled {
                                        Sense::click_and_drag()
                                    } else {
                                        Sense::hover()
                                    },
                                );
                                let origin = response.rect.min;
                                let visible = VisibleRange {
                                    rows: visible_row_range(
                                        sheet,
                                        clip_rect.height(),
                                        clip_rect.min.y - origin.y,
                                        cell_height,
                                    ),
                                    cols: visible_col_range(
                                        self,
                                        sheet,
                                        clip_rect.width(),
                                        (clip_rect.min.x - origin.x).max(0.0),
                                        settings.cell_scale,
                                        settings,
                                        vcols,
                                    ),
                                };

                                self.hovered_cell = response
                                    .hover_pos()
                                    .filter(|pointer_pos| {
                                        clip_rect.contains(*pointer_pos)
                                            && !self
                                                .minimap_hit_rect
                                                .is_some_and(|rect| rect.contains(*pointer_pos))
                                            && !is_pointer_over_scrollbar(
                                                *pointer_pos,
                                                clip_rect,
                                                body_size,
                                                scroll_style,
                                            )
                                    })
                                    .and_then(|pointer_pos| {
                                        body_cell_at(
                                            self,
                                            sheet,
                                            pointer_pos,
                                            origin,
                                            settings.cell_scale,
                                            settings,
                                            cell_height,
                                        )
                                    })
                                    .filter(|cell| cell.col < vcols);

                                if table_interaction_enabled
                                    && !self.popup_was_open
                                    && self.pressed_column_header.is_none()
                                    && self.pressed_row_header.is_none()
                                    && ctx.input(|i| i.pointer.primary_pressed())
                                {
                                    let press_in_body = ctx
                                        .input(|i| i.pointer.press_origin())
                                        .is_some_and(|pos| {
                                            clip_rect.contains(pos)
                                                && !self
                                                    .minimap_hit_rect
                                                    .is_some_and(|rect| rect.contains(pos))
                                                && !is_pointer_over_scrollbar(
                                                    pos,
                                                    clip_rect,
                                                    body_size,
                                                    scroll_style,
                                                )
                                        });
                                    if press_in_body {
                                        self.pressed_body_cell = ctx
                                            .input(|i| i.pointer.press_origin())
                                            .and_then(|pointer_pos| {
                                                body_cell_at(
                                                    self,
                                                    sheet,
                                                    pointer_pos,
                                                    origin,
                                                    settings.cell_scale,
                                                    settings,
                                                    cell_height,
                                                )
                                            })
                                            .filter(|cell| cell.col < vcols);

                                        if let Some(cell) = self.pressed_body_cell {
                                            let existing_selection = self.selection;
                                            let normalized_selection =
                                                existing_selection.map(SelectionRect::normalized);
                                            let drag_modifiers = ctx.input(|i| i.modifiers);
                                            let selection_drag_modifier =
                                                selection_drag_modifier(drag_modifiers);
                                            let shift_extend_selection = drag_modifiers.shift
                                                && !selection_drag_modifier
                                                && existing_selection.is_some();
                                            let start_selection_drag = selection_drag_modifier
                                                && !shift_extend_selection
                                                && normalized_selection.is_some_and(|selection| {
                                                    selection.contains(cell)
                                                });

                                            if let Some(original_selection) = start_selection_drag
                                                .then_some(normalized_selection)
                                                .flatten()
                                            {
                                                self.pending_selection_drag =
                                                    Some(PendingSelectionDrag {
                                                        origin_cell: cell,
                                                        original_selection,
                                                        moved: false,
                                                    });
                                                self.drag_anchor = Some(cell);
                                                self.drag_mode = None;
                                            } else if let Some(anchor) = shift_extend_selection
                                                .then_some(
                                                    existing_selection
                                                        .map(|selection| selection.start),
                                                )
                                                .flatten()
                                            {
                                                self.clear_pending_selection_drag();
                                                self.set_selection(Some(SelectionRect {
                                                    start: anchor,
                                                    end: cell,
                                                }));
                                                self.remember_selection_row_count(sheet);
                                                self.drag_anchor = Some(anchor);
                                                self.drag_mode = Some(DragMode::Cell);
                                            } else {
                                                self.clear_pending_selection_drag();
                                                self.set_selection(Some(SelectionRect {
                                                    start: cell,
                                                    end: cell,
                                                }));
                                                self.remember_selection_row_count(sheet);
                                                self.drag_anchor = Some(cell);
                                                self.drag_mode = Some(DragMode::Cell);
                                            }
                                            self.edit_buffer.clear();
                                        } else {
                                            self.clear_pending_selection_drag();
                                            self.set_selection(None);
                                            self.drag_anchor = None;
                                            self.drag_mode = None;
                                            self.edit_buffer.clear();
                                        }
                                        ctx.request_repaint();
                                    }
                                }

                                if table_interaction_enabled
                                    && !self.popup_was_open
                                    && response.double_clicked()
                                {
                                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                                        if clip_rect.contains(pointer_pos)
                                            && !self
                                                .minimap_hit_rect
                                                .is_some_and(|rect| rect.contains(pointer_pos))
                                            && !is_pointer_over_scrollbar(
                                                pointer_pos,
                                                clip_rect,
                                                body_size,
                                                scroll_style,
                                            )
                                        {
                                            if let Some(cell) = body_cell_at(
                                                self,
                                                sheet,
                                                pointer_pos,
                                                origin,
                                                settings.cell_scale,
                                                settings,
                                                cell_height,
                                            )
                                            .filter(|cell| cell.col < vcols)
                                            {
                                                self.clear_pending_selection_drag();
                                                self.set_selection(column_selection(
                                                    cell.col, cell.col, sheet,
                                                ));
                                                self.remember_selection_row_count(sheet);
                                                self.drag_anchor = None;
                                                self.drag_mode = None;
                                                self.pressed_body_cell = None;
                                                self.edit_buffer.clear();
                                                ctx.request_repaint();
                                            }
                                        }
                                    }
                                }

                                painter.rect_filled(response.rect, 0.0, ui.visuals().panel_fill);

                                if table_interaction_enabled
                                    && ctx.input(|i| i.pointer.primary_down())
                                {
                                    if let Some(pending_drag) = self.pending_selection_drag {
                                        if let Some(pointer_pos) =
                                            ctx.input(|i| i.pointer.latest_pos())
                                        {
                                            if !self
                                                .minimap_hit_rect
                                                .is_some_and(|rect| rect.contains(pointer_pos))
                                            {
                                                if let Some(current) = body_cell_at(
                                                    self,
                                                    sheet,
                                                    pointer_pos,
                                                    origin,
                                                    settings.cell_scale,
                                                    settings,
                                                    cell_height,
                                                ) {
                                                    let next_selection = move_selection_block(
                                                        pending_drag.original_selection,
                                                        current.col as isize
                                                            - pending_drag.origin_cell.col as isize,
                                                        current.row as isize
                                                            - pending_drag.origin_cell.row as isize,
                                                        sheet,
                                                    );
                                                    self.set_selection(Some(next_selection));
                                                    if next_selection
                                                        != pending_drag.original_selection
                                                    {
                                                        self.pending_selection_drag =
                                                            Some(PendingSelectionDrag {
                                                                moved: true,
                                                                ..pending_drag
                                                            });
                                                        self.drag_mode =
                                                            Some(DragMode::SelectionMove);
                                                    }
                                                    self.edit_buffer.clear();
                                                    ctx.request_repaint();
                                                }
                                            }
                                        }
                                    } else if self.drag_mode == Some(DragMode::Cell)
                                        || self.pressed_body_cell.is_some()
                                    {
                                        self.drag_anchor =
                                            self.drag_anchor.or(self.pressed_body_cell);
                                        self.drag_mode = Some(DragMode::Cell);
                                        if let (Some(anchor), Some(pointer_pos)) = (
                                            self.drag_anchor,
                                            ctx.input(|i| i.pointer.latest_pos()),
                                        ) {
                                            if !self
                                                .minimap_hit_rect
                                                .is_some_and(|rect| rect.contains(pointer_pos))
                                            {
                                                if let Some(current) = body_cell_at(
                                                    self,
                                                    sheet,
                                                    pointer_pos,
                                                    origin,
                                                    settings.cell_scale,
                                                    settings,
                                                    cell_height,
                                                ) {
                                                    self.set_selection(Some(SelectionRect {
                                                        start: anchor,
                                                        end: current,
                                                    }));
                                                    self.remember_selection_row_count(sheet);
                                                    self.edit_buffer.clear();
                                                    ctx.request_repaint();
                                                }
                                            }
                                        }
                                    }
                                }

                                if response.drag_stopped() {
                                    if let Some(pending_drag) = self.pending_selection_drag {
                                        if pending_drag.moved {
                                            let copy_on_drop = ctx.input(|i| {
                                                copy_selection_drag_modifier(i.modifiers)
                                            });
                                            let moved_selection = self
                                                .selection
                                                .map(SelectionRect::normalized)
                                                .unwrap_or(pending_drag.original_selection);
                                            transfer_range_on_sheet(
                                                sheet,
                                                DisplayMode::Keyframe,
                                                pending_drag.original_selection.start.col,
                                                pending_drag.original_selection.end.col,
                                                pending_drag.original_selection.start.row,
                                                pending_drag.original_selection.end.row,
                                                moved_selection.start.col,
                                                moved_selection.start.row,
                                                !copy_on_drop,
                                            );
                                            self.set_selection(Some(moved_selection));
                                            self.remember_selection_row_count(sheet);
                                        } else if let Some(cell) = self.hovered_cell {
                                            self.set_selection(Some(SelectionRect {
                                                start: cell,
                                                end: cell,
                                            }));
                                            self.remember_selection_row_count(sheet);
                                        }
                                        self.clear_pending_selection_drag();
                                    }
                                    self.drag_anchor = None;
                                    self.drag_mode = None;
                                    self.pressed_body_cell = None;
                                }

                                if !ctx.input(|i| i.pointer.primary_down()) {
                                    if let Some(pending_drag) = self.pending_selection_drag {
                                        if !pending_drag.moved {
                                            if let Some(cell) = self.hovered_cell {
                                                self.set_selection(Some(SelectionRect {
                                                    start: cell,
                                                    end: cell,
                                                }));
                                                self.remember_selection_row_count(sheet);
                                            } else {
                                                self.set_selection(Some(
                                                    pending_drag.original_selection,
                                                ));
                                            }
                                        }
                                        self.clear_pending_selection_drag();
                                    }
                                    self.drag_anchor = None;
                                    self.drag_mode = None;
                                    self.pressed_body_cell = None;
                                    self.pressed_column_header = None;
                                    self.pressed_row_header = None;
                                }

                                draw_body_cells(
                                    &painter,
                                    origin,
                                    self.selection,
                                    self.hovered_cell,
                                    editing_cell,
                                    (!self.edit_buffer.is_empty())
                                        .then_some(self.edit_buffer.as_str()),
                                    settings.cell_scale,
                                    cell_height,
                                    settings.body_cell_font_size,
                                    &settings.alternate_column_mode,
                                    settings.alternate_darken_amount,
                                    settings.alternate_second_darken_amount,
                                    settings.alternate_saturation_amount,
                                    settings.alternate_column_color,
                                    settings.cell_background_color,
                                    settings.punched_row_background_color(),
                                    settings.zero_cell_background_color,
                                    settings.use_zero_cell_background_color,
                                    settings.show_zero_value_markers,
                                    settings.special_inserted_row_background_color,
                                    settings.selection_color,
                                    settings.hover_color,
                                    ctx.input(|i| copy_selection_drag_modifier(i.modifiers)),
                                    vcols,
                                    self,
                                    settings,
                                    sheet,
                                    &visible,
                                    kara_cell_x_value,
                                );
                                draw_second_dividers(
                                    &painter,
                                    response.rect,
                                    0.0,
                                    cell_height,
                                    sheet,
                                    visible.rows,
                                    ctx.global_style().visuals.dark_mode,
                                );
                            })
                        });
                        let scroll_output = inner.inner;

                        let mut state = scroll_output.state;
                        let viewport_rect = scroll_output.inner_rect;
                        let content_size = scroll_output.content_size;
                        self.last_body_viewport_size = viewport_rect.size();
                        let max_offset_y = (content_size.y - viewport_rect.height()).max(0.0);

                        if table_interaction_enabled && row_header_hovered {
                            let scroll_delta = ctx.input(|input| input.smooth_scroll_delta.y);
                            let scrolling_up = state.offset.y > 0.0 && scroll_delta > 0.0;
                            let scrolling_down =
                                state.offset.y < max_offset_y && scroll_delta < 0.0;

                            if scrolling_up || scrolling_down {
                                state.offset.y =
                                    (state.offset.y - scroll_delta).clamp(0.0, max_offset_y);
                                state.store(&ctx, scroll_output.id);
                                ctx.input_mut(|input| {
                                    input.smooth_scroll_delta.y = 0.0;
                                });
                                ctx.request_repaint();
                            }
                        }

                        if self.scroll_selection_into_view
                            || self.pending_scroll_top_row.is_some()
                            || self.pending_scroll_to_cell.is_some()
                        {
                            apply_pending_scroll_request(
                                &mut state,
                                viewport_rect,
                                content_size,
                                self.pending_vertical_scroll_direction,
                                self.suppress_vertical_scroll_adjustment,
                                self.previous_selection,
                                self.selection,
                                self.pending_scroll_top_row,
                                self.pending_scroll_to_cell,
                                self.preserve_horizontal_scroll_for_pending_cell,
                                self.preserve_horizontal_scroll_for_selection,
                                self,
                                settings,
                                settings.cell_scale,
                                cell_height,
                            );
                            state.store(&ctx, scroll_output.id);
                            self.scroll_selection_into_view = false;
                            self.pending_vertical_scroll_direction = None;
                            self.suppress_vertical_scroll_adjustment = false;
                            self.pending_scroll_top_row = None;
                            self.pending_scroll_to_cell = None;
                            self.preserve_horizontal_scroll_for_pending_cell = false;
                            self.preserve_horizontal_scroll_for_selection = false;
                        }

                        let pointer_state = ctx.input(|i| {
                            (
                                i.pointer.button_down(egui::PointerButton::Middle),
                                i.pointer.latest_pos(),
                            )
                        });

                        if table_interaction_enabled
                            && let (true, Some(pointer_pos)) = pointer_state
                        {
                            if viewport_rect.contains(pointer_pos) || self.middle_pan_last.is_some()
                            {
                                if let Some(last_pos) = self.middle_pan_last {
                                    let drag_delta = pointer_pos - last_pos;
                                    let max_offset_x =
                                        (content_size.x - viewport_rect.width()).max(0.0);
                                    let max_offset_y =
                                        (content_size.y - viewport_rect.height()).max(0.0);

                                    state.offset.x =
                                        (state.offset.x - drag_delta.x).clamp(0.0, max_offset_x);
                                    state.offset.y =
                                        (state.offset.y - drag_delta.y).clamp(0.0, max_offset_y);
                                    state.store(&ctx, scroll_output.id);
                                    ctx.request_repaint();
                                }
                                self.middle_pan_last = Some(pointer_pos);
                            }
                        } else {
                            self.middle_pan_last = None;
                        }

                        if table_interaction_enabled
                            && let (Some(anchor), Some(pointer_pos), Some(drag_mode)) = (
                                self.drag_anchor,
                                ctx.input(|i| i.pointer.latest_pos()),
                                self.drag_mode,
                            )
                        {
                            let scroll_delta =
                                auto_scroll_delta(pointer_pos, viewport_rect, content_size, state);
                            let scroll_delta = match drag_mode {
                                DragMode::ColumnHeader => Vec2::new(scroll_delta.x, 0.0),
                                DragMode::RowHeader => Vec2::new(0.0, scroll_delta.y),
                                DragMode::Cell | DragMode::SelectionMove => scroll_delta,
                            };
                            if scroll_delta != Vec2::ZERO {
                                state.offset.x += scroll_delta.x;
                                state.offset.y += scroll_delta.y;
                                state.store(&ctx, scroll_output.id);
                                ctx.request_repaint();
                            }

                            let content_origin = viewport_rect.min - state.offset;
                            let clamped_pointer = clamp_pos(pointer_pos, viewport_rect);

                            match drag_mode {
                                DragMode::Cell => {
                                    if let Some(current) = body_cell_at(
                                        self,
                                        sheet,
                                        clamped_pointer,
                                        content_origin,
                                        settings.cell_scale,
                                        settings,
                                        cell_height,
                                    ) {
                                        self.set_selection(Some(SelectionRect {
                                            start: anchor,
                                            end: current,
                                        }));
                                        self.remember_selection_row_count(sheet);
                                    }
                                }
                                DragMode::SelectionMove => {
                                    if let Some(pending_drag) = self.pending_selection_drag
                                        && let Some(current) = body_cell_at(
                                            self,
                                            sheet,
                                            clamped_pointer,
                                            content_origin,
                                            settings.cell_scale,
                                            settings,
                                            cell_height,
                                        )
                                    {
                                        let next_selection = move_selection_block(
                                            pending_drag.original_selection,
                                            current.col as isize
                                                - pending_drag.origin_cell.col as isize,
                                            current.row as isize
                                                - pending_drag.origin_cell.row as isize,
                                            sheet,
                                        );
                                        self.set_selection(Some(next_selection));
                                    }
                                }
                                DragMode::RowHeader => {
                                    if let Some(current_row) = row_at_y(
                                        clamped_pointer.y,
                                        viewport_rect.min.y,
                                        state.offset.y,
                                        cell_height,
                                        sheet,
                                    ) {
                                        self.set_selection(row_selection(
                                            anchor.row,
                                            current_row,
                                            sheet,
                                        ));
                                        self.remember_selection_row_count(sheet);
                                    }
                                }
                                DragMode::ColumnHeader => {
                                    if let Some(current_col) = col_at_x_with_scroll(
                                        self,
                                        sheet,
                                        clamped_pointer.x,
                                        viewport_rect.min.x,
                                        state.offset.x,
                                        settings.cell_scale,
                                        settings,
                                    ) {
                                        let anchor_col =
                                            anchor.col.min(sheet.column_count().saturating_sub(1));
                                        self.set_active_column_headers(Some((
                                            anchor_col,
                                            current_col,
                                        )));
                                    }
                                }
                            }
                        }

                        if self.show_minimap && (table_interaction_enabled || context_menu_open) {
                            if let Some(minimap_layout) = compute_minimap_overlay_layout(
                                viewport_rect,
                                content_size,
                                state,
                                settings.minimap_width,
                                settings.minimap_height,
                            ) {
                                let response = ui.interact(
                                    minimap_layout.outer_rect,
                                    ui.id().with("minimap_overlay"),
                                    if table_interaction_enabled {
                                        Sense::click_and_drag()
                                    } else {
                                        Sense::hover()
                                    },
                                );
                                self.minimap_hit_rect = Some(minimap_layout.outer_rect);

                                if table_interaction_enabled
                                    && let Some(pointer_pos) = response.interact_pointer_pos()
                                {
                                    if response.drag_started()
                                        || response.clicked_by(egui::PointerButton::Primary)
                                    {
                                        self.pressed_body_cell = None;
                                        if matches!(
                                            self.drag_mode,
                                            Some(DragMode::Cell | DragMode::SelectionMove)
                                        ) {
                                            self.drag_anchor = None;
                                            self.drag_mode = None;
                                        }
                                        self.pending_selection_drag = None;
                                        let resize_handle =
                                            minimap_resize_handle_rect(minimap_layout);
                                        if resize_handle.contains(pointer_pos)
                                            && response.drag_started()
                                        {
                                            self.minimap_drag_mode = Some(MinimapDragMode::Resize);
                                        } else {
                                            let drag_offset = if minimap_layout
                                                .viewport_rect
                                                .contains(pointer_pos)
                                            {
                                                pointer_pos - minimap_layout.viewport_rect.min
                                            } else {
                                                minimap_layout.viewport_rect.size() * 0.5
                                            };
                                            self.minimap_drag_mode =
                                                Some(MinimapDragMode::Scroll(drag_offset));

                                            let next_offset = minimap_scroll_offset_for_pointer(
                                                pointer_pos,
                                                drag_offset,
                                                minimap_layout,
                                                viewport_rect,
                                                content_size,
                                            );
                                            state.offset = next_offset;
                                            state.store(&ctx, scroll_output.id);
                                            ctx.request_repaint();
                                        }
                                    } else if response.dragged() {
                                        match self.minimap_drag_mode {
                                            Some(MinimapDragMode::Scroll(drag_offset)) => {
                                                let next_offset = minimap_scroll_offset_for_pointer(
                                                    pointer_pos,
                                                    drag_offset,
                                                    minimap_layout,
                                                    viewport_rect,
                                                    content_size,
                                                );
                                                state.offset = next_offset;
                                                state.store(&ctx, scroll_output.id);
                                                ctx.request_repaint();
                                            }
                                            Some(MinimapDragMode::Resize) => {
                                                settings.set_minimap_width(
                                                    minimap_layout.outer_rect.max.x - pointer_pos.x,
                                                );
                                                settings.set_minimap_height(
                                                    minimap_layout.outer_rect.max.y - pointer_pos.y,
                                                );
                                                ctx.request_repaint();
                                            }
                                            None => {}
                                        }
                                    }
                                }

                                if response.drag_stopped()
                                    || !ctx.input(|i| i.pointer.primary_down())
                                {
                                    self.minimap_drag_mode = None;
                                }

                                if response.hovered()
                                    && minimap_resize_handle_rect(minimap_layout).contains(
                                        ctx.input(|i| i.pointer.hover_pos().unwrap_or(Pos2::ZERO)),
                                    )
                                {
                                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                                } else if matches!(
                                    self.minimap_drag_mode,
                                    Some(MinimapDragMode::Resize)
                                ) {
                                    ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                                }

                                draw_minimap_overlay(
                                    ui.painter(),
                                    minimap_layout,
                                    settings.cell_scale,
                                    cell_height,
                                    &settings.alternate_column_mode,
                                    settings.alternate_darken_amount,
                                    settings.alternate_second_darken_amount,
                                    settings.alternate_saturation_amount,
                                    settings.alternate_column_color,
                                    settings.cell_background_color,
                                    settings.zero_cell_background_color,
                                    settings.use_zero_cell_background_color,
                                    vcols,
                                    self,
                                    settings,
                                    sheet,
                                );
                            } else {
                                self.minimap_hit_rect = None;
                                self.minimap_drag_mode = None;
                            }
                        } else {
                            self.minimap_hit_rect = None;
                            self.minimap_drag_mode = None;
                        }

                        self.horizontal_scroll = state.offset.x;
                        self.vertical_scroll = state.offset.y;
                        if self.pending_selection_drag.is_some() {
                            if ctx.input(|i| copy_selection_drag_modifier(i.modifiers)) {
                                ctx.set_cursor_icon(egui::CursorIcon::Copy);
                            } else {
                                ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                            }
                        }
                    });

                if let Some(row_header_rect) = row_header_rect_for_paint {
                    let visible_rows = visible_row_range(
                        sheet,
                        row_header_rect.height(),
                        self.vertical_scroll,
                        cell_height,
                    );
                    let painter = ui.painter_at(row_header_rect);
                    draw_row_headers(
                        &painter,
                        row_header_rect,
                        self.vertical_scroll,
                        cell_height,
                        settings.cell_scale,
                        settings.row_header_font_size,
                        header_width,
                        sheet,
                        self.selection,
                        self.hovered_row,
                        settings.frame_header_mode,
                        settings.segment_header_mode,
                        default_frames_per_page,
                        settings.frame_header_density,
                        settings.segment_header_density,
                        settings.row_header_background_color,
                        settings.selection_color,
                        settings.hover_color,
                        visible_rows.clone(),
                    );
                    draw_second_dividers(
                        &painter,
                        row_header_rect,
                        self.vertical_scroll,
                        cell_height,
                        sheet,
                        visible_rows,
                        ctx.global_style().visuals.dark_mode,
                    );
                }

                if let (Some(header_rect), Some(painter)) =
                    (column_header_rect_for_paint, header_painter.clone())
                {
                    let visible_cols = visible_col_range(
                        self,
                        sheet,
                        header_rect.width(),
                        self.horizontal_scroll,
                        settings.cell_scale,
                        settings,
                        vcols,
                    );
                    let current_time = ctx.input(|i| i.time);
                    draw_column_headers(
                        &painter,
                        header_rect,
                        settings.cell_scale,
                        settings.column_header_font_size,
                        header_height,
                        sheet,
                        self.selection,
                        self.hovered_col,
                        self.active_column_headers,
                        settings.column_header_background_color,
                        settings.selection_color,
                        settings.hover_color,
                        self.flashing_column_header,
                        current_time,
                        self,
                        settings,
                        visible_cols,
                    );
                }

                if let (Some(corner_rect), Some(painter)) =
                    (corner_rect_for_paint, header_painter.clone())
                {
                    draw_corner_header(
                        &painter.with_clip_rect(corner_rect),
                        corner_rect,
                        settings.column_header_background_color,
                    );
                }
            });

        self.take_events()
    }

    pub fn selected_cell(&self) -> Option<(usize, usize)> {
        self.selection
            .map(|selection| selection.normalized().start)
            .map(|cell| (cell.col, cell.row))
    }

    pub fn selected_column(&self) -> Option<usize> {
        self.active_column_headers
            .map(|(start, _)| start)
            .or(self.column_header_context_col)
            .or_else(|| self.selected_cell().map(|(col, _)| col))
    }

    pub fn selected_header_column(&self) -> Option<usize> {
        self.active_column_headers
            .map(|(start, _)| start)
            .or(self.column_header_context_col)
    }

    pub fn selected_range(&self) -> Option<TableSelection> {
        self.selection
            .map(SelectionRect::normalized)
            .map(|selection| TableSelection {
                start_col: selection.start.col,
                end_col: selection.end.col,
                start_row: selection.start.row,
                end_row: selection.end.row,
            })
    }

    pub fn selected_range_size(&self) -> Option<(usize, usize)> {
        self.selection
            .map(|selection| selection.normalized())
            .map(|selection| (selection.width(), selection.height()))
    }

    pub fn selected_timeline_row_count(&self, sheet: &Sheet) -> Option<usize> {
        self.selection
            .map(SelectionRect::normalized)
            .map(|selection| timeline_row_count_in_selection(selection, sheet))
    }

    pub fn edit_buffer_text(&self) -> Option<&str> {
        (!self.edit_buffer.is_empty()).then_some(self.edit_buffer.as_str())
    }

    pub fn toggle_minimap(&mut self) {
        if self.global_shortcuts_blocked() {
            return;
        }
        self.show_minimap = !self.show_minimap;
        self.minimap_drag_mode = None;
    }

    pub fn show_minimap(&self) -> bool {
        self.show_minimap
    }

    pub fn set_show_minimap(&mut self, show: bool) {
        self.show_minimap = show;
        if !show {
            self.minimap_drag_mode = None;
        }
    }

    pub fn move_selection_up(&mut self, sheet: &Sheet) -> bool {
        if self.has_all_rows_selected(sheet) {
            self.scroll_to_row_preserving_horizontal(0, sheet);
            return true;
        }
        self.move_selection_vertically(sheet, -1, Some(VerticalScrollDirection::Up))
    }

    pub fn move_selection_down(&mut self, sheet: &Sheet) -> bool {
        if self.has_all_rows_selected(sheet) {
            if let Some(last_row) = sheet.row_count().checked_sub(1) {
                self.scroll_to_row_preserving_horizontal(last_row, sheet);
                return true;
            }
            return false;
        }
        self.move_selection_vertically(sheet, 1, Some(VerticalScrollDirection::Down))
    }

    pub fn move_selection_left(&mut self, sheet: &Sheet) -> bool {
        self.move_selection_horizontally(sheet, -1)
    }

    pub fn move_selection_right(&mut self, sheet: &Sheet) -> bool {
        self.move_selection_horizontally(sheet, 1)
    }

    pub fn jump_selection_up(&mut self, sheet: &Sheet) -> bool {
        self.jump_selection(sheet, JumpDirection::Up)
    }

    pub fn jump_selection_down(&mut self, sheet: &Sheet) -> bool {
        self.jump_selection(sheet, JumpDirection::Down)
    }

    pub fn decrease_selection(&mut self, sheet: &Sheet) -> bool {
        if !self.can_run_selection_command(sheet) {
            return false;
        }

        let Some(selection) = self.selection else {
            return false;
        };
        let normalized = selection.normalized();
        if normalized.height() <= 1 {
            return false;
        }

        let next_selection = SelectionRect {
            start: normalized.start,
            end: CellIndex {
                col: normalized.end.col,
                row: previous_timeline_row(normalized.end.row, normalized.start.row, sheet)
                    .unwrap_or(normalized.start.row),
            },
        };
        self.apply_selection_change(Some(next_selection), None, false, false, false);
        self.remember_selection_row_count(sheet);
        true
    }

    pub fn increase_selection(&mut self, sheet: &Sheet) -> bool {
        if !self.can_run_selection_command(sheet) {
            return false;
        }

        let Some(selection) = self.selection else {
            return false;
        };
        let normalized = selection.normalized();
        let Some(next_row) = next_timeline_row(normalized.end.row, sheet) else {
            return false;
        };

        let next_selection = SelectionRect {
            start: normalized.start,
            end: CellIndex {
                col: normalized.end.col,
                row: next_row,
            },
        };
        self.apply_selection_change(Some(next_selection), None, false, false, false);
        self.remember_selection_row_count(sheet);
        true
    }

    fn apply_scrollbar_style(&self, ui: &mut egui::Ui, settings: &TableSettings) {
        let style = ui.style_mut();
        style.spacing.scroll.floating = false;
        style.spacing.scroll.foreground_color = false;
        let visuals = &mut style.visuals;
        visuals.extreme_bg_color = settings.scrollbar_background_color;
        visuals.widgets.inactive.bg_fill = settings.scrollbar_handle_color;
        visuals.widgets.hovered.bg_fill = settings.scrollbar_handle_hovered_color;
        visuals.widgets.active.bg_fill = settings.scrollbar_handle_active_color;
    }

    fn has_all_rows_selected(&self, sheet: &Sheet) -> bool {
        self.selection
            .map(SelectionRect::normalized)
            .is_some_and(|selection| {
                sheet.row_count() > 0
                    && selection.start.row == 0
                    && selection.end.row + 1 == sheet.row_count()
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JumpDirection {
    Up,
    Down,
}

impl TableViewState {
    fn jump_selection(&mut self, sheet: &Sheet, direction: JumpDirection) -> bool {
        if !self.can_run_selection_command(sheet) {
            return false;
        }

        let Some(selection) = self.selection.map(SelectionRect::normalized) else {
            return false;
        };

        let Some(target_row) = find_jump_target_row(selection, sheet, direction) else {
            return false;
        };

        let next_selection = selection_with_start_row(selection, target_row, sheet);
        if Some(next_selection) == self.selection.map(SelectionRect::normalized) {
            return false;
        }

        let vertical_scroll_direction = match direction {
            JumpDirection::Up => Some(VerticalScrollDirection::Up),
            JumpDirection::Down => Some(VerticalScrollDirection::Down),
        };
        self.apply_selection_change(
            Some(next_selection),
            vertical_scroll_direction,
            false,
            false,
            true,
        );
        true
    }
}

fn find_jump_target_row(
    selection: SelectionRect,
    sheet: &Sheet,
    direction: JumpDirection,
) -> Option<usize> {
    let cols = selection.start.col..=selection.end.col;

    match direction {
        JumpDirection::Down => ((selection.end.row + 1)..sheet.row_count())
            .find(|&row| is_jump_target_row(row, cols.clone(), sheet)),
        JumpDirection::Up => (0..selection.start.row)
            .rev()
            .find(|&row| is_jump_target_row(row, cols.clone(), sheet)),
    }
}

fn is_jump_target_row(row: usize, cols: RangeInclusive<usize>, sheet: &Sheet) -> bool {
    if !sheet.row_participates_in_timeline(row) {
        return false;
    }

    let Some(previous_row) = previous_visible_row(row, sheet) else {
        return row == 0;
    };

    cols.into_iter().any(|col| {
        let current = sheet.cell(col, row);
        let previous = sheet.cell(col, previous_row);
        current != previous
    })
}

fn previous_visible_row(row: usize, sheet: &Sheet) -> Option<usize> {
    sheet.previous_timeline_row(row)
}

fn selection_with_start_row(
    selection: SelectionRect,
    start_row: usize,
    sheet: &Sheet,
) -> SelectionRect {
    let visible_row_count = (selection.start.row..=selection.end.row)
        .filter(|&row| sheet.row_participates_in_timeline(row))
        .count()
        .max(1);

    let mut end_row = start_row;
    let mut remaining = visible_row_count.saturating_sub(1);
    while remaining > 0 {
        let Some(next_row) = next_timeline_row(end_row, sheet) else {
            break;
        };
        end_row = next_row;
        remaining -= 1;
    }

    SelectionRect {
        start: CellIndex {
            col: selection.start.col,
            row: start_row,
        },
        end: CellIndex {
            col: selection.end.col,
            row: end_row,
        },
    }
}

use std::ops::RangeInclusive;

#[cfg(test)]
mod tests {
    use super::{
        CellIndex, JumpDirection, SelectionRect, find_jump_target_row, selection_with_start_row,
    };
    use sheet::{CellValue, Sheet, SheetColumn};

    fn sample_sheet(columns: Vec<Vec<i64>>) -> Sheet {
        Sheet::new(
            columns
                .into_iter()
                .enumerate()
                .map(|(index, values)| {
                    SheetColumn::new(
                        format!("C{index}"),
                        values.into_iter().map(CellValue::from).collect(),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn jump_down_uses_nearest_change_across_selected_columns() {
        let sheet = sample_sheet(vec![
            vec![0, 0, 0, 0, 1, 1, 2, 2, 2, 3, 3, 3],
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 3],
        ]);
        let selection = SelectionRect {
            start: CellIndex { col: 0, row: 0 },
            end: CellIndex { col: 1, row: 1 },
        };

        assert_eq!(
            find_jump_target_row(selection, &sheet, JumpDirection::Down),
            Some(3)
        );

        let jumped = selection_with_start_row(selection, 3, &sheet);
        assert_eq!(jumped.start.row, 3);
        assert_eq!(jumped.end.row, 4);
    }

    #[test]
    fn jump_up_finds_previous_change_row() {
        let sheet = sample_sheet(vec![vec![0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]]);
        let selection = SelectionRect {
            start: CellIndex { col: 0, row: 6 },
            end: CellIndex { col: 0, row: 7 },
        };

        assert_eq!(
            find_jump_target_row(selection, &sheet, JumpDirection::Up),
            Some(3)
        );
    }
}

fn visible_col_range(
    table: &TableViewState,
    sheet: &Sheet,
    viewport_width: f32,
    horizontal_scroll: f32,
    cell_scale: f32,
    settings: &TableSettings,
    col_limit: usize,
) -> Range<usize> {
    let start_col = table
        .col_at_x(sheet, horizontal_scroll, cell_scale, settings)
        .unwrap_or(0);
    let end_col = table
        .col_at_x(
            sheet,
            horizontal_scroll + viewport_width,
            cell_scale,
            settings,
        )
        .map(|col| col + 1)
        .unwrap_or(col_limit);
    start_col.min(col_limit)..end_col.min(col_limit)
}

fn visible_row_range(
    sheet: &Sheet,
    viewport_height: f32,
    vertical_scroll: f32,
    cell_height: f32,
) -> Range<usize> {
    let start_row = (vertical_scroll / cell_height).floor().max(0.0) as usize;
    let end_row = ((vertical_scroll + viewport_height) / cell_height).ceil() as usize + 1;
    start_row.min(sheet.row_count())..end_row.min(sheet.row_count())
}

fn body_cell_at(
    table: &TableViewState,
    sheet: &Sheet,
    pointer_pos: Pos2,
    origin: Pos2,
    cell_scale: f32,
    settings: &TableSettings,
    cell_height: f32,
) -> Option<CellIndex> {
    let x = pointer_pos.x - origin.x;
    let y = pointer_pos.y - origin.y;

    if y < 0.0 {
        return None;
    }

    let col = table.col_at_x(sheet, x, cell_scale, settings)?;
    let row = (y / cell_height).floor() as usize;

    if row < sheet.row_count() && !sheet.is_punched_row(row) {
        Some(CellIndex { col, row })
    } else {
        None
    }
}

fn column_header_at(
    table: &TableViewState,
    sheet: &Sheet,
    pointer_pos: Pos2,
    area: Rect,
    horizontal_scroll: f32,
    cell_scale: f32,
    settings: &TableSettings,
) -> Option<usize> {
    col_at_x_with_scroll(
        table,
        sheet,
        pointer_pos.x,
        area.min.x,
        horizontal_scroll,
        cell_scale,
        settings,
    )
}

fn row_header_at(
    pointer_pos: Pos2,
    area: Rect,
    vertical_scroll: f32,
    cell_height: f32,
    sheet: &Sheet,
) -> Option<usize> {
    row_at_y(
        pointer_pos.y,
        area.min.y,
        vertical_scroll,
        cell_height,
        sheet,
    )
}

fn column_header_rect(
    table: &TableViewState,
    col: usize,
    area: Rect,
    cell_scale: f32,
    header_height: f32,
    settings: &TableSettings,
) -> Rect {
    Rect::from_min_size(
        Pos2::new(
            area.min.x + table.column_left(col, cell_scale, settings) - table.horizontal_scroll,
            area.min.y,
        ),
        Vec2::new(table.column_width(col, cell_scale, settings), header_height),
    )
}

fn row_at_y(
    pointer_y: f32,
    area_top: f32,
    vertical_scroll: f32,
    cell_height: f32,
    sheet: &Sheet,
) -> Option<usize> {
    let y = pointer_y - area_top + vertical_scroll;
    if y < 0.0 {
        return None;
    }

    let row = (y / cell_height).floor() as usize;
    (row < sheet.row_count()).then_some(row)
}

fn col_at_x_with_scroll(
    table: &TableViewState,
    sheet: &Sheet,
    pointer_x: f32,
    area_left: f32,
    horizontal_scroll: f32,
    cell_scale: f32,
    settings: &TableSettings,
) -> Option<usize> {
    let x = pointer_x - area_left + horizontal_scroll;
    table.col_at_x(sheet, x, cell_scale, settings)
}

fn selection_drag_modifier(modifiers: egui::Modifiers) -> bool {
    modifiers.command
}

fn copy_selection_drag_modifier(modifiers: egui::Modifiers) -> bool {
    selection_drag_modifier(modifiers) && modifiers.shift
}

fn column_action_state_for_context_col(
    _selection: Option<SelectionRect>,
    context_col: Option<usize>,
) -> ColumnActionState {
    ColumnActionState {
        context_col,
        target_col: context_col,
    }
}

fn row_action_state_from_selection(selection: Option<SelectionRect>) -> RowActionState {
    let target_rows: Vec<usize> = selection
        .map(SelectionRect::normalized)
        .map(|selection| (selection.start.row..=selection.end.row).collect())
        .unwrap_or_default();
    let context_row = target_rows.first().copied();

    RowActionState {
        context_row,
        target_rows,
    }
}

fn row_action_state_for_context_row(
    selection: Option<SelectionRect>,
    context_row: Option<usize>,
) -> RowActionState {
    if let Some(context_row) = context_row {
        let target_rows: Vec<usize> = selection
            .map(SelectionRect::normalized)
            .map(|selection| (selection.start.row..=selection.end.row).collect())
            .unwrap_or_else(|| vec![context_row]);
        return RowActionState {
            context_row: Some(context_row),
            target_rows,
        };
    }

    RowActionState::default()
}
