#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::render::RenderTheme;
    use crate::types::Message;
    use crate::ui::events::{LlmEvent, UiEvent};
    use crate::ui::runtime_helpers::{PreheatResult, TabState};
    use crate::ui::runtime_tick::{
        ActiveFrameData, apply_preheat_results, build_exec_header_note,
        collect_stream_events_from_batch, preheat_inactive_tabs, prepare_active_frame,
        sync_code_exec_overlay, sync_file_patch_overlay, update_code_exec_results,
        update_tab_widths,
    };
    use crate::ui::runtime_view::ViewState;
    use crate::ui::state::{CodeExecLive, PendingCodeExec, RequestHandle};
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use std::sync::{Arc, Mutex, atomic::AtomicBool, mpsc};
    use std::time::Instant;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn stream_tab() -> TabState {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        tab.app.active_request = Some(RequestHandle {
            id: 1,
            cancel: Arc::new(AtomicBool::new(false)),
        });
        tab.app.busy = true;
        tab.app.pending_assistant = Some(0);
        tab.app.messages.push(Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: String::new(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab
    }

    fn send_stream_events(out: &mut Vec<UiEvent>) {
        out.push(UiEvent {
            tab: 0,
            request_id: 1,
            event: LlmEvent::Chunk("hi".to_string()),
        });
        out.push(UiEvent {
            tab: 0,
            request_id: 1,
            event: LlmEvent::Done { usage: None },
        });
    }

    #[test]
    fn drain_preheat_results_updates_cache() {
        let tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        let msg = Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "line".to_string(),
            tool_call_id: None,
            tool_calls: None,
        };
        let entry = crate::render::build_cache_entry(&msg, 20, &theme(), false);
        let mut preheat = vec![PreheatResult {
            tab: 0,
            idx: 0,
            entry,
        }];
        apply_preheat_results(&mut preheat, &mut [tab]);
    }

    #[test]
    fn collect_stream_events_returns_done_and_tools() {
        let tab = stream_tab();
        let mut llm_events = Vec::new();
        send_stream_events(&mut llm_events);
        let (_processed, done, tools) =
            collect_stream_events_from_batch(&mut llm_events, &mut [tab], &theme());
        assert_eq!(done, vec![0]);
        assert!(tools.is_empty());
    }

    #[test]
    fn update_tab_widths_sets_last_width() {
        let mut tabs = vec![TabState::new(
            "id".into(),
            "cat".into(),
            "",
            false,
            "m",
            "p",
        )];
        update_tab_widths(&mut tabs, 99);
    }

    #[test]
    fn preheat_inactive_tabs_sends_tasks() {
        let mut tabs = vec![
            TabState::new("a".into(), "cat".into(), "", false, "m", "p"),
            TabState::new("b".into(), "cat".into(), "", false, "m", "p"),
        ];
        tabs[1].app.messages.push(Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tabs[1].app.dirty_indices = vec![0];
        let (tx, rx) = mpsc::channel();
        preheat_inactive_tabs(&mut tabs, 0, &theme(), 80, &tx);
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn sync_overlays_toggle() {
        let mut tabs = vec![TabState::new(
            "id".into(),
            "cat".into(),
            "",
            false,
            "m",
            "p",
        )];
        let mut view = ViewState::new();
        sync_code_exec_overlay(&mut tabs, 0, &mut view);
        assert!(view.overlay.is_chat());
        tabs[0].app.pending_code_exec = Some(pending_code_exec());
        sync_code_exec_overlay(&mut tabs, 0, &mut view);
        assert!(view.overlay.is(crate::ui::overlay::OverlayKind::CodeExec));
        tabs[0].app.pending_file_patch = Some(pending_file_patch());
        sync_file_patch_overlay(&mut tabs, 0, &mut view);
        assert!(view.overlay.is(crate::ui::overlay::OverlayKind::CodeExec));
    }

    fn pending_code_exec() -> PendingCodeExec {
        PendingCodeExec {
            call_id: "c".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: Instant::now(),
            stop_reason: None,
        }
    }

    fn pending_file_patch() -> crate::ui::state::PendingFilePatch {
        crate::ui::state::PendingFilePatch {
            call_id: "f".to_string(),
            path: None,
            diff: String::new(),
            preview: String::new(),
        }
    }

    #[test]
    fn update_code_exec_results_sets_finished_output() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        tab.app.pending_code_exec = Some(PendingCodeExec {
            call_id: "c".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: Instant::now(),
            stop_reason: None,
        });
        let live = Arc::new(Mutex::new(CodeExecLive {
            started_at: Instant::now(),
            finished_at: Some(Instant::now()),
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            done: true,
        }));
        tab.app.code_exec_live = Some(live);
        update_code_exec_results(&mut [tab]);
    }

    #[test]
    fn prepare_active_frame_outputs_text() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        tab.app.messages.push(Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hello".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let args = default_args();
        let ActiveFrameData {
            text, total_lines, ..
        } = prepare_active_frame(
            &mut tab,
            &args,
            &theme(),
            40,
            10,
            Rect::new(0, 0, 40, 3),
            None,
        );
        assert!(total_lines >= 1);
        assert!(!text.lines.is_empty());
    }

    fn default_args() -> Args {
        Args {
            model: None,
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            resume: None,
            replay_fork_last: false,
            enable: None,
            log_requests: None,
            perf: false,
            question_set: None,
            workspace: "/tmp/deepchat-workspace".to_string(),
            yolo: false,
            read_only: false,
            wait_gdb: false,
        }
    }

    #[test]
    fn build_exec_header_note_lists_tabs() {
        let mut tabs = vec![TabState::new(
            "id".into(),
            "cat".into(),
            "",
            false,
            "m",
            "p",
        )];
        tabs[0].app.pending_code_exec = Some(PendingCodeExec {
            call_id: "c".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: Instant::now(),
            stop_reason: None,
        });
        let note = build_exec_header_note(&tabs, &["cat".to_string()]);
        assert!(note.is_some());
    }
}
