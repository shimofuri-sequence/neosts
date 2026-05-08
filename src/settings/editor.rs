use crate::{AppLocale, strings};
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    FullFrame,
    Keyframe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipboardExportFormat {
    AfterEffects,
    Autograph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AeKeyframeDataLocale {
    Japanese,
    English,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AeKaraCellMode {
    Blinds,
    MaxFrameCount,
}

pub const DEFAULT_AE_KEYFRAME_VERSION: &str = "7.0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeybindAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    JumpUp,
    JumpDown,
    DecreaseSelection,
    IncreaseSelection,
    KaraZeroInput,
    ToggleMinimap,
    OpenPreferences,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub modifiers: KeyModifiers,
    trigger: KeyBindingTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyBindingTrigger {
    Key(egui::Key),
    Text(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBindings {
    pub move_up: KeyBinding,
    pub move_down: KeyBinding,
    pub move_left: KeyBinding,
    pub move_right: KeyBinding,
    pub jump_up: KeyBinding,
    pub jump_down: KeyBinding,
    pub decrease_selection: KeyBinding,
    pub increase_selection: KeyBinding,
    pub kara_zero_input: KeyBinding,
    pub toggle_minimap: KeyBinding,
    pub open_preferences: KeyBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorSettings {
    pub display_mode: DisplayMode,
    pub clipboard_export_format: ClipboardExportFormat,
    pub ae_keyframe_data_locale: AeKeyframeDataLocale,
    pub ae_keyframe_version: String,
    pub ae_kara_cell_mode: AeKaraCellMode,
    pub keybindings: KeyBindings,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::FullFrame,
            clipboard_export_format: ClipboardExportFormat::AfterEffects,
            ae_keyframe_data_locale: AeKeyframeDataLocale::Japanese,
            ae_keyframe_version: DEFAULT_AE_KEYFRAME_VERSION.to_owned(),
            ae_kara_cell_mode: AeKaraCellMode::Blinds,
            keybindings: KeyBindings::default(),
        }
    }
}

impl EditorSettings {
    pub fn normalized_ae_keyframe_version(&self) -> String {
        normalize_ae_keyframe_version(&self.ae_keyframe_version)
    }

    pub fn reset_ae_settings(&mut self) {
        self.clipboard_export_format = ClipboardExportFormat::AfterEffects;
        self.ae_keyframe_data_locale = AeKeyframeDataLocale::Japanese;
        self.ae_keyframe_version = DEFAULT_AE_KEYFRAME_VERSION.to_owned();
        self.ae_kara_cell_mode = AeKaraCellMode::Blinds;
    }
}

impl ClipboardExportFormat {
    pub const fn storage_id(self) -> u8 {
        match self {
            Self::AfterEffects => 0,
            Self::Autograph => 1,
        }
    }

    pub const fn from_storage_id(id: u8) -> Self {
        match id {
            1 => Self::Autograph,
            _ => Self::AfterEffects,
        }
    }

    pub fn label(self) -> &'static str {
        self.localized_label(AppLocale::Japanese)
    }

    pub fn localized_label(self, _locale: AppLocale) -> &'static str {
        match self {
            Self::AfterEffects => "After Effects",
            Self::Autograph => "Autograph",
        }
    }
}

impl AeKeyframeDataLocale {
    pub const fn storage_id(self) -> u8 {
        match self {
            Self::Japanese => 0,
            Self::English => 1,
        }
    }

    pub const fn from_storage_id(id: u8) -> Self {
        match id {
            1 => Self::English,
            _ => Self::Japanese,
        }
    }

    pub fn label(self) -> &'static str {
        self.localized_label(AppLocale::Japanese)
    }

    pub fn localized_label(self, locale: AppLocale) -> &'static str {
        match self {
            Self::Japanese => strings::ae_keyframe_locale_japanese(locale),
            Self::English => strings::ae_keyframe_locale_english(locale),
        }
    }
}

impl AeKaraCellMode {
    pub const fn storage_id(self) -> u8 {
        match self {
            Self::Blinds => 0,
            Self::MaxFrameCount => 1,
        }
    }

    pub const fn from_storage_id(id: u8) -> Self {
        match id {
            1 => Self::MaxFrameCount,
            _ => Self::Blinds,
        }
    }

    pub fn label(self) -> &'static str {
        self.localized_label(AppLocale::Japanese)
    }

    pub fn localized_label(self, locale: AppLocale) -> &'static str {
        match self {
            Self::Blinds => strings::ae_kara_cell_mode_blinds(locale),
            Self::MaxFrameCount => strings::ae_kara_cell_mode_max_frame_count(locale),
        }
    }
}

impl KeyModifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };

    pub const fn ctrl() -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: false,
        }
    }
}

impl KeyBinding {
    pub const fn new(key: egui::Key) -> Self {
        Self {
            modifiers: KeyModifiers::NONE,
            trigger: KeyBindingTrigger::Key(key),
        }
    }

    pub const fn with_modifiers(key: egui::Key, modifiers: KeyModifiers) -> Self {
        Self {
            modifiers,
            trigger: KeyBindingTrigger::Key(key),
        }
    }

    pub const fn text(text: &'static str) -> Self {
        Self {
            modifiers: KeyModifiers::NONE,
            trigger: KeyBindingTrigger::Text(text),
        }
    }

    pub const fn with_shift(self) -> Self {
        Self {
            modifiers: KeyModifiers {
                ctrl: self.modifiers.ctrl,
                alt: self.modifiers.alt,
                shift: true,
            },
            trigger: self.trigger,
        }
    }

    pub fn matches(self, input: &egui::InputState) -> bool {
        match self.trigger {
            KeyBindingTrigger::Key(key) => {
                input.key_pressed(key)
                    && primary_modifier(input.modifiers) == self.modifiers.ctrl
                    && input.modifiers.alt == self.modifiers.alt
                    && input.modifiers.shift == self.modifiers.shift
            }
            KeyBindingTrigger::Text(text) => input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Text(event_text) if event_text == text)),
        }
    }

    pub fn from_modifiers(key: egui::Key, modifiers: egui::Modifiers) -> Option<Self> {
        if key_name(key).is_none() {
            return None;
        }

        Some(Self {
            modifiers: KeyModifiers {
                ctrl: primary_modifier(modifiers),
                alt: modifiers.alt,
                shift: modifiers.shift,
            },
            trigger: KeyBindingTrigger::Key(key),
        })
    }

    pub fn from_text(text: &str) -> Option<Self> {
        match text {
            "*" => Some(Self::text("*")),
            _ => None,
        }
    }

    pub fn display_text(self) -> String {
        if let KeyBindingTrigger::Text(text) = self.trigger {
            return text.to_owned();
        }

        let mut parts = Vec::new();
        if self.modifiers.ctrl {
            parts.push(primary_modifier_name());
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        let key_name = match self.trigger {
            KeyBindingTrigger::Key(key) => key_name(key).unwrap_or("Unknown"),
            KeyBindingTrigger::Text(_) => "Unknown",
        };
        parts.push(key_name);
        parts.join("+")
    }

    pub fn storage_value(self) -> String {
        match self.trigger {
            KeyBindingTrigger::Key(key) => {
                let key = key_identifier(key).unwrap_or("Unknown");
                format!(
                    "{}:{}:{}:{}",
                    if self.modifiers.ctrl { 1 } else { 0 },
                    if self.modifiers.alt { 1 } else { 0 },
                    if self.modifiers.shift { 1 } else { 0 },
                    key
                )
            }
            KeyBindingTrigger::Text(text) => format!("text:{text}"),
        }
    }

    pub fn from_storage_value(value: &str) -> Option<Self> {
        if let Some(text) = value.strip_prefix("text:") {
            return Self::from_text(text);
        }

        let mut parts = value.split(':');
        let ctrl = parts.next()? == "1";
        let alt = parts.next()? == "1";
        let shift = parts.next()? == "1";
        let key = key_from_identifier(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }

        Some(Self {
            modifiers: KeyModifiers { ctrl, alt, shift },
            trigger: KeyBindingTrigger::Key(key),
        })
    }
}

fn primary_modifier(modifiers: egui::Modifiers) -> bool {
    modifiers.command
}

fn primary_modifier_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Cmd"
    }

    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl"
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_up: KeyBinding::new(egui::Key::ArrowUp),
            move_down: KeyBinding::new(egui::Key::ArrowDown),
            move_left: KeyBinding::new(egui::Key::ArrowLeft),
            move_right: KeyBinding::new(egui::Key::ArrowRight),
            jump_up: KeyBinding::new(egui::Key::J),
            jump_down: KeyBinding::new(egui::Key::K),
            decrease_selection: Self::default_decrease_selection_binding(),
            increase_selection: Self::default_increase_selection_binding(),
            kara_zero_input: Self::default_kara_zero_input_binding(),
            toggle_minimap: KeyBinding::with_modifiers(egui::Key::M, KeyModifiers::ctrl()),
            open_preferences: KeyBinding::with_modifiers(egui::Key::Comma, KeyModifiers::ctrl()),
        }
    }
}

impl KeybindAction {
    pub const ALL: [Self; 11] = [
        Self::MoveUp,
        Self::MoveDown,
        Self::MoveLeft,
        Self::MoveRight,
        Self::JumpUp,
        Self::JumpDown,
        Self::DecreaseSelection,
        Self::IncreaseSelection,
        Self::KaraZeroInput,
        Self::ToggleMinimap,
        Self::OpenPreferences,
    ];

    pub fn label(self) -> &'static str {
        self.localized_label(AppLocale::Japanese)
    }

    pub fn localized_label(self, locale: AppLocale) -> &'static str {
        match self {
            Self::MoveUp => strings::keybind_move_up(locale),
            Self::MoveDown => strings::keybind_move_down(locale),
            Self::MoveLeft => strings::keybind_move_left(locale),
            Self::MoveRight => strings::keybind_move_right(locale),
            Self::JumpUp => strings::keybind_jump_up(locale),
            Self::JumpDown => strings::keybind_jump_down(locale),
            Self::DecreaseSelection => strings::keybind_decrease_selection(locale),
            Self::IncreaseSelection => strings::keybind_increase_selection(locale),
            Self::KaraZeroInput => strings::keybind_blank_cel_input(locale),
            Self::ToggleMinimap => strings::keybind_toggle_minimap(locale),
            Self::OpenPreferences => strings::keybind_open_preferences(locale),
        }
    }
}

impl KeyBindings {
    pub const fn legacy_default_decrease_selection_binding() -> KeyBinding {
        KeyBinding::new(egui::Key::ArrowUp).with_shift()
    }

    pub const fn legacy_default_increase_selection_binding() -> KeyBinding {
        KeyBinding::new(egui::Key::ArrowDown).with_shift()
    }

    pub const fn default_decrease_selection_binding() -> KeyBinding {
        KeyBinding::new(egui::Key::Slash)
    }

    pub const fn default_increase_selection_binding() -> KeyBinding {
        KeyBinding::text("*")
    }

    pub const fn default_kara_zero_input_binding() -> KeyBinding {
        KeyBinding::new(egui::Key::Period)
    }

    pub fn binding(&self, action: KeybindAction) -> KeyBinding {
        match action {
            KeybindAction::MoveUp => self.move_up,
            KeybindAction::MoveDown => self.move_down,
            KeybindAction::MoveLeft => self.move_left,
            KeybindAction::MoveRight => self.move_right,
            KeybindAction::JumpUp => self.jump_up,
            KeybindAction::JumpDown => self.jump_down,
            KeybindAction::DecreaseSelection => self.decrease_selection,
            KeybindAction::IncreaseSelection => self.increase_selection,
            KeybindAction::KaraZeroInput => self.kara_zero_input,
            KeybindAction::ToggleMinimap => self.toggle_minimap,
            KeybindAction::OpenPreferences => self.open_preferences,
        }
    }

    pub fn set_binding(&mut self, action: KeybindAction, binding: KeyBinding) {
        match action {
            KeybindAction::MoveUp => self.move_up = binding,
            KeybindAction::MoveDown => self.move_down = binding,
            KeybindAction::MoveLeft => self.move_left = binding,
            KeybindAction::MoveRight => self.move_right = binding,
            KeybindAction::JumpUp => self.jump_up = binding,
            KeybindAction::JumpDown => self.jump_down = binding,
            KeybindAction::DecreaseSelection => self.decrease_selection = binding,
            KeybindAction::IncreaseSelection => self.increase_selection = binding,
            KeybindAction::KaraZeroInput => self.kara_zero_input = binding,
            KeybindAction::ToggleMinimap => self.toggle_minimap = binding,
            KeybindAction::OpenPreferences => self.open_preferences = binding,
        }
    }

    pub fn migrated_binding(action: KeybindAction, binding: KeyBinding) -> KeyBinding {
        match action {
            KeybindAction::DecreaseSelection
                if binding == Self::legacy_default_decrease_selection_binding() =>
            {
                Self::default_decrease_selection_binding()
            }
            KeybindAction::IncreaseSelection
                if binding == Self::legacy_default_increase_selection_binding() =>
            {
                Self::default_increase_selection_binding()
            }
            _ => binding,
        }
    }
}

pub fn keybind_action_id(action: KeybindAction) -> u8 {
    match action {
        KeybindAction::MoveUp => 0,
        KeybindAction::MoveDown => 1,
        KeybindAction::MoveLeft => 2,
        KeybindAction::MoveRight => 3,
        KeybindAction::JumpUp => 4,
        KeybindAction::JumpDown => 5,
        KeybindAction::DecreaseSelection => 6,
        KeybindAction::IncreaseSelection => 7,
        KeybindAction::KaraZeroInput => 8,
        KeybindAction::ToggleMinimap => 9,
        KeybindAction::OpenPreferences => 10,
    }
}

pub fn keybind_action_from_id(id: u8) -> KeybindAction {
    match id {
        1 => KeybindAction::MoveDown,
        2 => KeybindAction::MoveLeft,
        3 => KeybindAction::MoveRight,
        4 => KeybindAction::JumpUp,
        5 => KeybindAction::JumpDown,
        6 => KeybindAction::DecreaseSelection,
        7 => KeybindAction::IncreaseSelection,
        8 => KeybindAction::KaraZeroInput,
        9 => KeybindAction::ToggleMinimap,
        10 => KeybindAction::OpenPreferences,
        _ => KeybindAction::MoveUp,
    }
}

pub fn key_name(key: egui::Key) -> Option<&'static str> {
    match key {
        egui::Key::ArrowUp => Some("Up"),
        egui::Key::ArrowDown => Some("Down"),
        egui::Key::ArrowLeft => Some("Left"),
        egui::Key::ArrowRight => Some("Right"),
        egui::Key::Escape => Some("Esc"),
        egui::Key::Tab => Some("Tab"),
        egui::Key::Backspace => Some("Backspace"),
        egui::Key::Enter => Some("Enter"),
        egui::Key::Space => Some("Space"),
        egui::Key::Comma => Some(","),
        egui::Key::Plus => Some("+"),
        egui::Key::Minus => Some("-"),
        egui::Key::Period => Some("."),
        egui::Key::Slash => Some("/"),
        egui::Key::Num0 => Some("0"),
        egui::Key::Num1 => Some("1"),
        egui::Key::Num2 => Some("2"),
        egui::Key::Num3 => Some("3"),
        egui::Key::Num4 => Some("4"),
        egui::Key::Num5 => Some("5"),
        egui::Key::Num6 => Some("6"),
        egui::Key::Num7 => Some("7"),
        egui::Key::Num8 => Some("8"),
        egui::Key::Num9 => Some("9"),
        egui::Key::A => Some("A"),
        egui::Key::B => Some("B"),
        egui::Key::C => Some("C"),
        egui::Key::D => Some("D"),
        egui::Key::E => Some("E"),
        egui::Key::F => Some("F"),
        egui::Key::G => Some("G"),
        egui::Key::H => Some("H"),
        egui::Key::I => Some("I"),
        egui::Key::J => Some("J"),
        egui::Key::K => Some("K"),
        egui::Key::L => Some("L"),
        egui::Key::M => Some("M"),
        egui::Key::N => Some("N"),
        egui::Key::O => Some("O"),
        egui::Key::P => Some("P"),
        egui::Key::Q => Some("Q"),
        egui::Key::R => Some("R"),
        egui::Key::S => Some("S"),
        egui::Key::T => Some("T"),
        egui::Key::U => Some("U"),
        egui::Key::V => Some("V"),
        egui::Key::W => Some("W"),
        egui::Key::X => Some("X"),
        egui::Key::Y => Some("Y"),
        egui::Key::Z => Some("Z"),
        _ => None,
    }
}

fn key_identifier(key: egui::Key) -> Option<&'static str> {
    match key {
        egui::Key::ArrowUp => Some("ArrowUp"),
        egui::Key::ArrowDown => Some("ArrowDown"),
        egui::Key::ArrowLeft => Some("ArrowLeft"),
        egui::Key::ArrowRight => Some("ArrowRight"),
        egui::Key::Escape => Some("Escape"),
        egui::Key::Tab => Some("Tab"),
        egui::Key::Backspace => Some("Backspace"),
        egui::Key::Enter => Some("Enter"),
        egui::Key::Space => Some("Space"),
        egui::Key::Comma => Some("Comma"),
        egui::Key::Plus => Some("Plus"),
        egui::Key::Minus => Some("Minus"),
        egui::Key::Period => Some("Period"),
        egui::Key::Slash => Some("Slash"),
        egui::Key::A => Some("A"),
        egui::Key::B => Some("B"),
        egui::Key::C => Some("C"),
        egui::Key::D => Some("D"),
        egui::Key::E => Some("E"),
        egui::Key::F => Some("F"),
        egui::Key::G => Some("G"),
        egui::Key::H => Some("H"),
        egui::Key::I => Some("I"),
        egui::Key::J => Some("J"),
        egui::Key::K => Some("K"),
        egui::Key::L => Some("L"),
        egui::Key::M => Some("M"),
        egui::Key::N => Some("N"),
        egui::Key::O => Some("O"),
        egui::Key::P => Some("P"),
        egui::Key::Q => Some("Q"),
        egui::Key::R => Some("R"),
        egui::Key::S => Some("S"),
        egui::Key::T => Some("T"),
        egui::Key::U => Some("U"),
        egui::Key::V => Some("V"),
        egui::Key::W => Some("W"),
        egui::Key::X => Some("X"),
        egui::Key::Y => Some("Y"),
        egui::Key::Z => Some("Z"),
        egui::Key::Num0 => Some("Num0"),
        egui::Key::Num1 => Some("Num1"),
        egui::Key::Num2 => Some("Num2"),
        egui::Key::Num3 => Some("Num3"),
        egui::Key::Num4 => Some("Num4"),
        egui::Key::Num5 => Some("Num5"),
        egui::Key::Num6 => Some("Num6"),
        egui::Key::Num7 => Some("Num7"),
        egui::Key::Num8 => Some("Num8"),
        egui::Key::Num9 => Some("Num9"),
        _ => None,
    }
}

fn key_from_identifier(value: &str) -> Option<egui::Key> {
    match value {
        "ArrowUp" => Some(egui::Key::ArrowUp),
        "ArrowDown" => Some(egui::Key::ArrowDown),
        "ArrowLeft" => Some(egui::Key::ArrowLeft),
        "ArrowRight" => Some(egui::Key::ArrowRight),
        "Escape" => Some(egui::Key::Escape),
        "Tab" => Some(egui::Key::Tab),
        "Backspace" => Some(egui::Key::Backspace),
        "Enter" => Some(egui::Key::Enter),
        "Space" => Some(egui::Key::Space),
        "Comma" => Some(egui::Key::Comma),
        "Plus" => Some(egui::Key::Plus),
        "Minus" => Some(egui::Key::Minus),
        "Period" => Some(egui::Key::Period),
        "Slash" => Some(egui::Key::Slash),
        "A" => Some(egui::Key::A),
        "B" => Some(egui::Key::B),
        "C" => Some(egui::Key::C),
        "D" => Some(egui::Key::D),
        "E" => Some(egui::Key::E),
        "F" => Some(egui::Key::F),
        "G" => Some(egui::Key::G),
        "H" => Some(egui::Key::H),
        "I" => Some(egui::Key::I),
        "J" => Some(egui::Key::J),
        "K" => Some(egui::Key::K),
        "L" => Some(egui::Key::L),
        "M" => Some(egui::Key::M),
        "N" => Some(egui::Key::N),
        "O" => Some(egui::Key::O),
        "P" => Some(egui::Key::P),
        "Q" => Some(egui::Key::Q),
        "R" => Some(egui::Key::R),
        "S" => Some(egui::Key::S),
        "T" => Some(egui::Key::T),
        "U" => Some(egui::Key::U),
        "V" => Some(egui::Key::V),
        "W" => Some(egui::Key::W),
        "X" => Some(egui::Key::X),
        "Y" => Some(egui::Key::Y),
        "Z" => Some(egui::Key::Z),
        "Num0" => Some(egui::Key::Num0),
        "Num1" => Some(egui::Key::Num1),
        "Num2" => Some(egui::Key::Num2),
        "Num3" => Some(egui::Key::Num3),
        "Num4" => Some(egui::Key::Num4),
        "Num5" => Some(egui::Key::Num5),
        "Num6" => Some(egui::Key::Num6),
        "Num7" => Some(egui::Key::Num7),
        "Num8" => Some(egui::Key::Num8),
        "Num9" => Some(egui::Key::Num9),
        _ => None,
    }
}

fn normalize_ae_keyframe_version(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.is_empty()
        && trimmed.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        && trimmed.chars().any(|ch| ch.is_ascii_digit())
    {
        trimmed.to_owned()
    } else {
        DEFAULT_AE_KEYFRAME_VERSION.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AeKaraCellMode, AeKeyframeDataLocale, ClipboardExportFormat, DEFAULT_AE_KEYFRAME_VERSION,
        EditorSettings, KeyBinding, KeyBindings, KeybindAction, normalize_ae_keyframe_version,
    };

    #[test]
    fn text_binding_roundtrips_storage() {
        let binding = KeyBinding::text("*");
        assert_eq!(binding.storage_value(), "text:*");
        assert_eq!(KeyBinding::from_storage_value("text:*"), Some(binding));
        assert_eq!(binding.display_text(), "*");
    }

    #[test]
    fn legacy_selection_bindings_migrate_to_new_defaults() {
        assert_eq!(
            KeyBindings::migrated_binding(
                KeybindAction::DecreaseSelection,
                KeyBindings::legacy_default_decrease_selection_binding(),
            ),
            KeyBindings::default_decrease_selection_binding()
        );
        assert_eq!(
            KeyBindings::migrated_binding(
                KeybindAction::IncreaseSelection,
                KeyBindings::legacy_default_increase_selection_binding(),
            ),
            KeyBindings::default_increase_selection_binding()
        );
    }

    #[test]
    fn normalizes_ae_keyframe_version() {
        assert_eq!(normalize_ae_keyframe_version(" 8.0 "), "8.0");
        assert_eq!(
            normalize_ae_keyframe_version("nine"),
            DEFAULT_AE_KEYFRAME_VERSION
        );
        assert_eq!(
            normalize_ae_keyframe_version(""),
            DEFAULT_AE_KEYFRAME_VERSION
        );
    }

    #[test]
    fn resets_only_ae_settings_to_defaults() {
        let mut settings = EditorSettings {
            display_mode: super::DisplayMode::Keyframe,
            clipboard_export_format: ClipboardExportFormat::Autograph,
            ae_keyframe_data_locale: AeKeyframeDataLocale::English,
            ae_keyframe_version: "9.0".to_owned(),
            ae_kara_cell_mode: AeKaraCellMode::MaxFrameCount,
            keybindings: KeyBindings::default(),
        };

        settings.reset_ae_settings();

        assert_eq!(settings.display_mode, super::DisplayMode::Keyframe);
        assert_eq!(
            settings.clipboard_export_format,
            ClipboardExportFormat::AfterEffects
        );
        assert_eq!(
            settings.ae_keyframe_data_locale,
            AeKeyframeDataLocale::Japanese
        );
        assert_eq!(settings.ae_keyframe_version, DEFAULT_AE_KEYFRAME_VERSION);
        assert_eq!(settings.ae_kara_cell_mode, AeKaraCellMode::Blinds);
    }
}
