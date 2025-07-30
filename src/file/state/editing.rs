pub struct EditingState {
    cells: crate::file::cell::CellRect,
    buffer: crate::file::input_buffer::InputBuffer,
}

impl EditingState {
    pub fn new(cells: crate::file::cell::CellRect, opening_chars: &str) -> EditingState {
        EditingState {
            cells,
            buffer: crate::file::input_buffer::InputBuffer::new(opening_chars),
        }
    }

    pub fn render(
        &self,
        viewport: &crate::file::viewport::FileViewport,
        frame: &mut ratatui::Frame,
    ) {
        use crate::utils::usize_to_u16;

        let editor_frame = crate::widgets::Editor::new("Save: Enter/Tab", "Cancel: Esc");

        let cells_rect = viewport.cells_pos_to_screen_pos(self.cells);
        let text_width = cells_rect
            .width
            .max(usize_to_u16(self.buffer.required_width()))
            .max(usize_to_u16(editor_frame.text_widths()));
        let text_height = cells_rect
            .height
            .max(usize_to_u16(self.buffer.required_height()));
        let text_rect = viewport
            .grid_area()
            .intersection(ratatui::layout::Rect::new(
                cells_rect.x,
                cells_rect.y,
                text_width,
                text_height,
            ));
        let contour_rect = viewport
            .grid_area()
            .intersection(ratatui::layout::Rect::new(
                text_rect.x.saturating_sub(1),
                text_rect.y.saturating_sub(1),
                text_rect.width.saturating_add(2),
                text_rect.height.saturating_add(4),
            ));

        frame.render_widget(ratatui::widgets::Clear, contour_rect);
        frame.render_widget(editor_frame, contour_rect);
        self.buffer.render(frame, text_rect);
    }
}

pub struct EditingStateEventResponse {
    pub command: crate::file::command::Command,
    pub exit: bool,
}

impl crate::event::EventHandler for EditingState {
    type EventResponse = Option<EditingStateEventResponse>;
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        _: &mut String,
    ) -> Self::EventResponse {
        use crate::file::command::{Command, SelectionDirection};
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
            Event::Paste(pasted_value) => {
                pasted_value.chars().for_each(|ch| self.buffer.push(ch));
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
                let buffer = self.buffer.string();
                Some(EditingStateEventResponse {
                    command: Command::WriteCells {
                        cells: self.cells,
                        content: crate::file::cell::Cell::parse(&buffer),
                        next_selection: SelectionDirection::Return,
                    },
                    exit: true,
                })
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Tab,
                ..
            }) => {
                let buffer = self.buffer.string();
                Some(EditingStateEventResponse {
                    command: Command::WriteCells {
                        cells: self.cells,
                        content: crate::file::cell::Cell::parse(&buffer),
                        next_selection: SelectionDirection::Next,
                    },
                    exit: true,
                })
            }
            _ => None,
        }
    }
}
