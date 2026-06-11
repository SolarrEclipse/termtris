use std::{
    io::{Stdout, Write, stdout},
    time::Duration,
    vec,
};

use crossterm::{
    cursor,
    event::{
        self,
        Event::{self},
        KeyCode,
    },
    execute,
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

struct Grid {
    height: u16,
    width: u16,
    cells: Vec<Vec<(u8)>>,
}

impl Grid {
    fn new(width: u16, height: u16) -> Self {
        Self {width, height, cells: vec![vec![0u8; width.into()]; height.into()]}
    }
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    let mut grid = Grid::new(10, 20);

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        stdout.flush()?;
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    draw_grid(&mut stdout, &mut grid);

    Ok(())
}

fn draw_grid(stdout: &mut Stdout, grid: &mut Grid) {
    let vert_border = '│';
    let bot_left_corner = '└';
    let bot_border = '─';
    let bot_right_corner = '┘';

    let grid_width = 10;
    let grid_height = 20;

    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            println!("{}, {:?}, {}, {:?}", y, row, x, cell);
        }
    }
}
