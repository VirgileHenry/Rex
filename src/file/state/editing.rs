pub struct EditingState {
    cells: crate::file::cell::CellRect,
    buffer: crate::file::input_buffer::InputBuffer,
}

impl EditingState {
    pub fn new(cells: crate::file::cell::CellRect, opening_char: char) -> EditingState {
        EditingState {
            cells,
            buffer: crate::file::input_buffer::InputBuffer::new(opening_char),
        }
    }

    pub fn render(
        &self,
        viewport: &crate::file::viewport::FileViewport,
        frame: &mut ratatui::Frame,
    ) {
        use ratatui::style::Stylize;

        let text_rect = viewport.cells_pos_to_screen_pos(self.cells);
        let contour_rect = viewport
            .grid_area()
            .intersection(ratatui::layout::Rect::new(
                text_rect.x.saturating_sub(1),
                text_rect.y.saturating_sub(1),
                text_rect.width.saturating_add(2),
                text_rect.height.saturating_add(4),
            ));

        frame.render_widget(ratatui::widgets::Clear, contour_rect);
        frame.render_widget(
            ratatui::widgets::Block::bordered()
                .border_type(ratatui::widgets::block::BorderType::Rounded)
                .border_style(ratatui::style::Color::Cyan)
                .bg(ratatui::style::Color::Black),
            contour_rect,
        );
        self.buffer.render(frame, text_rect);
    }
}

pub struct EditingStateEventResponse {
    pub command: crate::file::command::Command,
    pub exit: bool,
}

impl crate::event::EventHandler for EditingState {
    type EventResponse = Option<EditingStateEventResponse>;
    fn handle_event(&mut self, event: crossterm::event::Event) -> Self::EventResponse {
        use crate::file::command::Command;
        use crossterm::event::Event;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

        match event {
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                code: KeyCode::Char(ch),
                ..
            }) => {
                self.buffer.push(ch);
                Some(EditingStateEventResponse {
                    command: Command::RedrawRequest,
                    exit: false,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                code: KeyCode::Backspace,
                ..
            }) => {
                self.buffer.del_front();
                Some(EditingStateEventResponse {
                    command: Command::RedrawRequest,
                    exit: false,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                code: KeyCode::Delete,
                ..
            }) => {
                self.buffer.del_back();
                Some(EditingStateEventResponse {
                    command: Command::RedrawRequest,
                    exit: false,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                code: KeyCode::Left,
                modifiers,
                ..
            }) => {
                let whole_word = modifiers.contains(KeyModifiers::CONTROL);
                self.buffer.decrement_cursor(whole_word);
                Some(EditingStateEventResponse {
                    command: Command::RedrawRequest,
                    exit: false,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                code: KeyCode::Right,
                modifiers,
                ..
            }) => {
                let whole_word = modifiers.contains(KeyModifiers::CONTROL);
                self.buffer.increment_cursor(whole_word);
                Some(EditingStateEventResponse {
                    command: Command::RedrawRequest,
                    exit: false,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Esc,
                ..
            }) => Some(EditingStateEventResponse {
                command: Command::RedrawRequest,
                exit: true,
            }),
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Enter,
                ..
            }) => {
                let next_selection = crate::file::cell::CellRect::new(
                    self.cells.x,
                    self.cells.y + self.cells.height,
                    self.cells.width,
                    self.cells.height,
                );
                let buffer = self.buffer.string();
                Some(EditingStateEventResponse {
                    command: Command::WriteCells {
                        cells: self.cells,
                        content: crate::file::cell::Cell::parse(&buffer),
                        next_selection: Some(next_selection),
                    },
                    exit: true,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Tab,
                ..
            }) => {
                let next_selection = crate::file::cell::CellRect::new(
                    self.cells.x + self.cells.width,
                    self.cells.y,
                    self.cells.width,
                    self.cells.height,
                );
                let buffer = self.buffer.string();
                Some(EditingStateEventResponse {
                    command: Command::WriteCells {
                        cells: self.cells,
                        content: crate::file::cell::Cell::parse(&buffer),
                        next_selection: Some(next_selection),
                    },
                    exit: true,
                })
            }
            _ => None,
        }
    }
}
