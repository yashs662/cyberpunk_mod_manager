use compress_tools::{uncompress_archive, Ownership};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{LevelFilter, info, error};
use tui_logger::TuiLoggerWidget;
use ui::ui::{draw_select_folder, draw_explore};
use walkdir::WalkDir;
use std::{
    error::Error,
    io,
    time::{Duration, Instant}, path::{PathBuf, Path}, fs::{self, File, create_dir_all, remove_dir_all}, env::temp_dir,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

pub mod ui;
pub mod constants;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Nothing,
    Submit,
    Input,
}

impl Focus {
    fn all() -> Vec<Focus> {
        vec![Focus::Submit, Focus::Input]
    }

    fn next(&self) -> Focus {
        let index = Focus::all().iter().position(|&r| r == *self).unwrap();
        let next = (index + 1) % Focus::all().len();
        Focus::all()[next]
    }

    fn previous(&self) -> Focus {
        let index = Focus::all().iter().position(|&r| r == *self).unwrap();
        let previous = if index == 0 {
            Focus::all().len() - 1
        } else {
            index - 1
        };
        Focus::all()[previous]
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UiMode {
    Explore,
    SelectFolder,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppMode {
    Normal,
    Input,
}

struct AppState {
    focus: Focus,
    current_input: String,
    app_mode: AppMode,
    ui_mode: UiMode,
    file_list: StatefulList<(String, usize)>,
    
}

impl AppState {
    fn new() -> AppState {
        AppState {
            focus: Focus::Nothing,
            current_input: String::new(),
            app_mode: AppMode::Normal,
            ui_mode: UiMode::Explore,
            file_list: StatefulList::with_items(vec![]),
        }
    }
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.items.is_empty() {
                    0
                } else if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.items.is_empty() {
                    0
                } else if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.

pub struct App {
    state: AppState,
    selected_folder: Option<PathBuf>,
    cyberpunk_folder: Option<PathBuf>,
}

impl App {  
    fn new() -> App {
        App {
            state: AppState::new(),
            selected_folder: None,
            cyberpunk_folder: None,
        }
    }

    fn on_tick(&mut self) {
        // Do nothing for now
    }
}

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

fn log_help() {
    info!("Press 's' to select a folder");
    info!("Use UP/DOWN to navigate the list");
    info!("Press ENTER to select a file");
    info!("Press 'i' to enter input mode (Green Highlight)");
    info!("Press TAB to switch between input and submit button (Blue Highlight)");
    info!("Press 'h' to see this help message again");
    info!("Press 'q' to quit");
}

fn check_if_mod_is_valid(file_path: PathBuf) -> bool {
    let mut is_valid = false;
    // make sure the file exists
    if !file_path.exists() {
        info!("{} does not exist", file_path.to_string_lossy());
        return false;
    }
    let file_name = &file_path.file_name().unwrap().to_string_lossy();
    let mut destination = temp_dir();
    // make a cyberpunk_mod_manager directory in the temp directory
    destination.push("cyberpunk_mod_manager");
    // make a directory with the name of the file
    destination.push(file_path.file_name().unwrap());
    // create the directory
    create_dir_all(&destination).unwrap();
    // extract the zip file to the destination
    let mut source = File::open(&file_path).unwrap();
    uncompress_archive(&mut source, &destination, Ownership::Preserve).unwrap();
    // check if the zip file contains any of the following folders
    // archive, bin, engine, mods
    // if it does, return true
    // if the zip file has .ARCHIVE files, return true
    // else return false
    for entry in WalkDir::new(&destination) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if dir_name == "archive" || dir_name == "bin" || dir_name == "engine" || dir_name == "mods" {
                is_valid = true;
                info!("Valid mod file: {}", file_name);
            }
        }
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            if file_name.ends_with(".archive") {
                is_valid = true;
                info!("Valid mod file: {}", file_name);
            }
        }
    }
    // remove the directory
    remove_dir_all(&destination).unwrap();
    is_valid
}