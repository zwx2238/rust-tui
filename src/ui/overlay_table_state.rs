use crate::ui::model_popup::model_popup_area;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_table::{row_at, visible_rows};
use crate::ui::prompt_popup::prompt_popup_area;
use crate::ui::runtime_view::ViewState;
use crate::ui::scroll::max_scroll;
use crate::ui::selection_state::SelectionState;
use crate::ui::shortcut_help::help_popup_area;
use ratatui::layout::Rect;

#[derive(Copy, Clone)]
pub(crate) struct OverlayAreas {
    pub(crate) full: Rect,
    pub(crate) msg: Rect,
}

#[derive(Copy, Clone)]
pub(crate) struct OverlayRowCounts {
    pub(crate) tabs: usize,
    pub(crate) jump: usize,
    pub(crate) models: usize,
    pub(crate) prompts: usize,
    pub(crate) help: usize,
}

#[derive(Copy, Clone)]
pub(crate) struct OverlayTableMetrics {
    pub(crate) area: Rect,
    pub(crate) rows: usize,
}

pub(crate) struct OverlayTableHandle<'a> {
    metrics: OverlayTableMetrics,
    selection: &'a mut SelectionState,
}

impl<'a> OverlayTableHandle<'a> {
    pub(crate) fn row_at(&self, mouse_x: u16, mouse_y: u16) -> Option<usize> {
        row_at(
            self.metrics.area,
            self.metrics.rows,
            self.selection.scroll,
            mouse_x,
            mouse_y,
        )
    }

    pub(crate) fn visible_rows(&self) -> usize {
        visible_rows(self.metrics.area)
    }

    pub(crate) fn clamp(&mut self) {
        let viewport = self.visible_rows();
        self.selection
            .clamp_with_viewport(self.metrics.rows, viewport);
    }

    pub(crate) fn scroll_by(&mut self, delta: i32) {
        let viewport = self.visible_rows();
        let max_scroll = max_scroll(self.metrics.rows, viewport);
        self.selection.scroll_by(delta, max_scroll, viewport);
    }
}

pub(crate) fn overlay_table_metrics(
    kind: OverlayKind,
    areas: OverlayAreas,
    counts: OverlayRowCounts,
) -> OverlayTableMetrics {
    match kind {
        OverlayKind::Summary => summary_metrics(areas, counts),
        OverlayKind::Jump => jump_metrics(areas, counts),
        OverlayKind::Model => model_metrics(areas, counts),
        OverlayKind::Prompt => prompt_metrics(areas, counts),
        OverlayKind::CodeExec | OverlayKind::FilePatch => empty_metrics(areas),
        OverlayKind::Help => help_metrics(areas, counts),
    }
}

fn summary_metrics(areas: OverlayAreas, counts: OverlayRowCounts) -> OverlayTableMetrics {
    OverlayTableMetrics {
        area: areas.msg,
        rows: counts.tabs,
    }
}

fn jump_metrics(areas: OverlayAreas, counts: OverlayRowCounts) -> OverlayTableMetrics {
    OverlayTableMetrics {
        area: areas.msg,
        rows: counts.jump,
    }
}

fn model_metrics(areas: OverlayAreas, counts: OverlayRowCounts) -> OverlayTableMetrics {
    OverlayTableMetrics {
        area: model_popup_area(areas.full, counts.models),
        rows: counts.models,
    }
}

fn prompt_metrics(areas: OverlayAreas, counts: OverlayRowCounts) -> OverlayTableMetrics {
    OverlayTableMetrics {
        area: prompt_popup_area(areas.full, counts.prompts),
        rows: counts.prompts,
    }
}

fn help_metrics(areas: OverlayAreas, counts: OverlayRowCounts) -> OverlayTableMetrics {
    OverlayTableMetrics {
        area: help_popup_area(areas.full, counts.help),
        rows: counts.help,
    }
}

fn empty_metrics(areas: OverlayAreas) -> OverlayTableMetrics {
    OverlayTableMetrics {
        area: areas.msg,
        rows: 0,
    }
}

pub(crate) fn overlay_visible_rows(
    kind: OverlayKind,
    areas: OverlayAreas,
    counts: OverlayRowCounts,
) -> usize {
    let metrics = overlay_table_metrics(kind, areas, counts);
    visible_rows(metrics.area)
}

pub(crate) fn with_active_table_handle<R>(
    view: &mut ViewState,
    areas: OverlayAreas,
    counts: OverlayRowCounts,
    f: impl FnOnce(OverlayTableHandle<'_>) -> R,
) -> Option<R> {
    let kind = view.overlay.active?;
    if matches!(kind, OverlayKind::CodeExec | OverlayKind::FilePatch) {
        return None;
    }
    let metrics = overlay_table_metrics(kind, areas, counts);
    let selection: &mut SelectionState = match kind {
        OverlayKind::Summary => &mut view.summary,
        OverlayKind::Jump => &mut view.jump,
        OverlayKind::Model => &mut view.model,
        OverlayKind::Prompt => &mut view.prompt,
        OverlayKind::Help => &mut view.help,
        OverlayKind::CodeExec | OverlayKind::FilePatch => &mut view.summary,
    };
    Some(f(OverlayTableHandle { metrics, selection }))
}
