use crate::file::cell;

/// History of all the changes for undo / redo
/// Stored as a tree changes, to explore what happened maybe ?
pub struct ChangeHistory {
    last_changes: Vec<Changes>,
    last_undos: Vec<Changes>,
}

impl ChangeHistory {
    pub fn new() -> ChangeHistory {
        ChangeHistory {
            last_changes: Vec::new(),
            last_undos: Vec::new(),
        }
    }

    pub fn push(&mut self, changes: Changes) {
        self.last_changes.push(changes);
    }

    pub fn last_changes(&self) -> Option<&Changes> {
        self.last_changes.last()
    }

    pub fn undo(&mut self) {
        if let Some(last_change) = self.last_changes.pop() {
            self.last_undos.push(last_change.rev());
        }
    }

    pub fn last_undos(&self) -> Option<&Changes> {
        self.last_undos.last()
    }

    pub fn redo(&mut self) {
        if let Some(last_undo) = self.last_undos.pop() {
            self.last_changes.push(last_undo.rev());
        }
    }
}

/// Changes made by a single action
pub enum Changes {
    Empty,
    Atomic(Change),
    Group(Vec<Change>),
}

impl Changes {
    fn rev(self) -> Changes {
        match self {
            Changes::Empty => Changes::Empty,
            Changes::Atomic(change) => Changes::Atomic(change.rev()),
            Changes::Group(changes) => Changes::Group(
                changes
                    .into_iter()
                    .map(|change| change.rev())
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

/// Single atomic change
pub struct Change {
    pub index: cell::CellIndex,
    pub previous_value: Option<cell::Cell>,
    pub new_value: Option<cell::Cell>,
}

impl Change {
    pub fn new(
        index: cell::CellIndex,
        previous_value: Option<cell::Cell>,
        new_value: Option<cell::Cell>,
    ) -> Change {
        Change {
            index,
            previous_value,
            new_value,
        }
    }
    fn rev(self) -> Change {
        Change {
            index: self.index,
            previous_value: self.new_value,
            new_value: self.previous_value,
        }
    }
}
