use std::io::Write;
use std::time::Instant;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
pub const CELL_WIDTH: u16 = 2;
pub const CELL_HEIGHT: u16 = 1;
pub const LOWER_BORDER: &str = "▀";
pub const UPPER_BORDER: &str = "▄";
pub const VERT_BORDER: &str = "█";

pub struct RenderLayout {
    pub cell_width: u16,
    pub cell_height: u16,
    pub board_start_x: u16,
    pub board_start_y: u16,
    pub board_width: u16,
    pub board_height: u16,
    pub hold_start_x: u16,
    pub hold_start_y: u16,
    pub queue_start_x: u16,
    pub queue_start_y: u16,
    pub game_box_start_x: u16,
    pub game_box_start_y: u16,
    pub game_box_height: u16,
    pub controls_start_x: u16,
    pub controls_start_y: u16,
}

impl RenderLayout {
    pub fn new(term_width: u16, term_height: u16, grid_width: u16, grid_height: u16) -> Self {
        let board_width = grid_width * CELL_WIDTH + 2;
        let board_height = grid_height * CELL_HEIGHT + 1;

        let board_start_x = term_width / 2 - board_width / 2;
        let board_start_y = term_height / 2 - board_height / 2;

        let hold_start_x = board_start_x.saturating_sub(16);
        let hold_start_y = board_start_y + 2;

        let queue_start_x = board_start_x + board_width + CELL_WIDTH;
        let queue_start_y = board_start_y + 2;

        let game_box_start_x = hold_start_x;
        let game_box_start_y = board_start_y + 11;
        let game_box_height = grid_height * CELL_HEIGHT - 9;

        let controls_start_x = board_start_x;
        let controls_start_y = board_start_y + board_height + 1;

        Self {
            cell_width: CELL_WIDTH,
            cell_height: CELL_HEIGHT,
            board_start_x,
            board_start_y,
            board_width,
            board_height,
            hold_start_x,
            hold_start_y,
            queue_start_x,
            queue_start_y,
            game_box_start_x,
            game_box_start_y,
            game_box_height,
            controls_start_x,
            controls_start_y,
        }
    }
}

pub struct ControlCenter {
    pub open: bool,
    pub selected: usize,
}

impl ControlCenter {
    pub fn new() -> Self {
        Self { open: false, selected: 0 }
    }

    pub fn draw(&self, stdout: &mut impl Write, layout: &RenderLayout) -> std::io::Result<()> {
        let start_x = layout.hold_start_x.saturating_sub(15);
        let start_y = layout.hold_start_y;
        let border_width: u16 = 12;
        let inner = (border_width - 2) as usize;

        if self.open {
            let open_height: u16 = 8;
            let options = ["PLAY", "MENU", "SETTINGS", "EXIT"];
            let mut top_row = String::new();
            for _ in 0..border_width {
                top_row.push_str(UPPER_BORDER);
            }
            queue!(stdout, MoveTo(start_x, start_y), Print(top_row))?;
            for y in 1..open_height {
                let by = y + start_y;
                if y == open_height - 1 {
                    let mut bottom = String::new();
                    for _ in 0..border_width {
                        bottom.push_str(LOWER_BORDER);
                    }
                    queue!(stdout, MoveTo(start_x, by), Print(bottom))?;
                    continue;
                }
                queue!(stdout, MoveTo(start_x, by), Print(VERT_BORDER))?;
                queue!(stdout, MoveTo(start_x + border_width - 1, by), Print(VERT_BORDER))?;
                let ix = start_x + 1;
                if y >= 2 && (y - 2) < options.len() as u16 {
                    let option_index = (y - 2) as usize;
                    let text = format!("{:^width$}", options[option_index], width = inner);
                    if option_index == self.selected {
                        queue!(
                            stdout,
                            MoveTo(ix, by),
                            SetBackgroundColor(Color::Rgb { r: 50, g: 50, b: 50 }),
                            Print(text),
                            ResetColor
                        )?;
                    } else {
                        queue!(stdout, MoveTo(ix, by), Print(text))?;
                    }
                } else {
                    queue!(stdout, MoveTo(ix, by), Print(format!("{:<width$}", "", width = inner)))?;
                }
            }
        } else {
            let closed_width: u16 = 7;
            let closed_inner = (closed_width - 2) as usize;
            let mut closed_top = String::new();
            for _ in 0..closed_width {
                closed_top.push_str(UPPER_BORDER);
            }
            queue!(stdout, MoveTo(start_x, start_y), Print(closed_top))?;
            queue!(stdout, MoveTo(start_x, start_y + 1), Print(VERT_BORDER))?;
            queue!(stdout, MoveTo(start_x + closed_width - 1, start_y + 1), Print(VERT_BORDER))?;
            queue!(
                stdout,
                MoveTo(start_x + 1, start_y + 1),
                Print(format!("{:^width$}", "ESC", width = closed_inner))
            )?;
            let mut closed_bottom = String::new();
            for _ in 0..closed_width {
                closed_bottom.push_str(LOWER_BORDER);
            }
            queue!(stdout, MoveTo(start_x, start_y + 2), Print(closed_bottom))?;
        }

        Ok(())
    }
}

pub struct GameOverScreen {
    pub active: bool,
    pub selected: usize,
}

impl GameOverScreen {
    pub fn new() -> Self {
        Self { active: false, selected: 0 }
    }

    pub fn draw(&self, stdout: &mut impl Write, layout: &RenderLayout, level: u16, score: u32) -> std::io::Result<()> {
        if !self.active {
            return Ok(());
        }

        let panel_width: u16 = 22;
        let panel_height: u16 = 9;
        let start_x = layout.board_start_x + layout.board_width / 2 - panel_width / 2;
        let start_y = layout.board_start_y + layout.board_height / 2 - panel_height / 2;
        let inner_width = (panel_width - 2) as usize;

        let title = " GAME OVER ";
        let title_offset = (panel_width as usize - title.len()) / 2;
        let mut top_row = String::new();
        for col in 0..panel_width as usize {
            if col >= title_offset && col < title_offset + title.len() {
                top_row.push(title.as_bytes()[col - title_offset] as char);
            } else {
                top_row.push_str(UPPER_BORDER);
            }
        }
        queue!(stdout, MoveTo(start_x, start_y), Print(top_row))?;

        for row in 1..panel_height {
            let row_y = row + start_y;
            if row == panel_height - 1 {
                queue!(stdout, MoveTo(start_x, row_y), Print(LOWER_BORDER.repeat(panel_width as usize)))?;
                continue;
            }
            queue!(stdout, MoveTo(start_x, row_y), Print(VERT_BORDER))?;
            queue!(stdout, MoveTo(start_x + panel_width - 1, row_y), Print(VERT_BORDER))?;
            let content_x = start_x + 1;
            match row {
                2 => queue!(stdout, MoveTo(content_x, row_y), Print(format!("{:^width$}", format!("Level: {}", level), width = inner_width)))?,
                3 => queue!(stdout, MoveTo(content_x, row_y), Print(format!("{:^width$}", format!("Score: {}", score), width = inner_width)))?,
                5 => {
                    let play_label = "[PLAY]";
                    let quit_label = "[QUIT]";
                    let side_pad = (inner_width - play_label.len() - quit_label.len()) / 3;
                    let mid_pad = inner_width - side_pad - play_label.len() - quit_label.len() - side_pad;
                    queue!(stdout, MoveTo(content_x, row_y), Print(format!("{:width$}", "", width = side_pad)))?;
                    if self.selected == 0 {
                        queue!(stdout, SetBackgroundColor(Color::Rgb { r: 60, g: 60, b: 60 }))?;
                    }
                    queue!(stdout, Print(play_label))?;
                    if self.selected == 0 {
                        queue!(stdout, ResetColor)?;
                    }
                    queue!(stdout, Print(format!("{:width$}", "", width = mid_pad)))?;
                    if self.selected == 1 {
                        queue!(stdout, SetBackgroundColor(Color::Rgb { r: 60, g: 60, b: 60 }))?;
                    }
                    queue!(stdout, Print(quit_label))?;
                    if self.selected == 1 {
                        queue!(stdout, ResetColor)?;
                    }
                    queue!(stdout, Print(format!("{:width$}", "", width = side_pad)))?;
                }
                _ => queue!(stdout, MoveTo(content_x, row_y), Print(format!("{:<width$}", "", width = inner_width)))?,
            }
        }

        Ok(())
    }
}

pub fn draw_controls(
    stdout: &mut impl Write,
    layout: &RenderLayout,
    entries: &[(&str, &str)],
) -> std::io::Result<()> {
    let border_width: u16 = 20;
    let inner = (border_width - 2) as usize;
    let border_height = entries.len() as u16 + 2;
    let start_x = layout.controls_start_x;
    let start_y = layout.controls_start_y;

    let title = " KEYS ";
    let title_offset = (border_width as usize - title.len()) / 2;
    let mut top_row = String::new();
    for col in 0..border_width as usize {
        if col >= title_offset && col < title_offset + title.len() {
            top_row.push(title.as_bytes()[col - title_offset] as char);
        } else {
            top_row.push_str(UPPER_BORDER);
        }
    }
    queue!(stdout, MoveTo(start_x, start_y), Print(top_row))?;

    for y in 1..border_height {
        let by = y + start_y;
        if y == border_height - 1 {
            queue!(stdout, MoveTo(start_x, by), Print(LOWER_BORDER.repeat(border_width as usize)))?;
            continue;
        }
        queue!(stdout, MoveTo(start_x, by), Print(VERT_BORDER))?;
        queue!(stdout, MoveTo(start_x + border_width - 1, by), Print(VERT_BORDER))?;

        let entry_index = (y - 1) as usize;
        let (label, key) = entries[entry_index];
        let line = format!(" {:<11} {:>4} ", label, key);
        queue!(stdout, MoveTo(start_x + 1, by), Print(format!("{:<width$}", line, width = inner)))?;
    }

    Ok(())
}

pub fn draw_notification(
    stdout: &mut impl Write,
    last_clear: &Option<(Instant, u16, u32)>,
    layout: &RenderLayout,
) -> std::io::Result<()> {
    let nx = layout.hold_start_x;
    let ny = layout.hold_start_y + 6;
    let width = 14usize;

    if let Some((time, lines, pts)) = last_clear {
        let elapsed = time.elapsed().as_secs_f32();
        if elapsed < 3.0 {
            let (word, base_red, base_green, base_blue) = match lines {
                1 => ("SINGLE", 100u8, 230u8, 100u8),
                2 => ("DOUBLE", 100u8, 180u8, 255u8),
                3 => ("TRIPLE", 255u8, 190u8, 60u8),
                _ => ("TETRIS", 230u8, 80u8, 255u8),
            };
            let factor = if elapsed < 2.0 { 1.0f32 } else { 1.0 - (elapsed - 2.0) };
            let red = (base_red as f32 * factor) as u8;
            let green = (base_green as f32 * factor) as u8;
            let blue = (base_blue as f32 * factor) as u8;
            let color = Color::Rgb { r: red, g: green, b: blue };
            let pts_str = format!("+{}", pts);
            queue!(
                stdout,
                MoveTo(nx, ny),
                SetForegroundColor(color),
                Print(format!("{:^width$}", word, width = width)),
                ResetColor
            )?;
            queue!(
                stdout,
                MoveTo(nx, ny + 1),
                SetForegroundColor(color),
                Print(format!("{:^width$}", pts_str, width = width)),
                ResetColor
            )?;
            return Ok(());
        }
    }

    queue!(stdout, MoveTo(nx, ny), Print(format!("{:<width$}", "", width = width)))?;
    queue!(stdout, MoveTo(nx, ny + 1), Print(format!("{:<width$}", "", width = width)))?;
    Ok(())
}
