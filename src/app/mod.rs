use std::path::Path;
use std::vec;
use std::path::PathBuf;
use log::{debug, info};
use log::{
    error,
    warn
};

use self::actions::Actions;
use self::state::AppState;
use self::state::AppStatus;
use self::state::Focus;
use self::state::UiMode;
use self::utils::{ModPopup, ModOptions};
use self::utils::check_if_mod_is_valid;
use self::utils::log_help;
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
    pub mod_popup: Option<ModPopup>,
    pub state: AppState,
    pub mod_folder: Option<PathBuf>,
    pub cyberpunk_folder: Option<PathBuf>,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();
        let mod_popup = None;

        Self {
            io_tx,
            actions,
            is_loading,
            mod_popup,
            state,
            mod_folder: None,
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
                        Focus::ModFolderInput => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position > 0 {
                                self.state.select_folder_form[0].remove(cursor_position - 1);
                                self.state.cursor_position = Some(cursor_position - 1);
                            } else {
                                self.state.select_folder_form[0].remove(0);
                            }
                        },
                        Focus::CyberpunkFolderInput => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position > 0 {
                                self.state.select_folder_form[1].remove(cursor_position - 1);
                                self.state.cursor_position = Some(cursor_position - 1);
                            } else {
                                self.state.select_folder_form[1].remove(0);
                            }
                        },
                        _ => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position > 0 {
                                self.state.current_input.remove(cursor_position - 1);
                                self.state.cursor_position = Some(cursor_position - 1);
                            } else {
                                self.state.current_input.remove(0);
                            }
                        }
                    };
                    current_key = "".to_string();
                // check for Left and right
                } else if current_key == "<Left>" {
                    match self.state.focus {
                        Focus::ModFolderInput => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position > 0 {
                                self.state.cursor_position = Some(cursor_position - 1);
                            }
                        },
                        Focus::CyberpunkFolderInput => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position > 0 {
                                self.state.cursor_position = Some(cursor_position - 1);
                            }
                        },
                        _ => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position > 0 {
                                self.state.cursor_position = Some(cursor_position - 1);
                            }
                        }
                    };
                    current_key = "".to_string();
                } else if current_key == "<Right>" {
                    match self.state.focus {
                        Focus::ModFolderInput => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position < self.state.select_folder_form[0].len() {
                                self.state.cursor_position = Some(cursor_position + 1);
                            }
                        },
                        Focus::CyberpunkFolderInput => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position < self.state.select_folder_form[1].len() {
                                self.state.cursor_position = Some(cursor_position + 1);
                            }
                        },
                        _ => {
                            let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                            if cursor_position < self.state.current_input.len() {
                                self.state.cursor_position = Some(cursor_position + 1);
                            }
                        }
                    };
                    current_key = "".to_string();
                } else if current_key == "<Home>" {
                    match self.state.focus {
                        Focus::ModFolderInput => {
                            self.state.cursor_position = Some(0);
                        },
                        Focus::CyberpunkFolderInput => {
                            self.state.cursor_position = Some(0);
                        },
                        _ => {
                            self.state.cursor_position = Some(0);
                        }
                    };
                    current_key = "".to_string();
                } else if current_key == "<End>" {
                    match self.state.focus {
                        Focus::ModFolderInput => {
                            self.state.cursor_position = Some(self.state.select_folder_form[0].len());
                        },
                        Focus::CyberpunkFolderInput => {
                            self.state.cursor_position = Some(self.state.select_folder_form[1].len());
                        },
                        _ => {
                            self.state.cursor_position = Some(self.state.current_input.len());
                        }
                    };
                    current_key = "".to_string();
                } else if current_key.starts_with("<") && current_key.ends_with(">") {
                    current_key = current_key[1..current_key.len() - 1].to_string();
                }

                if self.state.focus == Focus::ModFolderInput && current_key != "" {
                    let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                    self.state.select_folder_form[0].insert(cursor_position, current_key.chars().next().unwrap());
                    self.state.cursor_position = Some(cursor_position + 1);
                } else if self.state.focus == Focus::CyberpunkFolderInput && current_key != "" {
                    let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                    self.state.select_folder_form[1].insert(cursor_position, current_key.chars().next().unwrap());
                    self.state.cursor_position = Some(cursor_position + 1);
                } else if current_key != "" {
                    let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                    self.state.current_input.insert(cursor_position, current_key.chars().next().unwrap());
                    self.state.cursor_position = Some(cursor_position + 1);
                }
            } else {
                self.state.status = AppStatus::Initialized;
                debug!("Exiting user input mode");
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
                        if self.mod_popup.is_some() {
                            self.state.mod_options.previous();
                        } else {
                            self.state.file_list.previous()
                        }
                        AppReturn::Continue
                    }
                    Action::Down => {
                        if self.mod_popup.is_some() {
                            self.state.mod_options.next();
                        } else {
                            self.state.file_list.next()
                        }
                        AppReturn::Continue
                    }
                    Action::Right => {
                        if self.state.status == AppStatus::UserInput {
                            if self.state.focus == Focus::ModFolderInput {
                                let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                                self.state.cursor_position = Some((cursor_position + 1).min(self.state.select_folder_form[0].len()));
                            } else if self.state.focus == Focus::CyberpunkFolderInput {
                                let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                                self.state.cursor_position = Some((cursor_position + 1).min(self.state.select_folder_form[1].len()));
                            } else {
                                let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                                self.state.cursor_position = Some((cursor_position + 1).min(self.state.current_input.len()));
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::Left => {
                        if self.state.status == AppStatus::UserInput {
                            if self.state.focus == Focus::ModFolderInput {
                                let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                                if cursor_position > 0 {
                                    self.state.cursor_position = Some(cursor_position - 1);
                                }
                            } else if self.state.focus == Focus::CyberpunkFolderInput {
                                let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                                if cursor_position > 0 {
                                    self.state.cursor_position = Some(cursor_position - 1);
                                }
                            } else {
                                let cursor_position = self.state.cursor_position.unwrap_or_else(||0);
                                if cursor_position > 0 {
                                    self.state.cursor_position = Some(cursor_position - 1);
                                }
                            }
                        }
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
                                    // ensure the cursor is at the end of the string
                                    self.state.cursor_position = Some(self.state.select_folder_form[0].len());
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
                                    // ensure the cursor is at the end of the string
                                    self.state.cursor_position = Some(self.state.select_folder_form[1].len());
                                }
                            }
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::Escape => {
                        if self.state.status == AppStatus::UserInput {
                            self.state.status = AppStatus::Initialized;
                            self.state.cursor_position = None;
                        } else if self.state.status == AppStatus::Initialized {
                            if self.state.ui_mode == UiMode::SelectFolder {
                                self.state.ui_mode = UiMode::Explore;
                            } else if self.state.ui_mode == UiMode::Explore && self.mod_popup.is_none() {
                                return AppReturn::Exit;
                            }
                        }
                        if self.mod_popup.is_some() {
                            self.mod_popup = None;
                        }
                        
                        AppReturn::Continue
                    }
                    Action::Enter => {
                        if self.state.status == AppStatus::UserInput {
                            self.state.status = AppStatus::Initialized;
                            self.state.cursor_position = None;
                        }
                        if self.state.focus == Focus::Submit {
                            self.dispatch(IoEvent::LoadMods).await;
                        }
                        if self.state.ui_mode == UiMode::Explore {
                            if self.mod_popup.is_some() {
                                let current_selected_option_index = self.state.mod_options.state.selected();
                                let available_options = ModOptions::get_all_options();
                                if let Some(selected_option_index) = current_selected_option_index {
                                    let selected_option = available_options[selected_option_index].clone();
                                    match selected_option {
                                        ModOptions::Install => {
                                            self.dispatch(IoEvent::InstallMod).await;
                                        }
                                        ModOptions::Uninstall => {
                                            self.dispatch(IoEvent::UninstallMod).await;
                                        }
                                        ModOptions::Repair => {
                                            self.dispatch(IoEvent::UninstallMod).await;
                                            self.dispatch(IoEvent::InstallMod).await;
                                        }
                                    }
                                }
                            }
                            else if let Some(selected) = self.state.file_list.state.selected() {
                                let selected_file = self.state.file_list.items[selected].0.clone();
                                let selected_file_path = Path::new(&self.mod_folder.as_ref().unwrap()).join(&selected_file);
                                if !check_if_mod_is_valid(selected_file_path.clone()) {
                                    error!("{} is not a valid mod", selected_file_path.to_string_lossy());
                                } else {
                                    info!("Selected mod: {}", selected_file_path.to_string_lossy());
                                    self.mod_popup = Some(ModPopup::new(selected_file.clone()));
                                    self.dispatch(IoEvent::CheckIfModIsInstalled).await;
                                    info!("popup: {:?}", self.mod_popup);
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::SelectFolder => {
                        if self.state.ui_mode != UiMode::SelectFolder {
                            self.state.ui_mode = UiMode::SelectFolder;
                            // if mod_folder or cyberpunk_folder is set, set the input value to the current value
                            if let Some(mod_folder) = &self.mod_folder {
                                self.state.select_folder_form[0] = mod_folder.clone().to_string_lossy().to_string();
                            }
                            if let Some(cyberpunk_folder) = &self.cyberpunk_folder {
                                self.state.select_folder_form[1] = cyberpunk_folder.clone().to_string_lossy().to_string();
                            }
                            self.state.focus = Focus::ModFolderInput;
                        } else {
                            self.state.ui_mode = UiMode::Explore;
                            // check if state.file_list has any selected items
                            if self.state.file_list.state.selected().is_none() {
                                self.state.file_list.next();
                            }
                            self.state.focus = Focus::NoFocus;
                        }
                        AppReturn::Continue
                    }
                    Action::LogHelp => {
                        log_help();
                        AppReturn::Continue
                    }
                    Action::SaveSettings => {
                        self.dispatch(IoEvent::SaveSettings).await;
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