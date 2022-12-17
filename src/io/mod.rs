pub mod handler;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,      // Launch to initialize the application
    InstallMod,      // Install a mod
    UninstallMod,    // Uninstall a mod
    CheckIfModIsInstalled, // Check if a mod is installed
    SaveSettings,    // Save settings
    LoadMods,        // Load mods into app
    DeleteTempDir,   // Delete the temp dir on exit
}
