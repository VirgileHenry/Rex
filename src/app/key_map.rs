use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

struct KeyShortcut {
    key: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Command {
    Exit,
}

pub struct KeyMap {
    exit: KeyShortcut,
    command_map: HashMap<(KeyCode, KeyModifiers), Command>,
}

impl KeyMap {
    pub fn command(&self, event: crossterm::event::KeyEvent) -> Option<Command> {
        self.command_map
            .get(&(event.code, event.modifiers))
            .cloned()
    }

    fn recompute_command_map(&mut self) {}
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut map = KeyMap {
            exit: KeyShortcut {
                key: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            },
            command_map: HashMap::new(),
        };
        map.recompute_command_map();
        map
    }
}
