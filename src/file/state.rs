mod editing;
pub use editing::*;

pub enum State {
    Idle,
    Editing(editing::EditingState),
}
