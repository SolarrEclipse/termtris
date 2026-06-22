use std::io::Stdout;
use crossterm::{
    cursor::MoveTo,
    event::KeyCode,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use serde::{Deserialize, Serialize};
use crate::render::{ColorMode, RenderLayout, LOWER_BORDER, UPPER_BORDER, VERT_BORDER};

const SETTINGS_PATH: &str = "settings.toml";
const KEYBIND_COUNT: usize = 8;

#[derive(Serialize, Deserialize)]
pub struct KeyBindings {
    pub move_left: String,
    pub move_right: String,
    pub move_down: String,
    pub rotate_left: String,
    pub rotate_right: String,
    pub hard_drop: String,
    pub hold: String,
    pub pause: String,
}

#[derive(Serialize, Deserialize)]
pub struct DisplaySettings {
    pub color_mode: String,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub keys: KeyBindings,
    pub display: DisplaySettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            keys: KeyBindings {
                move_left: "Left".to_string(),
                move_right: "Right".to_string(),
                move_down: "Down".to_string(),
                rotate_left: "a".to_string(),
                rotate_right: "d".to_string(),
                hard_drop: "Space".to_string(),
                hold: "c".to_string(),
                pause: "Escape".to_string(),
            },
            display: DisplaySettings {
                color_mode: "Normal".to_string(),
            },
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if let Ok(contents) = std::fs::read_to_string(SETTINGS_PATH) {
            toml::from_str(&contents).unwrap_or_else(|_| Self::default())
        } else {
            let default = Self::default();
            let _ = default.save();
            default
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(SETTINGS_PATH, contents)
    }

    pub fn color_mode(&self) -> ColorMode {
        match self.display.color_mode.as_str() {
            "Deut." => ColorMode::Deuteranopia,
            "Prot." => ColorMode::Protanopia,
            "Trit." => ColorMode::Tritanopia,
            _ => ColorMode::Normal,
        }
    }
}

pub fn keycode_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Esc => "Escape".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        _ => "Unknown".to_string(),
    }
}

pub fn string_to_keycode(s: &str) -> Option<KeyCode> {
    match s {
        "Left" => Some(KeyCode::Left),
        "Right" => Some(KeyCode::Right),
        "Up" => Some(KeyCode::Up),
        "Down" => Some(KeyCode::Down),
        "Escape" => Some(KeyCode::Esc),
        "Enter" => Some(KeyCode::Enter),
        "Backspace" => Some(KeyCode::Backspace),
        "Tab" => Some(KeyCode::Tab),
        "Space" => Some(KeyCode::Char(' ')),
        s if s.len() == 1 => Some(KeyCode::Char(s.chars().next().unwrap())),
        _ => None,
    }
}

fn key_display(binding: &str) -> String {
    match binding {
        "Left" => "←".to_string(),
        "Right" => "→".to_string(),
        "Up" => "↑".to_string(),
        "Down" => "↓".to_string(),
        "Escape" => "Esc".to_string(),
        "Space" => "Spc".to_string(),
        "Enter" => "Ret".to_string(),
        "Backspace" => "Bsp".to_string(),
        "Tab" => "Tab".to_string(),
        s if s.len() == 1 => s.to_uppercase(),
        s => s.to_string(),
    }
}

fn next_color_mode(current: &str) -> String {
    match current {
        "Normal" => "Deut.".to_string(),
        "Deut." => "Prot.".to_string(),
        "Prot." => "Trit.".to_string(),
        _ => "Normal".to_string(),
    }
}

pub struct SettingsScreen {
    pub open: bool,
    selected: usize,
    rebinding: bool,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            open: false,
            selected: 0,
            rebinding: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, settings: &mut Settings) {
        if self.rebinding {
            if key == KeyCode::Esc {
                self.rebinding = false;
                return;
            }
            let key_str = keycode_to_string(key);
            match self.selected {
                0 => settings.keys.move_left = key_str,
                1 => settings.keys.move_right = key_str,
                2 => settings.keys.move_down = key_str,
                3 => settings.keys.rotate_left = key_str,
                4 => settings.keys.rotate_right = key_str,
                5 => settings.keys.hard_drop = key_str,
                6 => settings.keys.hold = key_str,
                7 => settings.keys.pause = key_str,
                _ => {}
            }
            let _ = settings.save();
            self.rebinding = false;
            return;
        }

        match key {
            KeyCode::Esc => {
                self.open = false;
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected < KEYBIND_COUNT {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if self.selected < KEYBIND_COUNT {
                    self.rebinding = true;
                } else {
                    settings.display.color_mode = next_color_mode(&settings.display.color_mode);
                    let _ = settings.save();
                }
            }
            _ => {}
        }
    }

    pub fn draw(
        &self,
        stdout: &mut Stdout,
        settings: &Settings,
        term_width: u16,
        term_height: u16,
        _layout: &RenderLayout,
    ) -> std::io::Result<()> {
        let border_width: u16 = 30;
        let border_height: u16 = 15;
        let inner = (border_width - 2) as usize;

        let start_x = term_width / 2 - border_width / 2;
        let start_y = term_height / 2 - border_height / 2;

        let labels = [
            "Move Left",
            "Move Right",
            "Move Down",
            "Rotate Left",
            "Rotate Right",
            "Hard Drop",
            "Hold",
            "Pause",
        ];

        let bindings = [
            settings.keys.move_left.as_str(),
            settings.keys.move_right.as_str(),
            settings.keys.move_down.as_str(),
            settings.keys.rotate_left.as_str(),
            settings.keys.rotate_right.as_str(),
            settings.keys.hard_drop.as_str(),
            settings.keys.hold.as_str(),
            settings.keys.pause.as_str(),
        ];

        let title = " SETTINGS ";
        let title_local = (border_width as usize - title.len()) / 2;
        let mut top_row = String::new();
        for x in 0..border_width as usize {
            if x >= title_local && x < title_local + title.len() {
                top_row.push(title.as_bytes()[x - title_local] as char);
            } else {
                top_row.push_str(UPPER_BORDER);
            }
        }
        queue!(stdout, MoveTo(start_x, start_y), Print(top_row))?;

        for y in 1..border_height {
            let by = y + start_y;

            if y == border_height - 1 {
                let bottom: String = LOWER_BORDER.repeat(border_width as usize);
                queue!(stdout, MoveTo(start_x, by), Print(bottom))?;
                continue;
            }

            queue!(stdout, MoveTo(start_x, by), Print(VERT_BORDER))?;
            queue!(stdout, MoveTo(start_x + border_width - 1, by), Print(VERT_BORDER))?;

            let ix = start_x + 1;
            let item_y = y as usize - 1;

            if item_y < KEYBIND_COUNT {
                let label = labels[item_y];
                let key_str = if self.rebinding && self.selected == item_y {
                    " ? ".to_string()
                } else {
                    key_display(bindings[item_y])
                };
                let line = format!(" {:<14}[ {:>4} ] ", label, key_str);
                let padded = format!("{:<width$}", line, width = inner);

                if self.selected == item_y && !self.rebinding {
                    queue!(
                        stdout,
                        MoveTo(ix, by),
                        SetBackgroundColor(Color::Rgb { r: 40, g: 40, b: 40 }),
                        Print(padded),
                        ResetColor
                    )?;
                } else if self.rebinding && self.selected == item_y {
                    queue!(
                        stdout,
                        MoveTo(ix, by),
                        SetForegroundColor(Color::Rgb { r: 255, g: 200, b: 80 }),
                        Print(padded),
                        ResetColor
                    )?;
                } else {
                    queue!(stdout, MoveTo(ix, by), Print(padded))?;
                }
            } else if item_y == KEYBIND_COUNT {
                let blank = format!("{:<width$}", "", width = inner);
                queue!(stdout, MoveTo(ix, by), Print(blank))?;
            } else if item_y == KEYBIND_COUNT + 1 {
                let mode_str = &settings.display.color_mode;
                let line = format!(" {:<14}[ {:<6} ] ", "Color Mode", mode_str);
                let padded = format!("{:<width$}", line, width = inner);

                if self.selected == KEYBIND_COUNT {
                    queue!(
                        stdout,
                        MoveTo(ix, by),
                        SetBackgroundColor(Color::Rgb { r: 40, g: 40, b: 40 }),
                        Print(padded),
                        ResetColor
                    )?;
                } else {
                    queue!(stdout, MoveTo(ix, by), Print(padded))?;
                }
            } else if item_y == 12 {
                let saved = format!("{:^width$}", "auto-saved", width = inner);
                queue!(
                    stdout,
                    MoveTo(ix, by),
                    SetBackgroundColor(Color::Rgb { r: 30, g: 50, b: 30 }),
                    SetForegroundColor(Color::Rgb { r: 100, g: 180, b: 100 }),
                    Print(saved),
                    ResetColor
                )?;
            } else {
                let blank = format!("{:<width$}", "", width = inner);
                queue!(stdout, MoveTo(ix, by), Print(blank))?;
            }
        }

        Ok(())
    }
}
