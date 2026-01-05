use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use crate::ui::state::App;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
use crate::ui::runtime_loop_steps::FrameLayout;

use super::context::{EventCtx, UpdateOutput};

pub(crate) struct ActiveTabBinding<'a> {
    tab: &'a mut TabState,
}

impl<'a> ActiveTabBinding<'a> {
    pub(crate) fn app(&mut self) -> &mut App {
        &mut self.tab.app
    }

    pub(crate) fn tab(&mut self) -> &mut TabState {
        self.tab
    }
}

pub(crate) fn bind_active_tab<'a>(
    tabs: &'a mut [TabState],
    active_tab: usize,
) -> Option<ActiveTabBinding<'a>> {
    tabs.get_mut(active_tab).map(|tab| ActiveTabBinding { tab })
}

pub(crate) struct EventBinding<'a> {
    pub(crate) dispatch: DispatchContext<'a>,
    pub(crate) layout: LayoutContext,
    pub(crate) view: &'a mut ViewState,
}

pub(crate) fn bind_event<'a>(
    ctx: &'a mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
) -> EventBinding<'a> {
    let EventCtx {
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        args,
        view,
        ..
    } = ctx;
    let dispatch = DispatchContext {
        tabs,
        active_tab,
        categories,
        active_category,
        msg_width: layout.layout.msg_width,
        theme,
        registry,
        prompt_registry,
        args,
    };
    let layout_ctx = build_layout(layout, update);
    EventBinding {
        dispatch,
        layout: layout_ctx,
        view,
    }
}

fn build_layout(layout: &FrameLayout, update: &UpdateOutput) -> LayoutContext {
    LayoutContext {
        size: layout.size,
        tabs_area: layout.layout.tabs_area,
        msg_area: layout.layout.msg_area,
        input_area: layout.layout.input_area,
        category_area: layout.layout.category_area,
        view_height: layout.layout.view_height,
        total_lines: update.active_data.total_lines,
    }
}
