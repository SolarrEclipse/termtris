use std::{
    io::{Stdout, Write, stdout},
    range::Range,
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

const CELL_WIDTH: u16 = 2;
const CELL_HEIGHT: u16 = 1;
const LOWER_BORDER: &str = "▀";
const UPPER_BORDER: &str = "▄";
const VERT_BORDER: &str = "█";
const TET_BLOCK: &str = "██";

struct Grid {
    width: u16,
    height: u16,
    cells: Vec<Vec<Option<TetrominoKind>>>,
}

impl Grid {
    fn new(width: u16, height: u16) -> Self {
        let mut cells = vec![vec![None; width.into()]; height.into()];

        Self {
            width,
            height,
            cells,
        }
    }

    fn active_tet_at(
        &self,
        active: Option<&ActiveTetromino>,
        x: u16,
        y: u16,
    ) -> Option<TetrominoKind> {
        let active = active?;

        for (block_x, block_y) in active.kind.blocks() {
            let board_x = active.x + block_x;
            let board_y = active.y + block_y;

            if board_x == x as i32 && board_y == y as i32 {
                return Some(active.kind);
            }
        }
        None
    }

    fn draw(&self, stdout: &mut Stdout, active: Option<&ActiveTetromino>) -> std::io::Result<()> {
        let (term_width, term_height) = terminal::size()?;
        let board_height = self.height * CELL_HEIGHT + 1;
        let board_width = self.width * CELL_WIDTH + 2;
        let board_left = 0;
        let board_top = 0;

        let border_left = board_left;

        let start_y = term_height / 2 - board_height / 2;
        let start_x = term_width / 2 - board_width / 2;

        let cells_left = board_left + 1;
        let cells_top = board_top + 1;

        for y in 0..self.height {
            for x in 0..self.width {
                let terminal_x = cells_left + x * CELL_WIDTH;
                let terminal_y = cells_top + y * CELL_HEIGHT;

                let visible_cell = self
                    .active_tet_at(active, x, y)
                    .or(self.cells[y as usize][x as usize]);

                let (tile, color) = match visible_cell {
                    Some(kind) => (TET_BLOCK, kind.color()),
                    None => ("  ", Color::Rgb { r: 0, g: 0, b: 0 }),
                };

                if x == border_left {
                    queue!(
                        stdout,
                        MoveTo(terminal_x - 1 + start_x, terminal_y + start_y),
                        Print(VERT_BORDER)
                    )?;
                }

                if x == self.width - 1 {
                    queue!(
                        stdout,
                        MoveTo(terminal_x + CELL_WIDTH + start_x, terminal_y + start_y),
                        Print(VERT_BORDER)
                    )?;
                }

                if y == self.height - 1 {
                    if x == border_left {
                        queue!(
                            stdout,
                            MoveTo(terminal_x - 1 + start_x, terminal_y + CELL_HEIGHT + start_y),
                            Print([LOWER_BORDER, LOWER_BORDER, LOWER_BORDER].concat())
                        )?;
                    } else if x == self.width - 1 {
                        queue!(
                            stdout,
                            MoveTo(terminal_x + start_x, terminal_y + CELL_HEIGHT + start_y),
                            Print([LOWER_BORDER, LOWER_BORDER, LOWER_BORDER].concat())
                        )?;
                    } else {
                        queue!(
                            stdout,
                            MoveTo(terminal_x + start_x, terminal_y + CELL_HEIGHT + start_y),
                            Print([LOWER_BORDER, LOWER_BORDER].concat())
                        )?;
                    }
                }
                queue!(
                    stdout,
                    MoveTo(terminal_x + start_x, terminal_y + start_y),
                    SetForegroundColor(color),
                    Print(tile),
                    ResetColor
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum TetrominoKind {
    T,
    I,
    J,
    L,
    S,
    Z,
    O,
}

impl TetrominoKind {
    fn color(&self) -> Color {
        match self {
            TetrominoKind::I => Color::Rgb {
                r: 91,
                g: 206,
                b: 250,
            },
            TetrominoKind::O => Color::Rgb {
                r: 250,
                g: 222,
                b: 91,
            },
            TetrominoKind::T => Color::Rgb {
                r: 190,
                g: 126,
                b: 240,
            },
            TetrominoKind::S => Color::Rgb {
                r: 94,
                g: 214,
                b: 137,
            },
            TetrominoKind::Z => Color::Rgb {
                r: 240,
                g: 96,
                b: 113,
            },
            TetrominoKind::J => Color::Rgb {
                r: 92,
                g: 124,
                b: 250,
            },
            TetrominoKind::L => Color::Rgb {
                r: 245,
                g: 163,
                b: 89,
            },
        }
    }

    fn blocks(&self) -> [(i32, i32); 4] {
        match self {
            TetrominoKind::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            TetrominoKind::O => [(1, 0), (2, 0), (1, 1), (2, 1)],
            TetrominoKind::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            TetrominoKind::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            TetrominoKind::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            TetrominoKind::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            TetrominoKind::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        }
    }
}

struct Tetromino {
    kind: TetrominoKind,
    color: Color,
    blocks: [(i32, i32); 4],
}

impl Tetromino {
    fn new(kind: TetrominoKind) -> Self {
        Self {
            color: kind.color(),
            blocks: kind.blocks(),
            kind: kind,
        }
    }
}

struct ActiveTetromino {
    kind: TetrominoKind,
    x: i32,
    y: i32,
}

impl ActiveTetromino {
    fn from(tet: Tetromino) -> Self {
        Self {
            kind: tet.kind,
            x: 3,
            y: 0,
        }
    }
}

struct TetrominoBag {
    pieces: Vec<Tetromino>,
}

impl TetrominoBag {
    fn new() -> Self {
        Self {
            pieces: vec![
                Tetromino::new(TetrominoKind::T),
                Tetromino::new(TetrominoKind::I),
                Tetromino::new(TetrominoKind::J),
                Tetromino::new(TetrominoKind::L),
                Tetromino::new(TetrominoKind::S),
                Tetromino::new(TetrominoKind::Z),
                Tetromino::new(TetrominoKind::O),
            ],
        }
    }
}

struct Game {
    grid: Grid,
    active: ActiveTetromino,
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();

    let grid = Grid::new(10, 20);
    let mut bag = TetrominoBag::new();

    let tetromino = spawn_from_bag(&mut bag);
    let mut active = ActiveTetromino::from(tetromino);

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        grid.draw(&mut stdout, Some(&active))?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => active.x -= 1,
                    KeyCode::Right => active.x += 1,
                    KeyCode::Down => active.y += 1,
                    _ => {}
                }
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}

fn spawn_from_bag(bag: &mut TetrominoBag) -> Tetromino {
    if bag.pieces.is_empty() {
        *bag = TetrominoBag::new();
    }

    let selector = rand::random_range(0..bag.pieces.len());
    bag.pieces.remove(selector)
}
