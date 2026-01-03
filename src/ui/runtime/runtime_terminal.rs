use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, IsTerminal};

pub(crate) fn ensure_tty_ready() -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("未检测到终端 TTY。请在真实终端运行，或在 CLion 运行/调试配置中勾选 \"Emulate terminal in output console\"。".into());
    }
    ensure_controlling_tty()?;
    Ok(())
}

pub(crate) fn setup_terminal()
-> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub(crate) fn teardown_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(unix)]
fn ensure_controlling_tty() -> Result<(), Box<dyn std::error::Error>> {
    if std::fs::File::open("/dev/tty").is_err() {
        return Err("未检测到控制终端 (无法打开 /dev/tty)。CLion Debug 可能未分配 TTY，请改用外部终端运行或启用 \"Run in terminal\"。".into());
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_controlling_tty() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
