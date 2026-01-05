use crate::args::Args;
use crate::llm::prompts::{PromptRegistry, SystemPrompt};
use crate::model_registry::{ModelProfile, ModelRegistry};
use crate::render::RenderTheme;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Color;

pub(crate) fn theme() -> RenderTheme {
    RenderTheme {
        bg: Color::Black,
        fg: Some(Color::White),
        code_bg: Color::Black,
        code_theme: "base16-ocean.dark",
        heading_fg: Some(Color::Cyan),
    }
}

pub(crate) fn registry() -> ModelRegistry {
    ModelRegistry {
        default_key: "m1".to_string(),
        models: vec![ModelProfile {
            key: "m1".to_string(),
            base_url: "http://example.com".to_string(),
            api_key: "k".to_string(),
            model: "model".to_string(),
        }],
    }
}

pub(crate) fn prompt_registry() -> PromptRegistry {
    PromptRegistry {
        default_key: "p1".to_string(),
        prompts: vec![SystemPrompt {
            key: "p1".to_string(),
            content: "sys1".to_string(),
        }],
    }
}

pub(crate) fn args() -> Args {
    Args {
        model: "m".to_string(),
        system: "sys".to_string(),
        base_url: "http://example.com".to_string(),
        show_reasoning: false,
        config: None,
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

pub(crate) fn layout() -> LayoutContext {
    LayoutContext {
        size: Rect::new(0, 0, 120, 50),
        tabs_area: Rect::new(0, 1, 120, 1),
        msg_area: Rect::new(0, 2, 120, 40),
        input_area: Rect::new(0, 42, 120, 5),
        category_area: Rect::new(0, 1, 10, 5),
        view_height: 20,
        total_lines: 0,
    }
}

pub(crate) struct CtxParams<'a> {
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: &'a mut usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub theme: &'a RenderTheme,
    pub registry: &'a ModelRegistry,
    pub prompt_registry: &'a PromptRegistry,
    pub args: &'a Args,
}

pub(crate) fn ctx<'a>(params: CtxParams<'a>) -> DispatchContext<'a> {
    DispatchContext {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        msg_width: 60,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    }
}

pub(crate) struct OverlayTestState {
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
    pub categories: Vec<String>,
    pub active_category: usize,
    pub theme: RenderTheme,
    pub registry: ModelRegistry,
    pub prompt_registry: PromptRegistry,
    pub args: Args,
    pub view: ViewState,
}

pub(crate) fn base_state(
    tab: TabState,
    overlay: crate::ui::overlay::OverlayKind,
) -> OverlayTestState {
    let theme = theme();
    let registry = registry();
    let prompt_registry = prompt_registry();
    let args = args();
    let mut view = ViewState::new();
    view.overlay.open(overlay);
    OverlayTestState {
        tabs: vec![tab],
        active_tab: 0,
        categories: vec!["默认".to_string()],
        active_category: 0,
        theme,
        registry,
        prompt_registry,
        args,
        view,
    }
}

pub(crate) fn ctx_and_view<'a>(
    state: &'a mut OverlayTestState,
) -> (DispatchContext<'a>, &'a mut ViewState) {
    let OverlayTestState {
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        args,
        view,
    } = state;
    let ctx = ctx(CtxParams {
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        args,
    });
    (ctx, view)
}

pub(crate) fn code_exec_tab(code: &str) -> TabState {
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    tab.app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
        call_id: "call".to_string(),
        language: "python".to_string(),
        code: code.to_string(),
        exec_code: None,
        requested_at: std::time::Instant::now(),
        stop_reason: None,
    });
    tab
}

pub(crate) fn code_exec_live(tab: &mut TabState) {
    tab.app.code_exec_live = Some(std::sync::Arc::new(std::sync::Mutex::new(
        crate::ui::state::CodeExecLive {
            started_at: std::time::Instant::now(),
            finished_at: None,
            stdout: "out\n".repeat(50),
            stderr: "err\n".repeat(50),
            exit_code: None,
            done: false,
        },
    )));
}

pub(crate) fn file_patch_tab() -> TabState {
    let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
    tab.app.pending_file_patch = Some(crate::ui::state::PendingFilePatch {
        call_id: "call".to_string(),
        path: None,
        diff: "diff --git a/a b/a\n".to_string(),
        preview: "preview".to_string(),
    });
    tab
}

pub(crate) fn mouse_move_at(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Moved,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

pub(crate) fn mouse_click_at(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

pub(crate) fn mouse_scroll_down_at(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}
