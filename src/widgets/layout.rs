/// The application layout gets computed from the terminal area,
/// and stores infos about where to draw the varous app widgets.
pub struct AppLayout {
    menu_size: u16,
    footer_size: u16,
    pub top: ratatui::layout::Rect,
    pub content: ratatui::layout::Rect,
    pub footer: ratatui::layout::Rect,
}

impl AppLayout {
    /// Layout created from an empty rect.
    pub fn new(term_size: ratatui::layout::Size, menu_size: u16, footer_size: u16) -> AppLayout {
        let mut result = AppLayout {
            menu_size,
            footer_size,
            top: ratatui::layout::Rect::ZERO,
            content: ratatui::layout::Rect::ZERO,
            footer: ratatui::layout::Rect::ZERO,
        };
        result.recompute(term_size);
        result
    }

    /// Recomputes the layout from the given terminal size.
    pub fn recompute(&mut self, size: ratatui::layout::Size) {
        let area = ratatui::layout::Rect::new(
            1,
            1,
            size.width.saturating_sub(2),
            size.height.saturating_sub(2),
        );
        let constraints = [
            ratatui::layout::Constraint::Length(self.menu_size),
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(self.footer_size),
        ];
        let [top, content, footer] = ratatui::layout::Layout::vertical(constraints)
            .spacing(ratatui::layout::Spacing::Space(1))
            .areas(area);
        self.top = top;
        self.content = content;
        self.footer = footer;
    }
}
