use std::{sync::Arc, env::temp_dir, fs::{File, OpenOptions, self}, io::{Write, Read}, path::Path};
use crate::{
    app::{App, utils::{log_help, Settings, check_if_cyberpunk_dir_is_valid}, state::{UiMode, Focus}},
    constants::{WORKING_DIR_NAME, SAVE_DIR_NAME, SAVE_FILE_NAME, NOT_A_DIRECTORY_ERROR,
        NOT_A_VALID_CYBERPUNK_FOLDER_ERROR, CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR,
        MOD_FOLDER_INPUT_EMPTY_ERROR}
    };
use compress_tools::{uncompress_archive, Ownership};
use eyre::Result;
use log::{
    error,
    info,
};
use walkdir::WalkDir;

use super::IoEvent;

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::InstallMod => self.install_mod().await,
            IoEvent::UninstallMod => self.do_uninstall_mod().await,
            IoEvent::CheckIfModIsInstalled => self.do_check_if_mod_is_installed().await,
            IoEvent::SaveSettings => self.do_save_settings().await,
            IoEvent::LoadMods => self.do_load_mods(false).await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initializing the application");
        self.get_saved_settings().await?;
        self.do_load_mods(true).await?;
        let mut app = self.app.lock().await;
        app.initialized(); // we could update the app state
        log_help();
        info!("👍 Application initialized");
        Ok(())
    }

    async fn install_mod(&mut self) -> Result<()> {
        info!("🚀 Installing mod");
        let app = self.app.lock().await;
        let cyberpunk_dir = app.cyberpunk_folder.clone().unwrap();
        let mod_file_name = app.mod_popup.as_ref().unwrap().get_mod_name();
        let mod_path = app.mod_folder.clone().unwrap().join(mod_file_name);
        let temp_dir = temp_dir().join(WORKING_DIR_NAME);
        // remove extension
        let temp_mod_path = temp_dir.join(mod_file_name.split('.').next().unwrap());
        let source = File::open(&mod_path)?;
        uncompress_archive(source, &temp_mod_path, Ownership::Preserve)?;
        // mods are installed by copying the file structure from the zip file to the cyberpunk folder to uninstall we need to delete the files
        // we will not delete the directories because they may contain other files and only delete the files that are present in the zip file
        for entry in WalkDir::new(&temp_mod_path) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&temp_mod_path)?;
            let cyberpunk_path = cyberpunk_dir.join(relative_path);
            info!("👍 Checking file: {:?}", relative_path);
            if path.is_dir() {
                // we don't need to copy directories
                continue;
            }
            if !cyberpunk_path.exists() {
                // the file does not exist in the cyberpunk folder so we need to copy it
                // std::fs::copy(path, cyberpunk_path)?;
                info!("👍 Copying file: {:?}", relative_path);
            }
        }
        info!("👍 Mod installed");
        Ok(())
    }

    async fn do_uninstall_mod(&mut self) -> Result<()> {
        info!("🚀 Uninstalling mod");
        let app = self.app.lock().await;
        let cyberpunk_dir = app.cyberpunk_folder.clone().unwrap();
        let mod_file_name = app.mod_popup.as_ref().unwrap().get_mod_name();
        let mod_path = app.mod_folder.clone().unwrap().join(mod_file_name);
        let temp_dir = temp_dir().join(WORKING_DIR_NAME);
        // remove extension
        let temp_mod_path = temp_dir.join(mod_file_name.split('.').next().unwrap());
        let source = File::open(&mod_path)?;
        uncompress_archive(source, &temp_mod_path, Ownership::Preserve)?;
        // mods are installed by copying the file structure from the zip file to the cyberpunk folder to uninstall we need to delete the files
        // we will not delete the directories because they may contain other files and only delete the files that are present in the zip file
        for entry in WalkDir::new(&temp_mod_path) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&temp_mod_path)?;
            let cyberpunk_path = cyberpunk_dir.join(relative_path);
            info!("Checking file: {:?}", relative_path);
            if cyberpunk_path.exists() {
                info!("Deleting file: {:?}", cyberpunk_path);
                std::fs::remove_file(cyberpunk_path)?;
            }
        }
        info!("👍 Mod uninstalled");
        Ok(())
    }

    async fn do_check_if_mod_is_installed(&mut self) -> Result<()> {
        info!("🚀 Checking if mod is installed");
        let mut install_status = true;
        let mut app = self.app.lock().await;
        let cyberpunk_dir = app.cyberpunk_folder.clone().unwrap();
        let mod_file_name = app.mod_popup.as_ref().unwrap().get_mod_name();
        let mod_path = app.mod_folder.clone().unwrap().join(mod_file_name);
        let temp_dir = temp_dir().join(WORKING_DIR_NAME);
        // remove extension
        let temp_mod_path = temp_dir.join(mod_file_name.split('.').next().unwrap());
        let source = File::open(&mod_path)?;
        uncompress_archive(source, &temp_mod_path, Ownership::Preserve)?;
        // mods are installed by copying the file structure from the zip file to the cyberpunk folder to uninstall we need to delete the files
        // we will not delete the directories because they may contain other files and only delete the files that are present in the zip file
        for entry in WalkDir::new(&temp_mod_path) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&temp_mod_path)?;
            let cyberpunk_path = cyberpunk_dir.join(relative_path);
            info!("Comparing file: {:?}", relative_path);
            if path.is_dir() {
                // we don't need to copy directories
                continue;
            }
            if !cyberpunk_path.exists() {
                install_status = false;
            }
        }
        app.mod_popup.as_mut().unwrap().set_mod_install_status(install_status);
        if install_status {
            info!("👍 Mod is installed");
        } else {
            info!("👍 Mod is not installed");
        }
        Ok(())
    }
    
    async fn do_save_settings(&mut self) -> Result<()> {
        info!("🚀 Saving settings");
        let app = self.app.lock().await;
        // get pub mod_folder: Option<PathBuf>, pub cyberpunk_folder: Option<PathBuf>, from app and save them to SAVE_FILE_NAME in SAVE_FILE_DIR
        let mut mod_folder = app.mod_folder.clone();
        // check if unwrap is safe
        if mod_folder.is_none() {
            mod_folder = Some("".to_string().into());
        } else {
            mod_folder = Some(mod_folder.unwrap());
        }
        let mut cyberpunk_folder = app.cyberpunk_folder.clone();
        // check if unwrap is safe
        if cyberpunk_folder.is_none() {
            cyberpunk_folder = Some("".to_string().into());
        } else {
            cyberpunk_folder = Some(cyberpunk_folder.unwrap());
        }
        let settings = Settings {
            mod_folder: mod_folder,
            cyberpunk_folder: cyberpunk_folder,
        };
        let settings_json = serde_json::to_string(&settings)?;
        let save_file_path = temp_dir().join(SAVE_DIR_NAME).join(SAVE_FILE_NAME);
        // if file is not present create it, else overwrite it
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(save_file_path)?;
        file.write_all(settings_json.as_bytes())?;
        info!("👍 Settings saved");
        Ok(())
    }

    async fn get_saved_settings(&mut self) -> Result<()> {
        info!("🚀 Fetching saved settings");
        let save_file_path = temp_dir().join(SAVE_DIR_NAME).join(SAVE_FILE_NAME);
        // check if file exists
        if !save_file_path.exists() {
            info!("👍 No saved settings found");
            return Ok(());
        }
        let mut file = File::open(save_file_path)?;
        let mut settings_json = String::new();
        file.read_to_string(&mut settings_json)?;
        let settings: Settings = serde_json::from_str(&settings_json)?;
        let mut app = self.app.lock().await;
        // if the saved settings are empty set None
        if settings.mod_folder.is_none() || settings.mod_folder.as_ref().unwrap().to_str().unwrap() == "" {
            app.mod_folder = None;
        } else {
            app.mod_folder = Some(settings.mod_folder.unwrap());
        }
        if settings.cyberpunk_folder.is_none() || settings.cyberpunk_folder.as_ref().unwrap().to_str().unwrap() == "" {
            app.cyberpunk_folder = None;
        } else {
            app.cyberpunk_folder = Some(settings.cyberpunk_folder.unwrap());
        }
        info!("👍 Saved settings loaded");
        Ok(())
    }

    async fn do_load_mods(&mut self, from_save: bool) -> Result<()> {
        let mut app = self.app.lock().await;
        let mut mod_folder_ok = false;
        let mut cyberpunk_folder_ok = false;
        let mut mod_folder_input = app.state.select_folder_form[0].clone();
        let mut cyberpunk_folder_input = app.state.select_folder_form[1].clone();
        if from_save {
            if app.mod_folder.is_none() {
                mod_folder_input = "".to_string();
            } else {
                mod_folder_input = app.mod_folder.clone().unwrap().to_str().unwrap().to_string();
            }
            if app.cyberpunk_folder.is_none() {
                cyberpunk_folder_input = "".to_string();
            } else {
                cyberpunk_folder_input = app.cyberpunk_folder.clone().unwrap().to_str().unwrap().to_string();
            }
        }
        // remove " from the start and end of the string if they exist
        mod_folder_input = mod_folder_input.trim_start_matches('"').trim_end_matches('"').to_string();
        cyberpunk_folder_input = cyberpunk_folder_input.trim_start_matches('"').trim_end_matches('"').to_string();

        if mod_folder_input.ends_with(NOT_A_DIRECTORY_ERROR)
            || cyberpunk_folder_input.ends_with(NOT_A_DIRECTORY_ERROR)
            || cyberpunk_folder_input.ends_with(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
            {
            return Ok(());
        }

        let mod_folder_path = Path::new(&mod_folder_input);
        let cyberpunk_folder_path = Path::new(&cyberpunk_folder_input);
        if mod_folder_path.is_dir() {
            app.mod_folder = Some(mod_folder_path.to_path_buf());
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
            app.state.file_list.items = files;
            mod_folder_ok = true;
        } else {
            // check if input is empty, put error message in temp input store
            if mod_folder_input.trim().is_empty() {
                app.state.select_folder_form[0] = MOD_FOLDER_INPUT_EMPTY_ERROR.to_string();
            } else if !app.state.select_folder_form[0].contains(MOD_FOLDER_INPUT_EMPTY_ERROR) {
                app.state.select_folder_form[0] = format!("{} {}", mod_folder_input, NOT_A_DIRECTORY_ERROR);
            }
        }
        if cyberpunk_folder_path.is_dir() {
            if !check_if_cyberpunk_dir_is_valid(cyberpunk_folder_path.clone().to_path_buf()) {
                app.state.select_folder_form[1] = format!("{} {}", cyberpunk_folder_input, NOT_A_VALID_CYBERPUNK_FOLDER_ERROR);
                return Ok(());
            } else {
                app.cyberpunk_folder = Some(cyberpunk_folder_path.to_path_buf());
                cyberpunk_folder_ok = true;
            }
        } else {
            // check if input is empty, put error message in temp input store
            if cyberpunk_folder_input.trim().is_empty() {
                app.state.select_folder_form[1] = CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR.to_string();
            } else if !app.state.select_folder_form[1].contains(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR)
                || !app.state.select_folder_form[1].contains(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
                {
                app.state.select_folder_form[1] = format!("{} {}", cyberpunk_folder_input, NOT_A_DIRECTORY_ERROR);
            }
        }
        if mod_folder_ok && cyberpunk_folder_ok {
            app.state.ui_mode = UiMode::Explore;
            app.state.focus = Focus::NoFocus;
            // clear temp input store
            app.state.select_folder_form[0] = String::new();
            app.state.select_folder_form[1] = String::new();
        }
        Ok(())
    }
}