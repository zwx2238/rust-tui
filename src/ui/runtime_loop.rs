use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_loop_steps::{
    DispatchContextParams, FrameLayout, LayoutContextParams, active_frame_data, frame_layout,
    handle_pending_command_if_any, handle_pending_line, header_note, note_elapsed,
    poll_and_dispatch_event, prepare_categories, process_stream_updates, tab_labels_and_pos,
};
use crate::ui::runtime_render::render_view;
use crate::ui::runtime_tick::drain_preheat_results;
use crate::ui::runtime_view::ViewState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

#[derive(Copy, Clone, Eq, PartialEq)]
enum LoopControl {
    Continue,
    Break,
}

struct IterationData {
    frame: FrameLayout,
    tab_labels: Vec<String>,
    active_tab_pos: usize,
    active_data: crate::ui::runtime_tick::ActiveFrameData,
    jump_rows: Vec<crate::ui::jump::JumpRow>,
}

pub(crate) fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, tabs: &mut Vec<TabState>,
    active_tab: &mut usize, categories: &mut Vec<String>, active_category: &mut usize,
    session_location: &mut Option<crate::session::SessionLocation>, rx: &mpsc::Receiver<UiEvent>,
    tx: &mpsc::Sender<UiEvent>, preheat_tx: &mpsc::Sender<PreheatTask>,
    preheat_res_rx: &mpsc::Receiver<PreheatResult>, registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry, args: &Args, theme: &RenderTheme,
    start_time: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut startup_elapsed = None; let mut view = ViewState::new();
    loop {
        if run_loop_iteration(
            terminal, tabs, active_tab, categories, active_category, session_location, rx, tx,
            preheat_tx, preheat_res_rx, registry, prompt_registry, args, theme, start_time,
            &mut startup_elapsed, &mut view,
        )? == LoopControl::Break { break; }
        #[cfg(test)]
        if std::env::var("DEEPCHAT_TEST_RUN_LOOP_ONCE").is_ok() { break; }
    }
    Ok(())
}

fn run_loop_iteration(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, tabs: &mut Vec<TabState>,
    active_tab: &mut usize, categories: &mut Vec<String>, active_category: &mut usize,
    session_location: &mut Option<crate::session::SessionLocation>, rx: &mpsc::Receiver<UiEvent>,
    tx: &mpsc::Sender<UiEvent>, preheat_tx: &mpsc::Sender<PreheatTask>,
    preheat_res_rx: &mpsc::Receiver<PreheatResult>, registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry, args: &Args, theme: &RenderTheme,
    start_time: Instant, startup_elapsed: &mut Option<std::time::Duration>, view: &mut ViewState,
) -> Result<LoopControl, Box<dyn std::error::Error>> {
    let data = build_iteration_data(
        terminal, tabs, active_tab, categories, active_category, rx, tx, preheat_tx,
        preheat_res_rx, registry, prompt_registry, args, theme, start_time, startup_elapsed, view,
    )?;
    handle_pending_line(
        data.active_data.pending_line.clone(),
        tabs,
        *active_tab,
        registry,
        args,
        tx,
    );
    handle_pending_command_if_any(
        data.active_data.pending_command, tabs, active_tab, categories, active_category,
        session_location, registry, prompt_registry, args, tx,
    );
    if dispatch_events(
        tabs, active_tab, categories, active_category, theme, registry, prompt_registry, args,
        &data, view,
    )? { return Ok(LoopControl::Break); }
    Ok(LoopControl::Continue)
}

fn build_iteration_data(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, tabs: &mut Vec<TabState>,
    active_tab: &mut usize, categories: &mut Vec<String>, active_category: &mut usize,
    rx: &mpsc::Receiver<UiEvent>, tx: &mpsc::Sender<UiEvent>,
    preheat_tx: &mpsc::Sender<PreheatTask>, preheat_res_rx: &mpsc::Receiver<PreheatResult>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry, args: &Args, theme: &RenderTheme,
    start_time: Instant, startup_elapsed: &mut Option<std::time::Duration>, view: &mut ViewState,
) -> Result<IterationData, Box<dyn std::error::Error>> {
    let frame = frame_layout(terminal, view, tabs, *active_tab, categories)?; drain_preheat_results(preheat_res_rx, tabs);
    let active_category_name = prepare_categories(tabs, *active_tab, categories, active_category);
    let (tab_labels, active_tab_pos) = tab_labels_and_pos(tabs, *active_tab, &active_category_name);
    process_stream_updates(rx, tabs, *active_tab, theme, frame.layout.msg_width, registry, args, tx, preheat_tx, view)?;
    let active_data = active_frame_data(tabs, *active_tab, theme, frame.layout.msg_width, frame.layout.view_height, frame.layout.input_area, *startup_elapsed);
    let header_note = header_note(tabs, categories);
    let jump_rows = render_frame(terminal, tabs, *active_tab, &tab_labels, active_tab_pos, categories, *active_category, theme, active_data.startup_text.as_deref(), frame.size, frame.layout.input_height, frame.layout.msg_area, frame.layout.tabs_area, frame.layout.category_area, frame.layout.header_area, frame.layout.footer_area, frame.layout.msg_width, &active_data.text, active_data.total_lines, header_note.as_deref(), registry, prompt_registry, view)?;
    note_elapsed(start_time, startup_elapsed); terminal.hide_cursor()?;
    Ok(IterationData {
        frame,
        tab_labels: tab_labels.to_vec(),
        active_tab_pos,
        active_data,
        jump_rows,
    })
}

fn dispatch_events(
    tabs: &mut Vec<TabState>, active_tab: &mut usize, categories: &mut Vec<String>,
    active_category: &mut usize, theme: &RenderTheme, registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry, args: &Args, data: &IterationData,
    view: &mut ViewState,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut dispatch_params = DispatchContextParams {
        tabs, active_tab, categories, active_category, msg_width: data.frame.layout.msg_width,
        theme, registry, prompt_registry, args,
    };
    let layout_params = LayoutContextParams {
        size: data.frame.size, tabs_area: data.frame.layout.tabs_area,
        msg_area: data.frame.layout.msg_area, input_area: data.frame.layout.input_area,
        category_area: data.frame.layout.category_area, view_height: data.frame.layout.view_height,
        total_lines: data.active_data.total_lines,
    };
    poll_and_dispatch_event(&mut dispatch_params, layout_params, view, &data.jump_rows)
}

fn render_frame(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, tabs: &mut Vec<TabState>,
    active_tab: usize, tab_labels: &[String], active_tab_pos: usize, categories: &Vec<String>,
    active_category: usize, theme: &RenderTheme, startup_text: Option<&str>,
    full_area: ratatui::layout::Rect, input_height: u16, msg_area: ratatui::layout::Rect,
    tabs_area: ratatui::layout::Rect, category_area: ratatui::layout::Rect,
    header_area: ratatui::layout::Rect, footer_area: ratatui::layout::Rect, msg_width: usize,
    text: &ratatui::text::Text<'_>, total_lines: usize, header_note: Option<&str>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry, view: &mut ViewState,
) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn std::error::Error>> {
    let mut render_ctx = RenderContext {
        terminal, tabs, active_tab, tab_labels, active_tab_pos, categories, active_category,
        theme, startup_text, full_area, input_height, msg_area, tabs_area, category_area,
        header_area, footer_area, msg_width, text, total_lines, header_note,
        models: &registry.models, prompts: &prompt_registry.prompts,
    };
    render_view(&mut render_ctx, view)
}
