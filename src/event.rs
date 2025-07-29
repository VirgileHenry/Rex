pub trait EventHandler {
    type EventResponse;
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        info: &mut String,
    ) -> Self::EventResponse;
}
