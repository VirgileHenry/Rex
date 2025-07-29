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

    pub fn save(&self, content: &mut String) {
        match self {
            Cell::Text(text) => content.push_str(text.as_str()),
            Cell::Num(val) => content.push_str(&format!("{val}")),
            Cell::Formula(_) => content.push_str("=?"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Cell::Text(text) => text.clone(),
            Cell::Num(num) => format!("{num}"),
            Cell::Formula(_) => format!("=?"),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellIndex {
    pub x: u64,
    pub y: u64,
}

impl CellIndex {
    pub fn new(x: u64, y: u64) -> CellIndex {
        CellIndex { x, y }
    }

    pub fn alternate_color_index(&self) -> usize {
        usize::try_from((self.x + self.y) % 2).unwrap()
    }
}

impl std::fmt::Display for CellIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let column = format_column(self.x);
        let row = format_row(self.y);
        write!(f, "{column}{row}")
    }
}

impl PartialOrd for CellIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match self.y.cmp(&other.y) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => self.x.cmp(&other.x),
        })
    }
}
impl Ord for CellIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.y.cmp(&other.y) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => self.x.cmp(&other.x),
        }
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

    pub fn count(&self) -> u64 {
        self.width.saturating_mul(self.height)
    }
}

impl std::fmt::Display for CellRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.width, self.height) {
            (0, _) | (_, 0) => write!(f, "Empty cell rect"),
            (1, 1) => {
                let cell = CellIndex::new(self.x, self.y);
                write!(f, "{cell}")
            }
            (width, height) => {
                let start = CellIndex::new(self.x, self.y);
                let end = CellIndex::new(
                    self.x.saturating_add(width).saturating_sub(1),
                    self.y.saturating_add(height).saturating_sub(1),
                );
                write!(f, "{start}:{end}")
            }
        }
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

pub fn format_column(col: u64) -> String {
    let mut index = col;
    let mut column = String::new();

    index += 1; // Shift to 1-based (Excel is 1-indexed)

    while index > 0 {
        index -= 1;
        let ch = (b'A' + (index % 26) as u8) as char;
        column.insert(0, ch);
        index /= 26;
    }

    column
}

pub fn format_row(row: u64) -> String {
    format!("{}", row + 1)
}
