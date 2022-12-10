
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use cyberpunk_mod_manager::{App, AppMode, UiMode, Focus};
use log::{LevelFilter, info, error};

use ui::ui::{draw_select_folder, draw_explore};
use utils::check_if_mod_is_valid;

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
                    KeyCode::Tab => app.state.focus = app.state.focus.next(),
                    KeyCode::BackTab => app.state.focus = app.state.focus.previous(),
                    KeyCode::Char('s') => {
                        if app.state.app_mode == AppMode::Normal {
                            if app.state.ui_mode == UiMode::Explore {
                                app.state.focus = Focus::Input;
                                app.state.ui_mode = UiMode::SelectFolder;
                            } else {
                                app.state.focus = Focus::Nothing;
                                app.state.ui_mode = UiMode::Explore;
                            }
                        } else if app.state.app_mode == AppMode::Input {
                            app.state.current_input.push('s');
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
                            let current_input = app.state.current_input.clone();
                            // check if current input ends with is not a directory, if so do nothing
                            if current_input.ends_with("is not a directory") {
                                continue;
                            }
                            let input_path = Path::new(&current_input);
                            if input_path.is_dir() {
                                app.selected_folder = Some(input_path.to_path_buf());
                                let mut files = vec![];
                                for entry in fs::read_dir(input_path)? {
                                    if let Ok(entry) = entry {
                                        if let Ok(metadata) = entry.metadata() {
                                            if metadata.is_file() {
                                                files.push((entry.file_name().to_string_lossy().to_string(), metadata.len() as usize));
                                            }
                                        }
                                    }
                                }
                                app.state.file_list.items = files;
                                // clear input
                                app.state.current_input = String::new();
                                app.state.ui_mode = UiMode::Explore;
                                app.state.focus = Focus::Nothing;
                            } else {
                                app.state.current_input = format!("{} is not a directory", current_input)
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
                        if app.state.focus == Focus::Input {
                            // check if app.state.current_input ends with "is not a directory" if so, remove it
                            if app.state.current_input.ends_with("is not a directory") {
                                app.state.current_input = app.state.current_input.replace("is not a directory", "");
                                // trim whitespace
                                app.state.current_input = app.state.current_input.trim().to_string();
                            }
                            app.state.app_mode = AppMode::Input;
                        }
                    }
                    KeyCode::Esc => {
                        if app.state.app_mode == AppMode::Input {
                            app.state.app_mode = AppMode::Normal;
                        }
                    }
                    _ => {
                        if app.state.app_mode == AppMode::Input && app.state.focus == Focus::Input {
                            // if backspace, remove last char
                            if key.code == KeyCode::Backspace {
                                app.state.current_input.pop();
                            }
                            if let KeyCode::Char(c) = key.code {
                                app.state.current_input.push(c);
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
    match app.state.ui_mode {
        UiMode::SelectFolder => draw_select_folder(f, app),
        UiMode::Explore => draw_explore(f, app)
    }
}