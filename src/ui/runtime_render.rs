use crate::ui::jump::JumpRow;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::widget_system::WidgetSystem;
use std::error::Error;

pub(crate) fn render_view(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let mut system = WidgetSystem::new();
    system.render(ctx, view)
}
