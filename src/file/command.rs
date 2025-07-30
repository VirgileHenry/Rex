/// When moving (or validating, editing, etc..) cells, where do we go
pub enum SelectionDirection {
    /// Don't move at all
    Stay,
    /// Go to the next cell group (left)
    Next,
    /// Go back on the next line (down)
    Return,
}

pub enum Command {
    RedrawRequest,
    WriteCells {
        cells: crate::file::cell::CellRect,
        content: crate::file::cell::Cell,
        next_selection: SelectionDirection,
    },
    DeleteCells {
        cells: crate::file::cell::CellRect,
        next_selection: SelectionDirection,
    },
    CopyCells {
        cells: crate::file::cell::CellRect,
    },
    PasteCells {
        cells: crate::file::cell::CellRect,
        paste_buffer: String,
    },
    Undo,
    Redo,
}

impl super::FileApp {
    pub fn execute_command(&mut self, cmd: Command, info: &mut String) {
        use super::cell;
        use super::change_history;

        match cmd {
            Command::WriteCells {
                cells,
                content,
                next_selection,
            } => {
                let mut changes = Vec::new();
                for cell_x in cells.x..cells.x + cells.width {
                    for cell_y in cells.y..cells.y + cells.height {
                        let key = cell::CellIndex::new(cell_x, cell_y);
                        changes.push(change_history::Change::new(
                            key,
                            self.content.get(&key).cloned(),
                            Some(content.clone()),
                        ));
                        self.content.insert(key, content.clone());
                    }
                }

                let changes = match changes.len() {
                    0 => change_history::Changes::Empty,
                    1 => change_history::Changes::Atomic(changes.pop().unwrap()),
                    _ => change_history::Changes::Group(changes),
                };
                self.changes.push(changes);

                self.viewport.selection = match next_selection {
                    SelectionDirection::Stay => Some(cells),
                    SelectionDirection::Next => Some(cell::CellRect::new(
                        cells.x.saturating_add(cells.width),
                        cells.y,
                        cells.width,
                        cells.height,
                    )),
                    SelectionDirection::Return => Some(cell::CellRect::new(
                        cells.x,
                        cells.y.saturating_add(cells.height),
                        cells.width,
                        cells.height,
                    )),
                };
                self.saved = false;
                *info = format!("Wrote {cells} ({} cells)", cells.count());
            }
            Command::DeleteCells {
                cells,
                next_selection,
            } => {
                let mut changes = Vec::new();
                for cell_x in cells.x..cells.x + cells.width {
                    for cell_y in cells.y..cells.y + cells.height {
                        let key = cell::CellIndex::new(cell_x, cell_y);
                        match self.content.get(&key) {
                            Some(prev) => changes.push(change_history::Change::new(
                                key,
                                Some(prev.clone()),
                                None,
                            )),
                            None => { /* no changes, from empty to empty */ }
                        };
                        self.content.remove(&key);
                    }
                }

                let changes = match changes.len() {
                    0 => change_history::Changes::Empty,
                    1 => change_history::Changes::Atomic(changes.pop().unwrap()),
                    _ => change_history::Changes::Group(changes),
                };
                self.changes.push(changes);

                self.viewport.selection = match next_selection {
                    SelectionDirection::Stay => Some(cells),
                    SelectionDirection::Next => Some(cell::CellRect::new(
                        cells.x.saturating_add(cells.width),
                        cells.y,
                        cells.width,
                        cells.height,
                    )),
                    SelectionDirection::Return => Some(cell::CellRect::new(
                        cells.x,
                        cells.y.saturating_add(cells.height),
                        cells.width,
                        cells.height,
                    )),
                };
                self.saved = false;
                *info = format!("Deleted {cells} ({} cells)", cells.count());
            }
            Command::CopyCells { cells } => {
                let data = (cells.y..cells.y + cells.height)
                    .map(|cell_y| {
                        (cells.x..cells.x + cells.width)
                            .map(|cell_x| {
                                match self.content.get(&cell::CellIndex::new(cell_x, cell_y)) {
                                    Some(cell) => cell.to_string(),
                                    None => String::with_capacity(0),
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\t")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let bytes = data.as_bytes().len();
                match crossterm::execute!(
                    std::io::stdout(),
                    crossterm::clipboard::CopyToClipboard::to_clipboard_from(data)
                ) {
                    Ok(_) => *info = format!("Copied {bytes} bytes to clipboard"),
                    Err(e) => *info = format!("Failed to copy to clipboard: {e}"),
                }
            }
            Command::PasteCells {
                cells,
                paste_buffer,
            } => {
                let mut changes = Vec::new();
                for (offset_y, row) in paste_buffer.lines().enumerate() {
                    for (offset_x, content) in row.split('\t').enumerate() {
                        let key = cell::CellIndex::new(
                            cells
                                .x
                                .saturating_add(u64::try_from(offset_x).unwrap_or(u64::MAX)),
                            cells
                                .y
                                .saturating_add(u64::try_from(offset_y).unwrap_or(u64::MAX)),
                        );
                        let cell = cell::Cell::parse(content);
                        changes.push(change_history::Change::new(
                            key,
                            self.content.get(&key).cloned(),
                            Some(cell.clone()),
                        ));
                        self.content.insert(key, cell);
                    }
                }
                let changes = match changes.len() {
                    0 => change_history::Changes::Empty,
                    1 => change_history::Changes::Atomic(changes.pop().unwrap()),
                    _ => change_history::Changes::Group(changes),
                };
                self.changes.push(changes);
                *info = format!("Pasted {} bytes", paste_buffer.bytes().len());
            }
            Command::Undo => {
                let mut updated_cell_count = 0;
                match self.changes.last_changes() {
                    None => *info = format!("No changes to undo!"),
                    Some(change_history::Changes::Empty) => {
                        *info = format!("Undo changes with no impacts")
                    }
                    Some(change_history::Changes::Atomic(change)) => {
                        match &change.previous_value {
                            Some(value) => self.content.insert(change.index, value.clone()),
                            None => self.content.remove(&change.index),
                        };
                        updated_cell_count += 1;
                    }
                    Some(change_history::Changes::Group(changes)) => {
                        for change in changes {
                            match &change.previous_value {
                                Some(value) => self.content.insert(change.index, value.clone()),
                                None => self.content.remove(&change.index),
                            };
                            updated_cell_count += 1;
                        }
                    }
                }
                self.changes.undo();
                *info = format!("Undo: updated {updated_cell_count} cells");
            }
            Command::Redo => {
                let mut updated_cell_count = 0;
                match self.changes.last_undos() {
                    None => *info = format!("No changes to redo!"),
                    Some(change_history::Changes::Empty) => {
                        *info = format!("Redo changes with no impacts")
                    }
                    Some(change_history::Changes::Atomic(change)) => {
                        match &change.previous_value {
                            Some(value) => self.content.insert(change.index, value.clone()),
                            None => self.content.remove(&change.index),
                        };
                        updated_cell_count += 1;
                    }
                    Some(change_history::Changes::Group(changes)) => {
                        for change in changes {
                            match &change.previous_value {
                                Some(value) => self.content.insert(change.index, value.clone()),
                                None => self.content.remove(&change.index),
                            };
                            updated_cell_count += 1;
                        }
                    }
                }
                self.changes.redo();
                *info = format!("Redo: updated {updated_cell_count} cells");
            }
            Command::RedrawRequest => { /* bubble up, but nothing to do */ }
        }
    }
}
