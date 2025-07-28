use crate::file::command::Command;

const MIN_EDIT_SIZE: u16 = 16;

pub enum State {
    Idle,
    Editing {
        cells: crate::file::cell::CellRect,
        buffer: String,
        cursor: usize,
    },
}

impl State {
    pub fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        selection: Option<crate::file::cell::CellRect>,
    ) -> Option<Command> {
        use crossterm::event::Event;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

        match self {
            State::Idle => match (event, selection) {
                // when cells are selected and we press any writing chars, enter editing
                (
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code: KeyCode::Char(ch),
                        ..
                    }),
                    Some(cells),
                ) => {
                    let mut buffer = String::new();
                    buffer.push(ch);
                    *self = State::Editing {
                        cells,
                        buffer,
                        cursor: 0,
                    };
                    Some(Command::RedrawRequest)
                }
                // cells selected and supr / backspace, delete selected cells
                (
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code: KeyCode::Delete | KeyCode::Backspace,
                        ..
                    }),
                    Some(cells),
                ) => Some(Command::DeleteCells {
                    cells,
                    next_selection: None,
                }),
                _ => None,
            },
            State::Editing { buffer, cells, .. } => match event {
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char(ch),
                    ..
                }) => {
                    buffer.push(ch);
                    Some(Command::RedrawRequest)
                }
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    buffer.pop();
                    Some(Command::RedrawRequest)
                }
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Esc,
                    ..
                }) => {
                    *self = State::Idle;
                    Some(Command::RedrawRequest)
                }
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Enter,
                    ..
                }) => {
                    let next_selection = crate::file::cell::CellRect::new(
                        cells.x,
                        cells.y + cells.height,
                        cells.width,
                        cells.height,
                    );
                    let command = Command::WriteCells {
                        cells: *cells,
                        content: crate::file::cell::Cell::parse(&buffer),
                        next_selection: Some(next_selection),
                    };
                    *self = State::Idle;
                    Some(command)
                }
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Tab,
                    ..
                }) => {
                    let next_selection = crate::file::cell::CellRect::new(
                        cells.x + cells.width,
                        cells.y,
                        cells.width,
                        cells.height,
                    );
                    let command = Command::WriteCells {
                        cells: *cells,
                        content: crate::file::cell::Cell::parse(&buffer),
                        next_selection: Some(next_selection),
                    };
                    *self = State::Idle;
                    Some(command)
                }
                _ => None,
            },
        }
    }

    pub fn render(
        &self,
        viewport: &crate::file::viewport::FileViewport,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) {
        use ratatui::style::Stylize;

        match self {
            State::Idle => {}
            State::Editing {
                cells,
                buffer,
                cursor,
            } => {
                let selection_rect = viewport
                    .cells_pos_to_screen_pos(*cells)
                    .intersection(viewport.grid_area());
                let available_width = area
                    .x
                    .saturating_add(area.width)
                    .saturating_sub(selection_rect.x);
                let available_height = area
                    .y
                    .saturating_add(area.height)
                    .saturating_sub(selection_rect.y);
                let text_width = (buffer.len() as u16)
                    .max(MIN_EDIT_SIZE)
                    .min(available_width);
                let text_rect =
                    ratatui::layout::Rect::new(selection_rect.x, selection_rect.y, text_width, 1);
                let edit_widget_rect = ratatui::layout::Rect::new(
                    text_rect.x.saturating_sub(1),
                    text_rect.y.saturating_sub(1),
                    text_rect.width + 2,
                    text_rect.height + 2,
                );

                frame.render_widget(ratatui::widgets::Clear, edit_widget_rect);
                frame.render_widget(
                    ratatui::widgets::Block::bordered()
                        .border_type(ratatui::widgets::block::BorderType::Rounded)
                        .border_style(ratatui::style::Color::Cyan)
                        .bg(ratatui::style::Color::Black),
                    edit_widget_rect,
                );
                frame.render_widget(ratatui::widgets::Paragraph::new(buffer.as_str()), text_rect);
            }
        }
    }
}
