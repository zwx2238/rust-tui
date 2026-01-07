use crate::ui::events::RuntimeEvent;
use crate::ui::notice::push_notice;
use crate::ui::runtime_helpers::TabState;
use crossterm::event::KeyEvent;
use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};

use super::events::TerminalEvent;
use super::keys::key_event_to_bytes;

type PtyHandles = (
    Box<dyn portable_pty::MasterPty + Send>,
    Box<dyn Write + Send>,
    Box<dyn portable_pty::Child + Send>,
    Box<dyn Read + Send>,
);

pub(crate) struct TerminalConfig {
    pub(crate) scrollback: usize,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { scrollback: 2000 }
    }
}

pub(crate) struct TerminalSession {
    parser: vt100::Parser,
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send>,
    alive: Arc<AtomicBool>,
    reader_thread: Option<std::thread::JoinHandle<()>>,
    pub(crate) scroll_offset: u16,
    last_cols: u16,
    last_rows: u16,
}

impl TerminalSession {
    pub(crate) fn new(
        shell: &str,
        cols: u16,
        rows: u16,
        config: TerminalConfig,
        tx: &Sender<RuntimeEvent>,
        conversation_id: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (master, writer, child, reader) = open_pty(shell, cols, rows)?;
        let alive = Arc::new(AtomicBool::new(true));
        let reader_thread = spawn_reader_thread(
            reader,
            tx.clone(),
            conversation_id,
            Arc::clone(&alive),
        );
        Ok(Self {
            parser: vt100::Parser::new(rows, cols, config.scrollback),
            master,
            writer,
            child,
            alive,
            reader_thread: Some(reader_thread),
            scroll_offset: 0,
            last_cols: cols,
            last_rows: rows,
        })
    }

    pub(crate) fn apply_output(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub(crate) fn resize_if_needed(&mut self, cols: u16, rows: u16) {
        if cols == 0 || rows == 0 {
            return;
        }
        if self.last_cols == cols && self.last_rows == rows {
            return;
        }
        self.last_cols = cols;
        self.last_rows = rows;
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        self.parser.set_size(rows, cols);
    }

    pub(crate) fn send_key(&mut self, key: KeyEvent) {
        self.scroll_offset = 0;
        let Some(bytes) = key_event_to_bytes(key) else {
            return;
        };
        self.send_bytes(&bytes);
    }

    pub(crate) fn send_paste(&mut self, s: &str) {
        self.scroll_offset = 0;
        self.send_bytes(s.as_bytes());
    }

    pub(crate) fn send_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    pub(crate) fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(h) = self.reader_thread.take() {
            let _ = h.join();
        }
    }
}

pub(crate) fn ensure_terminal_for_active_tab(
    tabs: &mut [TabState],
    active_tab: usize,
    cols: u16,
    rows: u16,
    tx: &Sender<RuntimeEvent>,
) {
    let Some(tab) = tabs.get_mut(active_tab) else {
        return;
    };
    if tab.app.terminal.is_none() {
        create_terminal_for_tab(tab, cols, rows, tx);
    }
    if let Some(terminal) = tab.app.terminal.as_mut() {
        terminal.resize_if_needed(cols, rows);
    }
}

fn create_terminal_for_tab(tab: &mut TabState, cols: u16, rows: u16, tx: &Sender<RuntimeEvent>) {
    let shell = choose_shell();
    match TerminalSession::new(
        &shell,
        cols.max(2),
        rows.max(2),
        TerminalConfig::default(),
        tx,
        tab.conversation_id.clone(),
    ) {
        Ok(s) => tab.app.terminal = Some(s),
        Err(_) => push_notice(&mut tab.app, "无法启动弹窗终端：未找到可用的 shell（$SHELL 或 bash）。"),
    }
}

fn choose_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string())
}

fn open_pty(
    shell: &str,
    cols: u16,
    rows: u16,
) -> Result<PtyHandles, Box<dyn std::error::Error>> {
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut cmd = CommandBuilder::new(shell);
    cmd.arg("-i");
    let child = pair.slave.spawn_command(cmd)?;
    let master = pair.master;
    let writer = master.take_writer()?;
    let reader = master.try_clone_reader()?;
    Ok((master, writer, child, reader))
}

fn spawn_reader_thread(
    mut reader: Box<dyn Read + Send>,
    tx: Sender<RuntimeEvent>,
    conversation_id: String,
    alive: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || read_loop(&mut *reader, &tx, &conversation_id, &alive))
}

fn read_loop(
    reader: &mut dyn Read,
    tx: &Sender<RuntimeEvent>,
    conversation_id: &str,
    alive: &AtomicBool,
) {
    let mut buf = [0u8; 8192];
    while alive.load(Ordering::Relaxed) {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let _ = tx.send(RuntimeEvent::Terminal(TerminalEvent {
            conversation_id: conversation_id.to_string(),
            bytes: buf[..n].to_vec(),
        }));
    }
}

