use crate::ui::runtime_helpers::TabState;

#[derive(Clone, Debug)]
pub(crate) struct TerminalEvent {
    pub(crate) conversation_id: String,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) fn apply_terminal_events(events: &mut Vec<TerminalEvent>, tabs: &mut [TabState]) {
    for ev in events.drain(..) {
        apply_one(&ev, tabs);
    }
}

fn apply_one(ev: &TerminalEvent, tabs: &mut [TabState]) {
    let Some(tab) = tabs
        .iter_mut()
        .find(|t| t.conversation_id == ev.conversation_id)
    else {
        return;
    };
    let Some(terminal) = tab.app.terminal.as_mut() else {
        return;
    };
    terminal.apply_output(&ev.bytes);
}
