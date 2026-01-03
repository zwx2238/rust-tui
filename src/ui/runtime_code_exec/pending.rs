use crate::types::Message;
use crate::ui::state::{App, PendingCodeExec};

pub(super) fn take_pending_or_notify(app: &mut App) -> Option<PendingCodeExec> {
    let Some(pending) = app.pending_code_exec.take() else {
        push_no_pending(app);
        return None;
    };
    Some(pending)
}

pub(super) fn clone_pending_or_notify(app: &mut App) -> Option<PendingCodeExec> {
    let Some(pending) = app.pending_code_exec.clone() else {
        push_no_pending(app);
        return None;
    };
    Some(pending)
}

fn push_no_pending(app: &mut App) {
    let idx = app.messages.len();
    app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content: "没有待审批的代码执行请求。".to_string(),
        tool_call_id: None,
        tool_calls: None,
    });
    app.dirty_indices.push(idx);
}
