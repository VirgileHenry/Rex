use std::u64;

use crate::file::cell;

pub struct FileViewport {
    pub cell_size: ratatui::layout::Size,
    pub area: ratatui::layout::Rect,
    pub top_left: cell::CellIndex,
    pub selection: Option<cell::CellRect>,
}

impl FileViewport {
    /// Creates a new viewport from the given screen rect area.
    pub fn new(area: ratatui::layout::Rect) -> FileViewport {
        FileViewport {
            cell_size: ratatui::layout::Size::new(12, 1),
            area,
            top_left: cell::CellIndex::new(0, 0),
            selection: None,
        }
    }

    /// Whether a given cell is in the viewport selection or not.
    pub fn is_selected(&self, cell: cell::CellIndex) -> bool {
        if let Some(selection) = self.selection {
            selection.contains(cell)
        } else {
            false
        }
    }

    /// Whether the given column is in the selection or not.
    pub fn is_selected_x(&self, cell_x: u64) -> bool {
        if let Some(selection) = self.selection {
            selection.x <= cell_x && cell_x < selection.x + selection.width
        } else {
            false
        }
    }

    /// Whether the given row is in the selection or not.
    pub fn is_selected_y(&self, cell_y: u64) -> bool {
        if let Some(selection) = self.selection {
            selection.y <= cell_y && cell_y < selection.y + selection.height
        } else {
            false
        }
    }

    /// Get the drawable grid rect area, which is the entire space in which
    /// we are allowed to draw the cells, without the axis.
    pub fn grid_area(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::new(
            self.area.x.saturating_add(self.cell_size.width),
            self.area.y.saturating_add(self.cell_size.height),
            self.area.width.saturating_sub(self.cell_size.width),
            self.area.height.saturating_sub(self.cell_size.height),
        )
    }

    /// Get the cell index from a given screen position.
    /// If the cells or out of the current view, returns None.
    pub fn screen_pos_to_cell_pos(
        &self,
        screen_pos: ratatui::layout::Position,
    ) -> Option<cell::CellIndex> {
        let grid_rect = self.grid_area();
        if grid_rect.contains(screen_pos) {
            let grid_x = screen_pos.x.checked_sub(grid_rect.x)?;
            let grid_y = screen_pos.y.checked_sub(grid_rect.y)?;
            let cell_x = grid_x / self.cell_size.width;
            let cell_y = grid_y / self.cell_size.height;
            Some(cell::CellIndex::new(
                u64::from(cell_x).saturating_add(self.top_left.x),
                u64::from(cell_y).saturating_add(self.top_left.y),
            ))
        } else {
            None
        }
    }

    /// Get the screen position from the given cell position, constrained to the grid view.
    pub fn cells_pos_to_screen_pos(
        &self,
        cells: crate::file::cell::CellRect,
    ) -> ratatui::layout::Rect {
        use crate::utils::u64_to_u16;

        let start_x = u64_to_u16(
            cells
                .x
                .saturating_add(1)
                .saturating_sub(self.top_left.x)
                .saturating_mul(u64::from(self.cell_size.width)),
        ) + self.area.x;
        let end_x =
            start_x.saturating_add(u64_to_u16(cells.width).saturating_mul(self.cell_size.width));

        let start_y = u64_to_u16(
            cells
                .y
                .saturating_add(1)
                .saturating_sub(self.top_left.y)
                .saturating_mul(u64::from(self.cell_size.height)),
        ) + self.area.y;
        let end_y =
            start_y.saturating_add(u64_to_u16(cells.height).saturating_mul(self.cell_size.height));

        let cells_rect = ratatui::layout::Rect::new(
            start_x,
            start_y,
            end_x.saturating_sub(start_x),
            end_y.saturating_sub(start_y),
        );

        cells_rect.intersection(self.grid_area())
    }

    fn keep_selection_in_view(&mut self) {
        if let Some(selection) = self.selection {
            self.top_left.x = self.top_left.x.min(selection.x);
            self.top_left.x = self.top_left.x.max(
                selection
                    .x
                    .saturating_add(1)
                    .saturating_add(selection.width)
                    .saturating_sub(u64::from(self.area.width / self.cell_size.width)),
            );
            self.top_left.y = self.top_left.y.min(selection.y);
            self.top_left.y = self.top_left.y.max(
                selection
                    .y
                    .saturating_add(1)
                    .saturating_add(selection.height)
                    .saturating_sub(u64::from(self.area.height)),
            );
        }
    }
}

impl crate::event::EventHandler for FileViewport {
    type EventResponse = Option<super::command::Command>;
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        info: &mut String,
    ) -> Self::EventResponse {
        use crossterm::event::Event;
        use crossterm::event::MouseEventKind;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => match self.selection {
                Some(_) => {
                    self.selection = None;
                    info.clear();
                    Some(super::command::Command::RedrawRequest)
                }
                None => None,
            },
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                modifiers,
                ..
            }) => {
                if let Some(selection) = &mut self.selection {
                    let offset = 1 + u64::from(modifiers.contains(KeyModifiers::CONTROL)) * 7;
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        selection.height = selection.height.saturating_sub(offset).max(1);
                    } else {
                        selection.y = selection.y.saturating_sub(offset);
                    }
                    *info = format!("Selected {selection} ({} cells)", selection.count());
                    self.keep_selection_in_view();
                    Some(super::command::Command::RedrawRequest)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                modifiers,
                ..
            }) => {
                if let Some(selection) = &mut self.selection {
                    let offset = 1 + u64::from(modifiers.contains(KeyModifiers::CONTROL)) * 7;
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        selection.height = selection.height.saturating_add(offset);
                    } else {
                        selection.y = selection.y.saturating_add(offset);
                    }
                    *info = format!("Selected {selection} ({} cells)", selection.count());
                    self.keep_selection_in_view();
                    Some(super::command::Command::RedrawRequest)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                modifiers,
                ..
            }) => {
                if let Some(selection) = &mut self.selection {
                    let offset = 1 + u64::from(modifiers.contains(KeyModifiers::CONTROL)) * 7;
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        selection.width = selection.width.saturating_sub(offset).max(1);
                    } else {
                        selection.x = selection.x.saturating_sub(offset);
                    }
                    *info = format!("Selected {selection} ({} cells)", selection.count());
                    self.keep_selection_in_view();
                    Some(super::command::Command::RedrawRequest)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                modifiers,
                ..
            }) => {
                if let Some(selection) = &mut self.selection {
                    let offset = 1 + u64::from(modifiers.contains(KeyModifiers::CONTROL)) * 7;
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        selection.width = selection.width.saturating_add(offset);
                    } else {
                        selection.x = selection.x.saturating_add(offset);
                    }
                    *info = format!("Selected {selection} ({} cells)", selection.count());
                    self.keep_selection_in_view();
                    Some(super::command::Command::RedrawRequest)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            }) => {
                if let Some(selection) = &mut self.selection {
                    selection.x = selection.x.saturating_add(selection.width);
                    *info = format!("Selected {selection} ({} cells)", selection.count());
                    self.keep_selection_in_view();
                    Some(super::command::Command::RedrawRequest)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            }) => {
                if let Some(selection) = &mut self.selection {
                    selection.y = selection.y.saturating_add(selection.height);
                    *info = format!("Selected {selection} ({} cells)", selection.count());
                    self.keep_selection_in_view();
                    Some(super::command::Command::RedrawRequest)
                } else {
                    None
                }
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(_) => {
                    let mouse_pos = ratatui::layout::Position::new(mouse.column, mouse.row);
                    match self.screen_pos_to_cell_pos(mouse_pos) {
                        Some(cell) => {
                            let selection = cell::CellRect::new(cell.x, cell.y, 1, 1);
                            *info = format!("Selected {selection} ({} cells)", selection.count());
                            self.selection = Some(selection);
                            Some(super::command::Command::RedrawRequest)
                        }
                        None => None,
                    }
                }
                MouseEventKind::Drag(_) => {
                    let mouse_pos = ratatui::layout::Position::new(mouse.column, mouse.row);
                    match self.screen_pos_to_cell_pos(mouse_pos) {
                        Some(cell) => {
                            match &mut self.selection {
                                Some(selection) => {
                                    selection.width =
                                        cell.x.saturating_add(1).saturating_sub(selection.x).max(1);
                                    selection.height =
                                        cell.y.saturating_add(1).saturating_sub(selection.y).max(1);
                                    *info = format!(
                                        "Selected {selection} ({} cells)",
                                        selection.count()
                                    );
                                }
                                None => {
                                    let selection = cell::CellRect::new(cell.x, cell.y, 1, 1);
                                    *info = format!(
                                        "Selected {selection} ({} cells)",
                                        selection.count()
                                    );
                                    self.selection = Some(selection);
                                }
                            }
                            Some(super::command::Command::RedrawRequest)
                        }
                        None => None,
                    }
                }
                MouseEventKind::ScrollUp => {
                    self.top_left.y = self.top_left.y.saturating_sub(1);
                    Some(super::command::Command::RedrawRequest)
                }
                MouseEventKind::ScrollDown => {
                    self.top_left.y = self.top_left.y.saturating_add(1);
                    Some(super::command::Command::RedrawRequest)
                }
                MouseEventKind::ScrollLeft => {
                    self.top_left.x = self.top_left.x.saturating_sub(1);
                    Some(super::command::Command::RedrawRequest)
                }
                MouseEventKind::ScrollRight => {
                    self.top_left.x = self.top_left.x.saturating_add(1);
                    Some(super::command::Command::RedrawRequest)
                }
                _ => None,
            },
            _ => None,
        }
    }
}
