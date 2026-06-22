mod game;
mod render;
mod settings;
mod tetromino;

use std::io::{Write, stdout};
use std::time::{Duration, Instant};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    queue,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::{Game, Timings};
use render::{ControlCenter, RenderLayout, draw_notification};
use settings::{Settings, SettingsScreen, string_to_keycode};
use tetromino::{ActiveTetromino, RotationDirection};

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = stdout();
    let mut game = Game::new();
    let timings = Timings::init();
    let mut settings = Settings::load();
    let mut last_terminal_size = terminal::size()?;
    let mut control = ControlCenter::new();
    let mut last_control_open = false;
    let mut settings_screen = SettingsScreen::new();
    let mut last_settings_open = false;

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        let paused = control.open || settings_screen.open;

        if !paused {
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
                    game.lock_and_advance();
                }
            }
            if game.last_drop.elapsed() > timings.drop_interval {
                if !game.active.move_down(&game.grid) {
                    game.lock_and_advance();
                }
                game.grounded_since = None;
                game.last_drop = Instant::now();
            }
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if settings_screen.open {
                    settings_screen.handle_key(key.code, &mut settings);
                } else if control.open {
                    match key.code {
                        KeyCode::Esc => {
                            control.open = false;
                            game.last_drop = Instant::now();
                            game.grounded_since = None;
                        }
                        KeyCode::Up => {
                            if control.selected == 0 {
                                control.selected = 3;
                            } else {
                                control.selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            control.selected = (control.selected + 1) % 4;
                        }
                        KeyCode::Enter => match control.selected {
                            0 => {
                                control.open = false;
                                game.last_drop = Instant::now();
                                game.grounded_since = None;
                            }
                            1 => {}
                            2 => {
                                settings_screen.open = true;
                                control.open = false;
                            }
                            _ => break,
                        },
                        _ => {}
                    }
                } else {
                    let k = key.code;
                    let binds = &settings.keys;

                    if Some(k) == string_to_keycode(&binds.pause) {
                        control.open = true;
                        control.selected = 0;
                    } else if Some(k) == string_to_keycode(&binds.move_left) {
                        game.active.move_left(&game.grid);
                    } else if Some(k) == string_to_keycode(&binds.move_right) {
                        game.active.move_right(&game.grid);
                    } else if Some(k) == string_to_keycode(&binds.move_down) {
                        if game.active.move_down(&game.grid) {
                            game.grounded_since = None;
                        }
                        game.last_drop = Instant::now();
                    } else if Some(k) == string_to_keycode(&binds.hard_drop) {
                        game.hard_drop();
                    } else if Some(k) == string_to_keycode(&binds.rotate_left) {
                        game.active.rotate(&game.grid, RotationDirection::Left);
                    } else if Some(k) == string_to_keycode(&binds.rotate_right) {
                        game.active.rotate(&game.grid, RotationDirection::Right);
                    } else if Some(k) == string_to_keycode(&binds.hold) {
                        if !game.has_swapped {
                            game.has_swapped = true;
                            game.active.reset();
                            let new_piece = game.buffer.swap(game.active);
                            game.active = if let Some(swapped) = new_piece {
                                swapped
                            } else {
                                ActiveTetromino::from(game.queue.swap(&mut game.bag))
                            };
                        }
                    } else if k == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        if !paused {
            if !game.grid.is_valid_position(
                &game.active,
                game.active.x,
                game.active.y,
                game.active.rotation,
            ) {
                break;
            }
        }

        let (term_width, term_height) = terminal::size()?;
        let screen_changed = (term_width, term_height) != last_terminal_size
            || control.open != last_control_open
            || settings_screen.open != last_settings_open;

        if screen_changed {
            queue!(stdout, Clear(ClearType::All))?;
            last_terminal_size = (term_width, term_height);
            last_control_open = control.open;
            last_settings_open = settings_screen.open;
        }

        let color_mode = settings.color_mode();
        let layout = RenderLayout::new(term_width, term_height, game.grid.width, game.grid.height);

        if !settings_screen.open || screen_changed {
            game.grid.draw(&mut stdout, Some(&game.active), &layout, color_mode)?;
            game.buffer.draw(&mut stdout, &layout, color_mode)?;
            draw_notification(&mut stdout, &game.last_clear, &layout)?;
            game.queue.draw(&mut stdout, &layout, color_mode)?;
            game.progress.draw(&mut stdout, game.game_start.elapsed(), &layout)?;
            control.draw(&mut stdout, &layout)?;
        }

        if settings_screen.open {
            settings_screen.draw(&mut stdout, &settings, term_width, term_height, &layout)?;
        }

        stdout.flush()?;
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
