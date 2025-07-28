use std::collections::BTreeMap;

mod cell;
mod command;
mod input_buffer;
mod state;
mod viewport;

const TOP_LEFT: ratatui::style::Color = ratatui::style::Color::Gray;
const AXIS_STYLE: [ratatui::style::Color; 2] = [
    ratatui::style::Color::Indexed(242),
    ratatui::style::Color::Indexed(244),
];
const CELL_STYLE: [ratatui::style::Color; 2] = [
    ratatui::style::Color::Indexed(233),
    ratatui::style::Color::Indexed(234),
];
const SELECTED: ratatui::style::Color = ratatui::style::Color::Indexed(62);

/// App state for an opened file.
pub struct FileApp {
    path: std::path::PathBuf,
    saved: bool,
    content: BTreeMap<cell::CellIndex, cell::Cell>,
    viewport: viewport::FileViewport,
    state: state::State,
}

impl FileApp {
    pub fn open(path: &std::path::Path) -> std::io::Result<FileApp> {
        let content = std::fs::read_to_string(path)?;
        let content = Self::parse_csv(content.as_str())?;
        Ok(FileApp {
            path: path.to_owned(),
            saved: true,
            content,
            viewport: viewport::FileViewport::new(ratatui::layout::Rect::ZERO),
            state: state::State::Idle,
        })
    }

    fn parse_csv(content: &str) -> std::io::Result<BTreeMap<cell::CellIndex, cell::Cell>> {
        let mut result = BTreeMap::new();

        for (y, line) in content.lines().enumerate() {
            for (x, cell_content) in line.split(';').enumerate() {
                if !cell_content.is_empty() {
                    let cell_x = match u64::try_from(x) {
                        Ok(x) => x,
                        Err(_) => continue,
                    };
                    let cell_y = match u64::try_from(y) {
                        Ok(y) => y,
                        Err(_) => continue,
                    };
                    let index = cell::CellIndex::new(cell_x, cell_y);
                    result.insert(index, cell::Cell::parse(cell_content));
                }
            }
        }

        Ok(result)
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut content = String::new();

        let mut last_index = cell::CellIndex::new(0, 0);
        for (index, cell) in self.content.iter() {
            for _empty_row in last_index.y..index.y {
                content.push('\n');
                last_index.x = 0;
            }
            for _empty_col in last_index.x..index.x {
                content.push(';');
            }
            cell.save(&mut content);
            last_index = *index;
        }

        std::fs::write(&self.path, &content)?;

        Ok(())
    }

    pub fn execute_command(&mut self, cmd: command::Command) {
        match cmd {
            command::Command::WriteCells {
                cells,
                content,
                next_selection,
            } => {
                for cell_x in cells.x..cells.x + cells.width {
                    for cell_y in cells.y..cells.y + cells.height {
                        let key = cell::CellIndex::new(cell_x, cell_y);
                        self.content.insert(key, content.clone());
                    }
                }
                self.viewport.selection = next_selection;
                self.saved = false;
            }
            command::Command::DeleteCells {
                cells,
                next_selection,
            } => {
                for cell_x in cells.x..cells.x + cells.width {
                    for cell_y in cells.y..cells.y + cells.height {
                        self.content.remove(&cell::CellIndex::new(cell_x, cell_y));
                    }
                }
                self.viewport.selection = next_selection;
                self.saved = false;
            }
            command::Command::RedrawRequest => { /* bubble up, but nothing to do */ }
        }
    }

    pub fn update_content_area(&mut self, content_area: ratatui::layout::Rect) {
        self.viewport.area = content_area;
    }

    pub fn render(&self, frame: &mut ratatui::Frame) {
        use ratatui::layout::Rect;
        use ratatui::style::Stylize;

        let area = self.viewport.area;

        let top_left_area = area.intersection(ratatui::layout::Rect::new(
            area.x,
            area.y,
            self.viewport.cell_size.width,
            self.viewport.cell_size.height,
        ));
        let top_left_block = ratatui::widgets::Block::new().bg(TOP_LEFT);
        frame.render_widget(top_left_block, top_left_area);

        let start_cell_x = self.viewport.top_left.x;
        let end_cell_x = start_cell_x + u64::from(area.width / self.viewport.cell_size.width);
        let x_axis_rect = Rect::new(
            area.x.saturating_add(self.viewport.cell_size.width),
            area.y,
            area.width.saturating_sub(self.viewport.cell_size.width),
            self.viewport.cell_size.height,
        );
        self.render_x_axis(start_cell_x..end_cell_x, frame, x_axis_rect);

        let start_cell_y = self.viewport.top_left.y;
        let end_cell_y = start_cell_y + u64::from(area.height.saturating_sub(1));
        let y_axis_rect = Rect::new(
            area.x,
            area.y.saturating_add(self.viewport.cell_size.height),
            self.viewport.cell_size.width,
            area.height.saturating_sub(self.viewport.cell_size.height),
        );
        self.render_y_axis(start_cell_y..end_cell_y, frame, y_axis_rect);

        for cell_y in start_cell_y..end_cell_y {
            for cell_x in start_cell_x..end_cell_x {
                let cells = cell::CellRect::new(cell_x, cell_y, 1, 1);
                let cell_area = self.viewport.cells_pos_to_screen_pos(cells);

                let cell_index = cell::CellIndex::new(cell_x, cell_y);
                let bg_style = if self.viewport.is_selected(cell_index) {
                    SELECTED
                } else {
                    CELL_STYLE[cell_index.alternate_color_index()]
                };
                frame.render_widget(ratatui::widgets::Block::new().bg(bg_style), cell_area);

                let cell = self.content.get(&cell_index);
                if let Some(cell) = cell {
                    cell.render(frame, cell_area);
                }
            }
        }

        match &self.state {
            state::State::Idle => {}
            state::State::Editing(editor) => editor.render(&self.viewport, frame),
        }
    }

    fn render_x_axis(
        &self,
        cells: std::ops::Range<u64>,
        frame: &mut ratatui::Frame,
        axis_area: ratatui::layout::Rect,
    ) {
        use ratatui::style::Stylize;

        for (x, cell_index) in cells.enumerate() {
            let cell_area = axis_area.intersection(ratatui::layout::Rect::new(
                axis_area.x.saturating_add(
                    u16::try_from(x)
                        .unwrap_or(u16::MAX)
                        .saturating_mul(self.viewport.cell_size.width),
                ),
                axis_area.y,
                self.viewport.cell_size.width,
                self.viewport.cell_size.height,
            ));
            let bg_style = if self.viewport.is_selected_x(u64::from(cell_index)) {
                SELECTED
            } else {
                AXIS_STYLE[usize::try_from(cell_index % 2).unwrap()]
            };
            let cell_text = cell::format_column(cell_index);
            let text = ratatui::widgets::Paragraph::new(cell_text.as_str())
                .centered()
                .bg(bg_style)
                .fg(ratatui::style::Color::Black);
            frame.render_widget(text, cell_area);
        }
    }

    fn render_y_axis(
        &self,
        cells: std::ops::Range<u64>,
        frame: &mut ratatui::Frame,
        axis_area: ratatui::layout::Rect,
    ) {
        use ratatui::style::Stylize;

        for (y, cell_index) in cells.enumerate() {
            let cell_area = axis_area.intersection(ratatui::layout::Rect::new(
                axis_area.x,
                axis_area.y.saturating_add(
                    u16::try_from(y)
                        .unwrap_or(u16::MAX)
                        .saturating_mul(self.viewport.cell_size.height),
                ),
                self.viewport.cell_size.width,
                self.viewport.cell_size.height,
            ));
            let bg_style = if self.viewport.is_selected_y(u64::from(cell_index)) {
                SELECTED
            } else {
                AXIS_STYLE[usize::try_from(cell_index % 2).unwrap()]
            };
            let cell_text = cell::format_row(cell_index);
            let text = ratatui::widgets::Paragraph::new(cell_text)
                .centered()
                .bg(bg_style)
                .fg(ratatui::style::Color::Black);
            frame.render_widget(text, cell_area);
        }
    }
}

impl crate::event::EventHandler for FileApp {
    /// If we request the redraw or not after an event
    type EventResponse = bool;
    fn handle_event(&mut self, event: crossterm::event::Event) -> Self::EventResponse {
        use crossterm::event::Event;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

        let mut redraw_requested = false;

        let command_result = match &mut self.state {
            state::State::Idle => match (event, self.viewport.selection) {
                (
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }),
                    _,
                ) => {
                    _ = self.save();
                    None
                }
                // when cells are selected and we press any writing chars, enter editing
                (
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code: KeyCode::Char(opening_char),
                        modifiers,
                        ..
                    }),
                    Some(cells),
                ) => {
                    if !modifiers.contains(KeyModifiers::CONTROL) {
                        self.state =
                            state::State::Editing(state::EditingState::new(cells, opening_char));
                        Some(command::Command::RedrawRequest)
                    } else {
                        None
                    }
                }
                // cells selected and supr / backspace, delete selected cells
                (
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code: KeyCode::Delete | KeyCode::Backspace,
                        ..
                    }),
                    Some(cells),
                ) => Some(command::Command::DeleteCells {
                    cells,
                    next_selection: Some(cells),
                }),

                // Lastly, we can redirect the event to the viewport control
                (other, _) => self.viewport.handle_event(other),
            },
            state::State::Editing(editor) => match editor.handle_event(event) {
                Some(response) => {
                    if response.exit {
                        self.state = state::State::Idle;
                    }
                    Some(response.command)
                }
                None => None,
            },
        };

        if let Some(cmd) = command_result {
            self.execute_command(cmd);
            redraw_requested = true;
        }

        redraw_requested
    }
}
