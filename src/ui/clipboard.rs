use arboard::Clipboard;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn set(text: &str) {
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
        return;
    }
    if is_wsl() {
        if write_clip_utf16le(text) {
            return;
        }
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
            Err(_) => return,
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
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
    use super::{is_wsl, normalize};
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
}
