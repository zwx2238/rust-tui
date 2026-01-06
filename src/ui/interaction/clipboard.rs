use arboard::Clipboard;
use base64::Engine as _;
use std::io::Write;
use std::process::{Command, Stdio};

const OSC52_MAX_BYTES: usize = 100_000;

pub fn set(text: &str) {
    if set_arboard(text) {
        return;
    }
    if set_wsl(text) {
        return;
    }
    let _ = set_osc52(text);
}

pub fn get() -> Option<String> {
    if let Ok(mut clipboard) = Clipboard::new()
        && let Ok(text) = clipboard.get_text()
    {
        return Some(normalize(text));
    }
    if is_wsl()
        && let Ok(output) = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", "Get-Clipboard -Raw"])
            .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        return Some(normalize(text));
    }
    None
}

fn set_arboard(text: &str) -> bool {
    let Ok(mut clipboard) = Clipboard::new() else {
        return false;
    };
    clipboard.set_text(text.to_string()).is_ok()
}

fn set_wsl(text: &str) -> bool {
    if !is_wsl() {
        return false;
    }
    if write_clip_utf16le(text) {
        return true;
    }
    set_wsl_powershell(text)
}

fn set_wsl_powershell(text: &str) -> bool {
    let mut child = match Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "Set-Clipboard -Value ([Console]::In.ReadToEnd())",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    let Some(mut stdin) = child.stdin.take() else {
        return false;
    };
    if stdin.write_all(text.as_bytes()).is_err() {
        return false;
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

fn set_osc52(text: &str) -> bool {
    let text = truncate_utf8(text, OSC52_MAX_BYTES);
    let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());

    let mut osc = Vec::with_capacity(16 + b64.len());
    osc.extend_from_slice(b"\x1b]52;c;");
    osc.extend_from_slice(b64.as_bytes());
    osc.push(0x07);

    let out = if in_tmux() { tmux_wrap(&osc) } else { osc };
    write_stdout(&out)
}

fn truncate_utf8(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    &text[..end]
}

fn in_tmux() -> bool {
    std::env::var_os("TMUX").is_some()
}

fn tmux_wrap(seq: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(seq.len().saturating_mul(2));
    for &b in seq {
        if b == 0x1b {
            escaped.push(0x1b);
            escaped.push(0x1b);
        } else {
            escaped.push(b);
        }
    }
    let mut out = Vec::with_capacity(8 + escaped.len() + 2);
    out.extend_from_slice(b"\x1bPtmux;");
    out.extend_from_slice(&escaped);
    out.extend_from_slice(b"\x1b\\");
    out
}

fn write_stdout(bytes: &[u8]) -> bool {
    let mut out = std::io::stdout();
    out.write_all(bytes).and_then(|_| out.flush()).is_ok()
}

fn normalize(text: String) -> String {
    let text = text.replace("\r\n", "\n");
    text.replace('\r', "\n")
}

fn is_wsl() -> bool {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        return true;
    }
    if let Ok(osrelease) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        return osrelease.to_lowercase().contains("microsoft");
    }
    false
}

fn write_clip_utf16le(text: &str) -> bool {
    let mut child = match Command::new("clip.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let mut bytes = Vec::with_capacity(text.len().saturating_mul(2));
        for unit in text.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let _ = stdin.write_all(&bytes);
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{is_wsl, normalize, truncate_utf8};
    use crate::test_support::{env_lock, restore_env, set_env};

    #[test]
    fn normalize_rewrites_newlines() {
        let out = normalize("a\r\nb\rc".to_string());
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn is_wsl_checks_env_var() {
        let _guard = env_lock().lock().unwrap();
        let prev = set_env("WSL_DISTRO_NAME", "Ubuntu");
        assert!(is_wsl());
        restore_env("WSL_DISTRO_NAME", prev);
    }

    #[test]
    fn truncate_utf8_respects_char_boundaries() {
        let s = "a😊b";
        assert_eq!(truncate_utf8(s, 1), "a");
        assert_eq!(truncate_utf8(s, 2), "a");
        assert_eq!(truncate_utf8(s, 5), "a😊");
        assert_eq!(truncate_utf8(s, 1024), s);
    }
}
