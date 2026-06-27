mod game;
mod render;
mod settings;
mod tetromino;

use std::io::{BufWriter, Write, stdout};
use std::time::{Duration, Instant};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    queue,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::{Game, Timings};
use render::{ControlCenter, GameOverScreen, RenderLayout, draw_controls, draw_notification};
use settings::{Settings, SettingsScreen, key_display, string_to_keycode};
use tetromino::{ActiveTetromino, RotationDirection};

fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    let result = run();
    let _ = terminal::disable_raw_mode();
    result
}

fn run() -> std::io::Result<()> {
    let mut stdout = BufWriter::new(stdout());
    let mut game = Game::new();
    let mut timings = Timings::init();
    let mut settings = Settings::load();
    let mut last_terminal_size = terminal::size()?;
    let mut control = ControlCenter::new();
    let mut last_control_open = false;
    let mut settings_screen = SettingsScreen::new();
    let mut last_settings_open = false;
    let mut game_over_screen = GameOverScreen::new();
    let mut last_game_over = false;

    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    loop {
        let paused = control.open || settings_screen.open || game_over_screen.active;
        let clearing = game.clearing.is_some();

        if !paused && !clearing {
            timings.recalculate(game.progress.level);

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
                    game.lock_and_start_clear(timings.clear_animation_duration);
                }
            }
            if game.last_drop.elapsed() > timings.drop_interval {
                if game.active.move_down(&game.grid) {
                    game.grounded_since = None;
                }
                game.last_drop = Instant::now();
            }
        }

        if !paused && clearing {
            if let Some(ref anim) = game.clearing {
                if anim.started_at.elapsed() >= anim.duration {
                    game.finish_clear();
                }
            }
        }

        // Collect all pending events this frame
        let mut events = Vec::new();
        if event::poll(Duration::from_millis(16))? {
            events.push(event::read()?);
            while event::poll(Duration::from_millis(0))? {
                events.push(event::read()?);
            }
        }

        let mut should_quit = false;

        for event in events {
            if let Event::Key(key) = event {
                if game_over_screen.active {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Left => game_over_screen.selected = 0,
                            KeyCode::Right => game_over_screen.selected = 1,
                            KeyCode::Enter => {
                                if game_over_screen.selected == 0 {
                                    game = Game::new();
                                    timings = Timings::init();
                                    game_over_screen.active = false;
                                    game_over_screen.selected = 0;
                                } else {
                                    should_quit = true;
                                }
                            }
                            _ => {}
                        }
                    }
                } else if settings_screen.open {
                    if key.kind == KeyEventKind::Press {
                        settings_screen.handle_key(key.code, &mut settings);
                    }
                } else if control.open {
                    if key.kind == KeyEventKind::Press {
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
                                _ => should_quit = true,
                            },
                            _ => {}
                        }
                    }
                } else {
                    let k = key.code;
                    let binds = &settings.keys;
                    let left_key = string_to_keycode(&binds.move_left);
                    let right_key = string_to_keycode(&binds.move_right);
                    let is_left = Some(k) == left_key;
                    let is_right = Some(k) == right_key;

                    if is_left || is_right {
                        if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat {
                            if game.clearing.is_none() {
                                if is_left {
                                    game.active.move_left(&game.grid);
                                } else {
                                    game.active.move_right(&game.grid);
                                }
                            }
                        }
                    } else if key.kind == KeyEventKind::Press {
                        if Some(k) == string_to_keycode(&binds.pause) {
                            control.open = true;
                            control.selected = 0;
                        } else if k == KeyCode::Char('q') {
                            should_quit = true;
                        } else if game.clearing.is_none() {
                            if Some(k) == string_to_keycode(&binds.move_down) {
                                if game.active.move_down(&game.grid) {
                                    game.grounded_since = None;
                                }
                                game.last_drop = Instant::now();
                            } else if Some(k) == string_to_keycode(&binds.hard_drop) {
                                game.hard_drop(timings.clear_animation_duration);
                            } else if Some(k) == string_to_keycode(&binds.rotate_left) {
                                game.active.rotate(&game.grid, RotationDirection::Left);
                            } else if Some(k) == string_to_keycode(&binds.rotate_right) {
                                game.active.rotate(&game.grid, RotationDirection::Right);
                            } else if Some(k) == string_to_keycode(&binds.rotate_180) {
                                game.active.rotate(&game.grid, RotationDirection::Flip);
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
                            }
                        }
                    }
                }
            }
        }

        if should_quit {
            break;
        }

        if !paused && game.clearing.is_none() {
            if !game.grid.is_valid_position(
                &game.active,
                game.active.x,
                game.active.y,
                game.active.rotation,
            ) {
                game_over_screen.active = true;
            }
        }

        let (term_width, term_height) = terminal::size()?;
        let screen_changed = (term_width, term_height) != last_terminal_size
            || control.open != last_control_open
            || settings_screen.open != last_settings_open
            || game_over_screen.active != last_game_over;

        if screen_changed {
            queue!(stdout, Clear(ClearType::All))?;
            last_terminal_size = (term_width, term_height);
            last_control_open = control.open;
            last_settings_open = settings_screen.open;
            last_game_over = game_over_screen.active;
        }

        let block_str = settings.block_str();
        let layout = RenderLayout::new(term_width, term_height, game.grid.width, game.grid.height);

        if (!settings_screen.open && !game_over_screen.active) || screen_changed {
            let clear_anim_data = game.clearing.as_ref().map(|anim| {
                let progress = (anim.started_at.elapsed().as_secs_f32() / anim.duration.as_secs_f32()).min(1.0);
                (anim.rows.as_slice(), anim.center_column, progress)
            });
            let active_for_draw = if game.clearing.is_some() { None } else { Some(&game.active) };
            game.grid.draw(&mut stdout, active_for_draw, &layout, block_str, clear_anim_data)?;
            game.buffer.draw(&mut stdout, &layout, block_str)?;
            draw_notification(&mut stdout, &game.last_clear, &layout)?;
            game.queue.draw(&mut stdout, &layout, block_str)?;
            game.progress.draw(&mut stdout, game.game_start.elapsed(), &layout)?;
            control.draw(&mut stdout, &layout)?;
            let key_entries = [
                ("Move Left",  key_display(&settings.keys.move_left)),
                ("Move Right", key_display(&settings.keys.move_right)),
                ("Move Down",  key_display(&settings.keys.move_down)),
                ("Rot. Left",  key_display(&settings.keys.rotate_left)),
                ("Rot. Right", key_display(&settings.keys.rotate_right)),
                ("Rot. 180",   key_display(&settings.keys.rotate_180)),
                ("Hard Drop",  key_display(&settings.keys.hard_drop)),
                ("Hold",       key_display(&settings.keys.hold)),
                ("Pause",      key_display(&settings.keys.pause)),
            ];
            let entry_refs: Vec<(&str, &str)> = key_entries.iter().map(|(l, k)| (*l, k.as_str())).collect();
            draw_controls(&mut stdout, &layout, &entry_refs)?;
        }

        if settings_screen.open {
            settings_screen.draw(&mut stdout, &settings, term_width, term_height, &layout)?;
        }

        if game_over_screen.active {
            game_over_screen.draw(&mut stdout, &layout, game.progress.level, game.progress.score)?;
        }

        stdout.flush()?;
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
