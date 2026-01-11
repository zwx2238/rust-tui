use crate::session::SessionLocation;
use crate::types::Message;
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::state::PendingCommand;

pub(crate) fn handle_session_command(
    pending: PendingCommand,
    tabs: &mut [TabState],
    active_tab: &mut usize,
    categories: &mut [String],
    active_category: &mut usize,
    session_location: &mut Option<SessionLocation>,
) -> bool {
    if let PendingCommand::SaveSession = pending {
        handle_save_session(
            tabs,
            active_tab,
            categories,
            active_category,
            session_location,
        );
        return true;
    }
    false
}

fn handle_save_session(
    tabs: &mut [TabState],
    active_tab: &usize,
    categories: &mut [String],
    active_category: &mut usize,
    session_location: &mut Option<SessionLocation>,
) {
    save_all_conversations(tabs);
    let open_conversations = crate::framework::widget_system::runtime::runtime_helpers::collect_open_conversations(tabs);
    let active_conv = tabs.get(*active_tab).map(|t| t.conversation_id.clone());
    let save_result = crate::session::save_session(
        categories,
        &open_conversations,
        active_conv.as_deref(),
        categories.get(*active_category).map(|s| s.as_str()),
        session_location.as_ref(),
    );
    if let Some(tab_state) = tabs.get_mut(*active_tab) {
        update_save_result(tab_state, save_result, session_location);
    }
}

fn save_all_conversations(tabs: &[TabState]) {
    for tab in tabs {
        let _ = crate::conversation::save_conversation(
            &crate::framework::widget_system::runtime::runtime_helpers::tab_to_conversation(tab),
        );
    }
}

fn update_save_result(
    tab_state: &mut TabState,
    save_result: Result<SessionLocation, Box<dyn std::error::Error>>,
    session_location: &mut Option<SessionLocation>,
) {
    match save_result {
        Ok(loc) => {
            *session_location = Some(loc.clone());
            push_assistant_message(tab_state, format!("已保存会话：{}", loc.display_hint()));
        }
        Err(e) => {
            push_assistant_message(tab_state, format!("保存失败：{e}"));
        }
    }
}

fn push_assistant_message(tab_state: &mut TabState, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content,
        tool_call_id: None,
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}
