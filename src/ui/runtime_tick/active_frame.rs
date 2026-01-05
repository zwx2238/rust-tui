use crate::args::Args;
use crate::llm::prompt_manager::{augment_system, extract_system};
use crate::llm::templates::RigTemplates;
use crate::render::{
    RenderTheme, ViewportRenderParams, messages_to_viewport_text_cached,
    messages_to_viewport_text_cached_with_layout,
};
use crate::types::{Message, ROLE_SYSTEM};
use crate::ui::input_click::update_input_view_top;
use crate::ui::logic::{build_label_suffixes, timer_text};
use crate::ui::runtime_helpers::TabState;
use crate::ui::scroll::max_scroll_u16;
use crate::ui::state::PendingCommand;
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
    let (text, computed_total_lines) = update_text_and_scroll(
        tab_state,
        &render_messages,
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

fn update_text_and_scroll(
    tab_state: &mut TabState,
    messages: &[Message],
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> (Text<'static>, usize) {
    let prev_scroll = tab_state.app.scroll;
    let (mut text, total) =
        render_active_text(tab_state, messages, theme, msg_width, view_height, label_suffixes);
    update_scroll(&mut tab_state.app, total, view_height);
    if tab_state.app.scroll != prev_scroll {
        text = refresh_text_after_scroll(
            tab_state,
            messages,
            theme,
            msg_width,
            view_height,
            label_suffixes,
        );
    }
    (text, total)
}

fn take_pending(app: &mut crate::ui::state::App) -> (Option<String>, Option<PendingCommand>) {
    let pending_line = app.pending_send.take();
    let pending_command = app.pending_command.take();
    (pending_line, pending_command)
}

fn build_active_label_suffixes(app: &crate::ui::state::App) -> Vec<(usize, String)> {
    let timer = timer_text(app);
    build_label_suffixes(app, &timer)
}

fn render_active_text(
    tab_state: &mut TabState,
    messages: &[Message],
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> (Text<'static>, usize) {
    let app = &mut tab_state.app;
    let params = ViewportRenderParams {
        messages,
        width: msg_width,
        theme,
        label_suffixes,
        streaming_idx: app.pending_assistant,
        scroll: app.scroll,
        height: view_height,
    };
    let (text, total, layouts) =
        messages_to_viewport_text_cached_with_layout(params, &mut tab_state.render_cache);
    app.message_layouts = layouts;
    (text, total)
}

fn update_scroll(app: &mut crate::ui::state::App, total_lines: usize, view_height: u16) -> u16 {
    let max_scroll = max_scroll_u16(total_lines, view_height);
    if app.follow || app.scroll > max_scroll {
        app.scroll = max_scroll;
    }
    max_scroll
}

fn refresh_text_after_scroll(
    tab_state: &mut TabState,
    messages: &[Message],
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> Text<'static> {
    let app = &mut tab_state.app;
    let (text, _) = messages_to_viewport_text_cached(
        crate::render::ViewportRenderParams {
            messages,
            width: msg_width,
            theme,
            label_suffixes,
            streaming_idx: app.pending_assistant,
            scroll: app.scroll,
            height: view_height,
        },
        &mut tab_state.render_cache,
    );
    text
}

fn format_startup_text(startup_elapsed: Option<Duration>) -> Option<String> {
    startup_elapsed.map(|d| format!("启动耗时 {:.2}s", d.as_secs_f32()))
}

fn reset_active_cache(app: &mut crate::ui::state::App) {
    app.dirty_indices.clear();
    app.cache_shift = None;
}

pub fn build_display_messages(app: &crate::ui::state::App, args: &Args) -> Vec<Message> {
    if app.messages.is_empty() {
        return Vec::new();
    }
    let mut messages = app.messages.clone();
    if let Some(msg) = messages.first_mut()
        && msg.role == ROLE_SYSTEM
        && let Some(full) = build_full_prompt_for_display(&app.messages, &app.prompts_dir, args)
        && !full.trim().is_empty()
    {
        msg.content = full;
    }
    messages
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
