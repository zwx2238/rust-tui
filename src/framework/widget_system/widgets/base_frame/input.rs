use crate::render::RenderTheme;
use crate::framework::widget_system::draw::layout::{PADDING_X, PADDING_Y};
use crate::framework::widget_system::draw::style::{base_style, focus_border_style};
use crate::framework::widget_system::events::{
    MouseEventParams, handle_key_event, handle_mouse_event, handle_paste_event,
};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::runtime::state::Focus;
use crate::framework::widget_system::BoxConstraints;
use crate::framework::widget_system::bindings::{bind_active_tab, bind_event};
use crate::framework::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use ratatui::layout::Size;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{Block, Borders};
use std::error::Error;
use tui_textarea::TextArea;

use super::helpers::point_in_rect;

pub(super) struct InputWidget;

impl Widget for InputWidget {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let rect = ratatui::layout::Rect::new(0, 0, bc.max.width, bc.max.height);
        let height = crate::framework::widget_system::layout::compute_input_height(
            rect,
            ctx.view,
            ctx.tabs,
            ctx.active_tab,
        );
        Ok(Size {
            width: bc.max.width,
            height,
        })
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Mouse(m) => handle_input_mouse(ctx, layout, update, rect, *m),
            crossterm::event::Event::Key(key) => handle_input_key(ctx, layout, *key),
            crossterm::event::Event::Paste(paste) => handle_input_paste(ctx, paste),
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let theme = frame.state.theme;
        if let Some(app) = frame.state.active_app_mut() {
            draw_input(
                frame.frame,
                InputDrawParams {
                    area: rect,
                    input: &mut app.input,
                    theme,
                    focused: app.focus == Focus::Input,
                    busy: app.busy,
                    model_key: &app.model_key,
                    prompt_key: &app.prompt_key,
                },
            );
        }
        Ok(())
    }
}

struct InputDrawParams<'a, 'b> {
    area: ratatui::layout::Rect,
    input: &'a mut TextArea<'b>,
    theme: &'a RenderTheme,
    focused: bool,
    busy: bool,
    model_key: &'a str,
    prompt_key: &'a str,
}

fn draw_input<'a, 'b>(f: &mut ratatui::Frame<'_>, params: InputDrawParams<'a, 'b>) {
    let style = base_style(params.theme);
    let border_style = focus_border_style(params.theme, params.focused);
    let status = build_status(
        params.input,
        params.busy,
        params.model_key,
        params.prompt_key,
    );
    let block = build_block(status, style, border_style);
    params.input.set_block(block);
    params.input.set_style(style);
    params
        .input
        .set_selection_style(Style::default().bg(Color::DarkGray));
    params
        .input
        .set_cursor_style(cursor_style(params.focused, params.busy));
    params
        .input
        .set_placeholder_text(placeholder_text(params.busy));
    params
        .input
        .set_placeholder_style(Style::default().fg(Color::DarkGray));
    f.render_widget(&*params.input, params.area);
}

fn build_status(input: &TextArea<'_>, busy: bool, model_key: &str, prompt_key: &str) -> String {
    let (line_idx, col) = input.cursor();
    let total_lines = input.lines().len().max(1);
    format!(
        "{} · 模型 {} · 角色 {} · 行 {}/{} 列 {}",
        if busy { "输入(禁用)" } else { "输入" },
        model_key,
        prompt_key,
        line_idx + 1,
        total_lines,
        col + 1
    )
}

fn build_block(status: String, style: Style, border_style: Style) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title_top(status)
        .title_top(Line::from("Enter 发送 · Ctrl+J 换行").right_aligned())
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style)
}

fn cursor_style(focused: bool, busy: bool) -> Style {
    if focused && !busy {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    }
}

fn placeholder_text(busy: bool) -> &'static str {
    if busy {
        "正在生成回复，输入已禁用"
    } else {
        "输入内容后按 Enter 发送"
    }
}

fn handle_input_mouse(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    if !point_in_rect(m.column, m.row, rect) {
        return Ok(EventResult::ignored());
    }
    let binding = bind_event(ctx, layout, update);
    let _ = handle_mouse_event(MouseEventParams {
        m,
        tabs: binding.dispatch.tabs,
        active_tab: binding.dispatch.active_tab,
        categories: binding.dispatch.categories,
        active_category: binding.dispatch.active_category,
        tabs_area: binding.layout.tabs_area,
        msg_area: binding.layout.msg_area,
        input_area: binding.layout.input_area,
        category_area: binding.layout.category_area,
        msg_width: binding.dispatch.msg_width,
        view_height: binding.layout.view_height,
        total_lines: update.active_data.total_lines,
        theme: binding.dispatch.theme,
    });
    Ok(EventResult::handled())
}

fn handle_input_key(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    key: crossterm::event::KeyEvent,
) -> Result<EventResult, Box<dyn Error>> {
    if let Some(mut active) = bind_active_tab(ctx.tabs, *ctx.active_tab) {
        let app = active.app();
        if app.focus == Focus::Input && !app.busy {
            let handled = handle_key_event(
                key,
                ctx.tabs,
                *ctx.active_tab,
                ctx.args,
                layout.layout.msg_width,
                ctx.theme,
            )?;
            return Ok(if handled {
                EventResult::handled()
            } else {
                EventResult::ignored()
            });
        }
    }
    Ok(EventResult::ignored())
}

fn handle_input_paste(ctx: &mut EventCtx<'_>, paste: &str) -> Result<EventResult, Box<dyn Error>> {
    handle_paste_event(paste, ctx.tabs, *ctx.active_tab);
    Ok(EventResult::handled())
}
