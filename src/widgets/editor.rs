use ratatui::{buffer::Buffer, symbols::line::TOP_LEFT};

const HORIZONTAL: char = '─';
const VERTICAL: char = '│';
const TOPS: (char, char) = ('╭', '╮');
const CROSSES: (char, char) = ('├', '┤');
const BOTTOMS: (char, char) = ('╰', '╯');

pub struct Editor<'a> {
    validate_text: &'a str,
    cancel_text: &'a str,
}

impl<'a> Editor<'a> {
    pub fn new(validate_text: &'a str, cancel_text: &'a str) -> Editor<'a> {
        Editor {
            validate_text,
            cancel_text,
        }
    }

    pub fn text_widths(&self) -> usize {
        self.validate_text
            .len()
            .max(self.cancel_text.len())
            .saturating_mul(2)
            .saturating_add(1)
    }

    fn draw_line(
        &self,
        top_left: ratatui::layout::Position,
        at: u16,
        width: u16,
        sides: (char, char),
        buf: &mut Buffer,
    ) {
        if let Some(cell) = buf.cell_mut((top_left.x, top_left.y.saturating_add(at))) {
            cell.set_char(sides.0);
        }
        for x in 1..width - 1 {
            if let Some(cell) =
                buf.cell_mut((top_left.x.saturating_add(x), top_left.y.saturating_add(at)))
            {
                cell.set_char(HORIZONTAL);
            }
        }
        if let Some(cell) = buf.cell_mut((
            top_left.x.saturating_add(width).saturating_sub(1),
            top_left.y.saturating_add(at),
        )) {
            cell.set_char(sides.1);
        }
    }

    fn draw_sides(
        &self,
        top_left: ratatui::layout::Position,
        start: u16,
        end: u16,
        width: u16,
        buf: &mut Buffer,
    ) {
        for y in start..end {
            if let Some(cell) = buf.cell_mut((top_left.x, top_left.y.saturating_add(y))) {
                cell.set_char(VERTICAL);
            }
            if let Some(cell) = buf.cell_mut((
                top_left.x.saturating_add(width).saturating_sub(1),
                top_left.y.saturating_add(y),
            )) {
                cell.set_char(VERTICAL);
            }
        }
    }

    fn draw_full(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        buf.set_style(area, ratatui::style::Color::Cyan);

        let mut line_height = 0;
        self.draw_line(area.as_position(), line_height, area.width, TOPS, buf);
        line_height = line_height.saturating_add(1);

        let sides_height = area.height.saturating_sub(4);
        self.draw_sides(
            area.as_position(),
            line_height,
            line_height.saturating_add(sides_height),
            area.width,
            buf,
        );
        line_height = line_height.saturating_add(sides_height);

        self.draw_line(area.as_position(), line_height, area.width, CROSSES, buf);
        line_height = line_height.saturating_add(1);

        let sides_height = 1;
        self.draw_sides(
            area.as_position(),
            line_height,
            line_height.saturating_add(sides_height),
            area.width,
            buf,
        );
        line_height = line_height.saturating_add(sides_height);

        self.draw_line(area.as_position(), line_height, area.width, BOTTOMS, buf);

        let text_area = ratatui::layout::Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(area.height).saturating_sub(2),
            area.width.saturating_sub(2),
            1,
        );
        self.draw_text(text_area, buf);
    }

    fn draw_only_exterior(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        buf.set_style(area, ratatui::style::Color::Cyan);

        let mut line_height = 0;
        self.draw_line(area.as_position(), line_height, area.width, TOPS, buf);
        line_height = line_height.saturating_add(1);

        let sides_height = area.height.saturating_sub(2);
        self.draw_sides(
            area.as_position(),
            line_height,
            line_height.saturating_add(sides_height),
            area.width,
            buf,
        );
        line_height = line_height.saturating_add(sides_height);

        self.draw_line(area.as_position(), line_height, area.width, BOTTOMS, buf);
    }

    fn draw_text(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        buf.set_style(area, ratatui::style::Color::LightYellow);

        let text_available_space = area.width / 2;
        for (x, ch) in self
            .validate_text
            .chars()
            .take(usize::from(text_available_space))
            .enumerate()
        {
            buf.cell_mut((
                area.x.saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
            ))
            .map(|cell| cell.set_char(ch));
        }
        for (x, ch) in self
            .cancel_text
            .chars()
            .take(usize::from(text_available_space))
            .enumerate()
        {
            buf.cell_mut((
                area.x
                    .saturating_add(area.width % 2)
                    .saturating_add(text_available_space)
                    .saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
            ))
            .map(|cell| cell.set_char(ch));
        }
    }
}

impl<'a> ratatui::widgets::Widget for Editor<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        match area.height {
            0..5 => self.draw_only_exterior(area, buf),
            _ => self.draw_full(area, buf),
        }
    }
}
