//! Declarative keyboard and input-rule tables.

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Logical key code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyCode {
    /// Character key.
    Char(char),
    /// Enter.
    Enter,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Escape.
    Escape,
}

/// Platform-independent modifiers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "modifier keys are independent flags"
)]
pub struct Mods {
    /// Control.
    pub ctrl: bool,
    /// Alt.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Meta/Command/Windows.
    pub meta: bool,
}

/// One logical key chord.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    /// Key code.
    pub code: KeyCode,
    /// Modifiers.
    pub mods: Mods,
}

/// Stable command identifier used by keymaps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandId {
    /// Insert text.
    InsertText,
    /// Delete backward.
    DeleteBackward,
    /// Delete forward.
    DeleteForward,
    /// Split block.
    SplitBlock,
    /// Toggle strong.
    ToggleStrong,
    /// Toggle emphasis.
    ToggleEm,
    /// Select all.
    SelectAll,
    /// Undo.
    Undo,
    /// Redo.
    Redo,
}

/// Deterministic key table; the last binding for a key wins.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Keymap {
    entries: Vec<(Key, CommandId)>,
}

impl Keymap {
    /// Creates an empty keymap.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds a chord.
    pub fn bind(&mut self, key: Key, command: CommandId) {
        self.entries.retain(|(current, _)| *current != key);
        self.entries.push((key, command));
        self.entries.sort_by_key(|(key, _)| *key);
    }

    /// Resolves a chord.
    #[must_use]
    pub fn command(&self, key: Key) -> Option<CommandId> {
        self.entries
            .binary_search_by_key(&key, |(current, _)| *current)
            .ok()
            .and_then(|index| self.entries.get(index))
            .map(|(_, command)| *command)
    }

    /// Returns the default keymap.
    #[must_use]
    pub fn default_map() -> Self {
        let mut map = Self::new();
        map.bind(
            Key {
                code: KeyCode::Enter,
                mods: Mods::default(),
            },
            CommandId::SplitBlock,
        );
        map.bind(
            Key {
                code: KeyCode::Backspace,
                mods: Mods::default(),
            },
            CommandId::DeleteBackward,
        );
        map.bind(
            Key {
                code: KeyCode::Delete,
                mods: Mods::default(),
            },
            CommandId::DeleteForward,
        );
        map.bind(
            Key {
                code: KeyCode::Char('a'),
                mods: Mods {
                    ctrl: true,
                    ..Mods::default()
                },
            },
            CommandId::SelectAll,
        );
        map.bind(
            Key {
                code: KeyCode::Char('b'),
                mods: Mods {
                    ctrl: true,
                    ..Mods::default()
                },
            },
            CommandId::ToggleStrong,
        );
        map.bind(
            Key {
                code: KeyCode::Char('i'),
                mods: Mods {
                    ctrl: true,
                    ..Mods::default()
                },
            },
            CommandId::ToggleEm,
        );
        map
    }
}

/// Literal input rule triggered at the start of a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputRule {
    /// Literal prefix.
    pub prefix: Box<str>,
    /// Command emitted when matched.
    pub command: CommandId,
}

impl InputRule {
    /// Creates a rule.
    #[must_use]
    pub fn new(prefix: &str, command: CommandId) -> Self {
        Self {
            prefix: Box::from(prefix),
            command,
        }
    }

    /// Returns whether a text prefix matches.
    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        text.starts_with(&*self.prefix)
    }
}
