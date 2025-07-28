mod input;
mod prompt;

/// Main application.
pub struct App {
    last_command: String,
    file: Option<crate::file::FileApp>,
    layout: crate::widgets::AppLayout,
}

impl App {
    pub fn empty(term_size: ratatui::layout::Size) -> App {
        App {
            last_command: String::new(),
            file: None,
            layout: crate::widgets::AppLayout::new(term_size, 8, 1),
        }
    }

    pub fn with_file(
        term_size: ratatui::layout::Size,
        path: &std::path::Path,
    ) -> std::io::Result<App> {
        let mut file = crate::file::FileApp::open(path)?;
        let layout = crate::widgets::AppLayout::new(term_size, 8, 1);
        file.update_content_area(layout.content);
        Ok(App {
            last_command: String::new(),
            file: Some(file),
            layout,
        })
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // draw app background
        let main_frame = crate::widgets::MainFrame::new(&self.layout);
        frame.render_widget(main_frame, area);

        match &self.file {
            Some(file) => file.render(frame),
            None => frame.render_widget(
                ratatui::widgets::Paragraph::new("\n\nPress Ctrl+O to open a new file!").centered(),
                self.layout.content,
            ),
        }

        frame.render_widget(&self.last_command, self.layout.footer);
    }

    pub fn run(&mut self, mut terminal: ratatui::DefaultTerminal) -> std::io::Result<()> {
        let mut redraw_requested = false;
        terminal.draw(|frame| self.render(frame))?;

        loop {
            match crossterm::event::read() {
                Ok(event) => {
                    match event {
                        crossterm::event::Event::Key(input) => {
                            match (input.code, input.modifiers) {
                                (
                                    crossterm::event::KeyCode::Char('q'),
                                    crossterm::event::KeyModifiers::CONTROL,
                                ) => break Ok(()),
                                _ => {}
                            }
                        }
                        crossterm::event::Event::FocusGained => redraw_requested = true,
                        crossterm::event::Event::Resize(width, height) => {
                            let size = ratatui::layout::Size::new(width, height);
                            self.layout.recompute(size);
                            if let Some(file) = &mut self.file {
                                file.update_content_area(self.layout.content);
                            }
                            redraw_requested = true;
                        }
                        _ => { /* unhandled! */ }
                    }
                    match &mut self.file {
                        Some(file) => redraw_requested |= file.handle_event(event),
                        None => {}
                    }
                }
                Err(e) => break Err(e),
            }

            if redraw_requested {
                terminal.draw(|frame| self.render(frame))?;
                redraw_requested = false;
            }
        }?;

        Ok(())
    }
}
