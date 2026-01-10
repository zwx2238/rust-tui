use crate::args::Args;
use crate::render::{RenderTheme, messages_to_plain_lines};
use crate::ui::clipboard;
use crate::ui::input::handle_key;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_tick::build_display_messages;
use crate::ui::selection::extract_selection;
use crate::ui::state::Focus;
use crossterm::event::{KeyEvent, KeyModifiers};

pub(crate) fn handle_key_event(
    key: KeyEvent,
    tabs: &mut [TabState],
    active_tab: usize,
    args: &Args,
    msg_width: usize,
    theme: &RenderTheme,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(result) = handle_ctrl_c(key, tabs, active_tab, args, msg_width, theme)? {
        return Ok(result);
    }
    if let Some(tab_state) = tabs.get_mut(active_tab)
        && handle_key(key, &mut tab_state.app)?
    {
        return Ok(true);
    }
    Ok(false)
}

fn handle_ctrl_c(
    key: KeyEvent,
    tabs: &mut [TabState],
    active_tab: usize,
    args: &Args,
    msg_width: usize,
    theme: &RenderTheme,
) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    if !is_ctrl_c(key) {
        return Ok(None);
    }
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        if copy_input_selection(&mut tab_state.app) {
            return Ok(Some(false));
        }
        if copy_chat_selection(&mut tab_state.app, args, msg_width, theme) {
            return Ok(Some(false));
        }
    }
    Ok(Some(true))
}

fn is_ctrl_c(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('c')
}

fn copy_input_selection(app: &mut crate::ui::state::App) -> bool {
    if app.focus != Focus::Input || !app.input.is_selecting() {
        return false;
    }
    app.input.copy();
    let text = app.input.yank_text();
    clipboard::set(&text);
    true
}

fn copy_chat_selection(
    app: &mut crate::ui::state::App,
    args: &Args,
    msg_width: usize,
    theme: &RenderTheme,
) -> bool {
    if app.focus != Focus::Chat {
        return false;
    }
    let Some(selection) = app.chat_selection else {
        return false;
    };
    let messages = build_display_messages(app, args);
    let lines = messages_to_plain_lines(&messages, msg_width, theme);
    let text = extract_selection(&lines, selection);
    if !text.is_empty() {
        clipboard::set(&text);
    }
    true
}
