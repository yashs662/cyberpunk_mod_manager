use std::path::PathBuf;

use tui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Nothing,
    Submit,
    Input,
}

impl Focus {
    pub fn all() -> Vec<Focus> {
        vec![Focus::Submit, Focus::Input]
    }

    pub fn next(&self) -> Focus {
        let index = Focus::all().iter().position(|&r| r == *self).unwrap();
        let next = (index + 1) % Focus::all().len();
        Focus::all()[next]
    }

    pub fn previous(&self) -> Focus {
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
pub enum UiMode {
    Explore,
    SelectFolder,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Input,
}

pub struct AppState {
    pub focus: Focus,
    pub current_input: String,
    pub app_mode: AppMode,
    pub ui_mode: UiMode,
    pub file_list: StatefulList<(String, usize)>,
    
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

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct App {
    pub state: AppState,
    pub selected_folder: Option<PathBuf>,
    pub cyberpunk_folder: Option<PathBuf>,
}

impl App {  
    pub fn new() -> App {
        App {
            state: AppState::new(),
            selected_folder: None,
            cyberpunk_folder: None,
        }
    }

    pub fn on_tick(&mut self) {
        // Do nothing for now
    }
}