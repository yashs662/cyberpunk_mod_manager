use std::path::PathBuf;
use tui::{
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    backend::Backend,
    Frame,
    text::{Spans, Span, Text},
    widgets::{Paragraph, Block, Borders, Wrap, ListItem, List, ListState, Clear}
};
use tui_logger::TuiLoggerWidget;

use crate::{
    constants::{MIN_TERM_WIDTH, MIN_TERM_HEIGHT, ERROR_TEXT_STYLE,
                APP_TITLE, FOCUS_STYLE, LOG_ERROR_STYLE,
                LOG_DEBUG_STYLE, LOG_WARN_STYLE, LOG_TRACE_STYLE,
                LOG_INFO_STYLE, MOD_FOLDER_INPUT_EMPTY_ERROR, NOT_A_DIRECTORY_ERROR,
                CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR, NOT_A_VALID_CYBERPUNK_FOLDER_ERROR,
                CYBERPUNK_STYLE_YELLOW, CYBERPUNK_STYLE_PINK, CYBERPUNK_STYLE_CYAN,
                CYBERPUNK_STYLE_YELLOW_DARK, CYBERPUNK_STYLE_PINK_DARK, CYBERPUNK_STYLE_CYAN_DARK
    },
    App, app::{state::{Focus, AppStatus}, utils::ModOptions},
};

/// Helper function to check terminal size
pub fn check_size(rect: &Rect) -> String {
    let mut msg = String::new();
    if rect.width < MIN_TERM_WIDTH {
        msg.push_str(&format!("For optimal viewing experience, Terminal width should be >= {}, (current {})",MIN_TERM_WIDTH, rect.width));
    }
    else if rect.height < MIN_TERM_HEIGHT {
        msg.push_str(&format!("For optimal viewing experience, Terminal height should be >= {}, (current {})",MIN_TERM_HEIGHT, rect.height));
    }
    else {
        msg.push_str("Size OK");
    }
    msg
}

/// Draws size error screen if the terminal is too small
pub fn draw_size_error<B>(rect: &mut Frame<B>, size: &Rect, msg: String)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(*size);

    let title = draw_title(false);
    rect.render_widget(title, chunks[0]);

    let mut text = vec![Spans::from(Span::styled(&msg, ERROR_TEXT_STYLE))];
    text.append(&mut vec![Spans::from(Span::raw("Resize the window to continue, or press 'q' to quit."))]);
    let body = Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    rect.render_widget(body, chunks[1]);
}

/// Draws the title bar
pub fn draw_title<'a>(dark_mode: bool) -> Paragraph<'a> {
    
    let title_style = if dark_mode {
        CYBERPUNK_STYLE_YELLOW_DARK
    } else {
        CYBERPUNK_STYLE_YELLOW
    };

    Paragraph::new(APP_TITLE)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(title_style)
        )
}

pub fn draw_select_folder<B: Backend>(f: &mut Frame<B>, app: &App) {

    let submit_style = if app.state.focus == Focus::Submit {
        FOCUS_STYLE
    } else {
        CYBERPUNK_STYLE_CYAN
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(10)
            ].as_ref())
        .split(f.size());

    let title = Paragraph::new(Text::styled("Select Folder, Press <i> to edit, <Tab> to change focus and <Enter> to submit", CYBERPUNK_STYLE_CYAN))
        .block(Block::default().borders(Borders::ALL))
        .style(CYBERPUNK_STYLE_CYAN)
        .wrap(Wrap { trim: true });

    let mod_folder_text = app.state.select_folder_form[0].clone();
    let mod_folder_input_style = if app.state.focus == Focus::ModFolderInput {
        if app.state.status == AppStatus::UserInput {
            CYBERPUNK_STYLE_PINK
        } else {
            FOCUS_STYLE
        }
    } else if mod_folder_text.contains(MOD_FOLDER_INPUT_EMPTY_ERROR) || mod_folder_text.contains(NOT_A_DIRECTORY_ERROR){
        ERROR_TEXT_STYLE
    } else {
        CYBERPUNK_STYLE_CYAN
    };
    let mod_folder = Paragraph::new(Text::raw(mod_folder_text))
        .block(Block::default().borders(Borders::ALL).title("Mods Folder"))
        .style(mod_folder_input_style)
        .wrap(Wrap { trim: true });

    let cyberpunk_folder_text = app.state.select_folder_form[1].clone();
    let cyberpunk_folder_input_style = if app.state.focus == Focus::CyberpunkFolderInput {
        if app.state.status == AppStatus::UserInput {
            CYBERPUNK_STYLE_PINK
        } else {
            FOCUS_STYLE
        }
    } else if cyberpunk_folder_text.contains(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR)
        || cyberpunk_folder_text.contains(NOT_A_DIRECTORY_ERROR)
        || cyberpunk_folder_text.contains(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
        {
        ERROR_TEXT_STYLE
    } else {
        CYBERPUNK_STYLE_CYAN
    };
    let cyberpunk_folder = Paragraph::new(Text::raw(cyberpunk_folder_text))
        .block(Block::default().borders(Borders::ALL).title("Cyberpunk Folder"))
        .style(cyberpunk_folder_input_style)
        .wrap(Wrap { trim: true });

    let submit_button = Paragraph::new("Submit")
        .block(Block::default().borders(Borders::ALL))
        .style(submit_style)
        .wrap(Wrap { trim: true });

    // check if input mode is active, if so, show cursor
    if app.state.status == AppStatus::UserInput && app.state.focus == Focus::ModFolderInput {
        f.set_cursor(
            chunks[1].x + app.state.select_folder_form[0].len() as u16 + 1,
            chunks[1].y + 1,
        );
    } else if app.state.status == AppStatus::UserInput && app.state.focus == Focus::CyberpunkFolderInput {
        f.set_cursor(
            chunks[2].x + app.state.select_folder_form[1].len() as u16 + 1,
            chunks[2].y + 1,
        );
    }

    f.render_widget(title, chunks[0]);
    f.render_widget(mod_folder, chunks[1]);
    f.render_widget(cyberpunk_folder, chunks[2]);
    f.render_widget(submit_button, chunks[3]);
}

pub fn draw_explore<B: Backend>(f: &mut Frame<B>, app: &App, file_list_state: &mut ListState) {
    // Create two chunks with equal horizontal screen space
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(10),
            Constraint::Percentage(10)
            ].as_ref())
        .split(f.size());
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30)
            ].as_ref())
        .split(main_chunks[1]);

    let title_widget = draw_title(app.mod_popup.is_some());
    
    let current_folder = app.mod_folder.clone().unwrap_or_else(|| PathBuf::new());
    // check if current folder is a directory if not set it to No folder selected
    let current_folder_string = if current_folder.is_dir() {
        current_folder.to_string_lossy().to_string()
    } else {
        "No folder selected".to_string()
    };
    let current_folder_widget_style = if app.mod_popup.is_some() {
        CYBERPUNK_STYLE_PINK_DARK
    } else {
        CYBERPUNK_STYLE_PINK
    };
    let current_folder_widget = Paragraph::new(Text::raw(current_folder_string))
        .block(Block::default().borders(Borders::ALL).title("Mod Folder"))
        .style(current_folder_widget_style)
        .wrap(Wrap { trim: true });

    let cyberpunk_folder = app.cyberpunk_folder.clone().unwrap_or_else(|| PathBuf::new());
    // check if current folder is a directory if not set it to No folder selected
    let cyberpunk_folder_string = if cyberpunk_folder.is_dir() {
        cyberpunk_folder.to_string_lossy().to_string()
    } else {
        "No folder selected".to_string()
    };
    let cyberpunk_folder_widget_style = if app.mod_popup.is_some() {
        CYBERPUNK_STYLE_YELLOW_DARK
    } else {
        CYBERPUNK_STYLE_YELLOW
    };
    let cyberpunk_folder_widget = Paragraph::new(Text::raw(cyberpunk_folder_string))
        .block(Block::default().borders(Borders::ALL).title("Cyberpunk Folder"))
        .style(cyberpunk_folder_widget_style)
        .wrap(Wrap { trim: true });

    // Create a list of ListItems from the list of files
    let items: Vec<ListItem> = app
        .state.file_list
        .items
        .iter()
        .map(|(name, _size)| {
            ListItem::new(Text::from(name.clone()))
        })
        .collect();

    let item_list_style = if app.mod_popup.is_some() {
        CYBERPUNK_STYLE_CYAN_DARK
    } else {
        CYBERPUNK_STYLE_CYAN
    };

    // Create a List from all list items and highlight the currently selected one
    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available files"))
        .highlight_style(current_folder_widget_style)
        .highlight_symbol(">> ")
        .style(item_list_style);

    let log_widget = TuiLoggerWidget::default()
        .style_error(LOG_ERROR_STYLE)
        .style_debug(LOG_DEBUG_STYLE)
        .style_warn(LOG_WARN_STYLE)
        .style_trace(LOG_TRACE_STYLE)
        .style_info(LOG_INFO_STYLE)
        .block(
            Block::default()
                .title("Logs")
                .border_style(item_list_style)
                .borders(Borders::ALL),
        )
        .output_timestamp(None)
        .output_target(false)
        .output_level(None);
    
    f.render_widget(title_widget, main_chunks[0]);
    f.render_stateful_widget(items_list, chunks[0], file_list_state);
    f.render_widget(log_widget, chunks[1]);
    f.render_widget(current_folder_widget, main_chunks[2]);
    f.render_widget(cyberpunk_folder_widget, main_chunks[3]);
}

// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
    }

pub fn draw_mod_popup<B: Backend>(f: &mut Frame<B>, app: &App, mod_options_state: &mut ListState) {
    let clear_area = centered_rect(90, 90, f.size());
    let popup_area = centered_rect(80, 80, f.size());
    // clear the popup area
    f.render_widget(Clear, clear_area);
    f.render_widget(Block::default()
        .borders(Borders::ALL)    
        .border_style(CYBERPUNK_STYLE_YELLOW)
        .title("What do you want to do?"), clear_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(75),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(popup_area);
        
    let mod_name = app.mod_popup.as_ref().unwrap().get_mod_name();
    let mod_name_widget = Paragraph::new(Text::raw(mod_name))
        .block(Block::default().borders(Borders::ALL).title("Mod Name"))
        .style(CYBERPUNK_STYLE_YELLOW)
        .wrap(Wrap { trim: true });

    let items: Vec<ListItem> = ModOptions::get_all_options()
        .iter()
        .map(|mod_option| {
            ListItem::new(Text::from(mod_option.to_string()))
        })
        .collect();
    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available files"))
        .highlight_style(CYBERPUNK_STYLE_PINK)
        .highlight_symbol(">> ")
        .style(CYBERPUNK_STYLE_CYAN);

    let mod_install_status_bool = app.mod_popup.as_ref().unwrap().get_mod_install_status();
    let mod_install_status = if mod_install_status_bool.is_none() {
        "Checking...".to_string()
    } else {
        if mod_install_status_bool.unwrap() {
            "Installed".to_string()
        } else {
            "Not Installed".to_string()
        }
    };
    let mod_install_status_widget = Paragraph::new(Text::raw(mod_install_status))
        .block(Block::default().borders(Borders::ALL).title("Mod Install Status"))
        .style(CYBERPUNK_STYLE_YELLOW)
        .wrap(Wrap { trim: true });

    f.render_widget(mod_name_widget, chunks[0]);
    f.render_stateful_widget(items_list, chunks[1], mod_options_state);
    f.render_widget(mod_install_status_widget, chunks[2]);
}