pub mod ui;
use tui::backend::Backend;
use tui::Frame;

use crate::app::{
    App,
    state::{AppState, UiMode}
};

use self::ui::{check_size, draw_size_error, draw_explore, draw_select_folder};

/// Main UI Drawing handler
pub fn draw<B>(rect: &mut Frame<B>, app: &App, states: &mut AppState)
where
    B: Backend,
{   
    let msg = check_size(&rect.size());
    if &msg != "Size OK" {
        draw_size_error(rect, &rect.size(), msg);
        return;
    }

    match &app.state.ui_mode {
        UiMode::Explore => {
            draw_explore(rect, app, &mut states.file_list.state)
        }
        UiMode::SelectFolder => {
            draw_select_folder(rect, app)
        }
    }
}