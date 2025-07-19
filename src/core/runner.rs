use crate::core::app::App;
use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

pub fn run(mut app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = setup_terminal()?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_event(Event::Key(key))?;
                if app.should_quit() {
                    break;
                }
            }
        }

        app.update()?;

        terminal.draw(|f| {
            app.draw(f);
        })?;
    }

    cleanup_terminal()?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn cleanup_terminal() -> Result<(), io::Error> {
    disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}