use std::io::Stdout;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use crate::render::{
    ColorMode, RenderLayout, CELL_HEIGHT, CELL_WIDTH, LOWER_BORDER, TET_BLOCK, UPPER_BORDER,
    VERT_BORDER,
};

#[derive(Clone, Copy)]
pub enum TetrominoKind {
    T,
    I,
    J,
    L,
    S,
    Z,
    O,
}

impl TetrominoKind {
    pub fn color(&self, mode: ColorMode) -> Color {
        match mode {
            ColorMode::Normal => match self {
                TetrominoKind::I => Color::Rgb { r: 91, g: 206, b: 250 },
                TetrominoKind::O => Color::Rgb { r: 250, g: 222, b: 91 },
                TetrominoKind::T => Color::Rgb { r: 190, g: 126, b: 240 },
                TetrominoKind::S => Color::Rgb { r: 94, g: 214, b: 137 },
                TetrominoKind::Z => Color::Rgb { r: 240, g: 96, b: 113 },
                TetrominoKind::J => Color::Rgb { r: 92, g: 124, b: 250 },
                TetrominoKind::L => Color::Rgb { r: 245, g: 163, b: 89 },
            },
            ColorMode::Deuteranopia | ColorMode::Protanopia => match self {
                TetrominoKind::I => Color::Rgb { r: 80, g: 200, b: 255 },
                TetrominoKind::O => Color::Rgb { r: 255, g: 220, b: 80 },
                TetrominoKind::T => Color::Rgb { r: 200, g: 130, b: 240 },
                TetrominoKind::S => Color::Rgb { r: 80, g: 130, b: 255 },
                TetrominoKind::Z => Color::Rgb { r: 255, g: 165, b: 40 },
                TetrominoKind::J => Color::Rgb { r: 50, g: 80, b: 200 },
                TetrominoKind::L => Color::Rgb { r: 255, g: 200, b: 50 },
            },
            ColorMode::Tritanopia => match self {
                TetrominoKind::I => Color::Rgb { r: 100, g: 220, b: 220 },
                TetrominoKind::O => Color::Rgb { r: 240, g: 130, b: 210 },
                TetrominoKind::T => Color::Rgb { r: 200, g: 130, b: 240 },
                TetrominoKind::S => Color::Rgb { r: 100, g: 210, b: 130 },
                TetrominoKind::Z => Color::Rgb { r: 240, g: 100, b: 110 },
                TetrominoKind::J => Color::Rgb { r: 220, g: 160, b: 80 },
                TetrominoKind::L => Color::Rgb { r: 240, g: 100, b: 60 },
            },
        }
    }

    pub fn blocks(&self, rotation: usize) -> [(i32, i32); 4] {
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

pub struct Tetromino {
    pub kind: TetrominoKind,
}

impl Tetromino {
    pub fn new(kind: TetrominoKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Copy)]
pub struct ActiveTetromino {
    pub kind: TetrominoKind,
    pub x: i32,
    pub y: i32,
    pub rotation: usize,
}

pub enum RotationDirection {
    Left,
    Right,
}

impl ActiveTetromino {
    pub fn from(tet: Tetromino) -> Self {
        Self {
            kind: tet.kind,
            x: 3,
            y: 0,
            rotation: 0,
        }
    }

    pub fn reset(&mut self) {
        self.x = 3;
        self.y = 0;
        self.rotation = 0;
    }

    pub fn slot(&mut self, queue: &mut TetrominoQueue, bag: &mut TetrominoBag) {
        *self = ActiveTetromino::from(queue.swap(bag));
    }

    pub fn fall(&mut self, grid: &Grid) {
        while self.move_down(grid) {}
    }

    pub fn blocks(&self) -> [(i32, i32); 4] {
        self.kind.blocks(self.rotation)
    }

    pub fn rotate(&mut self, grid: &Grid, dir: RotationDirection) {
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

    pub fn move_left(&mut self, grid: &Grid) {
        if grid.is_valid_position(self, self.x - 1, self.y, self.rotation) {
            self.x -= 1;
        }
    }

    pub fn move_right(&mut self, grid: &Grid) {
        if grid.is_valid_position(self, self.x + 1, self.y, self.rotation) {
            self.x += 1;
        }
    }

    pub fn move_down(&mut self, grid: &Grid) -> bool {
        if grid.is_valid_position(self, self.x, self.y + 1, self.rotation) {
            self.y += 1;
            true
        } else {
            false
        }
    }
}

pub struct TetrominoBag {
    pieces: Vec<Tetromino>,
}

impl TetrominoBag {
    pub fn new() -> Self {
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

    pub fn spawn(&mut self) -> Tetromino {
        if self.pieces.is_empty() {
            *self = TetrominoBag::new();
        }
        let selector = rand::random_range(0..self.pieces.len());
        self.pieces.remove(selector)
    }
}

pub struct TetrominoQueue {
    pub tetrominos: Vec<Tetromino>,
    pub size: u16,
}

impl TetrominoQueue {
    pub fn new(bag: &mut TetrominoBag, size: u16) -> Self {
        let mut tetrominos = vec![];
        for _ in 0..size {
            tetrominos.push(bag.spawn());
        }
        Self { tetrominos, size }
    }

    pub fn swap(&mut self, bag: &mut TetrominoBag) -> Tetromino {
        let next = self.tetrominos.remove(0);
        self.tetrominos.push(bag.spawn());
        next
    }

    pub fn draw(&self, stdout: &mut Stdout, layout: &RenderLayout, color_mode: ColorMode) -> std::io::Result<()> {
        let border_width = 14u16;
        let border_height = 20u16;

        let title = " NEXT ";
        let title_x = layout.queue_start_x + (border_width - title.len() as u16) / 2;
        let title_local = (title_x - layout.queue_start_x) as usize;

        for y in 0..border_height {
            if y == 0 {
                let mut top_row = String::new();
                for x in 0..border_width as usize {
                    if x >= title_local && x < title_local + title.len() {
                        top_row.push(title.as_bytes()[x - title_local] as char);
                    } else {
                        top_row.push_str(UPPER_BORDER);
                    }
                }
                queue!(
                    stdout,
                    MoveTo(layout.queue_start_x, layout.queue_start_y),
                    Print(top_row)
                )?;
                continue;
            }

            let mut skip_x: Option<u16> = None;
            for x in 0..border_width {
                let border_y = y + layout.queue_start_y;
                let border_x = x + layout.queue_start_x;

                if y == border_height - 1 {
                    queue!(stdout, MoveTo(border_x, border_y), Print(LOWER_BORDER))?;
                } else if x == 0 || x == border_width - 1 {
                    queue!(stdout, MoveTo(border_x, border_y), Print(VERT_BORDER))?;
                } else if skip_x == Some(x) {
                    skip_x = None;
                } else {
                    let mut drew_block = false;
                    for (i, block) in self.tetrominos.iter().enumerate() {
                        let y_offset = i as u16 * 4;
                        for (block_x, block_y) in block.kind.blocks(0) {
                            let bx = block_x as u16 * layout.cell_width + border_width / 4;
                            let by = block_y as u16 * layout.cell_height
                                + y_offset
                                + self.size / 2
                                + CELL_HEIGHT;
                            if bx == x && by == y {
                                queue!(
                                    stdout,
                                    MoveTo(border_x, border_y),
                                    SetForegroundColor(block.kind.color(color_mode)),
                                    Print(TET_BLOCK),
                                    ResetColor
                                )?;
                                skip_x = Some(x + 1);
                                drew_block = true;
                                break;
                            }
                        }
                        if drew_block {
                            break;
                        }
                    }
                    if !drew_block {
                        queue!(stdout, MoveTo(border_x, border_y), Print(" "))?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct TetrominoBuffer {
    pub held: Option<ActiveTetromino>,
}

impl TetrominoBuffer {
    pub fn init() -> Self {
        Self { held: None }
    }

    pub fn swap(&mut self, tetromino: ActiveTetromino) -> Option<ActiveTetromino> {
        let piece = self.held;
        self.held = Some(tetromino);
        piece
    }

    pub fn draw(&self, stdout: &mut Stdout, layout: &RenderLayout, color_mode: ColorMode) -> std::io::Result<()> {
        let border_width = 14;
        let border_height = 6;
        let title = " HOLD ";
        let title_x = layout.hold_start_x + (border_width - title.len() as u16) / 2;
        let title_local = (title_x - layout.hold_start_x) as usize;

        let mut top_row = String::new();
        for x in 0..border_width as usize {
            if x >= title_local && x < title_local + title.len() {
                top_row.push(title.as_bytes()[x - title_local] as char);
            } else {
                top_row.push_str(UPPER_BORDER);
            }
        }
        queue!(
            stdout,
            MoveTo(layout.hold_start_x, layout.hold_start_y),
            Print(top_row)
        )?;

        for y in 1..border_height {
            let mut skip_x: Option<u16> = None;
            for x in 0..border_width {
                let border_y = y + layout.hold_start_y;
                let border_x = x + layout.hold_start_x;

                if y == border_height - 1 {
                    queue!(stdout, MoveTo(border_x, border_y), Print(LOWER_BORDER))?;
                } else if x == 0 || x == border_width - 1 {
                    queue!(stdout, MoveTo(border_x, border_y), Print(VERT_BORDER))?;
                } else if skip_x == Some(x) {
                    skip_x = None;
                } else {
                    let mut drew_block = false;
                    if let Some(held) = self.held {
                        for (bx, by) in held.blocks() {
                            let block_x = bx as u16 * layout.cell_width + border_width / 4;
                            let block_y = by as u16 * layout.cell_height + border_height / 3;
                            if block_x == x && block_y == y {
                                queue!(
                                    stdout,
                                    MoveTo(border_x, border_y),
                                    SetForegroundColor(held.kind.color(color_mode)),
                                    Print(TET_BLOCK),
                                    ResetColor
                                )?;
                                skip_x = Some(x + 1);
                                drew_block = true;
                                break;
                            }
                        }
                    }
                    if !drew_block {
                        queue!(stdout, MoveTo(border_x, border_y), Print(" "))?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct Grid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Vec<Option<TetrominoKind>>>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![None; width.into()]; height.into()];
        Self { width, height, cells }
    }

    pub fn active_tet_at(
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

    pub fn is_valid_position(
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

    pub fn lock_piece(&mut self, active: &ActiveTetromino) {
        for (block_x, block_y) in active.blocks() {
            let x = active.x + block_x;
            let y = active.y + block_y;
            if y >= 0 {
                self.cells[y as usize][x as usize] = Some(active.kind);
            }
        }
    }

    fn clear_line(&mut self, row: &u16) {
        for x in 0..self.width {
            self.cells[*row as usize][x as usize] = None;
        }
    }

    fn shift_row_down(&mut self, row: &u16) {
        for above_row in (0..*row).rev() {
            for x in 0..self.width {
                let above = self.cells[above_row as usize][x as usize];
                self.cells[above_row as usize][x as usize] = None;
                self.cells[(above_row + 1) as usize][x as usize] = above;
            }
        }
    }

    pub fn destroy_lines(&mut self) -> u16 {
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
        for y in &full_rows {
            self.clear_line(y);
            self.shift_row_down(y);
        }
        full_rows.len() as u16
    }

    pub fn hits_wall(&self, active: &ActiveTetromino, next_x: i32, next_rotation: usize) -> bool {
        for (block_x, _) in active.kind.blocks(next_rotation) {
            let x = next_x + block_x;
            if x < 0 || x >= self.width.into() {
                return true;
            }
        }
        false
    }

    pub fn generate_ghost(&self, active: Option<&ActiveTetromino>) -> Option<ActiveTetromino> {
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

    pub fn draw(
        &self,
        stdout: &mut Stdout,
        active: Option<&ActiveTetromino>,
        layout: &RenderLayout,
        color_mode: ColorMode,
    ) -> std::io::Result<()> {
        let border_left = 0u16;
        let cells_left = border_left + 1;
        let cells_top = 1u16;

        let ghost = self.generate_ghost(active);

        for y in 0..self.height {
            for x in 0..self.width {
                let terminal_x = cells_left + x * layout.cell_width;
                let terminal_y = cells_top + y * layout.cell_height;

                let active_cell = self.active_tet_at(active, x, y);
                let locked_cell = self.cells[y as usize][x as usize];
                let ghost_cell = self.active_tet_at(ghost.as_ref(), x, y);

                let (color, tile) = if let Some(kind) = active_cell {
                    (kind.color(color_mode), TET_BLOCK)
                } else if let Some(kind) = locked_cell {
                    (kind.color(color_mode), TET_BLOCK)
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
