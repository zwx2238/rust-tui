use crate::args::Args;
use crate::llm::prompt_manager::{augment_system, extract_system};
use crate::llm::templates::RigTemplates;
use crate::render::{
    RenderTheme, SingleMessageRenderParams, message_to_viewport_text_cached,
    message_to_viewport_text_cached_with_layout,
};
use crate::types::{Message, ROLE_SYSTEM};
use crate::framework::widget_system::interaction::input_click::update_input_view_top;
use crate::framework::widget_system::runtime::logic::{build_label_suffixes, timer_text};
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::interaction::scroll::max_scroll_u16;
use crate::framework::widget_system::runtime::state::{App, PendingCommand};
use ratatui::layout::Rect;
use ratatui::text::Text;
use std::time::Duration;

pub struct ActiveFrameData {
    pub text: Text<'static>,
    pub total_lines: usize,
    pub startup_text: Option<String>,
    pub pending_line: Option<String>,
    pub pending_command: Option<PendingCommand>,
}

#[derive(Clone)]
pub struct DisplayMessage {
    pub index: usize,
    pub message: Message,
}

#[derive(Copy, Clone)]
struct DetailMessage<'a> {
    index: usize,
    message: &'a Message,
}

pub fn prepare_active_frame(
    tab_state: &mut TabState,
    args: &Args,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    input_area: Rect,
    startup_elapsed: Option<Duration>,
) -> ActiveFrameData {
    let label_suffixes = build_active_label_suffixes(&tab_state.app);
    let render_messages = build_display_messages(&tab_state.app, args);
    let detail = resolve_detail_message(&mut tab_state.app, &render_messages);
    let (text, computed_total_lines) = update_detail_text_and_scroll(
        tab_state,
        detail,
        theme,
        msg_width,
        view_height,
        &label_suffixes,
    );
    finalize_active_frame(
        tab_state,
        input_area,
        startup_elapsed,
        text,
        computed_total_lines,
    )
}

fn finalize_active_frame(
    tab_state: &mut TabState,
    input_area: Rect,
    startup_elapsed: Option<Duration>,
    text: Text<'static>,
    total_lines: usize,
) -> ActiveFrameData {
    update_input_view_top(&mut tab_state.app, input_area);
    let startup_text = format_startup_text(startup_elapsed);
    let (pending_line, pending_command) = take_pending(&mut tab_state.app);
    reset_active_cache(&mut tab_state.app);
    ActiveFrameData {
        text,
        total_lines,
        startup_text,
        pending_line,
        pending_command,
    }
}

fn update_detail_text_and_scroll(
    tab_state: &mut TabState,
    detail: Option<DetailMessage<'_>>,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> (Text<'static>, usize) {
    let prev_scroll = tab_state.app.scroll;
    let (mut text, total) = render_detail_text(
        tab_state,
        detail,
        theme,
        msg_width,
        view_height,
        label_suffixes,
    );
    update_detail_scroll(&mut tab_state.app, total, view_height);
    if tab_state.app.scroll != prev_scroll {
        text = refresh_detail_text_after_scroll(
            tab_state,
            detail,
            theme,
            msg_width,
            view_height,
            label_suffixes,
        );
    }
    (text, total)
}

fn take_pending(app: &mut crate::framework::widget_system::runtime::state::App) -> (Option<String>, Option<PendingCommand>) {
    let pending_line = app.pending_send.take();
    let pending_command = app.pending_command.take();
    (pending_line, pending_command)
}

fn build_active_label_suffixes(app: &crate::framework::widget_system::runtime::state::App) -> Vec<(usize, String)> {
    let timer = timer_text(app);
    build_label_suffixes(app, &timer)
}

fn resolve_detail_message<'a>(
    app: &mut App,
    messages: &'a [DisplayMessage],
) -> Option<DetailMessage<'a>> {
    let prev_selected = app.message_history.selected;
    let index = select_visible_message(app, messages)?;
    if app.message_history.selected != prev_selected {
        app.chat_selection = None;
        app.chat_selecting = false;
    }
    find_display_message(messages, index)
        .map(|message| DetailMessage { index, message })
}

fn render_detail_text(
    tab_state: &mut TabState,
    detail: Option<DetailMessage<'_>>,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> (Text<'static>, usize) {
    let Some(detail) = detail else {
        tab_state.app.message_layouts.clear();
        return (Text::default(), 0);
    };
    let app = &mut tab_state.app;
    let params = SingleMessageRenderParams {
        message: detail.message,
        message_index: detail.index,
        width: msg_width,
        theme,
        label_suffixes,
        streaming: app.pending_assistant == Some(detail.index),
        scroll: app.scroll,
        height: view_height,
    };
    let (text, total, layouts) =
        message_to_viewport_text_cached_with_layout(params, &mut tab_state.render_cache);
    app.message_layouts = layouts;
    (text, total)
}

fn update_detail_scroll(app: &mut crate::framework::widget_system::runtime::state::App, total_lines: usize, view_height: u16) -> u16 {
    let max_scroll = max_scroll_u16(total_lines, view_height);
    if app.follow || app.scroll > max_scroll {
        app.scroll = max_scroll;
    }
    max_scroll
}

fn refresh_detail_text_after_scroll(
    tab_state: &mut TabState,
    detail: Option<DetailMessage<'_>>,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> Text<'static> {
    let Some(detail) = detail else {
        return Text::default();
    };
    let app = &mut tab_state.app;
    let params = SingleMessageRenderParams {
        message: detail.message,
        message_index: detail.index,
        width: msg_width,
        theme,
        label_suffixes,
        streaming: app.pending_assistant == Some(detail.index),
        scroll: app.scroll,
        height: view_height,
    };
    let (text, _) = message_to_viewport_text_cached(params, &mut tab_state.render_cache);
    text
}

fn format_startup_text(startup_elapsed: Option<Duration>) -> Option<String> {
    startup_elapsed.map(|d| format!("启动耗时 {:.2}s", d.as_secs_f32()))
}

fn reset_active_cache(app: &mut crate::framework::widget_system::runtime::state::App) {
    app.dirty_indices.clear();
    app.cache_shift = None;
}

pub fn build_display_messages(app: &App, args: &Args) -> Vec<DisplayMessage> {
    if app.messages.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (idx, msg) in app.messages.iter().enumerate() {
        if !args.show_system_prompt && msg.role == ROLE_SYSTEM {
            continue;
        }
        let mut message = msg.clone();
        if idx == 0
            && msg.role == ROLE_SYSTEM
            && let Some(full) = build_full_prompt_for_display(&app.messages, &app.prompts_dir, args)
            && !full.trim().is_empty()
        {
            message.content = full;
        }
        out.push(DisplayMessage { index: idx, message });
    }
    out
}

pub fn select_visible_message(app: &mut App, messages: &[DisplayMessage]) -> Option<usize> {
    if messages.is_empty() {
        app.message_history.selected = 0;
        return None;
    }
    let last = messages.last().map(|m| m.index).unwrap_or(0);
    let mut selected = app.message_history.selected;
    if app.follow {
        selected = last;
    } else if !has_display_index(messages, selected) {
        selected = next_display_index(messages, selected).unwrap_or(last);
    }
    app.message_history.selected = selected;
    Some(selected)
}

fn has_display_index(messages: &[DisplayMessage], index: usize) -> bool {
    messages.iter().any(|msg| msg.index == index)
}

fn next_display_index(messages: &[DisplayMessage], index: usize) -> Option<usize> {
    messages.iter().map(|msg| msg.index).find(|idx| *idx > index)
}

fn find_display_message(
    messages: &[DisplayMessage],
    index: usize,
) -> Option<&Message> {
    messages
        .iter()
        .find(|msg| msg.index == index)
        .map(|msg| &msg.message)
}

fn build_full_prompt_for_display(
    messages: &[Message],
    prompts_dir: &str,
    args: &Args,
) -> Option<String> {
    let templates = RigTemplates::load(prompts_dir).ok()?;
    let enabled = enabled_tool_names(args);
    let tools = templates.tool_defs().ok()?;
    let filtered = filter_tools_for_display(tools, &enabled);
    let base_system = augment_system(&extract_system(messages));
    if filtered.is_empty() {
        return Some(base_system);
    }
    templates.render_preamble(&base_system, &filtered).ok()
}

fn enabled_tool_names(args: &Args) -> Vec<&'static str> {
    let mut out = Vec::new();
    if args.web_search_enabled() {
        out.push("web_search");
    }
    if args.code_exec_enabled() {
        out.push("code_exec");
    }
    if args.read_file_enabled() {
        out.push("read_file");
        out.push("list_dir");
    }
    if args.read_code_enabled() {
        out.push("read_code");
    }
    if args.modify_file_enabled() {
        out.push("modify_file");
    }
    if args.ask_questions_enabled() {
        out.push("ask_questions");
    }
    out
}

fn filter_tools_for_display(
    tools: Vec<crate::llm::templates::ToolSchema>,
    enabled: &[&str],
) -> Vec<crate::llm::templates::ToolSchema> {
    if enabled.is_empty() {
        return Vec::new();
    }
    tools
        .into_iter()
        .filter(|tool| enabled.iter().any(|name| *name == tool.name))
        .collect()
}
