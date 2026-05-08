use crate::settings::AppLocale;
use sheet::SheetError;
use std::fmt::Display;

pub fn close(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "閉じる",
        AppLocale::English => "Close",
    }
}

pub fn menu_file(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ファイル",
        AppLocale::English => "File",
    }
}

pub fn menu_sheet(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列操作",
        AppLocale::English => "Column Actions",
    }
}

pub fn menu_edit(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "編集",
        AppLocale::English => "Edit",
    }
}

pub fn menu_view(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "表示",
        AppLocale::English => "View",
    }
}

pub fn menu_help(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ヘルプ",
        AppLocale::English => "Help",
    }
}

pub fn menu_rows(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "行操作",
        AppLocale::English => "Row Actions",
    }
}

pub fn menu_columns(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列操作",
        AppLocale::English => "Columns",
    }
}

pub fn reset(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "デフォルトに戻す",
        AppLocale::English => "Reset",
    }
}

pub fn cancel(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "キャンセル",
        AppLocale::English => "Cancel",
    }
}

pub fn ok(_locale: AppLocale) -> &'static str {
    "OK"
}

pub fn change(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "変更",
        AppLocale::English => "Change",
    }
}

pub fn waiting(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "入力待ち...",
        AppLocale::English => "Waiting...",
    }
}

pub fn rename(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "変更",
        AppLocale::English => "Rename",
    }
}

pub fn preferences_title(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "環境設定",
        AppLocale::English => "Preferences",
    }
}

pub fn display_tab(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "表示",
        AppLocale::English => "Display",
    }
}

pub fn ae_tab(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "外部アプリ連携",
        AppLocale::English => "External Apps",
    }
}

pub fn colors_tab(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "色/サイズ",
        AppLocale::English => "Colors",
    }
}

pub fn sheet_tab(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "シート",
        AppLocale::English => "Sheet",
    }
}

pub fn keys_tab(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "キー",
        AppLocale::English => "Keys",
    }
}

pub fn key_bindings(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "キーバインド",
        AppLocale::English => "Key Bindings",
    }
}

pub fn keybind_instruction(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "変更を押したあと、割り当てたいキーを入力します。",
        AppLocale::English => "Click Change, then press the key you want to assign.",
    }
}

pub fn display_settings(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "表示設定",
        AppLocale::English => "Display Settings",
    }
}

pub fn language(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "言語",
        AppLocale::English => "Language",
    }
}

pub fn locale_native_label(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "日本語",
        AppLocale::English => "English",
    }
}

pub fn theme(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "テーマ",
        AppLocale::English => "Theme",
    }
}

pub fn system(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "システム",
        AppLocale::English => "System",
    }
}

pub fn light(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ライト",
        AppLocale::English => "Light",
    }
}

pub fn dark(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ダーク",
        AppLocale::English => "Dark",
    }
}

pub fn custom(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "カスタム",
        AppLocale::English => "Custom",
    }
}

pub fn imported_theme(locale: AppLocale, name: &str) -> String {
    match locale {
        AppLocale::Japanese => format!("登録テーマ: {name}"),
        AppLocale::English => format!("Imported: {name}"),
    }
}

pub fn ae_keyframe_locale_japanese(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "日本語AE向け",
        AppLocale::English => "Japanese AE format",
    }
}

pub fn ae_keyframe_locale_english(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "英語AE向け",
        AppLocale::English => "English AE format",
    }
}

pub fn ae_kara_cell_mode_blinds(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ブラインド",
        AppLocale::English => "Blinds",
    }
}

pub fn ae_kara_cell_mode_max_frame_count(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "最大コマ数",
        AppLocale::English => "Comp frame count",
    }
}

pub fn after_effects_settings(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "外部アプリ連携設定",
        AppLocale::English => "External App Settings",
    }
}

pub fn column_double_click_copy(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列ダブルクリックコピー",
        AppLocale::English => "Column double-click copy",
    }
}

pub fn ae_keyframe_data(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "AEキーフレームデータ",
        AppLocale::English => "AE keyframe data",
    }
}

pub fn ae_keyframe_version(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "AE KeyFrameバージョン",
        AppLocale::English => "AE KeyFrame version",
    }
}

pub fn blank_cel_mode(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "送受信時のカラセル方式",
        AppLocale::English => "Blank cel mode for send/receive",
    }
}

pub fn sheet_settings(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "シート設定",
        AppLocale::English => "Sheet Settings",
    }
}

pub fn sheet_name(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "シート名",
        AppLocale::English => "Sheet Name",
    }
}

pub fn sheet_name_empty(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "シート名は空欄にできません。",
        AppLocale::English => "Sheet name cannot be empty.",
    }
}

pub fn duration(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "尺",
        AppLocale::English => "Duration",
    }
}

pub fn duration_with_colon(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "尺:",
        AppLocale::English => "Duration:",
    }
}

pub fn duration_input_help(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "`0+12` の形式で入力してください。最小は `0+1`、最大は 100 秒です。",
        AppLocale::English => "Use the `0+12` format. Minimum is `0+1`, maximum is 100 seconds.",
    }
}

pub fn columns(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列数",
        AppLocale::English => "Columns",
    }
}

pub fn new_sheet(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "新規シート",
        AppLocale::English => "New Sheet",
    }
}

pub fn rename_column(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列の名前を変更",
        AppLocale::English => "Rename Column",
    }
}

pub fn new_column_name(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "新しい列名",
        AppLocale::English => "New Column Name",
    }
}

pub fn append_rows_above(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲より上に継ぎ足し",
        AppLocale::English => "Append Rows Above Selection",
    }
}

pub fn append_rows_below(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲より下に継ぎ足し",
        AppLocale::English => "Append Rows Below Selection",
    }
}

pub fn rows_to_append(locale: AppLocale, max_count: usize) -> String {
    match locale {
        AppLocale::Japanese => format!("継ぎ足すコマ数: (1-{max_count})"),
        AppLocale::English => format!("Rows to append: (1-{max_count})"),
    }
}

pub fn append_rows_help(locale: AppLocale, max_count: usize) -> String {
    match locale {
        AppLocale::Japanese => {
            format!("1ページ分まで入力できます。最小は 1、最大は {max_count} コマです。")
        }
        AppLocale::English => {
            format!("You can enter up to one page. Minimum is 1, maximum is {max_count} rows.")
        }
    }
}

pub fn ae_copy_note_1(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => {
            "列ダブルクリックコピーの After Effects 形式は、現在ブラインド方式で出力します。"
        }
        AppLocale::English => {
            "After Effects column copy currently exports blank cels with the Blinds method."
        }
    }
}

pub fn ae_copy_note_2(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ブラインドはブラインドエフェクトを付けてカラセルを表現します。",
        AppLocale::English => "Blinds represents a blank cel by adding a Blinds effect.",
    }
}

pub fn ae_copy_note_3(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "最大コマ数は予約値 100 をカラセルとして書き出します。",
        AppLocale::English => "Comp frame count writes the reserved value 100 as a blank cel.",
    }
}

pub fn recent_files(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "最近開いたファイル...",
        AppLocale::English => "Recent Files...",
    }
}

pub fn no_recent_files(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "最近開いたファイルはありません",
        AppLocale::English => "No recent files",
    }
}

pub fn row_actions(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "行操作",
        AppLocale::English => "Row Actions",
    }
}

pub fn column_actions(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列操作",
        AppLocale::English => "Column Actions",
    }
}

pub fn about_description(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "STSベースのタイムシートエディタ",
        AppLocale::English => "An STS-based timesheet editor",
    }
}

pub fn cmd_open_sheet(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "シートを開く...",
        AppLocale::English => "Open Sheet...",
    }
}

pub fn cmd_new_sheet(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "新規作成...",
        AppLocale::English => "New Sheet...",
    }
}

pub fn cmd_save(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存...",
        AppLocale::English => "Save...",
    }
}

pub fn cmd_save_as(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "別名で保存...",
        AppLocale::English => "Save As...",
    }
}

pub fn cmd_exit(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "終了",
        AppLocale::English => "Quit",
    }
}

pub fn cmd_resize_sheet(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "シート設定...",
        AppLocale::English => "Sheet Settings...",
    }
}

pub fn cmd_send_column_to_ae(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択列をAEへ送信",
        AppLocale::English => "Send Selected Column to AE",
    }
}

pub fn cmd_new_sheet_from_ae(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "AEの選択レイヤーから新規シートを作成",
        AppLocale::English => "Create New Sheet from AE Selection",
    }
}

pub fn cmd_cut(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "カット",
        AppLocale::English => "Cut",
    }
}
pub fn cmd_copy(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "コピー",
        AppLocale::English => "Copy",
    }
}
pub fn cmd_paste(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ペースト",
        AppLocale::English => "Paste",
    }
}
pub fn cmd_repeat_selection_down(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を下に繰り返し複製",
        AppLocale::English => "Repeat Selection Down",
    }
}
pub fn cmd_undo(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "元に戻す",
        AppLocale::English => "Undo",
    }
}
pub fn cmd_redo(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "やり直す",
        AppLocale::English => "Redo",
    }
}
pub fn cmd_move_selection_up(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を上に移動",
        AppLocale::English => "Move Selection Up",
    }
}
pub fn cmd_move_selection_down(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を下に移動",
        AppLocale::English => "Move Selection Down",
    }
}
pub fn cmd_move_selection_left(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を左に移動",
        AppLocale::English => "Move Selection Left",
    }
}
pub fn cmd_move_selection_right(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を右に移動",
        AppLocale::English => "Move Selection Right",
    }
}
pub fn cmd_jump_selection_up(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を上にジャンプ",
        AppLocale::English => "Jump Selection Up",
    }
}
pub fn cmd_jump_selection_down(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を下にジャンプ",
        AppLocale::English => "Jump Selection Down",
    }
}
pub fn cmd_decrease_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を縮小",
        AppLocale::English => "Shrink Selection",
    }
}
pub fn cmd_increase_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を拡張",
        AppLocale::English => "Expand Selection",
    }
}
pub fn cmd_zoom_in(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "拡大",
        AppLocale::English => "Zoom In",
    }
}
pub fn cmd_zoom_out(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "縮小",
        AppLocale::English => "Zoom Out",
    }
}
pub fn cmd_reset_zoom(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "拡大率をリセット",
        AppLocale::English => "Reset Zoom",
    }
}
pub fn cmd_toggle_minimap(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ミニマップを表示",
        AppLocale::English => "Show Minimap",
    }
}
pub fn cmd_toggle_always_on_top(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "常に手前に表示",
        AppLocale::English => "Always on Top",
    }
}
pub fn cmd_open_preferences(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "環境設定...",
        AppLocale::English => "Preferences...",
    }
}
pub fn cmd_show_about(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "NeoSTSについて...",
        AppLocale::English => "About NeoSTS...",
    }
}
pub fn cmd_rename_column(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列の名前を変更...",
        AppLocale::English => "Rename Column...",
    }
}
pub fn cmd_delete_column(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列の削除",
        AppLocale::English => "Delete Column",
    }
}
pub fn cmd_insert_column_left(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "左に列を挿入",
        AppLocale::English => "Insert Column on Left",
    }
}
pub fn cmd_insert_column_right(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "右に列を挿入",
        AppLocale::English => "Insert Column on Right",
    }
}
pub fn cmd_punch_rows(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "中抜き",
        AppLocale::English => "Ignore Rows",
    }
}
pub fn cmd_unpunch_rows(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "中抜きを解除",
        AppLocale::English => "Unignore Rows",
    }
}
pub fn cmd_append_rows_above(locale: AppLocale) -> &'static str {
    append_rows_above(locale)
}
pub fn cmd_append_rows_below(locale: AppLocale) -> &'static str {
    append_rows_below(locale)
}
pub fn cmd_delete_special_rows(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "継ぎ足しシートの削除",
        AppLocale::English => "Delete Appended Rows",
    }
}

pub fn keybind_move_up(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "上へ移動",
        AppLocale::English => "Move Up",
    }
}
pub fn keybind_move_down(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "下へ移動",
        AppLocale::English => "Move Down",
    }
}
pub fn keybind_move_left(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "左へ移動",
        AppLocale::English => "Move Left",
    }
}
pub fn keybind_move_right(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "右へ移動",
        AppLocale::English => "Move Right",
    }
}
pub fn keybind_jump_up(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "上にジャンプ",
        AppLocale::English => "Jump Up",
    }
}
pub fn keybind_jump_down(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "下にジャンプ",
        AppLocale::English => "Jump Down",
    }
}
pub fn keybind_decrease_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を縮小",
        AppLocale::English => "Shrink Selection",
    }
}
pub fn keybind_increase_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を拡張",
        AppLocale::English => "Expand Selection",
    }
}
pub fn keybind_blank_cel_input(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "カラセル入力",
        AppLocale::English => "Blank Cel Input",
    }
}
pub fn keybind_toggle_minimap(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ミニマップ表示切替",
        AppLocale::English => "Toggle Minimap",
    }
}
pub fn keybind_open_preferences(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "環境設定を開く",
        AppLocale::English => "Open Preferences",
    }
}
pub fn prefs_scroll_up_boundary(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "上スクロール境界",
        AppLocale::English => "Upper scroll boundary",
    }
}
pub fn prefs_scroll_down_boundary(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "下スクロール境界",
        AppLocale::English => "Lower scroll boundary",
    }
}
pub fn prefs_continuation_min_frames(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "継続線の最小連続コマ数",
        AppLocale::English => "Minimum continuation run length",
    }
}
pub fn prefs_continuation_zero_note(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "0 にすると継続線を表示しません。",
        AppLocale::English => "Set to 0 to hide continuation lines.",
    }
}
pub fn prefs_continuation_type(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "継続線のタイプ",
        AppLocale::English => "Continuation line style",
    }
}
pub fn prefs_continuation_vertical(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "縦線",
        AppLocale::English => "Vertical",
    }
}
pub fn prefs_continuation_horizontal(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "横線",
        AppLocale::English => "Horizontal",
    }
}
pub fn prefs_open_new_sheet_on_startup(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "起動時に新規シートダイアログを開く",
        AppLocale::English => "Open the New Sheet dialog on startup",
    }
}
pub fn prefs_show_blank_cel_markers(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "カラセル表記と波線を表示",
        AppLocale::English => "Show blank cel markers and wavy lines",
    }
}
pub fn prefs_show_header_ghosts(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列ヘッダにゴースト表示",
        AppLocale::English => "Show header ghosts",
    }
}
pub fn prefs_frame_display(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "コマ表示",
        AppLocale::English => "Frame display",
    }
}
pub fn prefs_frame_number(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "コマ番号",
        AppLocale::English => "Frame numbering",
    }
}
pub fn prefs_frame_number_frames(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "コマ",
        AppLocale::English => "Frame",
    }
}
pub fn prefs_frame_number_absolute(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "通し番号",
        AppLocale::English => "Sequential",
    }
}
pub fn prefs_page_per_seconds(locale: AppLocale, seconds: u32) -> String {
    match locale {
        AppLocale::Japanese => format!("ページ毎 ({}秒)", seconds),
        AppLocale::English => format!("Per page ({} sec)", seconds),
    }
}
pub fn prefs_display_density(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "表示頻度",
        AppLocale::English => "Display density",
    }
}
pub fn prefs_density_all(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "全コマ",
        AppLocale::English => "Every frame",
    }
}
pub fn prefs_density_odd(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "奇数コマ",
        AppLocale::English => "Odd frames",
    }
}
pub fn prefs_density_even(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "偶数コマ",
        AppLocale::English => "Even frames",
    }
}
pub fn prefs_segment_display(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "セグメント表示",
        AppLocale::English => "Segment display",
    }
}
pub fn prefs_unit(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "単位",
        AppLocale::English => "Unit",
    }
}
pub fn prefs_unit_seconds(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "秒",
        AppLocale::English => "Seconds",
    }
}
pub fn prefs_unit_pages(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ページ",
        AppLocale::English => "Pages",
    }
}
pub fn prefs_seconds_per_page(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "1ページ当たりの秒数",
        AppLocale::English => "Seconds per page",
    }
}
pub fn prefs_initial_sheet_duration(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "新規シートの初期尺",
        AppLocale::English => "Default new sheet duration",
    }
}
pub fn prefs_invalid_format(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "形式が正しくありません。",
        AppLocale::English => "The format is invalid.",
    }
}
pub fn prefs_initial_column_count(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "起動時の初期列数",
        AppLocale::English => "Default initial column count",
    }
}
pub fn prefs_sheet_note_fps(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => {
            "fps は新規シートや fps 情報を持たないデータを開くときの初期値です。"
        }
        AppLocale::English => {
            "FPS is the default value for new sheets and for opening data that has no FPS information."
        }
    }
}
pub fn prefs_sheet_note_seconds_per_page(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => {
            "1ページ当たりの秒数は、紙1枚ぶんの区切りと継ぎ足し上限に使われます。"
        }
        AppLocale::English => "Seconds per page is used for page breaks and the append-row limit.",
    }
}
pub fn prefs_sheet_note_duration_independent(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "新規シートの初期尺とは連動しません。",
        AppLocale::English => "It does not automatically change the default new sheet duration.",
    }
}
pub fn prefs_sheet_note_duration_limit(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "新規シート初期尺は `0+1` 以上、100 秒まで設定できます。",
        AppLocale::English => {
            "The default new sheet duration can be set from `0+1` up to 100 seconds."
        }
    }
}
pub fn prefs_color_settings(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "色設定",
        AppLocale::English => "Color Settings",
    }
}
pub fn prefs_reset_colors(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "色を戻す",
        AppLocale::English => "Reset Colors",
    }
}
pub fn prefs_import(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "インポート",
        AppLocale::English => "Import",
    }
}
pub fn prefs_save(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存",
        AppLocale::English => "Save",
    }
}
pub fn prefs_colors(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "色",
        AppLocale::English => "Colors",
    }
}
pub fn prefs_custom_theme_note(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "色を個別に変えるとテーマはカスタムになります。",
        AppLocale::English => "Changing colors individually will switch the theme to Custom.",
    }
}
pub fn prefs_alternate_column_brightness(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "1列おきの明るさ",
        AppLocale::English => "Alternate column brightness",
    }
}
pub fn prefs_per_second_brightness(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "1秒ごとの明るさ",
        AppLocale::English => "Per-second brightness",
    }
}
pub fn prefs_background_color(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "背景色",
        AppLocale::English => "Background color",
    }
}
pub fn prefs_blank_cel_background(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "カラセル背景",
        AppLocale::English => "Blank cel background",
    }
}
pub fn prefs_enabled(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "有効",
        AppLocale::English => "Enabled",
    }
}
pub fn prefs_selection_color(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択色",
        AppLocale::English => "Selection color",
    }
}
pub fn prefs_hover_color(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ホバー色",
        AppLocale::English => "Hover color",
    }
}
pub fn prefs_header_color(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "ヘッダの色",
        AppLocale::English => "Header color",
    }
}
pub fn overlay_drop_folder_title(_locale: AppLocale) -> &'static str {
    "Drop Folder"
}
pub fn overlay_set_image_folder(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "画像フォルダを設定",
        AppLocale::English => "Set image folder",
    }
}
pub fn overlay_only_folders_allowed(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "フォルダだけドロップできます",
        AppLocale::English => "Only folders can be dropped here",
    }
}
pub fn overlay_drop_to_open_title(_locale: AppLocale) -> &'static str {
    "Drop to Open"
}
pub fn overlay_open_sheet_files(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "対応シートファイルを開く",
        AppLocale::English => "Open a supported sheet file",
    }
}
pub fn overlay_only_sheet_files_allowed(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "対応シートファイルだけドロップできます",
        AppLocale::English => "Only supported sheet files can be dropped here",
    }
}
pub fn ae_error_could_not_connect(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "NeoSTSに接続できませんでした",
        AppLocale::English => "Could not connect to NeoSTS",
    }
}
pub fn ae_error_select_composition(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "コンポジションを選択してください",
        AppLocale::English => "Select a composition",
    }
}
pub fn ae_error_select_layers(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "レイヤーを選択してください",
        AppLocale::English => "Select one or more layers",
    }
}
pub fn ae_error_select_av_layer(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "AVLayerを選択してください",
        AppLocale::English => "Select an AVLayer",
    }
}
pub fn ae_error_unknown(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "不明な After Effects エラー",
        AppLocale::English => "Unknown After Effects error",
    }
}

pub fn right_click_column_header(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列ヘッダ上で右クリック",
        AppLocale::English => "Right-click a column header",
    }
}

pub fn right_click_row_header(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "行ヘッダ上で右クリック",
        AppLocale::English => "Right-click a row header",
    }
}

pub fn selected_rows(locale: AppLocale, count: usize) -> String {
    match locale {
        AppLocale::Japanese => format!("選択行数: {count}"),
        AppLocale::English => format!("Selected rows: {count}"),
    }
}

pub fn dirty_title(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存されていない変更があります",
        AppLocale::English => "You have unsaved changes",
    }
}

pub fn status_opened_path(locale: AppLocale, path: &std::path::Path) -> String {
    match locale {
        AppLocale::Japanese => format!("{} を開きました", path.display()),
        AppLocale::English => format!("Opened {}", path.display()),
    }
}

pub fn status_select_column_to_send_ae(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "After Effectsへ送る列を選択してください",
        AppLocale::English => "Select a column to send to After Effects",
    }
}

pub fn status_ae_not_found_for_send(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "After Effects が見つからないため送信できません",
        AppLocale::English => "Cannot send because After Effects was not found",
    }
}

pub fn status_sent_column_to_ae(locale: AppLocale, column_name: &str) -> String {
    match locale {
        AppLocale::Japanese => format!("{column_name} をAfter Effectsへ送信しました"),
        AppLocale::English => format!("Sent {column_name} to After Effects"),
    }
}

pub fn status_failed_send_to_ae(locale: AppLocale, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("After Effectsへの送信に失敗しました: {error}"),
        AppLocale::English => format!("Failed to send to After Effects: {error}"),
    }
}

pub fn status_saved_path(locale: AppLocale, path: &std::path::Path) -> String {
    match locale {
        AppLocale::Japanese => format!("{} を保存しました", path.display()),
        AppLocale::English => format!("Saved {}", path.display()),
    }
}

pub fn status_copied_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲をコピーしました",
        AppLocale::English => "Copied selection",
    }
}
pub fn status_copied_column_values(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列の値をコピーしました",
        AppLocale::English => "Copied column values",
    }
}
pub fn status_cut_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲をカットしました",
        AppLocale::English => "Cut selection",
    }
}
pub fn status_failed_clipboard_read(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "クリップボードの取得に失敗しました",
        AppLocale::English => "Failed to read the clipboard",
    }
}
pub fn status_cannot_paste_into_ignored_rows(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "中抜き行には貼り付けできません",
        AppLocale::English => "Cannot paste into ignored rows",
    }
}
pub fn status_pasted_clipboard(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "クリップボードを貼り付けました",
        AppLocale::English => "Pasted clipboard contents",
    }
}
pub fn status_cannot_repeat_down_ignored_only(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "中抜き行だけでは繰り返し複製できません",
        AppLocale::English => "Cannot repeat down from ignored rows only",
    }
}
pub fn status_no_rows_below_selection(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲の下に複製先がありません",
        AppLocale::English => "There are no rows below the selection to fill",
    }
}
pub fn status_repeated_selection_down(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "選択範囲を下に繰り返し複製しました",
        AppLocale::English => "Repeated the selection downward",
    }
}
pub fn status_deleted_column_values(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "列の値を削除しました",
        AppLocale::English => "Deleted column values",
    }
}
pub fn status_ae_not_found_for_receive(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "After Effects が見つからないため取得できません",
        AppLocale::English => "Cannot fetch because After Effects was not found",
    }
}
pub fn status_created_sheet_from_ae(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "AEの選択レイヤーから新規シートを作成しました",
        AppLocale::English => "Created a new sheet from the AE selection",
    }
}
pub fn status_failed_convert_ae_data(locale: AppLocale, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("AEデータのシート変換に失敗しました: {error}"),
        AppLocale::English => format!("Failed to convert AE data into a sheet: {error}"),
    }
}
pub fn status_failed_receive_from_ae(locale: AppLocale, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("AEからの取得に失敗しました: {error}"),
        AppLocale::English => format!("Failed to receive data from AE: {error}"),
    }
}
pub fn status_failed_open_path(locale: AppLocale, path: &std::path::Path) -> String {
    match locale {
        AppLocale::Japanese => format!("{} を開けませんでした", path.display()),
        AppLocale::English => format!("Could not open {}", path.display()),
    }
}
pub fn file_path_missing_for_save(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存先のファイルパスがありません",
        AppLocale::English => "There is no file path to save to",
    }
}
pub fn file_save_failed(locale: AppLocale, path: &std::path::Path, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("保存に失敗しました: {} ({error})", path.display()),
        AppLocale::English => format!("Failed to save: {} ({error})", path.display()),
    }
}
pub fn startup_load_failed(
    locale: AppLocale,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => format!(
            "起動引数の読み込みに失敗しました: {} ({error})",
            path.display()
        ),
        AppLocale::English => format!(
            "Failed to load startup argument: {} ({error})",
            path.display()
        ),
    }
}
pub fn file_open_failed(locale: AppLocale, path: &std::path::Path, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("読み込みに失敗しました: {} ({error})", path.display()),
        AppLocale::English => format!("Failed to open: {} ({error})", path.display()),
    }
}
pub fn unsupported_file_format_with_path(locale: AppLocale, path: &std::path::Path) -> String {
    match locale {
        AppLocale::Japanese => format!("未対応のファイル形式です: {}", path.display()),
        AppLocale::English => format!("Unsupported file format: {}", path.display()),
    }
}
pub fn unsupported_file_format(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "未対応のファイル形式です",
        AppLocale::English => "Unsupported file format",
    }
}
pub fn theme_colors_reset(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "テーマ色を現在のテーマ既定値に戻しました。",
        AppLocale::English => "Reset theme colors to the current theme defaults.",
    }
}
pub fn theme_imported(locale: AppLocale, path: &std::path::Path) -> String {
    match locale {
        AppLocale::Japanese => format!("テーマをインポートしました: {}", path.display()),
        AppLocale::English => format!("Imported theme: {}", path.display()),
    }
}
pub fn theme_saved(locale: AppLocale, path: &std::path::Path) -> String {
    match locale {
        AppLocale::Japanese => format!("テーマを保存しました: {}", path.display()),
        AppLocale::English => format!("Saved theme: {}", path.display()),
    }
}
pub fn theme_export_cancelled(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "テーマ保存をキャンセルしました。",
        AppLocale::English => "Theme export was cancelled.",
    }
}
pub fn theme_export_serialize_failed(locale: AppLocale, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("テーマ保存データの生成に失敗しました: {error}"),
        AppLocale::English => format!("Failed to build theme export data: {error}"),
    }
}
pub fn theme_export_write_failed(
    locale: AppLocale,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => format!("テーマ保存に失敗しました: {} ({error})", path.display()),
        AppLocale::English => format!("Failed to save theme: {} ({error})", path.display()),
    }
}
pub fn theme_import_cancelled(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "テーマインポートをキャンセルしました。",
        AppLocale::English => "Theme import was cancelled.",
    }
}
pub fn theme_read_failed(locale: AppLocale, path: &std::path::Path, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("テーマ読込に失敗しました: {} ({error})", path.display()),
        AppLocale::English => format!("Failed to read theme: {} ({error})", path.display()),
    }
}
pub fn theme_parse_failed(locale: AppLocale, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => format!("テーマファイルの解析に失敗しました: {error}"),
        AppLocale::English => format!("Failed to parse theme file: {error}"),
    }
}
pub fn theme_library_read_failed(
    locale: AppLocale,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => format!(
            "テーマ保存フォルダの読込に失敗しました: {} ({error})",
            path.display()
        ),
        AppLocale::English => format!("Failed to read theme library: {} ({error})", path.display()),
    }
}
pub fn theme_library_list_failed(
    locale: AppLocale,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => format!(
            "テーマ保存フォルダの一覧取得に失敗しました: {} ({error})",
            path.display()
        ),
        AppLocale::English => format!("Failed to list theme library: {} ({error})", path.display()),
    }
}
pub fn theme_library_dir_create_failed(
    locale: AppLocale,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => format!(
            "テーマ保存フォルダの作成に失敗しました: {} ({error})",
            path.display()
        ),
        AppLocale::English => format!(
            "Failed to create theme library directory: {} ({error})",
            path.display()
        ),
    }
}
pub fn theme_import_copy_failed(
    locale: AppLocale,
    source: &std::path::Path,
    destination: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => format!(
            "テーマのインポートに失敗しました: {} -> {} ({error})",
            source.display(),
            destination.display()
        ),
        AppLocale::English => format!(
            "Failed to import theme: {} -> {} ({error})",
            source.display(),
            destination.display()
        ),
    }
}
pub fn theme_library_dir_unavailable(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "テーマ保存フォルダを取得できませんでした。",
        AppLocale::English => "Could not resolve the theme library directory.",
    }
}

pub fn dirty_body(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "現在の変更はまだ保存されていません。",
        AppLocale::English => "Your current changes have not been saved yet.",
    }
}

pub fn save_and_continue(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存して続行",
        AppLocale::English => "Save and Continue",
    }
}

pub fn discard_open_without_saving(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存せず開く",
        AppLocale::English => "Open Without Saving",
    }
}

pub fn discard_create_without_saving(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存せず新規作成",
        AppLocale::English => "Create Without Saving",
    }
}

pub fn discard_continue_without_saving(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存せず続行",
        AppLocale::English => "Continue Without Saving",
    }
}

pub fn discard_quit_without_saving(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::Japanese => "保存せず終了",
        AppLocale::English => "Quit Without Saving",
    }
}

pub fn builtin_theme_label(theme_id: u8, locale: AppLocale) -> &'static str {
    match (theme_id, locale) {
        (0, _) => system(locale),
        (1, _) => light(locale),
        (2, _) => dark(locale),
        (3, AppLocale::Japanese) => "あさぎ",
        (3, AppLocale::English) => "Light Blue",
        (4, AppLocale::Japanese) => "さくら",
        (4, AppLocale::English) => "Sakura",
        (5, AppLocale::Japanese) => "レモン",
        (5, AppLocale::English) => "Lemon",
        (6, AppLocale::Japanese) => "若草",
        (6, AppLocale::English) => "Light Green",
        (_, _) => custom(locale),
    }
}

pub fn lowlevel_read_file(
    locale: AppLocale,
    format_name: &str,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => {
            format!(
                "{format_name} ファイルの読込に失敗しました: {} ({error})",
                path.display()
            )
        }
        AppLocale::English => {
            format!(
                "Failed to read {format_name} file: {} ({error})",
                path.display()
            )
        }
    }
}

pub fn lowlevel_write_file(
    locale: AppLocale,
    format_name: &str,
    path: &std::path::Path,
    error: impl Display,
) -> String {
    match locale {
        AppLocale::Japanese => {
            format!(
                "{format_name} ファイルの書き込みに失敗しました: {} ({error})",
                path.display()
            )
        }
        AppLocale::English => {
            format!(
                "Failed to write {format_name} file: {} ({error})",
                path.display()
            )
        }
    }
}

pub fn lowlevel_read_bytes(locale: AppLocale, format_name: &str, error: impl Display) -> String {
    match locale {
        AppLocale::Japanese => {
            format!("{format_name} データの読込に失敗しました: {error}")
        }
        AppLocale::English => format!("Failed to read {format_name} data: {error}"),
    }
}

pub fn lowlevel_invalid_sheet_model(
    locale: AppLocale,
    format_name: &str,
    error: &SheetError,
) -> String {
    let detail = localized_sheet_error(locale, error);
    match locale {
        AppLocale::Japanese => {
            format!("{format_name} をシートに変換できませんでした: {detail}")
        }
        AppLocale::English => {
            format!("Failed to convert {format_name} into a sheet: {detail}")
        }
    }
}

pub fn localized_sheet_error(locale: AppLocale, error: &SheetError) -> String {
    match error {
        SheetError::InconsistentColumnLength {
            column_name,
            expected,
            actual,
        } => match locale {
            AppLocale::Japanese => {
                format!(
                    "列 `{column_name}` の値数が不正です。期待値は {expected}、実際は {actual} です"
                )
            }
            AppLocale::English => {
                format!("Column `{column_name}` has {actual} values, expected {expected}")
            }
        },
    }
}
