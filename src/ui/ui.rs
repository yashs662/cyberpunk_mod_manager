use std::path::PathBuf;
use tui::{
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    backend::Backend,
    Frame,
    text::{Spans, Span, Text},
    widgets::{Paragraph, Block, Borders, Wrap, ListItem, List, ListState}
};
use tui_logger::TuiLoggerWidget;

use crate::{
    constants::{MIN_TERM_WIDTH, MIN_TERM_HEIGHT, ERROR_TEXT_STYLE, DEFAULT_STYLE,
                APP_TITLE, INPUT_STYLE, FOCUS_STYLE, LIST_SELECT_STYLE, LOG_ERROR_STYLE,
                LOG_DEBUG_STYLE, LOG_WARN_STYLE, LOG_TRACE_STYLE, LOG_INFO_STYLE,
                MOD_FOLDER_INPUT_EMPTY_ERROR, NOT_A_DIRECTORY_ERROR, CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR,
                NOT_A_VALID_CYBERPUNK_FOLDER_ERROR
    },
    App, app::state::{Focus, AppStatus},
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

    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    let mut text = vec![Spans::from(Span::styled(msg, ERROR_TEXT_STYLE))];
    text.append(&mut vec![Spans::from(Span::raw("Resize the window to continue, or press 'q' to quit."))]);
    let body = Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    rect.render_widget(body, chunks[1]);
}

/// Draws the title bar
pub fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new(APP_TITLE)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(DEFAULT_STYLE)
        )
}

pub fn draw_select_folder<B: Backend>(f: &mut Frame<B>, app: &App) {

    let submit_style = if app.state.focus == Focus::Submit {
        FOCUS_STYLE
    } else {
        DEFAULT_STYLE
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

    let title = Paragraph::new(Text::styled("Select Folder, Press <i> to edit and <Enter> to submit", DEFAULT_STYLE))
        .block(Block::default().borders(Borders::ALL))
        .style(DEFAULT_STYLE)
        .wrap(Wrap { trim: true });

    let mod_folder_text = app.state.select_folder_form[0].clone();
    let mod_folder_input_style = if app.state.focus == Focus::ModFolderInput {
        if app.state.status == AppStatus::UserInput {
            INPUT_STYLE
        } else {
            FOCUS_STYLE
        }
    } else if mod_folder_text.contains(MOD_FOLDER_INPUT_EMPTY_ERROR) || mod_folder_text.contains(NOT_A_DIRECTORY_ERROR){
        ERROR_TEXT_STYLE
    } else {
        DEFAULT_STYLE
    };
    let mod_folder = Paragraph::new(Text::raw(mod_folder_text))
        .block(Block::default().borders(Borders::ALL).title("Mods Folder"))
        .style(mod_folder_input_style)
        .wrap(Wrap { trim: true });

    let cyberpunk_folder_text = app.state.select_folder_form[1].clone();
    let cyberpunk_folder_input_style = if app.state.focus == Focus::CyberpunkFolderInput {
        if app.state.status == AppStatus::UserInput {
            INPUT_STYLE
        } else {
            FOCUS_STYLE
        }
    } else if cyberpunk_folder_text.contains(CYBERPUNK_FOLDER_INPUT_EMPTY_ERROR)
        || cyberpunk_folder_text.contains(NOT_A_DIRECTORY_ERROR)
        || cyberpunk_folder_text.contains(NOT_A_VALID_CYBERPUNK_FOLDER_ERROR)
        {
        ERROR_TEXT_STYLE
    } else {
        DEFAULT_STYLE
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

    let title_widget = Paragraph::new(Text::styled("Cyberpunk Mod Manager", DEFAULT_STYLE))
        .block(Block::default().borders(Borders::ALL))
        .style(DEFAULT_STYLE)
        .wrap(Wrap { trim: true });
    
    let current_folder = app.selected_folder.clone().unwrap_or_else(|| PathBuf::new());
    // check if current folder is a directory if not set it to No folder selected
    let current_folder_string = if current_folder.is_dir() {
        current_folder.to_string_lossy().to_string()
    } else {
        "No folder selected".to_string()
    };
    let current_folder_widget = Paragraph::new(Text::raw(current_folder_string))
        .block(Block::default().borders(Borders::ALL).title("Current Folder"))
        .style(DEFAULT_STYLE)
        .wrap(Wrap { trim: true });

    let cyberpunk_folder = app.cyberpunk_folder.clone().unwrap_or_else(|| PathBuf::new());
    // check if current folder is a directory if not set it to No folder selected
    let cyberpunk_folder_string = if cyberpunk_folder.is_dir() {
        cyberpunk_folder.to_string_lossy().to_string()
    } else {
        "No folder selected".to_string()
    };
    let cyberpunk_folder_widget = Paragraph::new(Text::raw(cyberpunk_folder_string))
        .block(Block::default().borders(Borders::ALL).title("Cyberpunk Folder"))
        .style(DEFAULT_STYLE)
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

    // Create a List from all list items and highlight the currently selected one
    let items_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available files"))
        .highlight_style(LIST_SELECT_STYLE)
        .highlight_symbol(">> ");

    let log_widget = TuiLoggerWidget::default()
        .style_error(LOG_ERROR_STYLE)
        .style_debug(LOG_DEBUG_STYLE)
        .style_warn(LOG_WARN_STYLE)
        .style_trace(LOG_TRACE_STYLE)
        .style_info(LOG_INFO_STYLE)
        .block(
            Block::default()
                .title("Logs")
                .border_style(DEFAULT_STYLE)
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