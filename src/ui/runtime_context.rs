use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};

pub(crate) fn make_dispatch_context<'a>(
    tabs: &'a mut Vec<crate::ui::runtime_helpers::TabState>,
    active_tab: &'a mut usize,
    msg_width: usize,
    theme: &'a RenderTheme,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    args: &'a Args,
) -> DispatchContext<'a> {
    DispatchContext {
        tabs,
        active_tab,
        msg_width,
        theme,
        registry,
        prompt_registry,
        args,
    }
}

pub(crate) fn make_layout_context(
    size: ratatui::layout::Rect,
    tabs_area: ratatui::layout::Rect,
    msg_area: ratatui::layout::Rect,
    input_area: ratatui::layout::Rect,
    view_height: u16,
    total_lines: usize,
) -> LayoutContext {
    LayoutContext {
        size,
        tabs_area,
        msg_area,
        input_area,
        view_height,
        total_lines,
    }
}
