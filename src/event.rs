pub trait EventHandler {
    type EventResponse;
    fn handle_event(&mut self, event: crossterm::event::Event) -> Self::EventResponse;
}
