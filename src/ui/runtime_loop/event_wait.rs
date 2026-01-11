use crate::ui::events::{EventBatch, RuntimeEvent};
use crate::ui::runtime_helpers::{PreheatResult, TabState};
use crossterm::event::Event as CrosstermEvent;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(super) struct WaitOutcome {
    pub(super) ticked: bool,
    pub(super) disconnected: bool,
}

pub(super) fn wait_for_events(
    rx: &mpsc::Receiver<RuntimeEvent>,
    timeout: Option<Duration>,
    out: &mut EventBatch,
) -> WaitOutcome {
    out.clear();
    let Some(first) = recv_first(rx, timeout) else {
        return WaitOutcome {
            ticked: false,
            disconnected: true,
        };
    };
    match first {
        FirstEvent::Tick => WaitOutcome {
            ticked: true,
            disconnected: false,
        },
        FirstEvent::Event(ev) => {
            out.push(ev);
            drain_remaining(rx, out);
            WaitOutcome {
                ticked: false,
                disconnected: false,
            }
        }
    }
}

pub(super) fn compute_timeout(tabs: &[TabState], active_tab: usize) -> Option<Duration> {
    let tab = tabs.get(active_tab)?;
    if tab.app.busy {
        return Some(Duration::from_millis(100));
    }
    let idle = Duration::from_secs(1);
    match notice_timeout(&tab.app.notice) {
        Some(notice) => Some(notice.min(idle)),
        None => Some(idle),
    }
}

pub(super) fn input_batch_dirty(events: &[CrosstermEvent]) -> bool {
    events.iter().any(input_event_dirty)
}

pub(super) fn preheat_touches_active_tab(preheat: &[PreheatResult], active_tab: usize) -> bool {
    preheat.iter().any(|r| r.tab == active_tab)
}

enum FirstEvent {
    Tick,
    Event(RuntimeEvent),
}

fn recv_first(rx: &mpsc::Receiver<RuntimeEvent>, timeout: Option<Duration>) -> Option<FirstEvent> {
    match timeout {
        Some(dur) => match rx.recv_timeout(dur) {
            Ok(ev) => Some(FirstEvent::Event(ev)),
            Err(mpsc::RecvTimeoutError::Timeout) => Some(FirstEvent::Tick),
            Err(mpsc::RecvTimeoutError::Disconnected) => None,
        },
        None => rx.recv().ok().map(FirstEvent::Event),
    }
}

fn drain_remaining(rx: &mpsc::Receiver<RuntimeEvent>, out: &mut EventBatch) {
    while let Ok(ev) = rx.try_recv() {
        out.push(ev);
    }
}

fn notice_timeout(notice: &Option<crate::ui::state::Notice>) -> Option<Duration> {
    let notice = notice.as_ref()?;
    let dur = notice.expires_at.saturating_duration_since(Instant::now());
    Some(dur.min(Duration::from_secs(1)))
}

fn input_event_dirty(ev: &CrosstermEvent) -> bool {
    matches!(
        ev,
        CrosstermEvent::Key(_)
            | CrosstermEvent::Mouse(_)
            | CrosstermEvent::Resize(_, _)
            | CrosstermEvent::Paste(_)
    )
}
