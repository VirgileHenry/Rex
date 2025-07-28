pub struct InputBuffer {
    buffer: Vec<char>,
    cursor: usize,
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    pub fn handle_event(&mut self, event: crossterm::event::Event) {}
}
