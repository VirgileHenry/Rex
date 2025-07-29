/// Main application.
pub struct App {
    event_info: String,
    file: Option<crate::file::FileApp>,
    layout: crate::widgets::AppLayout,
}

impl App {
    pub fn empty(term_size: ratatui::layout::Size) -> App {
        App {
            event_info: String::new(),
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
            event_info: String::new(),
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

        let bottom_info =
            ratatui::widgets::Paragraph::new(self.event_info.as_str()).right_aligned();
        frame.render_widget(bottom_info, self.layout.footer);
    }

    pub fn run(&mut self, mut terminal: ratatui::DefaultTerminal) -> std::io::Result<()> {
        let mut redraw_requested = false;
        terminal.draw(|frame| self.render(frame))?;

        loop {
            let event = match crossterm::event::read() {
                Ok(event) => match event {
                    /* Some events need catching at the app level */
                    crossterm::event::Event::Key(crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Char('q'),
                        modifiers: crossterm::event::KeyModifiers::CONTROL,
                        ..
                    }) => break Ok(()),
                    crossterm::event::Event::FocusGained => {
                        redraw_requested = true;
                        None
                    }
                    crossterm::event::Event::Resize(width, height) => {
                        let size = ratatui::layout::Size::new(width, height);
                        self.layout.recompute(size);
                        if let Some(file) = &mut self.file {
                            file.update_content_area(self.layout.content);
                        }
                        redraw_requested = true;
                        None
                    }
                    other => Some(other),
                },
                Err(e) => break Err(e),
            };

            /* if the app didn't used the event, it left it here and we can redirect it to the file */
            match (&mut self.file, event) {
                (Some(file), Some(event)) => {
                    use crate::event::EventHandler;
                    redraw_requested |= file.handle_event(event, &mut self.event_info)
                }
                _ => { /* either the event have been consumed, either we have no file to redirect */
                }
            }

            if redraw_requested {
                terminal.draw(|frame| self.render(frame))?;
                redraw_requested = false;
            }
        }?;

        Ok(())
    }
}
