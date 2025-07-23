mod app;

fn main() {
    let terminal = ratatui::init();
    let mut application = app::App::empty();

    let result = application.run(terminal);

    ratatui::restore();
    if let Err(e) = result {
        println!("Error encountered: {e}")
    }
}
