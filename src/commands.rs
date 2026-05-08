use crate::{AppLocale, strings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMenu {
    File,
    Sheet,
    Edit,
    View,
    Help,
    Row,
    Column,
}

impl AppMenu {
    pub fn label(self) -> &'static str {
        self.localized_label(AppLocale::Japanese)
    }

    pub fn localized_label(self, locale: AppLocale) -> &'static str {
        match self {
            Self::File => strings::menu_file(locale),
            Self::Sheet => strings::menu_sheet(locale),
            Self::Edit => strings::menu_edit(locale),
            Self::View => strings::menu_view(locale),
            Self::Help => strings::menu_help(locale),
            Self::Row => strings::menu_rows(locale),
            Self::Column => strings::menu_columns(locale),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppCommand {
    OpenSheet,
    NewSheet,
    SaveSts,
    SaveStsAs,
    ExitApp,
    ResizeSheet,
    SendColumnToAfterEffects,
    NewSheetFromAfterEffectsSelection,
    Cut,
    Copy,
    Paste,
    RepeatSelectionDown,
    Undo,
    Redo,
    MoveSelectionUp,
    MoveSelectionDown,
    MoveSelectionLeft,
    MoveSelectionRight,
    JumpSelectionUp,
    JumpSelectionDown,
    DecreaseSelection,
    IncreaseSelection,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleMinimap,
    ToggleAlwaysOnTop,
    OpenPreferences,
    ShowAbout,
    RenameColumn,
    DeleteColumn,
    InsertColumnLeft,
    InsertColumnRight,
    PunchRows,
    UnpunchRows,
    AppendRowsAbove,
    AppendRowsBelow,
    DeleteSpecialRows,
}

impl AppCommand {
    pub fn label(self) -> &'static str {
        self.localized_label(AppLocale::Japanese)
    }

    pub fn localized_label(self, locale: AppLocale) -> &'static str {
        match self {
            Self::OpenSheet => strings::cmd_open_sheet(locale),
            Self::NewSheet => strings::cmd_new_sheet(locale),
            Self::SaveSts => strings::cmd_save(locale),
            Self::SaveStsAs => strings::cmd_save_as(locale),
            Self::ExitApp => strings::cmd_exit(locale),
            Self::ResizeSheet => strings::cmd_resize_sheet(locale),
            Self::SendColumnToAfterEffects => strings::cmd_send_column_to_ae(locale),
            Self::NewSheetFromAfterEffectsSelection => strings::cmd_new_sheet_from_ae(locale),
            Self::Cut => strings::cmd_cut(locale),
            Self::Copy => strings::cmd_copy(locale),
            Self::Paste => strings::cmd_paste(locale),
            Self::RepeatSelectionDown => strings::cmd_repeat_selection_down(locale),
            Self::Undo => strings::cmd_undo(locale),
            Self::Redo => strings::cmd_redo(locale),
            Self::MoveSelectionUp => strings::cmd_move_selection_up(locale),
            Self::MoveSelectionDown => strings::cmd_move_selection_down(locale),
            Self::MoveSelectionLeft => strings::cmd_move_selection_left(locale),
            Self::MoveSelectionRight => strings::cmd_move_selection_right(locale),
            Self::JumpSelectionUp => strings::cmd_jump_selection_up(locale),
            Self::JumpSelectionDown => strings::cmd_jump_selection_down(locale),
            Self::DecreaseSelection => strings::cmd_decrease_selection(locale),
            Self::IncreaseSelection => strings::cmd_increase_selection(locale),
            Self::ZoomIn => strings::cmd_zoom_in(locale),
            Self::ZoomOut => strings::cmd_zoom_out(locale),
            Self::ResetZoom => strings::cmd_reset_zoom(locale),
            Self::ToggleMinimap => strings::cmd_toggle_minimap(locale),
            Self::ToggleAlwaysOnTop => strings::cmd_toggle_always_on_top(locale),
            Self::OpenPreferences => strings::cmd_open_preferences(locale),
            Self::ShowAbout => strings::cmd_show_about(locale),
            Self::RenameColumn => strings::cmd_rename_column(locale),
            Self::DeleteColumn => strings::cmd_delete_column(locale),
            Self::InsertColumnLeft => strings::cmd_insert_column_left(locale),
            Self::InsertColumnRight => strings::cmd_insert_column_right(locale),
            Self::PunchRows => strings::cmd_punch_rows(locale),
            Self::UnpunchRows => strings::cmd_unpunch_rows(locale),
            Self::AppendRowsAbove => strings::cmd_append_rows_above(locale),
            Self::AppendRowsBelow => strings::cmd_append_rows_below(locale),
            Self::DeleteSpecialRows => strings::cmd_delete_special_rows(locale),
        }
    }
}

pub const FILE_MENU_COMMANDS: &[AppCommand] = &[
    AppCommand::NewSheet,
    AppCommand::OpenSheet,
    AppCommand::SaveSts,
    AppCommand::SaveStsAs,
    AppCommand::ExitApp,
];

pub const SHEET_MENU_COMMANDS: &[AppCommand] = &[
    AppCommand::ResizeSheet,
    AppCommand::NewSheetFromAfterEffectsSelection,
];

pub const EDIT_MENU_COMMANDS: &[AppCommand] = &[
    AppCommand::Cut,
    AppCommand::Copy,
    AppCommand::Paste,
    AppCommand::RepeatSelectionDown,
    AppCommand::Undo,
    AppCommand::Redo,
    AppCommand::OpenPreferences,
    AppCommand::PunchRows,
    AppCommand::UnpunchRows,
    AppCommand::AppendRowsAbove,
    AppCommand::AppendRowsBelow,
    AppCommand::DeleteSpecialRows,
];

pub const VIEW_MENU_COMMANDS: &[AppCommand] = &[
    AppCommand::ZoomIn,
    AppCommand::ZoomOut,
    AppCommand::ResetZoom,
    AppCommand::ToggleAlwaysOnTop,
    AppCommand::ToggleMinimap,
];

pub const HELP_MENU_COMMANDS: &[AppCommand] = &[AppCommand::ShowAbout];

pub const NAVIGATION_COMMANDS: &[AppCommand] = &[
    AppCommand::MoveSelectionUp,
    AppCommand::MoveSelectionDown,
    AppCommand::MoveSelectionLeft,
    AppCommand::MoveSelectionRight,
    AppCommand::JumpSelectionUp,
    AppCommand::JumpSelectionDown,
    AppCommand::DecreaseSelection,
    AppCommand::IncreaseSelection,
    AppCommand::ToggleMinimap,
];

pub const ROW_MENU_COMMANDS: &[AppCommand] = &[
    AppCommand::PunchRows,
    AppCommand::UnpunchRows,
    AppCommand::AppendRowsAbove,
    AppCommand::AppendRowsBelow,
    AppCommand::DeleteSpecialRows,
];

pub const COLUMN_MENU_COMMANDS: &[AppCommand] = &[
    AppCommand::SendColumnToAfterEffects,
    AppCommand::RenameColumn,
    AppCommand::DeleteColumn,
    AppCommand::InsertColumnLeft,
    AppCommand::InsertColumnRight,
];
