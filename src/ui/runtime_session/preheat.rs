use crate::ui::runtime_helpers::{PreheatResult, PreheatTask};
use crate::ui::events::{send_preheat, RuntimeEvent};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub(crate) fn spawn_preheat_workers(
    preheat_rx: mpsc::Receiver<PreheatTask>,
    tx: mpsc::Sender<RuntimeEvent>,
) {
    let workers = resolve_worker_count();
    let preheat_rx = Arc::new(Mutex::new(preheat_rx));
    for _ in 0..workers {
        spawn_preheat_worker(Arc::clone(&preheat_rx), tx.clone());
    }
}

fn resolve_worker_count() -> usize {
    std::env::var("PREHEAT_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| (n.get() / 2).max(1))
                .unwrap_or(1)
        })
}

fn spawn_preheat_worker(
    preheat_rx: Arc<Mutex<mpsc::Receiver<PreheatTask>>>,
    tx: mpsc::Sender<RuntimeEvent>,
) {
    std::thread::spawn(move || {
        loop {
            let Some(task) = recv_task(&preheat_rx) else {
                break;
            };
            let entry = crate::render::build_cache_entry(
                &task.msg,
                task.width,
                &task.theme,
                task.streaming,
            );
            send_preheat(&tx, PreheatResult {
                tab: task.tab,
                idx: task.idx,
                entry,
            });
        }
    });
}

fn recv_task(preheat_rx: &Arc<Mutex<mpsc::Receiver<PreheatTask>>>) -> Option<PreheatTask> {
    let guard = preheat_rx.lock().ok()?;
    guard.recv().ok()
}
