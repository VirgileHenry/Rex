use ratatui::buffer::Buffer;

const HORIZONTAL: char = '─';
const VERTICAL: char = '│';
const TOPS: (char, char) = ('╭', '╮');
const CROSSES: (char, char) = ('├', '┤');
const BOTTOMS: (char, char) = ('╰', '╯');

pub struct MainFrame<'lyt> {
    layout: &'lyt crate::widgets::AppLayout,
}

impl<'lyt> MainFrame<'lyt> {
    pub fn new(layout: &'lyt crate::widgets::AppLayout) -> MainFrame<'lyt> {
        MainFrame { layout }
    }

    fn draw_line(&self, at: u16, width: u16, sides: (char, char), buf: &mut Buffer) {
        if let Some(cell) = buf.cell_mut((0, at)) {
            cell.set_char(sides.0);
        }
        for x in 1..width - 1 {
            if let Some(cell) = buf.cell_mut((x, at)) {
                cell.set_char(HORIZONTAL);
            }
        }
        if let Some(cell) = buf.cell_mut((width - 1, at)) {
            cell.set_char(sides.1);
        }
    }

    fn draw_sides(&self, start: u16, end: u16, width: u16, buf: &mut Buffer) {
        for y in start..end {
            if let Some(cell) = buf.cell_mut((0, y)) {
                cell.set_char(VERTICAL);
            }
            if let Some(cell) = buf.cell_mut((width - 1, y)) {
                cell.set_char(VERTICAL);
            }
        }
    }

    fn draw_borders(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let mut line_height = 0;
        self.draw_line(line_height, area.width, TOPS, buf);
        line_height += 1;

        self.draw_sides(
            line_height,
            line_height + self.layout.top.height,
            area.width,
            buf,
        );
        line_height += self.layout.top.height;

        self.draw_line(line_height, area.width, CROSSES, buf);
        line_height += 1;

        self.draw_sides(
            line_height,
            line_height + self.layout.content.height,
            area.width,
            buf,
        );
        line_height += self.layout.content.height;

        self.draw_line(line_height, area.width, CROSSES, buf);
        line_height += 1;

        self.draw_sides(
            line_height,
            line_height + self.layout.footer.height,
            area.width,
            buf,
        );
        line_height += 1;

        self.draw_line(line_height, area.width, BOTTOMS, buf);
    }
}

impl<'lyt> ratatui::widgets::Widget for MainFrame<'lyt> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.draw_borders(area, buf);
    }
}
