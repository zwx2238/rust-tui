use crate::ui::events::RuntimeEvent;
use crossterm::event;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::time::Duration;

pub(crate) fn start_input_thread(
    tx: Sender<RuntimeEvent>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    std::thread::spawn(move || run_input_loop(tx, stop))
}

fn run_input_loop(tx: Sender<RuntimeEvent>, stop: Arc<AtomicBool>) {
    let poll_timeout = Duration::from_millis(50);
    loop {
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        if !poll_once(&tx, poll_timeout) {
            return;
        }
    }
}

fn poll_once(tx: &Sender<RuntimeEvent>, timeout: Duration) -> bool {
    let Ok(ready) = event::poll(timeout) else {
        return true;
    };
    if !ready {
        return true;
    }
    let Ok(ev) = event::read() else {
        return true;
    };
    tx.send(RuntimeEvent::Input(ev)).is_ok()
}
