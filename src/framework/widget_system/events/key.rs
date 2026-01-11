use crate::args::Args;
use crate::render::{RenderTheme, SingleMessageRenderParams, message_to_plain_lines};
use crate::framework::widget_system::interaction::clipboard;
use crate::framework::widget_system::interaction::input::handle_key;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::logic::{build_label_suffixes, timer_text};
use crate::framework::widget_system::runtime_tick::{
    DisplayMessage, build_display_messages, select_visible_message,
};
use crate::framework::widget_system::interaction::selection::extract_selection;
use crate::framework::widget_system::runtime::state::Focus;
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
        if copy_chat_selection(tab_state, args, msg_width, theme) {
            return Ok(Some(false));
        }
    }
    Ok(Some(true))
}

fn is_ctrl_c(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('c')
}

fn copy_input_selection(app: &mut crate::framework::widget_system::runtime::state::App) -> bool {
    if app.focus != Focus::Input || !app.input.is_selecting() {
        return false;
    }
    app.input.copy();
    let text = app.input.yank_text();
    clipboard::set(&text);
    true
}

fn copy_chat_selection(
    tab_state: &mut TabState,
    args: &Args,
    msg_width: usize,
    theme: &RenderTheme,
) -> bool {
    let app = &mut tab_state.app;
    if app.focus != Focus::Chat {
        return false;
    }
    let Some(selection) = app.chat_selection else {
        return false;
    };
    let messages = build_display_messages(app, args);
    let Some((idx, msg)) = active_message(app, &messages) else {
        return false;
    };
    let label_suffixes = build_label_suffixes(app, &timer_text(app));
    let params = SingleMessageRenderParams {
        message: msg,
        message_index: idx,
        width: msg_width,
        theme,
        label_suffixes: &label_suffixes,
        streaming: app.pending_assistant == Some(idx),
        scroll: 0,
        height: u16::MAX,
    };
    let lines = message_to_plain_lines(params, &mut tab_state.render_cache);
    let text = extract_selection(&lines, selection);
    if !text.is_empty() {
        clipboard::set(&text);
    }
    true
}

fn active_message<'a>(
    app: &mut crate::framework::widget_system::runtime::state::App,
    messages: &'a [DisplayMessage],
) -> Option<(usize, &'a crate::types::Message)> {
    let idx = select_visible_message(app, messages)?;
    messages
        .iter()
        .find(|msg| msg.index == idx)
        .map(|msg| (idx, &msg.message))
}
