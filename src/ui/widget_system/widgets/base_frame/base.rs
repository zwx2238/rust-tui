use crate::ui::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::widget_system::widget_pod::WidgetPod;
use crate::ui::{jump::JumpRow, runtime_loop_steps::FrameLayout};
use std::error::Error;

use super::categories::CategoriesWidget;
use super::command_suggestions::CommandSuggestionsWidget;
use super::edit_button::EditButtonWidget;
use super::footer::FooterWidget;
use super::global_key::GlobalKeyWidget;
use super::header::HeaderWidget;
use super::input::InputWidget;
use super::messages::MessagesWidget;
use super::tabs::TabsWidget;

pub(crate) struct BaseFrameWidget {
    pub(super) global_keys: WidgetPod<GlobalKeyWidget>,
    pub(super) header: WidgetPod<HeaderWidget>,
    pub(super) tabs: WidgetPod<TabsWidget>,
    pub(super) categories: WidgetPod<CategoriesWidget>,
    pub(super) messages: WidgetPod<MessagesWidget>,
    pub(super) edit_button: WidgetPod<EditButtonWidget>,
    pub(super) input: WidgetPod<InputWidget>,
    pub(super) footer: WidgetPod<FooterWidget>,
    pub(super) command_suggestions: WidgetPod<CommandSuggestionsWidget>,
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
        jump_rows: &[JumpRow],
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
