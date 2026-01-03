use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::runtime_context::{make_dispatch_context, make_layout_context};
use crate::ui::runtime_dispatch::{handle_key_event_loop, handle_mouse_event_loop};
use crate::ui::runtime_events::handle_paste_event;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use crossterm::event::{self, Event};
use ratatui::layout::Rect;
use std::time::Duration;

pub(crate) struct DispatchContextParams<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) msg_width: usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
}

pub(crate) struct LayoutContextParams {
    pub(crate) size: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) msg_area: Rect,
    pub(crate) input_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) view_height: u16,
    pub(crate) total_lines: usize,
}

pub(crate) fn poll_and_dispatch_event(
    mpsc_ctx: &mut DispatchContextParams<'_>,
    layout: LayoutContextParams,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(false);
    }
    match event::read()? {
        Event::Key(key) => handle_key_dispatch(key, mpsc_ctx, layout, view, jump_rows),
        Event::Paste(paste) => handle_paste_dispatch(paste, mpsc_ctx, view),
        Event::Mouse(m) => handle_mouse_dispatch(m, mpsc_ctx, layout, view, jump_rows),
        _ => Ok(false),
    }
}

fn build_dispatch_context<'a>(
    params: &'a mut DispatchContextParams<'_>,
) -> crate::ui::runtime_dispatch::DispatchContext<'a> {
    make_dispatch_context(crate::ui::runtime_context::DispatchContextParams {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        msg_width: params.msg_width,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    })
}

fn build_layout_context(params: LayoutContextParams) -> crate::ui::runtime_dispatch::LayoutContext {
    make_layout_context(
        params.size,
        params.tabs_area,
        params.msg_area,
        params.input_area,
        params.category_area,
        params.view_height,
        params.total_lines,
    )
}

fn handle_key_dispatch(
    key: crossterm::event::KeyEvent,
    params: &mut DispatchContextParams<'_>,
    layout: LayoutContextParams,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut ctx = build_dispatch_context(params);
    let layout_ctx = build_layout_context(layout);
    handle_key_event_loop(key, &mut ctx, layout_ctx, view, jump_rows)
}

fn handle_paste_dispatch(
    paste: String,
    params: &mut DispatchContextParams<'_>,
    view: &ViewState,
) -> Result<bool, Box<dyn std::error::Error>> {
    if view.is_chat() {
        handle_paste_event(&paste, params.tabs, *params.active_tab);
    }
    Ok(false)
}

fn handle_mouse_dispatch(
    m: crossterm::event::MouseEvent,
    params: &mut DispatchContextParams<'_>,
    layout: LayoutContextParams,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut ctx = build_dispatch_context(params);
    let layout_ctx = build_layout_context(layout);
    handle_mouse_event_loop(m, &mut ctx, layout_ctx, view, jump_rows);
    Ok(false)
}
