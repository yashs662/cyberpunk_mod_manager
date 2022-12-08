use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{LevelFilter, info};
use tui_logger::TuiLoggerWidget;
use std::{
    error::Error,
    io,
    time::{Duration, Instant}, path::{PathBuf, Path}, fs,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

const FOCUS_STYLE: Style = Style{
    fg: Some(Color::Cyan),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
const NORMAL_STYLE: Style = Style{
    fg: Some(Color::White),
    bg: Some(Color::Black),
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
const INPUT_STYLE: Style = Style{
    fg: Some(Color::LightGreen),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

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
    selected_folder: Option<PathBuf>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            focus: Focus::Nothing,
            current_input: String::new(),
            app_mode: AppMode::Normal,
            ui_mode: UiMode::Explore,
            file_list: StatefulList::with_items(vec![]),
            selected_folder: None,
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

struct App {
    state: AppState,
}

impl App {  
    fn new() -> App {
        App {
            state: AppState::new(),
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
                                app.state.selected_folder = Some(input_path.to_path_buf());
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

fn draw_select_folder<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    
    let input_style = if app.state.focus == Focus::Input {
        if app.state.app_mode == AppMode::Input {
            INPUT_STYLE
        } else {
            FOCUS_STYLE
        }
    } else {
        NORMAL_STYLE
    };

    let submit_style = if app.state.focus == Focus::Submit {
        FOCUS_STYLE
    } else {
        NORMAL_STYLE
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10)
            ].as_ref())
        .split(f.size());

    let title = Paragraph::new(Text::styled("Select Folder", Style::default().fg(Color::White)))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    let current_input = app.state.current_input.clone();

    let input = Paragraph::new(Text::raw(current_input))
        .block(Block::default().borders(Borders::ALL).title("Folder"))
        .style(input_style)
        .wrap(Wrap { trim: true });

    let button = Paragraph::new("Submit")
        .block(Block::default().borders(Borders::ALL))
        .style(submit_style)
        .wrap(Wrap { trim: true });

    // check if input mode is active, if so, show cursor
    if app.state.app_mode == AppMode::Input {
        f.set_cursor(
            chunks[1].x + app.state.current_input.len() as u16 + 1,
            chunks[1].y + 1,
        );
    }

    f.render_widget(title, chunks[0]);
    f.render_widget(input, chunks[1]);
    f.render_widget(button, chunks[2]);
}

fn draw_explore<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10)
            ].as_ref())
        .split(f.size());
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30)
            ].as_ref())
        .split(main_chunks[1]);

    let title_widget = Paragraph::new(Text::styled("Cyberpunk Mod Manager", Style::default().fg(Color::White)))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    
    let current_folder = app.state.selected_folder.clone().unwrap_or_else(|| PathBuf::new());
    // check if current folder is a directory if not set it to No folder selected
    let current_folder_string = if current_folder.is_dir() {
        current_folder.to_string_lossy().to_string()
    } else {
        "No folder selected".to_string()
    };
    let current_folder_widget = Paragraph::new(Text::raw(current_folder_string))
        .block(Block::default().borders(Borders::ALL).title("Current Folder"))
        .style(NORMAL_STYLE)
        .wrap(Wrap { trim: true });


    // Create a list of ListItems from the list of files
    let items: Vec<ListItem> = app
        .state.file_list
        .items
        .iter()
        .map(|(name, _size)| {
            ListItem::new(Text::from(name.clone()))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available files"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let log_widget = TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Blue))
        .style_info(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .title("Logs")
                .borders(Borders::ALL),
        )
        .output_timestamp(None)
        .output_target(false)
        .output_level(None);
    
    f.render_widget(title_widget, main_chunks[0]);
    f.render_stateful_widget(items_list, chunks[0], &mut app.state.file_list.state);
    f.render_widget(log_widget, chunks[1]);
    f.render_widget(current_folder_widget, main_chunks[2]);
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

fn check_if_mod_is_valid() -> bool {
    true
}