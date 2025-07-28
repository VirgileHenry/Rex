mod app;
mod args;
mod event;
mod file;
mod utils;
mod widgets;

fn setup_terminal(support_enhancement: bool, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
    use crossterm::event::*;

    if support_enhancement {
        let enhancement_flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
        crossterm::queue!(stdout, PushKeyboardEnhancementFlags(enhancement_flags))?;
    }

    crossterm::execute!(
        stdout,
        EnableBracketedPaste,
        EnableFocusChange,
        EnableMouseCapture,
    )?;

    Ok(())
}

fn restore_terminal(
    support_enhancement: bool,
    stdout: &mut std::io::Stdout,
) -> std::io::Result<()> {
    use crossterm::event::*;

    if support_enhancement {
        crossterm::queue!(stdout, PopKeyboardEnhancementFlags)?;
    }

    crossterm::execute!(
        stdout,
        DisableBracketedPaste,
        DisableFocusChange,
        DisableMouseCapture
    )?;

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::parse_args();
    let mut file = None;

    for arg in args.into_iter() {
        match arg {
            args::Args::File(path) => file = Some(path),
        }
    }

    let mut stdout = std::io::stdout();
    let support_enhancement = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );

    let terminal = ratatui::init();
    let term_size = terminal.size()?;
    setup_terminal(support_enhancement, &mut stdout)?;

    let mut application = match file {
        Some(file) => app::App::with_file(term_size, &file)?,
        None => app::App::empty(term_size),
    };

    let result = application.run(terminal);

    restore_terminal(support_enhancement, &mut stdout)?;
    ratatui::restore();

    result?;
    Ok(())
}
