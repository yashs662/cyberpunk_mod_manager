use super::utils::{StatefulList, ModOptions};

#[derive(Clone, PartialEq, Debug)]
pub enum AppStatus {
    Init,
    Initialized,
    UserInput
}

impl AppStatus {
    pub fn initialized() -> Self {
        Self::Initialized
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    NoFocus,
    Submit,
    ModFolderInput,
    CyberpunkFolderInput
}

impl Focus {
    pub fn to_str(&self) -> &str {
        match self {
            Focus::NoFocus => "Nothing",
            Focus::Submit => "Submit",
            Focus::ModFolderInput => "Mod Folder",
            Focus::CyberpunkFolderInput => "Cyberpunk Folder",
        }
    }

    pub fn all() -> Vec<Focus> {
        vec![Focus::Submit, Focus::ModFolderInput, Focus::CyberpunkFolderInput]
    }

    pub fn next(&self, available_tabs: &Vec<String>) -> Self {
        let current = self.to_str();
        let index = available_tabs.iter().position(|x| x == current);
        // check if index is None
        let index = match index {
            Some(i) => i,
            None => 0,
        };
        if available_tabs.len() <= 1 {
            return Self::NoFocus;
        }
        let next_index = (index + 1) % available_tabs.len();
        match available_tabs[next_index].as_str() {
            "Submit" => Focus::Submit,
            "Mod Folder" => Focus::ModFolderInput,
            "Cyberpunk Folder" => Focus::CyberpunkFolderInput,
            _ => Focus::NoFocus,
        }
    }

    pub fn prev(&self, available_tabs: &Vec<String>) -> Self {
        let current = self.to_str();
        let index = available_tabs.iter().position(|x| x == current);
        // check if index is None
        let index = match index {
            Some(i) => i,
            None => 0,
        };
        let prev_index = if index == 0 {
            available_tabs.len() - 1
        } else {
            index - 1
        };
        match available_tabs[prev_index].as_str() {
            "Submit" => Focus::Submit,
            "Mod Folder" => Focus::ModFolderInput,
            "Cyberpunk Folder" => Focus::CyberpunkFolderInput,
            _ => Focus::NoFocus,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Submit" => Focus::Submit,
            "Mod Folder" => Focus::ModFolderInput,
            "Cyberpunk Folder" => Focus::CyberpunkFolderInput,
            _ => Focus::NoFocus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiMode {
    Explore,
    SelectFolder,
}

impl UiMode {
    pub fn to_string(&self) -> String {
        match self {
            UiMode::Explore => "Explore".to_string(),
            UiMode::SelectFolder => "Select Folder".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<UiMode> {
        match s {
            "Explore" => Some(UiMode::Explore),
            "Select Folder" => Some(UiMode::SelectFolder),
            _ => None,
        }
    }

    pub fn get_available_targets(&self) -> Vec<String> {
        match self {
            UiMode::Explore => vec![],
            UiMode::SelectFolder => vec![
                "Mod Folder".to_string(),
                "Cyberpunk Folder".to_string(),
                "Submit".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Input,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub status: AppStatus,
    pub focus: Focus,
    pub current_input: String,
    pub select_folder_form: Vec<String>,
    pub ui_mode: UiMode,
    pub file_list: StatefulList<(String, usize)>,
    pub mod_options: StatefulList<String>,
    pub cursor_position: Option<usize>
}

impl Default for AppState {
    fn default() -> AppState {
        let mod_options_list = ModOptions::get_all_options_as_listitems();
        AppState {
            status: AppStatus::Init,
            focus: Focus::NoFocus,
            current_input: String::new(),
            select_folder_form: vec![String::new(), String::new()],
            ui_mode: UiMode::Explore,
            file_list: StatefulList::with_items(vec![]),
            mod_options: StatefulList::with_items(mod_options_list),
            cursor_position: None
        }
    }
}