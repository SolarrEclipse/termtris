use std::{
    io::{Stdout, Write, stdout},
    time::Duration,
    vec,
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{
        self,
        Event::{self},
        KeyCode,
    },
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

const CELL_WIDTH: u16 = 4;
const CELL_HEIGHT: u16 = 2;

struct Grid {
    width: u16,
    height: u16,
    cells: Vec<Vec<u8>>,
}

impl Grid {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![0u8; width.into()]; height.into()],
        }
    }
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    let grid = Grid::new(10, 20);

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        draw_grid(&mut stdout, &grid)?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn draw_grid(stdout: &mut Stdout, grid: &Grid) -> std::io::Result<()> {
    let (term_width, term_height) = terminal::size()?;
    let grid_screen_width = grid.width * CELL_WIDTH;
    let grid_screen_height = grid.height * CELL_HEIGHT;
    let board_x = term_width.saturating_sub(grid_screen_width) / 2;
    let board_y = term_height.saturating_sub(grid_screen_height) / 2;
    let right_edge = usize::from(grid.width - 1);
    let bottom_edge = usize::from(grid.height - 1);

    for (y, row) in grid.cells.iter().enumerate() {
        for (x, _cell) in row.iter().enumerate() {
            let screen_x = x as u16 * CELL_WIDTH;
            let screen_y = y as u16 * CELL_HEIGHT;
            let pos_y = screen_y + board_y + 3;
            let pos_x = screen_x + board_x;
            let wide_space = " ".repeat(CELL_WIDTH as usize);
            let wide_bot_border = "─".repeat(CELL_WIDTH as usize);

            if x == 0 {
                if y == bottom_edge {
                    queue!(
                        stdout,
                        MoveTo(pos_x, pos_y),
                        Print(format!("└{}", "─".repeat((CELL_WIDTH - 1) as usize)))
                    )?;
                } else {
                    for dy in 0..CELL_HEIGHT {
                        queue!(stdout, MoveTo(pos_x, pos_y + dy), Print("│"))?;
                    }
                }
            } else if x == right_edge {
                if y == bottom_edge {
                    queue!(stdout, MoveTo(pos_x, pos_y), Print("┘"))?;
                } else {
                    for dy in 0..CELL_HEIGHT {
                        queue!(stdout, MoveTo(pos_x, pos_y + dy), Print("│"))?;
                    }
                }
            } else if y == bottom_edge {
                queue!(
                    stdout,
                    MoveTo(pos_x, pos_y),
                    Print(wide_bot_border.as_str())
                )?;
            } else {
                // queue!(stdout, MoveTo(pos_x, pos_y), Print(wide_space))?;
                queue!(
                    stdout,
                    MoveTo(pos_x, pos_y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("·"),
                    ResetColor
                    )?;
            }
        }
    }

    Ok(())
}
