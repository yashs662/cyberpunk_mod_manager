use tui::style::{
    Style,
    Color,
    Modifier
};

pub const MIN_TERM_WIDTH: u16 = 110;
pub const MIN_TERM_HEIGHT: u16 = 30;
pub const APP_TITLE: &str = "Cyberpunk Mod Manager";
pub const MOD_FOLDER_INPUT_EMPTY_ERROR: &str = "Mods Folder input is empty";
pub const CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR: &str = "Cyberpunk Folder input is empty";
pub const NOT_A_VALID_CYBERPUNK_FOLDER_ERROR: &str = "is not a valid Cyberpunk folder";
pub const NOT_A_DIRECTORY_ERROR: &str = "is not a directory";

// Style
pub const ERROR_TEXT_STYLE: Style = Style {
    fg: Some(Color::LightRed),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const DEFAULT_STYLE: Style = Style{
    fg: Some(Color::White),
    bg: Some(Color::Black),
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
pub const LIST_SELECT_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::LightMagenta),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const FOCUS_STYLE: Style = Style{
    fg: Some(Color::Cyan),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const INPUT_STYLE: Style = Style{
    fg: Some(Color::LightGreen),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_ERROR_STYLE: Style = Style {
    fg: Some(Color::LightRed),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_DEBUG_STYLE: Style = Style {
    fg: Some(Color::LightGreen),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_WARN_STYLE: Style = Style {
    fg: Some(Color::LightYellow),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_TRACE_STYLE: Style = Style {
    fg: Some(Color::Gray),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_INFO_STYLE: Style = Style {
    fg: Some(Color::LightCyan),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};