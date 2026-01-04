use crate::ui::command_suggestions::{
    draw_command_suggestions, handle_command_suggestion_click,
};
use crate::ui::draw::{draw_categories, draw_footer, draw_header, draw_tabs};
use crate::ui::draw::{MessagesDrawParams, draw_messages};
use crate::ui::draw_input::{InputDrawParams, draw_input};
use crate::ui::runtime_dispatch::key_helpers::{
    handle_pre_key_actions, handle_view_action_flow, is_quit_key, resolve_view_action,
};
use crate::ui::runtime_events::handle_key_event;
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_events::handle_paste_event;
use crate::ui::runtime_events::handle_tab_category_click;
use crate::ui::draw::inner_area;
use crate::ui::draw::layout::{PADDING_X, PADDING_Y};
use crate::ui::state::Focus;
use crate::ui::widget_system::bindings::{bind_active_tab, bind_event};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::widget_system::widget_pod::WidgetPod;
use ratatui::style::{Color, Modifier, Style};
use std::error::Error;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::runtime_loop_steps::FrameLayout;
use super::button::ButtonWidget;

pub(crate) struct BaseFrameWidget {
    global_keys: WidgetPod<GlobalKeyWidget>,
    header: WidgetPod<HeaderWidget>,
    tabs: WidgetPod<TabsWidget>,
    categories: WidgetPod<CategoriesWidget>,
    messages: WidgetPod<MessagesWidget>,
    edit_button: WidgetPod<EditButtonWidget>,
    input: WidgetPod<InputWidget>,
    footer: WidgetPod<FooterWidget>,
    command_suggestions: WidgetPod<CommandSuggestionsWidget>,
}

impl BaseFrameWidget {
    pub(crate) fn new() -> Self {
        Self {
            global_keys: WidgetPod::new(GlobalKeyWidget),
            header: WidgetPod::new(HeaderWidget),
            tabs: WidgetPod::new(TabsWidget),
            categories: WidgetPod::new(CategoriesWidget),
            messages: WidgetPod::new(MessagesWidget),
            edit_button: WidgetPod::new(EditButtonWidget::new()),
            input: WidgetPod::new(InputWidget),
            footer: WidgetPod::new(FooterWidget),
            command_suggestions: WidgetPod::new(CommandSuggestionsWidget),
        }
    }

    fn handle_key(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        key: crossterm::event::KeyEvent,
    ) -> Result<EventResult, Box<dyn Error>> {
        let result = self
            .global_keys
            .event(ctx, &crossterm::event::Event::Key(key), layout, update, jump_rows)?;
        if result.handled || result.quit {
            return Ok(result);
        }
        if let Some(mut active) = bind_active_tab(ctx.tabs, *ctx.active_tab) {
            let app = active.app();
            if !app.busy {
                let handled = handle_key_event(
                    key,
                    ctx.tabs,
                    *ctx.active_tab,
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

    fn handle_paste(
        &mut self,
        ctx: &mut EventCtx<'_>,
        paste: &str,
    ) -> Result<EventResult, Box<dyn Error>> {
        handle_paste_event(paste, ctx.tabs, *ctx.active_tab);
        Ok(EventResult::handled())
    }

    fn handle_mouse(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        m: crossterm::event::MouseEvent,
    ) -> Result<EventResult, Box<dyn Error>> {
        if self
            .command_suggestions
            .event(ctx, &crossterm::event::Event::Mouse(m), layout, update, jump_rows)?
            .handled
        {
            return Ok(EventResult::handled());
        }
        if self.tabs.contains(m.column, m.row) {
            return self
                .tabs
                .event(ctx, &crossterm::event::Event::Mouse(m), layout, update, jump_rows);
        }
        if self.categories.contains(m.column, m.row) {
            return self
                .categories
                .event(ctx, &crossterm::event::Event::Mouse(m), layout, update, jump_rows);
        }
        if self.input.contains(m.column, m.row) {
            return self
                .input
                .event(ctx, &crossterm::event::Event::Mouse(m), layout, update, jump_rows);
        }
        if self
            .edit_button
            .event(ctx, &crossterm::event::Event::Mouse(m), layout, update, jump_rows)?
            .handled
        {
            return Ok(EventResult::handled());
        }
        if self.messages.contains(m.column, m.row)
            || scrollbar_hit(layout.layout.msg_area, m.column, m.row)
        {
            return self
                .messages
                .event(ctx, &crossterm::event::Event::Mouse(m), layout, update, jump_rows);
        }
        Ok(EventResult::ignored())
    }
}

impl Widget for BaseFrameWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        layout: &FrameLayout,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.global_keys.set_rect(rect);
        self.header.set_rect(layout.layout.header_area);
        self.tabs.set_rect(layout.layout.tabs_area);
        self.categories.set_rect(layout.layout.category_area);
        self.messages.set_rect(layout.layout.msg_area);
        self.edit_button.set_rect(layout.layout.msg_area);
        self.input.set_rect(layout.layout.input_area);
        self.footer.set_rect(layout.layout.footer_area);
        self.command_suggestions.set_rect(rect);
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.global_keys.update(ctx, layout, update)?;
        self.header.update(ctx, layout, update)?;
        self.tabs.update(ctx, layout, update)?;
        self.categories.update(ctx, layout, update)?;
        self.messages.update(ctx, layout, update)?;
        self.edit_button.update(ctx, layout, update)?;
        self.input.update(ctx, layout, update)?;
        self.command_suggestions.update(ctx, layout, update)?;
        self.footer.update(ctx, layout, update)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Key(key) => {
                self.handle_key(ctx, layout, update, jump_rows, *key)
            }
            crossterm::event::Event::Paste(paste) => self.handle_paste(ctx, paste),
            crossterm::event::Event::Mouse(m) => {
                self.handle_mouse(ctx, layout, update, jump_rows, *m)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.header.render(frame, layout, update)?;
        self.categories.render(frame, layout, update)?;
        self.tabs.render(frame, layout, update)?;
        self.messages.render(frame, layout, update)?;
        self.edit_button.render(frame, layout, update)?;
        self.input.render(frame, layout, update)?;
        self.command_suggestions.render(frame, layout, update)?;
        self.footer.render(frame, layout, update)?;
        Ok(())
    }
}

struct GlobalKeyWidget;

impl Widget for GlobalKeyWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
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
        jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Key(key) = event else {
            return Ok(EventResult::ignored());
        };
        if is_quit_key(*key) {
            return Ok(EventResult::quit());
        }
        let mut binding = crate::ui::widget_system::bindings::bind_event(ctx, layout, update);
        if handle_pre_key_actions(&mut binding.dispatch, binding.view, *key) {
            return Ok(EventResult::handled());
        }
        let action = resolve_view_action(&mut binding.dispatch, binding.view, *key, jump_rows);
        if handle_view_action_flow(
            &mut binding.dispatch,
            binding.layout,
            binding.view,
            jump_rows,
            action,
            *key,
        ) {
            return Ok(EventResult::handled());
        }
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        _frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

struct HeaderWidget;

impl Widget for HeaderWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _event: &crossterm::event::Event,
        _layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_header(
            frame.frame,
            rect,
            frame.state.theme,
            frame.state.header_note,
        );
        Ok(())
    }
}

struct TabsWidget;

impl Widget for TabsWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        if !point_in_rect(m.column, m.row, rect) {
            return Ok(EventResult::ignored());
        }
        let mut binding = bind_event(ctx, layout, update);
        let handled = handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
            mouse_x: m.column,
            mouse_y: m.row,
            tabs: binding.dispatch.tabs,
            active_tab: binding.dispatch.active_tab,
            categories: binding.dispatch.categories,
            active_category: binding.dispatch.active_category,
            tabs_area: binding.layout.tabs_area,
            category_area: binding.layout.category_area,
        });
        if handled {
            Ok(EventResult::handled())
        } else {
            Ok(EventResult::ignored())
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_tabs(
            frame.frame,
            rect,
            frame.state.tab_labels,
            frame.state.active_tab_pos,
            frame.state.theme,
            frame.state.startup_text,
        );
        Ok(())
    }
}

struct CategoriesWidget;

impl Widget for CategoriesWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        if !point_in_rect(m.column, m.row, rect) {
            return Ok(EventResult::ignored());
        }
        let mut binding = bind_event(ctx, layout, update);
        let handled = handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
            mouse_x: m.column,
            mouse_y: m.row,
            tabs: binding.dispatch.tabs,
            active_tab: binding.dispatch.active_tab,
            categories: binding.dispatch.categories,
            active_category: binding.dispatch.active_category,
            tabs_area: binding.layout.tabs_area,
            category_area: binding.layout.category_area,
        });
        if handled {
            Ok(EventResult::handled())
        } else {
            Ok(EventResult::ignored())
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_categories(
            frame.frame,
            rect,
            frame.state.categories,
            frame.state.active_category,
            frame.state.theme,
        );
        Ok(())
    }
}

struct MessagesWidget;

impl Widget for MessagesWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !point_in_rect(m.column, m.row, rect)
            && !scrollbar_hit(layout.layout.msg_area, m.column, m.row)
        {
            return Ok(EventResult::ignored());
        }
        let mut binding = bind_event(ctx, layout, update);
        let _ = handle_mouse_event(crate::ui::runtime_events::MouseEventParams {
            m: *m,
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

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(app) = frame.state.active_app() {
            draw_messages(
                frame.frame,
                MessagesDrawParams {
                    area: rect,
                    text: frame.state.text,
                    scroll: app.scroll,
                    theme: frame.state.theme,
                    focused: app.focus == Focus::Chat,
                    total_lines: frame.state.total_lines,
                    selection: app.chat_selection.clone(),
                },
            );
        }
        Ok(())
    }
}

struct EditButtonWidget {
    buttons: Vec<EditButtonEntry>,
}

struct EditButtonEntry {
    msg_index: usize,
    button: ButtonWidget,
}

impl EditButtonWidget {
    fn new() -> Self {
        Self { buttons: Vec::new() }
    }

    fn sync_buttons(
        &mut self,
        layouts: &[crate::render::MessageLayout],
        scroll: u16,
        msg_area: ratatui::layout::Rect,
    ) {
        let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
        let mut count = 0;
        for layout in layouts {
            let Some((start, end)) = layout.button_range else {
                continue;
            };
            let row = layout.label_line as i32 - scroll as i32;
            if row < 0 || row >= inner.height as i32 {
                continue;
            }
            let start_u = start as u16;
            if start_u >= inner.width {
                continue;
            }
            let end_u = (end as u16).min(inner.width);
            let width = end_u.saturating_sub(start_u);
            if width == 0 {
                continue;
            }
            let rect = ratatui::layout::Rect {
                x: inner.x + start_u,
                y: inner.y + row as u16,
                width,
                height: 1,
            };
            let entry = self.ensure_entry(count, layout.index);
            entry.msg_index = layout.index;
            entry.button.set_rect(rect);
            entry.button.set_label("[编辑]");
            entry.button.set_visible(true);
            entry.button.set_bordered(false);
            entry.button.set_style(edit_button_style());
            count += 1;
        }
        for entry in self.buttons.iter_mut().skip(count) {
            entry.button.set_visible(false);
        }
    }

    fn ensure_entry(&mut self, idx: usize, msg_index: usize) -> &mut EditButtonEntry {
        if self.buttons.len() <= idx {
            self.buttons.push(EditButtonEntry {
                msg_index,
                button: ButtonWidget::new("[编辑]"),
            });
        }
        &mut self.buttons[idx]
    }
}

impl Widget for EditButtonWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(tab_state) = ctx.tabs.get(*ctx.active_tab) {
            self.sync_buttons(
                &tab_state.app.message_layouts,
                tab_state.app.scroll,
                layout.layout.msg_area,
            );
        }
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        for entry in &mut self.buttons {
            if entry
                .button
                .event(ctx, event, layout, update, &[], layout.layout.msg_area)?
                .handled
            {
                let mut binding = bind_event(ctx, layout, update);
                let _ = crate::ui::runtime_dispatch::fork::fork_message_by_index(
                    &mut binding.dispatch,
                    entry.msg_index,
                );
                return Ok(EventResult::handled());
            }
        }
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        for entry in &mut self.buttons {
            let _ = entry
                .button
                .render(frame, layout, update, layout.layout.msg_area);
        }
        Ok(())
    }
}

fn edit_button_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

struct InputWidget;

impl Widget for InputWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
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
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Mouse(m) => {
                if !point_in_rect(m.column, m.row, rect) {
                    return Ok(EventResult::ignored());
                }
                let mut binding = bind_event(ctx, layout, update);
                let _ = handle_mouse_event(crate::ui::runtime_events::MouseEventParams {
                    m: *m,
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
            crossterm::event::Event::Key(key) => {
                if let Some(mut active) = bind_active_tab(ctx.tabs, *ctx.active_tab) {
                    let app = active.app();
                    if app.focus == Focus::Input && !app.busy {
                        let handled = handle_key_event(
                            *key,
                            ctx.tabs,
                            *ctx.active_tab,
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
            crossterm::event::Event::Paste(paste) => {
                handle_paste_event(paste, ctx.tabs, *ctx.active_tab);
                Ok(EventResult::handled())
            }
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

struct FooterWidget;

impl Widget for FooterWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
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
        _ctx: &mut EventCtx<'_>,
        _event: &crossterm::event::Event,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(app) = frame.state.active_app() {
            draw_footer(frame.frame, rect, frame.state.theme, app.nav_mode);
        }
        Ok(())
    }
}

struct CommandSuggestionsWidget;

impl Widget for CommandSuggestionsWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
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
        _update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            if handle_command_suggestion_click(
                &mut tab_state.app,
                layout.layout.msg_area,
                layout.layout.input_area,
                m.column,
                m.row,
            ) {
                return Ok(EventResult::handled());
            }
        }
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let msg_area = frame.state.msg_area;
        let input_area = frame.state.input_area;
        let theme = frame.state.theme;
        if let Some(app) = frame.state.active_app_mut() {
            draw_command_suggestions(
                frame.frame,
                msg_area,
                input_area,
                app,
                theme,
            );
        }
        Ok(())
    }
}

pub(crate) struct NoticeWidget;

impl Widget for NoticeWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
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
        _ctx: &mut EventCtx<'_>,
        _event: &crossterm::event::Event,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let full_area = frame.state.full_area;
        let theme = frame.state.theme;
        if let Some(app) = frame.state.active_app_mut() {
            crate::ui::notice::draw_notice(
                frame.frame,
                full_area,
                app,
                theme,
            );
        }
        Ok(())
    }
}

fn point_in_rect(column: u16, row: u16, rect: ratatui::layout::Rect) -> bool {
    column >= rect.x
        && column < rect.x + rect.width
        && row >= rect.y
        && row < rect.y + rect.height
}

fn scrollbar_hit(area: ratatui::layout::Rect, column: u16, row: u16) -> bool {
    let scroll = crate::ui::draw::scrollbar_area(area);
    point_in_rect(column, row, scroll)
}
