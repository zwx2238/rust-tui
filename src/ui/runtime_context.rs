use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};

pub(crate) struct DispatchContextParams<'a> {
    pub tabs: &'a mut Vec<crate::ui::runtime_helpers::TabState>,
    pub active_tab: &'a mut usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub msg_width: usize,
    pub theme: &'a RenderTheme,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
}

pub(crate) fn make_dispatch_context<'a>(params: DispatchContextParams<'a>) -> DispatchContext<'a> {
    DispatchContext {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        msg_width: params.msg_width,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    }
}

pub(crate) fn make_layout_context(
    size: ratatui::layout::Rect,
    tabs_area: ratatui::layout::Rect,
    msg_area: ratatui::layout::Rect,
    input_area: ratatui::layout::Rect,
    category_area: ratatui::layout::Rect,
    view_height: u16,
    total_lines: usize,
) -> LayoutContext {
    LayoutContext {
        size,
        tabs_area,
        msg_area,
        input_area,
        category_area,
        view_height,
        total_lines,
    }
}
