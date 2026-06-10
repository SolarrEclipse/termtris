use std::{io::{Write, stdout}, time::Duration};

use crossterm::{cursor, event::{self, Event::{self, Key}, KeyCode}, execute, style::Print, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
            Print("Welcome to Terminaltris"),
            cursor::MoveTo(0, 1),
            Print("Press q to quit")
        )?;
        stdout.flush()?;
    }

    execute!(
        stdout,
        LeaveAlternateScreen,
        cursor::Show
    )?;



    Ok(())
}
