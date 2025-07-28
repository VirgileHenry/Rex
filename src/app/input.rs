/// Trait for all interfaces that want to handle inputs in their own way.
pub trait InputHandler {
    type Response;
    fn handle(&mut self, input: crossterm::event::Event) -> Option<Self::Response>;
}
