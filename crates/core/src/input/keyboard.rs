//! Keyboard event types.

use bitflags::bitflags;

bitflags! {
    /// Keyboard modifier flags.
    ///
    /// These map to the Kitty keyboard protocol modifier bitfield:
    /// shift(1), alt(2), ctrl(4), super(8), hyper(16), meta(32),
    /// caps_lock(64), num_lock(128).
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct KeyModifiers: u8 {
        /// Shift key.
        const SHIFT = 0b0000_0001;
        /// Alt/Option key.
        const ALT = 0b0000_0010;
        /// Control key.
        const CTRL = 0b0000_0100;
        /// Super/Windows/Command key.
        const SUPER = 0b0000_1000;
        /// Hyper key (rare, X11/XKB systems).
        const HYPER = 0b0001_0000;
        /// Meta key (rare, distinct from Alt on some systems).
        const META = 0b0010_0000;
        /// Caps Lock state.
        const CAPS_LOCK = 0b0100_0000;
        /// Num Lock state.
        const NUM_LOCK = 0b1000_0000;
    }
}

/// Whether the key was pressed, repeated, or released.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyEventType {
    /// Key was pressed (the default if no event type is reported).
    #[default]
    Press,
    /// Key auto-repeat event.
    Repeat,
    /// Key was released.
    Release,
}

/// Which protocol produced this key event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeySource {
    /// Legacy / raw terminal protocol (VT sequences, modifyOtherKeys, etc.).
    #[default]
    Raw,
    /// Kitty keyboard protocol (CSI u encoding).
    Kitty,
}

/// A key code representing a keyboard key.
///
/// Covers standard PC keys, keypad variants, media keys, modifier keys,
/// and ISO level keys from the Kitty keyboard protocol functional key
/// definitions table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    /// Function key F1-F35 (35 is maximum in Kitty protocol).
    F(u8),
    /// A character key (includes space).
    Char(char),
    Escape,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    /// Keypad Begin (numpad 5 without numlock).
    KeypadBegin,
    /// Null (Ctrl+Space or Ctrl+@).
    Null,

    // ── Keypad variants ────────────────────────────────────────────────
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    KpSeparator,
    KpLeft,
    KpRight,
    KpUp,
    KpDown,
    KpHome,
    KpEnd,
    KpPageUp,
    KpPageDown,
    KpInsert,
    KpDelete,

    // ── Modifier keys (reported when all-keys-as-escapes is enabled) ──
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    LeftSuper,
    RightSuper,
    LeftHyper,
    RightHyper,
    LeftMeta,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,

    // ── Media keys ─────────────────────────────────────────────────────
    MediaPlay,
    MediaPause,
    MediaPlayPause,
    MediaReverse,
    MediaStop,
    MediaFastForward,
    MediaRewind,
    MediaTrackNext,
    MediaTrackPrevious,
    MediaRecord,
    MediaVolumeDown,
    MediaVolumeUp,
    MediaVolumeMute,
    MediaLowerVolume,
    MediaRaiseVolume,
}

impl KeyCode {
    /// Check if this is a function key.
    #[must_use]
    pub fn is_function_key(&self) -> bool {
        matches!(self, Self::F(_))
    }

    /// Check if this is a character key.
    #[must_use]
    pub fn is_char(&self) -> bool {
        matches!(self, Self::Char(_))
    }

    /// Check if this is a navigation key (arrows, home, end, page up/down).
    #[must_use]
    pub fn is_navigation(&self) -> bool {
        matches!(
            self,
            Self::Left
                | Self::Right
                | Self::Up
                | Self::Down
                | Self::Home
                | Self::End
                | Self::PageUp
                | Self::PageDown
        )
    }

    /// Check if this is a keypad key.
    #[must_use]
    pub fn is_keypad(&self) -> bool {
        matches!(
            self,
            Self::Kp0
                | Self::Kp1
                | Self::Kp2
                | Self::Kp3
                | Self::Kp4
                | Self::Kp5
                | Self::Kp6
                | Self::Kp7
                | Self::Kp8
                | Self::Kp9
                | Self::KpDecimal
                | Self::KpDivide
                | Self::KpMultiply
                | Self::KpSubtract
                | Self::KpAdd
                | Self::KpEnter
                | Self::KpEqual
                | Self::KpSeparator
                | Self::KpLeft
                | Self::KpRight
                | Self::KpUp
                | Self::KpDown
                | Self::KpHome
                | Self::KpEnd
                | Self::KpPageUp
                | Self::KpPageDown
                | Self::KpInsert
                | Self::KpDelete
                | Self::KeypadBegin
        )
    }

    /// Check if this is a modifier key (Shift, Ctrl, Alt, Super, etc.).
    #[must_use]
    pub fn is_modifier_key(&self) -> bool {
        matches!(
            self,
            Self::LeftShift
                | Self::RightShift
                | Self::LeftControl
                | Self::RightControl
                | Self::LeftAlt
                | Self::RightAlt
                | Self::LeftSuper
                | Self::RightSuper
                | Self::LeftHyper
                | Self::RightHyper
                | Self::LeftMeta
                | Self::RightMeta
                | Self::IsoLevel3Shift
                | Self::IsoLevel5Shift
        )
    }

    /// Check if this is a media key.
    #[must_use]
    pub fn is_media_key(&self) -> bool {
        matches!(
            self,
            Self::MediaPlay
                | Self::MediaPause
                | Self::MediaPlayPause
                | Self::MediaReverse
                | Self::MediaStop
                | Self::MediaFastForward
                | Self::MediaRewind
                | Self::MediaTrackNext
                | Self::MediaTrackPrevious
                | Self::MediaRecord
                | Self::MediaVolumeDown
                | Self::MediaVolumeUp
                | Self::MediaVolumeMute
                | Self::MediaLowerVolume
                | Self::MediaRaiseVolume
        )
    }

    /// Get the character if this is a character key.
    #[must_use]
    pub fn char(&self) -> Option<char> {
        match self {
            Self::Char(c) => Some(*c),
            _ => None,
        }
    }
}

/// A keyboard event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The logical key code.
    pub code: KeyCode,
    /// Modifier keys held.
    pub modifiers: KeyModifiers,
    /// Whether this is a press, repeat, or release event.
    pub event_type: KeyEventType,
    /// Which protocol produced this event.
    pub source: KeySource,
}

impl KeyEvent {
    /// Create a new key event.
    #[must_use]
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            code,
            modifiers,
            event_type: KeyEventType::Press,
            source: KeySource::Raw,
        }
    }

    /// Create a new key event with full metadata.
    #[must_use]
    pub fn with_event(
        code: KeyCode,
        modifiers: KeyModifiers,
        event_type: KeyEventType,
        source: KeySource,
    ) -> Self {
        Self {
            code,
            modifiers,
            event_type,
            source,
        }
    }

    /// Create a key event with no modifiers.
    #[must_use]
    pub fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::empty())
    }

    /// Create a character key event.
    #[must_use]
    pub fn char(c: char) -> Self {
        Self::key(KeyCode::Char(c))
    }

    /// Create a Ctrl+key event.
    #[must_use]
    pub fn with_ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CTRL)
    }

    /// Create an Alt+key event.
    #[must_use]
    pub fn with_alt(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::ALT)
    }

    /// Set the event type (builder pattern).
    #[must_use]
    pub fn with_event_type(mut self, event_type: KeyEventType) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set the source protocol (builder pattern).
    #[must_use]
    pub fn with_source(mut self, source: KeySource) -> Self {
        self.source = source;
        self
    }

    /// Check if Shift is held.
    #[must_use]
    pub fn shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }

    /// Check if Ctrl is held.
    #[must_use]
    pub fn ctrl(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CTRL)
    }

    /// Check if Alt is held.
    #[must_use]
    pub fn alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }

    /// Check if this matches a specific key with optional modifiers.
    #[must_use]
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code == code && self.modifiers == modifiers
    }

    /// Check if this is Ctrl+C.
    #[must_use]
    pub fn is_ctrl_c(&self) -> bool {
        self.matches(KeyCode::Char('c'), KeyModifiers::CTRL)
    }

    /// Check if this is Ctrl+D.
    #[must_use]
    pub fn is_ctrl_d(&self) -> bool {
        self.matches(KeyCode::Char('d'), KeyModifiers::CTRL)
    }

    /// Check if this is Escape.
    #[must_use]
    pub fn is_esc(&self) -> bool {
        self.code == KeyCode::Escape
    }

    /// Check if this is Enter.
    #[must_use]
    pub fn is_enter(&self) -> bool {
        self.code == KeyCode::Enter
    }

    /// Check if this is a press event.
    #[must_use]
    pub fn is_press(&self) -> bool {
        self.event_type == KeyEventType::Press
    }

    /// Check if this is a repeat event.
    #[must_use]
    pub fn is_repeat(&self) -> bool {
        self.event_type == KeyEventType::Repeat
    }

    /// Check if this is a release event.
    #[must_use]
    pub fn is_release(&self) -> bool {
        self.event_type == KeyEventType::Release
    }
}

impl From<char> for KeyEvent {
    fn from(c: char) -> Self {
        Self::char(c)
    }
}

impl From<KeyCode> for KeyEvent {
    fn from(code: KeyCode) -> Self {
        Self::key(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent::char('a');
        assert_eq!(event.code, KeyCode::Char('a'));
        assert!(event.modifiers.is_empty());
        assert_eq!(event.event_type, KeyEventType::Press);
        assert_eq!(event.source, KeySource::Raw);
    }

    #[test]
    fn test_key_event_modifiers() {
        let event = KeyEvent::with_ctrl(KeyCode::Char('c'));
        assert!(event.ctrl());
        assert!(!event.shift());
        assert!(!event.alt());
        assert!(event.is_ctrl_c());
    }

    #[test]
    fn test_key_code_checks() {
        assert!(KeyCode::F(1).is_function_key());
        assert!(KeyCode::Char('x').is_char());
        assert!(KeyCode::Up.is_navigation());
        assert!(!KeyCode::Enter.is_navigation());
    }

    #[test]
    fn test_key_event_from_char() {
        let event: KeyEvent = 'z'.into();
        assert_eq!(event.code, KeyCode::Char('z'));
    }

    #[test]
    fn test_event_type_builder() {
        let event = KeyEvent::key(KeyCode::Enter)
            .with_event_type(KeyEventType::Release)
            .with_source(KeySource::Kitty);
        assert!(event.is_release());
        assert_eq!(event.source, KeySource::Kitty);
    }

    #[test]
    fn test_keycode_categories() {
        assert!(KeyCode::Kp5.is_keypad());
        assert!(KeyCode::KpEnter.is_keypad());
        assert!(!KeyCode::Enter.is_keypad());

        assert!(KeyCode::LeftShift.is_modifier_key());
        assert!(KeyCode::RightControl.is_modifier_key());
        assert!(!KeyCode::Enter.is_modifier_key()); // Enter is not a modifier key

        assert!(KeyCode::MediaPlay.is_media_key());
        assert!(!KeyCode::F(5).is_media_key());
    }

    #[test]
    fn test_modifiers_all_bits() {
        let all = KeyModifiers::all();
        assert!(all.contains(KeyModifiers::SHIFT));
        assert!(all.contains(KeyModifiers::ALT));
        assert!(all.contains(KeyModifiers::CTRL));
        assert!(all.contains(KeyModifiers::SUPER));
        assert!(all.contains(KeyModifiers::HYPER));
        assert!(all.contains(KeyModifiers::META));
        assert!(all.contains(KeyModifiers::CAPS_LOCK));
        assert!(all.contains(KeyModifiers::NUM_LOCK));

        assert_eq!(all.bits(), 0xFF);
    }

    #[test]
    fn test_key_event_type_defaults() {
        assert_eq!(KeyEventType::default(), KeyEventType::Press);
        assert_eq!(KeySource::default(), KeySource::Raw);
    }
}
