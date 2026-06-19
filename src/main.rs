use std::{
    intrinsics::autodiff, io::{Stdout, Write, stdout}, time::{Duration, Instant}, vec
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
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

const CELL_WIDTH: u16 = 2;
const CELL_HEIGHT: u16 = 1;
const LOWER_BORDER: &str = "▀";
const UPPER_BORDER: &str = "▄";
const VERT_BORDER: &str = "█";
const TET_BLOCK: &str = "██";

struct RenderLayout {
    cell_width: u16,
    cell_height: u16,
    board_start_x: u16,
    board_start_y: u16,
    hold_start_x: u16,
    hold_start_y: u16,
}

impl RenderLayout {
    fn new(term_width: u16, term_height: u16, grid_width: u16, grid_height: u16) -> Self {
        let board_width = grid_width * CELL_WIDTH + 2;
        let board_height = grid_height * CELL_HEIGHT + 1;

        let board_start_x = term_width / 2 - board_width / 2;
        let board_start_y = term_height / 2 - board_height / 2;

        let hold_start_x = board_start_x.saturating_sub(8);
        let hold_start_y = board_start_y;

        Self {
            cell_width: CELL_WIDTH,
            cell_height: CELL_HEIGHT,
            board_start_x,
            board_start_y,
            hold_start_x,
            hold_start_y,
        }
    }
}

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

    fn clear_line(&mut self, row: u16) {
        for x in 0..self.width {
            self.cells[row as usize][x as usize] = None;
        }
    }

    fn shift_row_down(&mut self, row: u16) {
        for above_row in (0..row).rev() {
            for x in 0..self.width {
                let above = self.cells[above_row as usize][x as usize];
                self.cells[above_row as usize][x as usize] = None;
                self.cells[(above_row + 1) as usize][x as usize] = above;
            }
        }
    }

    fn destruct_lines(&mut self) {
        let mut full_rows: Vec<u16> = vec![];
        for y in 0..self.height {
            let mut filled = true;
            for x in 0..self.width {
                if self.cells[y as usize][x as usize].is_none() {
                    filled = false;
                }
            }

            if filled {
                full_rows.push(y);
            }
        }

        for y in full_rows {
            self.clear_line(y);
            self.shift_row_down(y);
        }
    }

    fn hits_wall(&self, active: &ActiveTetromino, next_x: i32, next_rotation: usize) -> bool {
        for (block_x, _) in active.kind.blocks(next_rotation) {
            let x = next_x + block_x;

            if x < 0 || x >= self.width.into() {
                return true;
            }
        }
        false
    }

    fn generate_ghost(&self, active: Option<&ActiveTetromino>) -> Option<ActiveTetromino> {
        let active = active?;

        let mut ghost = *active;

        for y in 0..self.height {
            if !self.is_valid_position(&ghost, ghost.x, ghost.y + y as i32, ghost.rotation) {
                ghost.y += y as i32 - 1;
                return Some(ghost);
            }
        }
        None
    }

    fn draw(
        &self,
        stdout: &mut Stdout,
        active: Option<&ActiveTetromino>,
        layout: &RenderLayout,
    ) -> std::io::Result<()> {
        let board_left = 0;
        let board_top = 0;

        let border_left = board_left;

        let cells_left = board_left + 1;
        let cells_top = board_top + 1;

        let ghost = self.generate_ghost(active);

        for y in 0..self.height {
            for x in 0..self.width {
                let terminal_x = cells_left + x * layout.cell_width;
                let terminal_y = cells_top + y * layout.cell_height;

                let active_cell = self.active_tet_at(active, x, y);
                let locked_cell = self.cells[y as usize][x as usize];
                let ghost_cell = self.active_tet_at(ghost.as_ref(), x, y);

                let (color, tile) = if let Some(kind) = active_cell {
                    (kind.color(), TET_BLOCK)
                } else if let Some(kind) = locked_cell {
                    (kind.color(), TET_BLOCK)
                } else if let Some(_kind) = ghost_cell {
                    (Color::DarkGrey, TET_BLOCK)
                } else {
                    (Color::Rgb { r: 0, g: 0, b: 0 }, "  ")
                };

                if x == border_left {
                    queue!(
                        stdout,
                        MoveTo(
                            terminal_x - 1 + layout.board_start_x,
                            terminal_y + layout.board_start_y
                        ),
                        Print(VERT_BORDER)
                    )?;
                }

                if x == self.width - 1 {
                    queue!(
                        stdout,
                        MoveTo(
                            terminal_x + layout.cell_width + layout.board_start_x,
                            terminal_y + layout.board_start_y
                        ),
                        Print(VERT_BORDER)
                    )?;
                }

                if y == self.height - 1 {
                    if x == border_left {
                        queue!(
                            stdout,
                            MoveTo(
                                terminal_x - 1 + layout.board_start_x,
                                terminal_y + layout.cell_height + layout.board_start_y
                            ),
                            Print([LOWER_BORDER, LOWER_BORDER, LOWER_BORDER].concat())
                        )?;
                    } else if x == self.width - 1 {
                        queue!(
                            stdout,
                            MoveTo(
                                terminal_x + layout.board_start_x,
                                terminal_y + layout.cell_height + layout.board_start_y
                            ),
                            Print([LOWER_BORDER, LOWER_BORDER, LOWER_BORDER].concat())
                        )?;
                    } else {
                        queue!(
                            stdout,
                            MoveTo(
                                terminal_x + layout.board_start_x,
                                terminal_y + layout.cell_height + layout.board_start_y
                            ),
                            Print([LOWER_BORDER, LOWER_BORDER].concat())
                        )?;
                    }
                }
                queue!(
                    stdout,
                    MoveTo(
                        terminal_x + layout.board_start_x,
                        terminal_y + layout.board_start_y
                    ),
                    SetForegroundColor(color),
                    Print(tile),
                    ResetColor
                )?;
            }
        }
        Ok(())
    }
}

struct TetrominoBuffer {
    held: Option<ActiveTetromino>,
}

impl TetrominoBuffer {
    fn init() -> Self {
        Self { held: None }
    }

    fn swap(&mut self, tetromino: ActiveTetromino) -> Option<ActiveTetromino> {
        let piece = self.held;
        self.held = Some(tetromino);
        piece
    }

    fn draw(&self, stdout: &mut Stdout, layout: &RenderLayout) -> std::io::Result<()> {
        //TODO: Draw border around buffer piece

        if let Some(held) = self.held {
            for (block_x, block_y) in held.blocks() {
                let x = layout.hold_start_x + block_x as u16 * layout.cell_width;
                let y = layout.hold_start_y + block_y as u16 * layout.cell_height + layout.cell_height * 2;

                queue!(
                    stdout,
                    MoveTo(x, y),
                    SetForegroundColor(held.kind.color()),
                    Print(TET_BLOCK),
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
}

impl Tetromino {
    fn new(kind: TetrominoKind) -> Self {
        Self { kind: kind }
    }
}

#[derive(Clone, Copy)]
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

    fn reset(&mut self) {
        self.x = 3;
        self.y = 0;
        self.rotation = 0;
    }

    fn slot(&mut self, bag: &mut TetrominoBag) {
        *self = ActiveTetromino::from(bag.spawn());
    }

    fn fall(&mut self, grid: &Grid) {
        while self.move_down(grid) {}
    }

    fn blocks(&self) -> [(i32, i32); 4] {
        self.kind.blocks(self.rotation)
    }

    fn rotate(&mut self, grid: &Grid, dir: RotationDirection) {
        let kicks = [1, -1, 2, -1];
        let next_rotation = match dir {
            RotationDirection::Left => (self.rotation + 3) % 4,
            RotationDirection::Right => (self.rotation + 1) % 4,
        };

        if grid.is_valid_position(self, self.x, self.y, next_rotation) {
            self.rotation = next_rotation;
        }

        if !grid.hits_wall(self, self.x, next_rotation) {
            return;
        }

        for dx in kicks {
            if grid.is_valid_position(self, self.x + dx, self.y, next_rotation) {
                self.x += dx;
                self.rotation = next_rotation;
                return;
            }
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

struct Timings {
    drop_interval: Duration,
    lock_delay: Duration,
}

impl Timings {
    fn init() -> Self {
        Self {
            drop_interval: Duration::from_millis(1500),
            lock_delay: Duration::from_millis(500),
        }
    }
}

struct Game {
    grid: Grid,
    bag: TetrominoBag,
    active: ActiveTetromino,
    buffer: TetrominoBuffer,
    has_swapped: bool,
    grounded_since: Option<Instant>,
    last_drop: Instant,
}

impl Game {
    fn new() -> Self {
        let mut bag = TetrominoBag::new();
        let active = ActiveTetromino::from(bag.spawn());
        Self {
            grid: Grid::new(10, 20),
            bag: bag,
            active: active,
            buffer: TetrominoBuffer::init(),
            has_swapped: false,
            grounded_since: None,
            last_drop: Instant::now(),
        }
    }
}

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    let mut game = Game::new();
    let timings = Timings::init();
    let mut last_terminal_size = terminal::size()?;

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        let grounded = !game.grid.is_valid_position(
            &game.active,
            game.active.x,
            game.active.y + 1,
            game.active.rotation,
        );

        if grounded {
            if game.grounded_since.is_none() {
                game.grounded_since = Some(Instant::now());
            }

            if game.grounded_since.unwrap().elapsed() >= timings.lock_delay {
                game.grid.lock_piece(&game.active);
                game.active.slot(&mut game.bag);
                game.grounded_since = None;
                game.has_swapped = false;
            }
        }
        if game.last_drop.elapsed() > timings.drop_interval {
            if !game.active.move_down(&game.grid) {
                game.grid.lock_piece(&game.active);
                game.active.slot(&mut game.bag);
                game.has_swapped = false;
            }
            game.grounded_since = None;
            game.last_drop = Instant::now();
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => game.active.move_left(&game.grid),
                    KeyCode::Right => game.active.move_right(&game.grid),
                    KeyCode::Down => {
                        if game.active.move_down(&game.grid) {
                            game.grounded_since = None;
                        }
                        game.last_drop = Instant::now();
                    }
                    KeyCode::Char(' ') => {
                        game.active.fall(&game.grid);
                        game.grid.lock_piece(&game.active);
                        game.active.slot(&mut game.bag);
                        game.last_drop = Instant::now();
                        game.has_swapped = false;
                    }
                    KeyCode::Char('a') => game.active.rotate(&game.grid, RotationDirection::Left),
                    KeyCode::Char('d') => game.active.rotate(&game.grid, RotationDirection::Right),
                    KeyCode::Char('c') => {
                        if !game.has_swapped {
                            game.has_swapped = true;
                            game.active.reset();
                            let new_piece = game.buffer.swap(game.active);
                            game.active = if let Some(swapped) = new_piece {
                                swapped
                            } else {
                                ActiveTetromino::from(game.bag.spawn())
                            }
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        game.grid.destruct_lines();
        if !game.grid.is_valid_position(
            &game.active,
            game.active.x,
            game.active.y,
            game.active.rotation,
        ) {
            break;
        }

        let (term_width, term_height) = terminal::size()?;
        if (term_width, term_height) != last_terminal_size {
            queue!(stdout, Clear(ClearType::All))?;
            last_terminal_size = (term_width, term_height);
        }

        let layout = RenderLayout::new(term_width, term_height, game.grid.width, game.grid.height);

        game.grid.draw(&mut stdout, Some(&game.active), &layout)?;
        game.buffer.draw(&mut stdout, &layout)?;
        stdout.flush()?;
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
