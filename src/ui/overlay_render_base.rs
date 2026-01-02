use crate::ui::draw::redraw;
use crate::ui::render_context::RenderContext;
use std::error::Error;

pub(crate) fn render_chat_view(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            ctx.tab_labels,
            ctx.active_tab_pos,
            ctx.categories,
            ctx.active_category,
            ctx.startup_text,
            ctx.input_height,
            ctx.header_note,
        )?;
    }
    Ok(())
}
