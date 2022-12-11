use std::{path::PathBuf, env::temp_dir, fs::{create_dir_all, File, remove_dir_all}};

use compress_tools::{uncompress_archive, Ownership};
use log::{info, error};
use serde::{Serialize, Deserialize};
use tui::widgets::ListState;
use walkdir::WalkDir;

use crate::constants::WORKING_DIR_NAME;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum ModOptions {
    Install,
    Uninstall,
    Repair
}

impl ModOptions {
    pub fn get_all_options() -> Vec<ModOptions> {
        vec![
            ModOptions::Install,
            ModOptions::Uninstall,
            ModOptions::Repair,
        ]
    }

    pub fn get_all_options_as_listitems() -> Vec<String> {
        let all_options = ModOptions::get_all_options();
        all_options
        .iter()
        .map(|mod_option| {
            mod_option.to_string()
        })
        .collect()
    }

    pub fn to_string(&self) -> String {
        match self {
            ModOptions::Install => "Install".to_string(),
            ModOptions::Uninstall => "Uninstall".to_string(),
            ModOptions::Repair => "Repair".to_string(),
        }
    }

    pub fn get_option_from_string(option: &str) -> Option<ModOptions> {
        match option {
            "Install" => Some(ModOptions::Install),
            "Uninstall" => Some(ModOptions::Uninstall),
            "Repair" => Some(ModOptions::Repair),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModPopup {
    pub mod_name: String,
    pub mod_install_status: Option<bool>,
}

impl ModPopup {
    pub fn new(mod_name: String) -> Self {
        Self {
            mod_name,
            mod_install_status: None,
        }
    }

    pub fn get_mod_name(&self) -> &str {
        &self.mod_name
    }

    pub fn get_mod_install_status(&self) -> Option<bool> {
        self.mod_install_status
    }

    pub fn set_mod_install_status(&mut self, status: bool) {
        self.mod_install_status = Some(status);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub cyberpunk_folder: Option<PathBuf>,
    pub mod_folder: Option<PathBuf>,
}

pub fn log_help() {
    info!("Press <f> to select Mod and Cyberpunk Folders");
    info!("Use UP/DOWN to navigate the list");
    info!("Press <Enter> to select a file");
    info!("Press <i> to enter input mode (Green Highlight)");
    info!("Press <Tab> to switch between input and submit button (Blue Highlight)");
    info!("Press <h> to see this help message again");
    info!("Press <Ctrl + s> to save settings");
    info!("Press <Esc> to exit input mode or close popup");
    info!("Press <q> to quit");
}

pub fn check_if_mod_is_valid(file_path: PathBuf) -> bool {
    let mut is_valid = false;
    // make sure the file exists
    if !file_path.exists() {
        info!("{} does not exist", file_path.to_string_lossy());
        return false;
    }
    let mut destination = temp_dir();
    // make a cyberpunk_mod_manager directory in the temp directory
    destination.push(WORKING_DIR_NAME);
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
            if dir_name == "archive" || dir_name == "bin" || dir_name == "engine" || dir_name == "mods" || dir_name == "red4ext" || dir_name == "r6"{
                is_valid = true;
            }
        }
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            if file_name.ends_with(".archive") {
                is_valid = true;
            }
        }
    }
    // remove the directory
    remove_dir_all(&destination).unwrap();
    is_valid
}

pub fn check_if_cyberpunk_dir_is_valid(file_path: PathBuf) -> bool {
    let mut is_valid = false;
    // make sure the file exists
    if !file_path.exists() {
        info!("{} does not exist", file_path.to_string_lossy());
        return false;
    }
    for entry in WalkDir::new(&file_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if dir_name == "archive" || dir_name == "bin" || dir_name == "engine" || dir_name == "mods" {
                is_valid = true;
                break;
            }
        }
    }
    if !is_valid {
        error!("{} is not a valid Cyberpunk 2077 directory", file_path.to_string_lossy());
    }
    is_valid
}