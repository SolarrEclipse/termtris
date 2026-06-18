use std::{
    io::{Stdout, Write, stdout},
    time::{Duration, Instant},
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
        let cells = vec![vec![None; width.into()]; height.into()];

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

        for (block_x, block_y) in active.blocks() {
            let board_x = active.x + block_x;
            let board_y = active.y + block_y;

            if board_x == x as i32 && board_y == y as i32 {
                return Some(active.kind);
            }
        }
        None
    }

    fn is_valid_position(
        &self,
        active: &ActiveTetromino,
        next_x: i32,
        next_y: i32,
        next_rotation: usize,
    ) -> bool {
        for (block_x, block_y) in active.kind.blocks(next_rotation) {
            let x = next_x + block_x;
            let y = next_y + block_y;

            if x < 0 || x >= self.width as i32 {
                return false;
            }

            if y >= self.height as i32 {
                return false;
            }

            if y >= 0 && self.cells[y as usize][x as usize].is_some() {
                return false;
            }
        }

        true
    }

    fn lock_piece(&mut self, active: &ActiveTetromino) {
        for (block_x, block_y) in active.blocks() {
            let x = active.x + block_x;
            let y = active.y + block_y;

            if y >= 0 {
                self.cells[y as usize][x as usize] = Some(active.kind);
            }
        }
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

    fn blocks(&self, rotation: usize) -> [(i32, i32); 4] {
        match (self, rotation % 4) {
            (TetrominoKind::I, 0) => [(0, 1), (1, 1), (2, 1), (3, 1)],
            (TetrominoKind::I, 1) => [(2, 0), (2, 1), (2, 2), (2, 3)],
            (TetrominoKind::I, 2) => [(0, 2), (1, 2), (2, 2), (3, 2)],
            (TetrominoKind::I, 3) => [(1, 0), (1, 1), (1, 2), (1, 3)],

            (TetrominoKind::O, _) => [(1, 0), (2, 0), (1, 1), (2, 1)],

            (TetrominoKind::T, 0) => [(0, 0), (1, 0), (2, 0), (1, 1)],
            (TetrominoKind::T, 1) => [(1, 0), (0, 1), (1, 1), (1, 2)],
            (TetrominoKind::T, 2) => [(1, 0), (0, 1), (1, 1), (2, 1)],
            (TetrominoKind::T, 3) => [(0, 0), (0, 1), (1, 1), (0, 2)],

            (TetrominoKind::S, 0) => [(1, 0), (2, 0), (0, 1), (1, 1)],
            (TetrominoKind::S, 1) => [(0, 0), (0, 1), (1, 1), (1, 2)],
            (TetrominoKind::S, 2) => [(1, 1), (2, 1), (0, 2), (1, 2)],
            (TetrominoKind::S, 3) => [(1, 0), (1, 1), (2, 1), (2, 2)],

            (TetrominoKind::Z, 0) => [(0, 0), (1, 0), (1, 1), (2, 1)],
            (TetrominoKind::Z, 1) => [(1, 0), (0, 1), (1, 1), (0, 2)],
            (TetrominoKind::Z, 2) => [(0, 1), (1, 1), (1, 2), (2, 2)],
            (TetrominoKind::Z, 3) => [(2, 0), (1, 1), (2, 1), (1, 2)],

            (TetrominoKind::J, 0) => [(0, 0), (0, 1), (1, 1), (2, 1)],
            (TetrominoKind::J, 1) => [(0, 0), (1, 0), (0, 1), (0, 2)],
            (TetrominoKind::J, 2) => [(0, 0), (1, 0), (2, 0), (2, 1)],
            (TetrominoKind::J, 3) => [(1, 0), (1, 1), (0, 2), (1, 2)],

            (TetrominoKind::L, 0) => [(2, 0), (0, 1), (1, 1), (2, 1)],
            (TetrominoKind::L, 1) => [(0, 0), (0, 1), (0, 2), (1, 2)],
            (TetrominoKind::L, 2) => [(0, 0), (1, 0), (2, 0), (0, 1)],
            (TetrominoKind::L, 3) => [(0, 0), (1, 0), (1, 1), (1, 2)],
            _ => unreachable!(),
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
            blocks: kind.blocks(0),
            kind: kind,
        }
    }
}

struct ActiveTetromino {
    kind: TetrominoKind,
    x: i32,
    y: i32,
    rotation: usize,
}

enum RotationDirection {
    Left,
    Right,
}

impl ActiveTetromino {
    fn from(tet: Tetromino) -> Self {
        Self {
            kind: tet.kind,
            x: 3,
            y: 0,
            rotation: 0,
        }
    }

    fn slot(&mut self, bag: &mut TetrominoBag) {
        *self = ActiveTetromino::from(bag.spawn());
    }

    fn blocks(&self) -> [(i32, i32); 4] {
        self.kind.blocks(self.rotation)
    }

    fn rotate(&mut self, grid: &Grid, dir: RotationDirection) {
        let next_rotation = match dir {
            RotationDirection::Left => (self.rotation + 3) % 4,
            RotationDirection::Right => (self.rotation + 1) % 4,
        };

        if grid.is_valid_position(self, self.x, self.y, next_rotation) {
            self.rotation = next_rotation;
        }
    }

    fn move_left(&mut self, grid: &Grid) {
        if grid.is_valid_position(self, self.x - 1, self.y, self.rotation) {
            self.x -= 1;
        }
    }

    fn move_right(&mut self, grid: &Grid) {
        if grid.is_valid_position(self, self.x + 1, self.y, self.rotation) {
            self.x += 1;
        }
    }

    fn move_down(&mut self, grid: &Grid) -> bool {
        if grid.is_valid_position(self, self.x, self.y + 1, self.rotation) {
            self.y += 1;
            true
        } else {
            false
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

    fn spawn(&mut self) -> Tetromino {
        if self.pieces.is_empty() {
            *self = TetrominoBag::new();
        }

        let selector = rand::random_range(0..self.pieces.len());
        self.pieces.remove(selector)
    }
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    let drop_interval = Duration::from_millis(1500);
    let mut last_drop = Instant::now();

    let mut grid = Grid::new(10, 20);
    let mut bag = TetrominoBag::new();

    let tetromino = bag.spawn();
    let mut active = ActiveTetromino::from(tetromino);

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        if last_drop.elapsed() > drop_interval {
            if !active.move_down(&grid) {
                grid.lock_piece(&active);
                active.slot(&mut bag);
            }
            last_drop = Instant::now();
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => active.move_left(&grid),
                    KeyCode::Right => active.move_right(&grid),
                    KeyCode::Down => {
                        if !active.move_down(&grid) {
                            grid.lock_piece(&active);
                            active.slot(&mut bag);
                        }
                        last_drop = Instant::now();
                    },
                    KeyCode::Char('a') => active.rotate(&grid, RotationDirection::Left),
                    KeyCode::Char('d') => active.rotate(&grid, RotationDirection::Right),
                    KeyCode::Char(' ') => todo!(),
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        grid.draw(&mut stdout, Some(&active))?;
        stdout.flush()?;
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
