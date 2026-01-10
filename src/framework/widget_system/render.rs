use crate::framework::widget_system::widgets::jump::JumpRow;
use crate::framework::widget_system::runtime::runtime_loop_steps::{FrameLayout, note_elapsed};
use std::error::Error;

use super::context::{RenderCtx, RenderState, UpdateOutput, WidgetFrame};
use super::lifecycle::Widget;
use super::widgets::RootWidget;

pub(crate) fn render_root(
    ctx: &mut RenderCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    root: &mut RootWidget,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let mut jump_rows = Vec::new();
    let result = draw_root(ctx, layout, update, root, &mut jump_rows);
    note_elapsed(ctx.start_time, &mut *ctx.startup_elapsed);
    ctx.terminal.hide_cursor()?;
    result?;
    Ok(jump_rows)
}

fn draw_root(
    ctx: &mut RenderCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    root: &mut RootWidget,
    jump_rows: &mut Vec<JumpRow>,
) -> Result<(), Box<dyn Error>> {
    let terminal = &mut *ctx.terminal;
    let tabs = &mut *ctx.tabs;
    let active_tab = ctx.active_tab;
    let categories = ctx.categories;
    let active_category = ctx.active_category;
    let theme = ctx.theme;
    let registry = ctx.registry;
    let prompt_registry = ctx.prompt_registry;
    let view = &mut *ctx.view;
    let mut render_result: Result<(), Box<dyn Error>> = Ok(());
    terminal.draw(|f| {
        let mut render_state = RenderState {
            tabs,
            active_tab,
            tab_labels: &update.tab_labels,
            active_tab_pos: update.active_tab_pos,
            categories,
            active_category,
            theme,
            startup_text: update.active_data.startup_text.as_deref(),
            full_area: layout.size,
            msg_area: layout.layout.msg_area,
            tabs_area: layout.layout.tabs_area,
            category_area: layout.layout.category_area,
            header_area: layout.layout.header_area,
            footer_area: layout.layout.footer_area,
            input_area: layout.layout.input_area,
            msg_width: layout.layout.msg_width,
            text: &update.active_data.text,
            total_lines: update.active_data.total_lines,
            header_note: update.header_note.as_deref(),
            models: &registry.models,
            prompts: &prompt_registry.prompts,
        };
        let mut frame = WidgetFrame {
            frame: f,
            state: &mut render_state,
            view,
            jump_rows,
        };
        if let Err(err) = root.render(&mut frame, layout, update, layout.size) {
            render_result = Err(err.to_string().into());
        }
    })?;
    render_result
}
