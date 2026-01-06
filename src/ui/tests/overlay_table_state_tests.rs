#[cfg(test)]
mod tests {
    use crate::ui::overlay::OverlayKind;
    use crate::ui::overlay_table_state::{
        OverlayAreas, OverlayRowCounts, overlay_table_metrics, overlay_visible_rows,
        with_active_table_handle,
    };
    use crate::ui::runtime_view::ViewState;
    use ratatui::layout::Rect;

    fn areas() -> OverlayAreas {
        OverlayAreas {
            full: Rect::new(0, 0, 80, 24),
            msg: Rect::new(0, 0, 60, 20),
        }
    }

    fn counts() -> OverlayRowCounts {
        OverlayRowCounts {
            tabs: 3,
            jump: 5,
            models: 2,
            prompts: 4,
            help: 6,
        }
    }

    #[test]
    fn metrics_match_kind() {
        let metrics = overlay_table_metrics(OverlayKind::Summary, areas(), counts());
        assert_eq!(metrics.rows, 3);
        let help = overlay_table_metrics(OverlayKind::Help, areas(), counts());
        assert_eq!(help.rows, 6);
    }

    #[test]
    fn visible_rows_nonzero() {
        let rows = overlay_visible_rows(OverlayKind::Summary, areas(), counts());
        assert!(rows >= 1);
    }

    #[test]
    fn with_active_table_handle_skips_code_exec() {
        let mut view = ViewState::new();
        view.overlay.open(OverlayKind::CodeExec);
        let res = with_active_table_handle(&mut view, areas(), counts(), |_| 1usize);
        assert!(res.is_none());
    }
}
