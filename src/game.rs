use std::io::Write;
use std::time::{Duration, Instant};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use crate::render::{RenderLayout, LOWER_BORDER, UPPER_BORDER, VERT_BORDER};
use crate::tetromino::{ActiveTetromino, Grid, TetrominoBag, TetrominoBuffer, TetrominoQueue};

pub struct Timings {
    pub drop_interval: Duration,
    pub lock_delay: Duration,
    pub clear_animation_duration: Duration,
}

impl Timings {
    pub fn init() -> Self {
        Self {
            drop_interval: Duration::from_millis(1500),
            lock_delay: Duration::from_millis(1000),
            clear_animation_duration: Duration::from_millis(400),
        }
    }

    pub fn recalculate(&mut self, level: u16) {
        let halvings = (level / 10) as u32;
        self.drop_interval = Duration::from_millis(1500 >> halvings);
    }
}

pub struct Progress {
    pub score: u32,
    pub lines: u16,
    pub level: u16,
    pub combo: u16,
    pub last_was_clear: bool,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            score: 0,
            lines: 0,
            level: 1,
            combo: 0,
            last_was_clear: false,
        }
    }

    pub fn increase(&mut self, cleared_lines: u16) -> u32 {
        if cleared_lines == 0 {
            self.combo = 0;
            self.last_was_clear = false;
            return 0;
        }

        if self.last_was_clear {
            self.combo += 1;
        } else {
            self.combo = 1;
        }
        self.last_was_clear = true;
        self.lines += cleared_lines;

        let pts = if cleared_lines == 1 {
            100 * self.level as u32
        } else if cleared_lines == 2 {
            300 * self.level as u32
        } else if cleared_lines == 3 {
            500 * self.level as u32
        } else {
            800 * self.level as u32
        };

        self.score += pts;
        self.level = self.lines / 10 + 1;
        pts
    }

    pub fn draw(
        &self,
        stdout: &mut impl Write,
        elapsed: Duration,
        layout: &RenderLayout,
    ) -> std::io::Result<()> {
        let border_width = 14u16;
        let border_height = layout.game_box_height;
        let x0 = layout.game_box_start_x;
        let y0 = layout.game_box_start_y;

        let title = " GAME ";
        let title_x = x0 + (border_width - title.len() as u16) / 2;
        let title_local = (title_x - x0) as usize;

        let mut top_row = String::new();
        for x in 0..border_width as usize {
            if x >= title_local && x < title_local + title.len() {
                top_row.push(title.as_bytes()[x - title_local] as char);
            } else {
                top_row.push_str(UPPER_BORDER);
            }
        }
        queue!(stdout, MoveTo(x0, y0), Print(top_row))?;

        for y in 1..border_height {
            let by = y + y0;
            if y == border_height - 1 {
                let bottom: String = LOWER_BORDER.repeat(border_width as usize);
                queue!(stdout, MoveTo(x0, by), Print(bottom))?;
                continue;
            }
            queue!(stdout, MoveTo(x0, by), Print(VERT_BORDER))?;
            queue!(stdout, MoveTo(x0 + border_width - 1, by), Print(VERT_BORDER))?;

            let inner = (border_width - 2) as usize;
            let ix = x0 + 1;

            if y == 1 {
                queue!(stdout, MoveTo(ix, by), Print(format!("{:<width$}", "", width = inner)))?;
            } else if y == 2 {
                let lv = format!("Lv. {}", self.level);
                queue!(stdout, MoveTo(ix, by), Print(format!("{:^width$}", lv, width = inner)))?;
            } else if y == 3 {
                let lines_in_level = self.lines % 10;
                let filled = (lines_in_level as usize * inner) / 10;
                let empty = inner - filled;
                let green_dashes: String = "-".repeat(filled);
                let grey_dashes: String = "-".repeat(empty);
                queue!(
                    stdout,
                    MoveTo(ix, by),
                    SetForegroundColor(Color::Green),
                    Print(green_dashes),
                    SetForegroundColor(Color::DarkGrey),
                    Print(grey_dashes),
                    ResetColor
                )?;
            } else if y == 4 {
                queue!(stdout, MoveTo(ix, by), Print(format!("{:<width$}", "", width = inner)))?;
            } else if y == 5 {
                queue!(stdout, MoveTo(ix, by), Print(format!("{:^width$}", "Score:", width = inner)))?;
            } else if y == 6 {
                queue!(stdout, MoveTo(ix, by), Print(format!("{:^width$}", self.score, width = inner)))?;
            } else if y == 7 {
                let combo_str = if self.combo > 1 { format!("x{}", self.combo) } else { String::new() };
                queue!(stdout, MoveTo(ix, by), Print(format!("{:^width$}", combo_str, width = inner)))?;
            } else if y == border_height - 2 {
                queue!(stdout, MoveTo(ix, by), Print(format!("{:^width$}", format_time(elapsed), width = inner)))?;
            } else {
                queue!(stdout, MoveTo(ix, by), Print(format!("{:<width$}", "", width = inner)))?;
            }
        }

        Ok(())
    }
}

fn format_time(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        format!("{}h {:02}m", secs / 3600, (secs % 3600) / 60)
    }
}

pub struct LineClearAnimation {
    pub rows: Vec<u16>,
    pub center_column: i32,
    pub started_at: Instant,
    pub duration: Duration,
}

pub struct Game {
    pub grid: Grid,
    pub bag: TetrominoBag,
    pub queue: TetrominoQueue,
    pub active: ActiveTetromino,
    pub buffer: TetrominoBuffer,
    pub progress: Progress,
    pub has_swapped: bool,
    pub grounded_since: Option<Instant>,
    pub last_drop: Instant,
    pub game_start: Instant,
    pub last_clear: Option<(Instant, u16, u32)>,
    pub clearing: Option<LineClearAnimation>,
}

impl Game {
    pub fn new() -> Self {
        let mut bag = TetrominoBag::new();
        let mut queue = TetrominoQueue::new(&mut bag, 4);
        let active = ActiveTetromino::from(queue.swap(&mut bag));
        Self {
            grid: Grid::new(10, 20),
            bag,
            queue,
            active,
            buffer: TetrominoBuffer::init(),
            progress: Progress::new(),
            has_swapped: false,
            grounded_since: None,
            last_drop: Instant::now(),
            game_start: Instant::now(),
            last_clear: None,
            clearing: None,
        }
    }

    pub fn lock_and_start_clear(&mut self, clear_duration: Duration) {
        self.grid.lock_piece(&self.active);
        let full_rows = self.grid.find_full_rows();
        if full_rows.is_empty() {
            self.active.slot(&mut self.queue, &mut self.bag);
            self.grounded_since = None;
            self.has_swapped = false;
        } else {
            let center_column = self.active.x + 1;
            self.clearing = Some(LineClearAnimation {
                rows: full_rows,
                center_column,
                started_at: Instant::now(),
                duration: clear_duration,
            });
        }
    }

    pub fn finish_clear(&mut self) {
        let rows = if let Some(ref anim) = self.clearing {
            anim.rows.clone()
        } else {
            return;
        };
        let lines = self.grid.remove_rows(&rows);
        let pts = self.progress.increase(lines);
        if lines > 0 {
            self.last_clear = Some((Instant::now(), lines, pts));
        }
        self.clearing = None;
        self.active.slot(&mut self.queue, &mut self.bag);
        self.grounded_since = None;
        self.has_swapped = false;
    }

    pub fn hard_drop(&mut self, clear_duration: Duration) {
        self.active.fall(&self.grid);
        self.lock_and_start_clear(clear_duration);
        self.last_drop = Instant::now();
    }
}
