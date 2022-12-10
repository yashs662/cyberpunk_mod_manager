use std::fs;
use std::path::Path;
use std::vec;
use std::path::PathBuf;
use log::{
    info,
    error,
    warn
};

use self::actions::Actions;
use self::state::AppState;
use self::state::AppStatus;
use self::state::Focus;
use self::state::UiMode;
use self::utils::check_if_cyberpunk_dir_is_valid;
use self::utils::check_if_mod_is_valid;
use crate::app::actions::Action;
use crate::constants::CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR;
use crate::constants::MOD_FOLDER_INPUT_EMPTY_ERROR;
use crate::constants::NOT_A_DIRECTORY_ERROR;
use crate::constants::NOT_A_VALID_CYBERPUNK_FOLDER_ERROR;
use crate::inputs::key::Key;
use crate::io::IoEvent;

pub mod actions;
pub mod state;
pub mod utils;


#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct App {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Actions,
    is_loading: bool,
    pub state: AppState,
    pub selected_folder: Option<PathBuf>,
    pub cyberpunk_folder: Option<PathBuf>,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();


        Self {
            io_tx,
            actions,
            is_loading,
            state,
            selected_folder: None,
            cyberpunk_folder: None,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        // check if we are in a user input mode
        if self.state.status == AppStatus::UserInput {
            // append to current user input if key is not enter else change state to Initialized
            if key != Key::Enter && key != Key::Esc {
                let mut current_key = key.to_string();
                if current_key == "<Space>" {
                    current_key = " ".to_string();
                } else if current_key == "<ShiftEnter>" {
                    current_key = "".to_string();
                } else if current_key == "<Tab>" {
                    current_key = "  ".to_string();
                } else if current_key == "<Backspace>" {
                    match self.state.focus {
                        Focus::ModFolderInput => self.state.select_folder_form[0].pop(),
                        Focus::CyberpunkFolderInput => self.state.select_folder_form[1].pop(),
                        _ => self.state.current_input.pop(),
                    };
                    return AppReturn::Continue;
                } else if current_key.starts_with("<") && current_key.ends_with(">") {
                    current_key = current_key[1..current_key.len() - 1].to_string();
                }

                if self.state.focus == Focus::ModFolderInput {
                    self.state.select_folder_form[0].push_str(&current_key);
                } else if self.state.focus == Focus::CyberpunkFolderInput {
                    self.state.select_folder_form[1].push_str(&current_key);
                } else {
                    self.state.current_input.push_str(&current_key);
                }
            } else {
                self.state.status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
            return AppReturn::Continue;
        } else {
            if let Some(action) = self.actions.find(key) {
                match action {
                    Action::Quit => {
                        AppReturn::Exit
                    }
                    Action::Tab => {
                        let current_focus = self.state.focus.clone();
                        let next_focus = self.state.focus.next(&UiMode::get_available_targets(&self.state.ui_mode));
                        // check if the next focus is the same as the current focus or NoFocus if so set back to the first focus
                        if next_focus == current_focus || next_focus == Focus::NoFocus {
                            self.state.focus = current_focus;
                        } else {
                            self.state.focus = next_focus;
                        }
                        AppReturn::Continue
                    }
                    Action::ShiftTab => {
                        let current_focus = self.state.focus.clone();
                        let next_focus = self.state.focus.prev(&UiMode::get_available_targets(&self.state.ui_mode));
                        // check if the next focus is the same as the current focus or NoFocus if so set back to the first focus
                        if next_focus == current_focus || next_focus == Focus::NoFocus {
                            self.state.focus = current_focus;
                        } else {
                            self.state.focus = next_focus;
                        }
                        AppReturn::Continue
                    }
                    Action::Up => {
                        self.state.file_list.previous();
                        AppReturn::Continue
                    }
                    Action::Down => {
                        self.state.file_list.next();
                        AppReturn::Continue
                    }
                    Action::Right => {
                        AppReturn::Continue
                    }
                    Action::Left => {
                        AppReturn::Continue
                    }
                    Action::TakeUserInput => {
                        match self.state.ui_mode {
                            UiMode::SelectFolder => {
                                self.state.status = AppStatus::UserInput;
                                if self.state.focus == Focus::ModFolderInput {
                                    if self.state.select_folder_form[0].ends_with(NOT_A_DIRECTORY_ERROR) {
                                        self.state.select_folder_form[0] = self.state.select_folder_form[0]
                                            .replace(NOT_A_DIRECTORY_ERROR, "").trim().to_string();
                                    } else if self.state.select_folder_form[0].ends_with(MOD_FOLDER_INPUT_EMPTY_ERROR) {
                                        self.state.select_folder_form[0] = self.state.select_folder_form[0]
                                            .replace(MOD_FOLDER_INPUT_EMPTY_ERROR, "").trim().to_string();
                                    }
                                } else if self.state.focus == Focus::CyberpunkFolderInput {
                                    if self.state.select_folder_form[1].ends_with(NOT_A_DIRECTORY_ERROR) {
                                        self.state.select_folder_form[1] = self.state.select_folder_form[1]
                                            .replace(NOT_A_DIRECTORY_ERROR, "").trim().to_string();
                                    } else if self.state.select_folder_form[1].ends_with(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR) {
                                        self.state.select_folder_form[1] = self.state.select_folder_form[1]
                                            .replace(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR, "").trim().to_string();
                                    } else if self.state.select_folder_form[1].ends_with(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR) {
                                        self.state.select_folder_form[1] = self.state.select_folder_form[1]
                                            .replace(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR, "").trim().to_string();
                                    }
                                }
                            }
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::Escape => {
                        if self.state.status == AppStatus::UserInput {
                            self.state.status = AppStatus::Initialized;
                        }
                        AppReturn::Continue
                    }
                    Action::Enter => {
                        if self.state.status == AppStatus::UserInput {
                            self.state.status = AppStatus::Initialized;
                        }
                        if self.state.focus == Focus::Submit {
                            let mut mod_folder_ok = false;
                            let mut cyberpunk_folder_ok = false;
                            let mod_folder_input = self.state.select_folder_form[0].clone();
                            let cyberpunk_folder_input = self.state.select_folder_form[1].clone();

                            if mod_folder_input.ends_with(NOT_A_DIRECTORY_ERROR)
                                || cyberpunk_folder_input.ends_with(NOT_A_DIRECTORY_ERROR)
                                || cyberpunk_folder_input.ends_with(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
                                {
                                return AppReturn::Continue;
                            }

                            let mod_folder_path = Path::new(&mod_folder_input);
                            let cyberpunk_folder_path = Path::new(&cyberpunk_folder_input);
                            if mod_folder_path.is_dir() {
                                self.selected_folder = Some(mod_folder_path.to_path_buf());
                                let mut files = vec![];
                                for entry in fs::read_dir(mod_folder_path).unwrap() {
                                    if let Ok(entry) = entry {
                                        if let Ok(metadata) = entry.metadata() {
                                            if metadata.is_file() {
                                                files.push((entry.file_name().to_string_lossy().to_string(), metadata.len() as usize));
                                            }
                                        }
                                    }
                                }
                                self.state.file_list.items = files;
                                mod_folder_ok = true;
                            } else {
                                // check if input is empty, put error message in temp input store
                                if mod_folder_input.trim().is_empty() {
                                    self.state.select_folder_form[0] = MOD_FOLDER_INPUT_EMPTY_ERROR.to_string();
                                } else if !self.state.select_folder_form[0].contains(MOD_FOLDER_INPUT_EMPTY_ERROR) {
                                    self.state.select_folder_form[0] = format!("{} {}", mod_folder_input, NOT_A_DIRECTORY_ERROR);
                                }
                            }
                            if cyberpunk_folder_path.is_dir() {
                                if !check_if_cyberpunk_dir_is_valid(cyberpunk_folder_path.clone().to_path_buf()) {
                                    self.state.select_folder_form[1] = format!("{} {}", cyberpunk_folder_input, NOT_A_VALID_CYBERPUNK_FOLDER_ERROR);
                                    return AppReturn::Continue;
                                } else {
                                    self.cyberpunk_folder = Some(cyberpunk_folder_path.to_path_buf());
                                    cyberpunk_folder_ok = true;
                                }
                            } else {
                                // check if input is empty, put error message in temp input store
                                if cyberpunk_folder_input.trim().is_empty() {
                                    self.state.select_folder_form[1] = CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR.to_string();
                                } else if !self.state.select_folder_form[1].contains(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR)
                                    || !self.state.select_folder_form[1].contains(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
                                    {
                                    self.state.select_folder_form[1] = format!("{} {}", cyberpunk_folder_input, NOT_A_DIRECTORY_ERROR);
                                }
                            }
                            if mod_folder_ok && cyberpunk_folder_ok {
                                self.state.ui_mode = UiMode::Explore;
                                self.state.focus = Focus::NoFocus;
                                // clear temp input store
                                self.state.select_folder_form[0] = String::new();
                                self.state.select_folder_form[1] = String::new();
                            }
                        }
                        if self.state.ui_mode == UiMode::Explore {
                            if let Some(selected) = self.state.file_list.state.selected() {
                                let selected_file = self.state.file_list.items[selected].0.clone();
                                let selected_file_path = Path::new(&self.selected_folder.as_ref().unwrap()).join(selected_file);
                                if !check_if_mod_is_valid(selected_file_path.clone()) {
                                    error!("{} is not a valid mod", selected_file_path.to_string_lossy());
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::SelectFolder => {
                        if self.state.ui_mode != UiMode::SelectFolder {
                            self.state.ui_mode = UiMode::SelectFolder;
                            self.state.focus = Focus::ModFolderInput;
                        } else {
                            self.state.ui_mode = UiMode::Explore;
                            self.state.focus = Focus::NoFocus;
                        }
                        AppReturn::Continue
                    }
                }
            } else {
                warn!("No action accociated to {}", key);
                AppReturn::Continue
            }
        }
    }
    
    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }
    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn status(&self) -> &AppStatus {
        &self.state.status
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = Action::all()
        .into();
        self.state.status = AppStatus::initialized()
    }
    pub fn loaded(&mut self) {
        self.is_loading = false;
    }
    pub fn current_focus(&self) -> &Focus {
        &self.state.focus
    }
    pub fn change_focus(&mut self, focus: Focus) {
        self.state.focus = focus;
    }
}