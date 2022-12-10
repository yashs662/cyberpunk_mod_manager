
use constants::{MOD_FOLDER_INPUT_EMPTY_ERROR, NOT_A_DIRECTORY_ERROR, CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR, NOT_A_VALID_CYBERPUNK_FOLDER_ERROR};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use cyberpunk_mod_manager::{App, AppMode, UiMode, Focus};
use log::{LevelFilter, info, error};

use ui::ui::{draw_select_folder, draw_explore, check_size, draw_size_error};
use utils::{check_if_mod_is_valid, check_if_cyberpunk_dir_is_valid};

use std::{
    error::Error,
    io,
    time::{Duration, Instant}, path::{Path}, fs::{self},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};

use crate::utils::log_help;

pub mod ui;
pub mod constants;
pub mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);
    
    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();

    info!("Starting Cyberpunk Mod Manager");
    info!(" ");
    log_help();

    let res = run_app(&mut terminal, app, tick_rate);


    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    Ok(loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                            break;
                        }
                    },
                    KeyCode::Tab => {
                        if app.state.app_mode != AppMode::Input {
                            app.state.focus = app.state.focus.next()
                        }
                    },
                    KeyCode::BackTab => {
                        if app.state.app_mode != AppMode::Input {
                            app.state.focus = app.state.focus.previous()
                        }
                    },
                    KeyCode::Char('s') => {
                        if app.state.app_mode == AppMode::Normal {
                            if app.state.ui_mode == UiMode::Explore {
                                app.state.focus = Focus::ModFolderInput;
                                app.state.ui_mode = UiMode::SelectFolder;
                            } else {
                                app.state.focus = Focus::Nothing;
                                app.state.ui_mode = UiMode::Explore;
                            }
                        } else if app.state.app_mode == AppMode::Input {
                            if app.state.focus == Focus::ModFolderInput {
                                app.state.temp_input_store[0].push('s');
                            } else if app.state.focus == Focus::CyberpunkFolderInput {
                                app.state.temp_input_store[1].push('s');
                            } else {
                                app.state.current_input.push('s');
                            }
                        }
                    },
                    KeyCode::Char('h') => log_help(),
                    KeyCode::Char('q') => if app.state.app_mode != AppMode::Input {
                        return Ok(())
                    }
                    KeyCode::Left => app.state.file_list.unselect(),
                    KeyCode::Down => app.state.file_list.next(),
                    KeyCode::Up => app.state.file_list.previous(),
                    KeyCode::Enter => {
                        if app.state.app_mode == AppMode::Input {
                            app.state.app_mode = AppMode::Normal;
                        }
                        if app.state.focus == Focus::Submit {
                            let mut mod_folder_ok = false;
                            let mut cyberpunk_folder_ok = false;
                            let mod_folder_input = app.state.temp_input_store[0].clone();
                            let cyberpunk_folder_input = app.state.temp_input_store[1].clone();

                            if mod_folder_input.ends_with(NOT_A_DIRECTORY_ERROR)
                                || cyberpunk_folder_input.ends_with(NOT_A_DIRECTORY_ERROR)
                                || cyberpunk_folder_input.ends_with(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
                                {
                                continue;
                            }

                            let mod_folder_path = Path::new(&mod_folder_input);
                            let cyberpunk_folder_path = Path::new(&cyberpunk_folder_input);
                            if mod_folder_path.is_dir() {
                                app.selected_folder = Some(mod_folder_path.to_path_buf());
                                let mut files = vec![];
                                for entry in fs::read_dir(mod_folder_path)? {
                                    if let Ok(entry) = entry {
                                        if let Ok(metadata) = entry.metadata() {
                                            if metadata.is_file() {
                                                files.push((entry.file_name().to_string_lossy().to_string(), metadata.len() as usize));
                                            }
                                        }
                                    }
                                }
                                app.state.file_list.items = files;
                                mod_folder_ok = true;
                            } else {
                                // check if input is empty, put error message in temp input store
                                if mod_folder_input.trim().is_empty() {
                                    app.state.temp_input_store[0] = MOD_FOLDER_INPUT_EMPTY_ERROR.to_string();
                                } else if !app.state.temp_input_store[0].contains(MOD_FOLDER_INPUT_EMPTY_ERROR) {
                                    app.state.temp_input_store[0] = format!("{} {}", mod_folder_input, NOT_A_DIRECTORY_ERROR);
                                }
                            }
                            if cyberpunk_folder_path.is_dir() {
                                if !check_if_cyberpunk_dir_is_valid(cyberpunk_folder_path.clone().to_path_buf()) {
                                    app.state.temp_input_store[1] = format!("{} {}", cyberpunk_folder_input, NOT_A_VALID_CYBERPUNK_FOLDER_ERROR);
                                    continue;
                                } else {
                                    app.cyberpunk_folder = Some(cyberpunk_folder_path.to_path_buf());
                                    cyberpunk_folder_ok = true;
                                }
                            } else {
                                // check if input is empty, put error message in temp input store
                                if cyberpunk_folder_input.trim().is_empty() {
                                    app.state.temp_input_store[1] = CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR.to_string();
                                } else if !app.state.temp_input_store[1].contains(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR)
                                    || !app.state.temp_input_store[1].contains(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
                                    {
                                    app.state.temp_input_store[1] = format!("{} {}", cyberpunk_folder_input, NOT_A_DIRECTORY_ERROR);
                                }
                            }
                            if mod_folder_ok && cyberpunk_folder_ok {
                                app.state.ui_mode = UiMode::Explore;
                                app.state.focus = Focus::Nothing;
                                // clear temp input store
                                app.state.temp_input_store[0] = String::new();
                                app.state.temp_input_store[1] = String::new();
                            }
                        }
                        if app.state.ui_mode == UiMode::Explore {
                            if let Some(selected) = app.state.file_list.state.selected() {
                                let selected_file = app.state.file_list.items[selected].0.clone();
                                let selected_file_path = Path::new(&app.selected_folder.as_ref().unwrap()).join(selected_file);
                                if !check_if_mod_is_valid(selected_file_path.clone()) {
                                    error!("{} is not a valid mod", selected_file_path.to_string_lossy());
                                }
                            }
                        }
                    }
                    KeyCode::Char('i') => {
                        if app.state.focus == Focus::ModFolderInput {
                            if app.state.temp_input_store[0].ends_with(NOT_A_DIRECTORY_ERROR) {
                                app.state.temp_input_store[0] = app.state.temp_input_store[0]
                                    .replace(NOT_A_DIRECTORY_ERROR, "").trim().to_string();
                            } else if app.state.temp_input_store[0].ends_with(MOD_FOLDER_INPUT_EMPTY_ERROR) {
                                app.state.temp_input_store[0] = app.state.temp_input_store[0]
                                    .replace(MOD_FOLDER_INPUT_EMPTY_ERROR, "").trim().to_string();
                            }
                        } else if app.state.focus == Focus::CyberpunkFolderInput {
                            if app.state.temp_input_store[1].ends_with(NOT_A_DIRECTORY_ERROR) {
                                app.state.temp_input_store[1] = app.state.temp_input_store[1]
                                    .replace(NOT_A_DIRECTORY_ERROR, "").trim().to_string();
                            } else if app.state.temp_input_store[1].ends_with(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR) {
                                app.state.temp_input_store[1] = app.state.temp_input_store[1]
                                    .replace(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR, "").trim().to_string();
                            } else if app.state.temp_input_store[1].ends_with(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR) {
                                app.state.temp_input_store[1] = app.state.temp_input_store[1]
                                    .replace(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR, "").trim().to_string();
                            }
                        }
                        app.state.app_mode = AppMode::Input;
                    }
                    KeyCode::Esc => {
                        if app.state.app_mode == AppMode::Input {
                            app.state.app_mode = AppMode::Normal;
                        }
                    }
                    _ => {
                        if app.state.app_mode == AppMode::Input && app.state.focus == Focus::ModFolderInput {
                            // if backspace, remove last char
                            if key.code == KeyCode::Backspace {
                                app.state.temp_input_store[0].pop();
                            }
                            if let KeyCode::Char(c) = key.code {
                                app.state.temp_input_store[0].push(c);
                            }
                        } else if app.state.app_mode == AppMode::Input && app.state.focus == Focus::CyberpunkFolderInput {
                            // if backspace, remove last char
                            if key.code == KeyCode::Backspace {
                                app.state.temp_input_store[1].pop();
                            }
                            if let KeyCode::Char(c) = key.code {
                                app.state.temp_input_store[1].push(c);
                            }
                        }
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    })
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let msg = check_size(&f.size());
    if &msg != "Size OK" {
        draw_size_error(f, &f.size(), msg);
        return;
    }
    
    match app.state.ui_mode {
        UiMode::SelectFolder => draw_select_folder(f, app),
        UiMode::Explore => draw_explore(f, app)
    }
}