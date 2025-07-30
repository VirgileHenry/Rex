pub struct InputBuffer {
    buffer: Vec<char>,
    cursor: usize,
}

impl InputBuffer {
    pub fn new(opening_chars: &str) -> InputBuffer {
        let mut buffer = Vec::new();
        for ch in opening_chars.chars() {
            buffer.push(ch);
        }
        let cursor = buffer.len();
        InputBuffer { buffer, cursor }
    }

    pub fn push(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn del_front(&mut self) {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1);
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    pub fn del_back(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }

    pub fn increment_cursor(&mut self, whole_word: bool) {
        match whole_word {
            true => {
                let start_at_alphanumeric = self
                    .buffer
                    .get(self.cursor)
                    .map(|ch| ch.is_alphanumeric())
                    .unwrap_or(false);
                while self.cursor < self.buffer.len()
                    && match self.buffer.get(self.cursor) {
                        Some(ch) => ch.is_alphanumeric() == start_at_alphanumeric,
                        None => false,
                    }
                {
                    self.cursor = self.cursor.saturating_add(1).min(self.buffer.len());
                }
            }
            false => self.cursor = self.cursor.saturating_add(1).min(self.buffer.len()),
        }
    }

    pub fn decrement_cursor(&mut self, whole_word: bool) {
        match whole_word {
            true => {
                let start_at_alphanumeric = self
                    .buffer
                    .get(self.cursor.saturating_sub(1))
                    .map(|ch| ch.is_alphanumeric())
                    .unwrap_or(false);
                while self.cursor > 0
                    && match self.buffer.get(self.cursor.saturating_sub(1)) {
                        Some(ch) => ch.is_alphanumeric() == start_at_alphanumeric,
                        None => false,
                    }
                {
                    self.cursor = self.cursor.saturating_sub(1);
                }
            }
            false => self.cursor = self.cursor.saturating_sub(1),
        }
    }

    pub fn string(&self) -> String {
        self.buffer.iter().cloned().collect()
    }

    pub fn required_width(&self) -> usize {
        self.buffer
            .split(|ch| *ch == '\n')
            .map(|subslice| subslice.len())
            .max()
            .unwrap_or(0)
            + 1 // for cursor
    }

    pub fn required_height(&self) -> usize {
        self.buffer.iter().filter(|ch| **ch == '\n').count()
    }

    pub fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::style::Stylize;

        let first_span = ratatui::text::Span::raw(
            self.buffer[0..self.cursor]
                .iter()
                .cloned()
                .collect::<String>(),
        );
        let cursor_span = ratatui::text::Span::styled(
            self.buffer
                .get(self.cursor)
                .cloned()
                .unwrap_or(' ')
                .to_string(),
            ratatui::style::Style::default().add_modifier(
                ratatui::style::Modifier::SLOW_BLINK | ratatui::style::Modifier::REVERSED,
            ),
        );
        let ending_span = ratatui::text::Span::raw(
            self.buffer
                .get(self.cursor + 1..self.buffer.len())
                .map(|slice| slice.iter().cloned().collect::<String>())
                .unwrap_or(String::with_capacity(0)),
        );
        let line = ratatui::text::Line::from(vec![first_span, cursor_span, ending_span]);
        let paragraph = ratatui::widgets::Paragraph::new(line)
            .fg(ratatui::style::Color::White)
            .bg(ratatui::style::Color::Black);
        frame.render_widget(paragraph, area);
    }
}
