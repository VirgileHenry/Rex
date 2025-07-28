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
const SELECTED: ratatui::style::Color = ratatui::style::Color::Indexed(28);

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

    pub fn handle_event(&mut self, event: crossterm::event::Event) -> bool {
        let mut redraw_requested = false;

        let command_result = self.viewport.handle_viewport_control_event(event.clone());
        if let Some(cmd) = command_result {
            self.execute_command(cmd);
            redraw_requested = true;
        }

        let command_result = self
            .state
            .handle_event(event.clone(), self.viewport.selection);
        if let Some(cmd) = command_result {
            self.execute_command(cmd);
            redraw_requested = true;
        }

        redraw_requested
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
            self.viewport.cell_width,
            1,
        ));
        let top_left_block = ratatui::widgets::Block::new().bg(TOP_LEFT);
        frame.render_widget(top_left_block, top_left_area);

        let start_cell_x = self.viewport.top_left.x;
        let end_cell_x = start_cell_x + u64::from(area.width / self.viewport.cell_width);
        let x_axis_rect = Rect::new(
            area.x + self.viewport.cell_width,
            area.y,
            area.width - self.viewport.cell_width,
            1,
        );
        self.render_x_axis(start_cell_x..end_cell_x, frame, x_axis_rect);

        let start_cell_y = self.viewport.top_left.y;
        let end_cell_y = start_cell_y + u64::from(area.height.saturating_sub(1));
        let y_axis_rect = Rect::new(
            area.x,
            area.y + 1,
            self.viewport.cell_width,
            area.height - 1,
        );
        self.render_y_axis(start_cell_y..end_cell_y, frame, y_axis_rect);

        let cells_area = Rect::new(
            area.x + self.viewport.cell_width,
            area.y + 1,
            area.width.saturating_sub(1),
            area.height.saturating_sub(1),
        );

        for (y, cell_y) in (start_cell_y..end_cell_y).enumerate() {
            for (x, cell_x) in (start_cell_x..end_cell_x).enumerate() {
                let cell_area = area.intersection(ratatui::layout::Rect::new(
                    cells_area.x + u16::try_from(x).unwrap_or(u16::MAX) * self.viewport.cell_width,
                    cells_area.y + u16::try_from(y).unwrap_or(u16::MAX),
                    self.viewport.cell_width,
                    1,
                ));

                let cell_index = cell::CellIndex::new(cell_x, cell_y);
                let bg_style = if self.viewport.is_selected(cell_index) {
                    SELECTED
                } else {
                    // SAFETY: safe to unwrap because the mod keeps us in 0, 2
                    CELL_STYLE[usize::try_from((cell_x + cell_y) % 2).unwrap()]
                };
                frame.render_widget(ratatui::widgets::Block::new().bg(bg_style), cell_area);

                let cell = self.content.get(&cell_index);
                if let Some(cell) = cell {
                    cell.render(frame, cell_area);
                }
            }
        }

        self.state.render(&self.viewport, frame, area);
    }

    fn render_x_axis(
        &self,
        cells: std::ops::Range<u64>,
        frame: &mut ratatui::Frame,
        axis_area: ratatui::layout::Rect,
    ) {
        use ratatui::style::Stylize;

        for (x, cell) in cells.enumerate() {
            let cell_area = axis_area.intersection(ratatui::layout::Rect::new(
                axis_area.x + u16::try_from(x).unwrap_or(u16::MAX) * self.viewport.cell_width,
                axis_area.y,
                self.viewport.cell_width,
                1,
            ));
            let bg_style = if self.viewport.is_selected_x(u64::from(cell)) {
                SELECTED
            } else {
                // SAFTEY: safe to unwrap, we are in the 0-2 range
                AXIS_STYLE[usize::try_from(cell % 2).unwrap()]
            };
            let text = ratatui::widgets::Paragraph::new(format!("{cell}"))
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

        for (y, cell) in cells.enumerate() {
            let cell_area = axis_area.intersection(ratatui::layout::Rect::new(
                axis_area.x,
                axis_area.y + u16::try_from(y).unwrap_or(u16::MAX),
                self.viewport.cell_width,
                1,
            ));
            let bg_style = if self.viewport.is_selected_y(u64::from(cell)) {
                SELECTED
            } else {
                // SAFTEY: safe to unwrap, we are in the 0-2 range
                AXIS_STYLE[usize::try_from(cell % 2).unwrap()]
            };
            let text = ratatui::widgets::Paragraph::new(format!("{cell}"))
                .centered()
                .bg(bg_style)
                .fg(ratatui::style::Color::Black);
            frame.render_widget(text, cell_area);
        }
    }
}
