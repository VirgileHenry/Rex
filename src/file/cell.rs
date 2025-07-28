const TEXT_COLOR: ratatui::style::Color = ratatui::style::Color::White;
const NUM_COLOR: ratatui::style::Color = ratatui::style::Color::Indexed(230);
const FORMULA_COLOR: ratatui::style::Color = ratatui::style::Color::Indexed(159);

/// Single cell in a spreadsheet!
#[derive(Debug, Clone)]
pub enum Cell {
    Text(String),
    Num(f32),
    Formula(()),
}

impl Cell {
    pub fn parse(content: &str) -> Cell {
        if content.chars().next() == Some('=') {
            return Cell::Formula(());
        }

        if let Ok(val) = content.parse::<f32>() {
            return Cell::Num(val);
        }

        Cell::Text(content.to_string())
    }

    pub fn render(&self, frame: &mut ratatui::Frame, cell_area: ratatui::layout::Rect) {
        use ratatui::style::Stylize;

        let paragraph = match self {
            Cell::Text(text) => ratatui::widgets::Paragraph::new(text.as_str())
                .left_aligned()
                .fg(TEXT_COLOR),
            Cell::Num(num) => ratatui::widgets::Paragraph::new(format!("{num}"))
                .right_aligned()
                .fg(NUM_COLOR),
            Cell::Formula(_) => ratatui::widgets::Paragraph::new("formula")
                .left_aligned()
                .fg(FORMULA_COLOR),
        };

        frame.render_widget(paragraph, cell_area);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellIndex {
    pub x: u64,
    pub y: u64,
}

impl CellIndex {
    pub fn new(x: u64, y: u64) -> CellIndex {
        CellIndex { x, y }
    }

    pub fn format_column(&self) -> String {
        let mut digits = Vec::new();
        let mut value = self.x;

        if value == 0 {
            digits.push('A');
        } else {
            while value > 0 {
                // SAFTEY: safe to unwrap because the mod keeps us in 0:25 range
                let digit = u8::try_from(value % 26).unwrap();
                digits.push(char::from(digit));
                value /= 26;
            }
        }

        digits.into_iter().rev().collect::<String>()
    }

    pub fn format_row(&self) -> String {
        format!("{}", self.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellRect {
    pub x: u64,
    pub y: u64,
    pub width: u64,
    pub height: u64,
}

impl CellRect {
    pub fn new(x: u64, y: u64, width: u64, height: u64) -> CellRect {
        CellRect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, position: CellIndex) -> bool {
        self.x <= position.x
            && position.x < self.x + self.width
            && self.y <= position.y
            && position.y < self.y + self.height
    }
}

impl From<ratatui::layout::Rect> for CellRect {
    fn from(value: ratatui::layout::Rect) -> Self {
        CellRect {
            x: u64::from(value.x),
            y: u64::from(value.y),
            width: u64::from(value.width),
            height: u64::from(value.height),
        }
    }
}
