mod key_map;

/// Main application.
pub struct App {
    key_map: key_map::KeyMap,
    file: Option<()>,
}

impl App {
    pub fn empty() -> App {
        App {
            key_map: key_map::KeyMap::default(),
            file: None,
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        frame.render_widget("hello world", frame.area());
    }

    pub fn run(&mut self, mut terminal: ratatui::DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Esc,
                    ..
                })) => break Ok(()),
                Err(e) => break Err(e),
                _ => {}
            }
        }?;

        Ok(())
    }
}
