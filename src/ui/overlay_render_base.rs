use crate::ui::draw::redraw;
use crate::ui::render_context::RenderContext;
use std::error::Error;

pub(crate) fn render_chat_view(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw(crate::ui::draw::RedrawParams {
            terminal: ctx.terminal,
            app: &mut tab_state.app,
            theme: ctx.theme,
            text: ctx.text,
            total_lines: ctx.total_lines,
            tab_labels: ctx.tab_labels,
            active_tab_pos: ctx.active_tab_pos,
            categories: ctx.categories,
            active_category: ctx.active_category,
            startup_text: ctx.startup_text,
            input_height: ctx.input_height,
            header_note: ctx.header_note,
        })?;
    }
    Ok(())
}
