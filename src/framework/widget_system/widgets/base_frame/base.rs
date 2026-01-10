use crate::framework::widget_system::draw::{inner_height, inner_width};
use crate::framework::widget_system::layout::LayoutInfo;
use crate::framework::widget_system::BoxConstraints;
use crate::framework::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use crate::framework::widget_system::widget_pod::WidgetPod;
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::widgets::jump::JumpRow;
use ratatui::layout::Size;
use std::error::Error;

use super::categories::CategoriesWidget;
use super::command_suggestions::CommandSuggestionsWidget;
use super::edit_button::EditButtonWidget;
use super::footer::FooterWidget;
use super::global_key::GlobalKeyWidget;
use super::header::HeaderWidget;
use super::input::InputWidget;
use super::message_history::MessageHistoryWidget;
use super::messages::MessagesWidget;
use super::tabs::TabsWidget;
use crate::framework::widget_system::widgets::layout::{Flex2, Flex3, FlexAxis, FlexParam, Stack2};

type MsgLayer = Stack2<MessagesWidget, EditButtonWidget>;
type MsgRow = Flex2<MsgLayer, MessageHistoryWidget>;
type Main = Flex3<TabsWidget, MsgRow, InputWidget>;
type Body = Flex2<CategoriesWidget, Main>;
type Root = Flex3<HeaderWidget, Body, FooterWidget>;

pub(crate) struct BaseFrameWidget {
    pub(super) global_keys: WidgetPod<GlobalKeyWidget>,
    pub(super) root: WidgetPod<Root>,
    pub(super) command_suggestions: WidgetPod<CommandSuggestionsWidget>,
}

impl BaseFrameWidget {
    pub(crate) fn new() -> Self {
        let msg_layer = MsgLayer::new(MessagesWidget, EditButtonWidget::new());
        let msg_row = MsgRow::new(
            FlexAxis::Horizontal,
            (msg_layer, FlexParam::Flex(1)),
            (MessageHistoryWidget, FlexParam::Intrinsic),
        );
        let main = Main::new(
            FlexAxis::Vertical,
            (TabsWidget, FlexParam::Fixed(1)),
            (msg_row, FlexParam::Flex(1)),
            (InputWidget, FlexParam::Intrinsic),
        );
        let body = Body::new(
            FlexAxis::Horizontal,
            (CategoriesWidget, FlexParam::Intrinsic),
            (main, FlexParam::Flex(1)),
        );
        let root = Root::new(
            FlexAxis::Vertical,
            (HeaderWidget, FlexParam::Fixed(1)),
            (body, FlexParam::Flex(1)),
            (FooterWidget, FlexParam::Fixed(1)),
        );
        Self {
            global_keys: WidgetPod::new(GlobalKeyWidget),
            root: WidgetPod::new(root),
            command_suggestions: WidgetPod::new(CommandSuggestionsWidget),
        }
    }
}

impl Widget for BaseFrameWidget {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let _ = self.root.measure(ctx, bc)?;
        Ok(bc.constrain(bc.max))
    }

    fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.global_keys.place(ctx, layout, rect)?;
        self.root.place(ctx, layout, rect)?;
        self.command_suggestions.place(ctx, layout, rect)?;
        layout.layout = build_layout_info(self.root.widget());
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.global_keys.update(ctx, layout, update)?;
        self.root.update(ctx, layout, update)?;
        self.command_suggestions.update(ctx, layout, update)?;
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
        self.root.render(frame, layout, update)?;
        self.command_suggestions.render(frame, layout, update)?;
        Ok(())
    }
}

fn build_layout_info(root: &Root) -> LayoutInfo {
    let header_area = root.a_rect();
    let footer_area = root.c_rect();
    let body = root.b_widget();
    let category_area = body.a_rect();
    let main = body.b_widget();
    let tabs_area = main.a_rect();
    let msg_row = main.b_widget();
    let msg_area = msg_row.a_rect();
    let input_area = main.c_rect();
    LayoutInfo {
        header_area,
        category_area,
        tabs_area,
        msg_area,
        input_area,
        footer_area,
        msg_width: inner_width(msg_area, 1),
        view_height: inner_height(msg_area, 0),
    }
}
