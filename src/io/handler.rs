use std::{sync::Arc, env::temp_dir, fs::{File, OpenOptions, self}, io::{Write, Read}, path::Path};
use crate::{
    app::{App, utils::{log_help, Settings, check_if_cyberpunk_dir_is_valid}, state::{UiMode, Focus}},
    constants::{WORKING_DIR_NAME, SAVE_DIR_NAME, SAVE_FILE_NAME, NOT_A_DIRECTORY_ERROR,
        NOT_A_VALID_CYBERPUNK_FOLDER_ERROR, CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR,
        MOD_FOLDER_INPUT_EMPTY_ERROR}
    };
use compress_tools::{uncompress_archive, Ownership};
use eyre::Result;
use fs_extra::dir::CopyOptions;
use log::{
    error,
    info, debug,
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
            IoEvent::InstallMod => {
                let result = self.install_mod().await;
                if let Err(err) = result {
                    error!("Oops, something wrong happened: {:?}", err);
                } else {
                    let check_install = self.check_if_mod_is_installed().await;
                    if let Err(err) = check_install {
                        error!("Oops, something wrong happened: {:?}", err);
                    }
                }
                Ok(())
            }
            IoEvent::UninstallMod => {
                let result = self.uninstall_mod().await;
                if let Err(err) = result {
                    error!("Oops, something wrong happened: {:?}", err);
                } else {
                    let check_install = self.check_if_mod_is_installed().await;
                    if let Err(err) = check_install {
                        error!("Oops, something wrong happened: {:?}", err);
                    }
                }
                Ok(())
            }
            IoEvent::CheckIfModIsInstalled => {
                let result = self.check_if_mod_is_installed().await;
                if let Err(_err) = result {
                    // do not log error if the mod is not installed
                }
                Ok(())
            }
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
        if app.state.file_list.state.selected().is_none() {
            app.state.file_list.next();
        }
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
        // check if the temp_mod_path has only .archive files if so copy them to archive/pc/mod in cyberpunk folder else use fs_extra to copy the files recursively to the cyberpunk folder
        let mut has_only_archive_files = false;
        for entry in WalkDir::new(&temp_mod_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().unwrap_or_default() == "archive" {
                has_only_archive_files = true;
            } else {
                has_only_archive_files = false;
                break;
            }
        }
        if has_only_archive_files {
            // copy the files to the cyberpunk folder
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                if path.extension().unwrap_or_default() == "archive" {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let dest_path = cyberpunk_dir.join("archive\\pc\\mod").join(file_name);
                    fs::copy(path, dest_path)?;
                }
            }
        } else {
            debug!("🚀 Copying files to the cyberpunk folder");
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                let mut copy_options = CopyOptions::new();
                copy_options.overwrite = true;
                copy_options.skip_exist = false;
                // remove mod_path from the start of the path
                let dir_name = path
                    .strip_prefix(&temp_mod_path)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let dest_path = cyberpunk_dir.join(dir_name);
                debug!("🚀 Copying {} to {}", &path.to_str().unwrap(), &dest_path.to_str().unwrap());
                // ensure directory exists
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs_extra::copy_items(&[path], &dest_path.parent().unwrap(), &copy_options)?;
            }
        }
        info!("👍 Mod installed");
        Ok(())
    }

    async fn uninstall_mod(&mut self) -> Result<()> {
        let app = self.app.lock().await;
        // check if the mod is installed
        if !app.mod_popup.as_ref().unwrap().get_mod_install_status().unwrap_or(false) {
            error!("🚫 Mod is not installed");
            return Ok(());
        }
        info!("🚀 Uninstalling mod");
        let cyberpunk_dir = app.cyberpunk_folder.clone().unwrap();
        let mod_file_name = app.mod_popup.as_ref().unwrap().get_mod_name();
        let mod_path = app.mod_folder.clone().unwrap().join(mod_file_name);
        let temp_dir = temp_dir().join(WORKING_DIR_NAME);
        // remove extension
        let temp_mod_path = temp_dir.join(mod_file_name.split('.').next().unwrap());
        let source = File::open(&mod_path)?;
        uncompress_archive(source, &temp_mod_path, Ownership::Preserve)?;
        let mut has_only_archive_files = false;
        for entry in WalkDir::new(&temp_mod_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().unwrap_or_default() == "archive" {
                has_only_archive_files = true;
            } else {
                has_only_archive_files = false;
                break;
            }
        }
        if has_only_archive_files {
            // check if the files exist in the cyberpunk folder
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                if path.extension().unwrap_or_default() == "archive" {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let dest_path = cyberpunk_dir.join("archive\\pc\\mod").join(file_name);
                    if dest_path.exists() {
                        fs::remove_file(dest_path)?;
                    }
                }
            }
        } else {
            debug!("🚀 Removing files from the cyberpunk folder");
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                // remove mod_path from the start of the path
                let dir_name = path
                    .strip_prefix(&temp_mod_path)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let dest_path = cyberpunk_dir.join(dir_name);
                if dest_path.exists() {
                    if dest_path.is_dir() {
                        // dont remove any directories
                    } else {
                        debug!("🚀 Removing {}", &dest_path.to_str().unwrap());
                        fs::remove_file(dest_path)?;
                    }
                }
            }
            // iterate again and remove empty directories
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                // remove mod_path from the start of the path
                let dir_name = path
                    .strip_prefix(&temp_mod_path)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let dest_path = cyberpunk_dir.join(dir_name);
                if dest_path.exists() {
                    if dest_path.is_dir() {
                        if dest_path.read_dir()?.next().is_none() {
                            debug!("🚀 Removing {}", &dest_path.to_str().unwrap());
                            fs::remove_dir(dest_path)?;
                        }
                    }
                }
            }
        }
        info!("👍 Mod uninstalled");
        Ok(())
    }

    async fn check_if_mod_is_installed(&mut self) -> Result<()> {
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
        // compare the files in the temp_mod_path with the files in the cyberpunk folder
        // if the files are not the same set install_status to false
        let mut has_only_archive_files = false;
        for entry in WalkDir::new(&temp_mod_path) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().unwrap_or_default() == "archive" {
                has_only_archive_files = true;
            } else {
                has_only_archive_files = false;
                break;
            }
        }
        if has_only_archive_files {
            // check if the files exist in the cyberpunk folder
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                if path.extension().unwrap_or_default() == "archive" {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let dest_path = cyberpunk_dir.join("archive\\pc\\mod").join(file_name);
                    if !dest_path.exists() {
                        install_status = false;
                        break;
                    }
                }
            }
        } else {
            for entry in WalkDir::new(&temp_mod_path) {
                let entry = entry?;
                let path = entry.path();
                // remove mod_path from the start of the path
                let dir_name = path
                    .strip_prefix(&temp_mod_path)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let dest_path = cyberpunk_dir.join(dir_name);
                if !dest_path.exists() {
                    install_status = false;
                    break;
                }
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
            if app.state.file_list.state.selected().is_none() {
                app.state.file_list.next();
            }
            app.state.focus = Focus::NoFocus;
            // clear temp input store
            app.state.select_folder_form[0] = String::new();
            app.state.select_folder_form[1] = String::new();
        }
        Ok(())
    }
}