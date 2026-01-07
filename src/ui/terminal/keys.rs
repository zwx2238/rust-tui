use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) fn key_event_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if is_plain_char(key) {
        return Some(char_bytes(key));
    }
    if is_ctrl_char(key) {
        return ctrl_char_bytes(key);
    }
    if is_alt_char(key) {
        return Some(alt_char_bytes(key));
    }
    special_key_bytes(key)
}

fn is_plain_char(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char(_)) && key.modifiers.is_empty()
}

fn char_bytes(key: KeyEvent) -> Vec<u8> {
    let KeyCode::Char(c) = key.code else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    out
}

fn is_ctrl_char(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char(_))
}

fn ctrl_char_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    let KeyCode::Char(c) = key.code else {
        return None;
    };
    let b = ctrl_byte(c)?;
    Some(vec![b])
}

fn ctrl_byte(c: char) -> Option<u8> {
    let b = u8::try_from(c.to_ascii_uppercase()).ok()?;
    Some(b & 0x1f)
}

fn is_alt_char(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Char(_))
}

fn alt_char_bytes(key: KeyEvent) -> Vec<u8> {
    let mut out = vec![0x1b];
    out.extend_from_slice(&char_bytes(KeyEvent::new(key.code, KeyModifiers::empty())));
    out
}

fn special_key_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    let mod_code = csi_modifier_code(key.modifiers);
    match key.code {
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(with_optional_escape(vec![0x7f], key.modifiers)),
        KeyCode::Tab => tab_bytes(key.modifiers),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Up => Some(csi_cursor("A", mod_code)),
        KeyCode::Down => Some(csi_cursor("B", mod_code)),
        KeyCode::Right => Some(csi_cursor("C", mod_code)),
        KeyCode::Left => Some(csi_cursor("D", mod_code)),
        KeyCode::Home => Some(csi_home_end("H", mod_code)),
        KeyCode::End => Some(csi_home_end("F", mod_code)),
        KeyCode::PageUp => Some(csi_tilde(5, mod_code)),
        KeyCode::PageDown => Some(csi_tilde(6, mod_code)),
        KeyCode::Insert => Some(csi_tilde(2, mod_code)),
        KeyCode::Delete => Some(csi_tilde(3, mod_code)),
        KeyCode::F(n) => fkey_bytes(n),
        _ => None,
    }
}

fn tab_bytes(modifiers: KeyModifiers) -> Option<Vec<u8>> {
    if modifiers.contains(KeyModifiers::SHIFT) {
        return Some(vec![0x1b, b'[', b'Z']);
    }
    Some(vec![b'\t'])
}

fn with_optional_escape(mut bytes: Vec<u8>, modifiers: KeyModifiers) -> Vec<u8> {
    if modifiers.contains(KeyModifiers::ALT) {
        let mut out = vec![0x1b];
        out.append(&mut bytes);
        return out;
    }
    bytes
}

fn csi_modifier_code(modifiers: KeyModifiers) -> Option<u8> {
    let mut code = 1u8;
    if modifiers.contains(KeyModifiers::SHIFT) {
        code += 1;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        code += 2;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        code += 4;
    }
    if code == 1 {
        None
    } else {
        Some(code)
    }
}

fn csi_cursor(suffix: &str, mod_code: Option<u8>) -> Vec<u8> {
    match mod_code {
        None => csi_simple(suffix),
        Some(m) => csi_with_mod(1, m, suffix),
    }
}

fn csi_home_end(suffix: &str, mod_code: Option<u8>) -> Vec<u8> {
    match mod_code {
        None => csi_simple(suffix),
        Some(m) => csi_with_mod(1, m, suffix),
    }
}

fn csi_tilde(code: u8, mod_code: Option<u8>) -> Vec<u8> {
    match mod_code {
        None => format!("\x1b[{code}~").into_bytes(),
        Some(m) => format!("\x1b[{code};{m}~").into_bytes(),
    }
}

fn csi_simple(suffix: &str) -> Vec<u8> {
    format!("\x1b[{suffix}").into_bytes()
}

fn csi_with_mod(prefix: u8, mod_code: u8, suffix: &str) -> Vec<u8> {
    format!("\x1b[{prefix};{mod_code}{suffix}").into_bytes()
}

fn fkey_bytes(n: u8) -> Option<Vec<u8>> {
    match n {
        1 => Some(b"\x1bOP".to_vec()),
        2 => Some(b"\x1bOQ".to_vec()),
        3 => Some(b"\x1bOR".to_vec()),
        4 => Some(b"\x1bOS".to_vec()),
        5 => Some(b"\x1b[15~".to_vec()),
        6 => Some(b"\x1b[17~".to_vec()),
        7 => Some(b"\x1b[18~".to_vec()),
        8 => Some(b"\x1b[19~".to_vec()),
        9 => Some(b"\x1b[20~".to_vec()),
        10 => Some(b"\x1b[21~".to_vec()),
        11 => Some(b"\x1b[23~".to_vec()),
        12 => Some(b"\x1b[24~".to_vec()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_is_etx() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(key_event_to_bytes(key), Some(vec![0x03]));
    }

    #[test]
    fn alt_x_prefixes_escape() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT);
        assert_eq!(key_event_to_bytes(key), Some(vec![0x1b, b'x']));
    }

    #[test]
    fn shift_tab_is_backtab_sequence() {
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);
        assert_eq!(key_event_to_bytes(key), Some(vec![0x1b, b'[', b'Z']));
    }

    #[test]
    fn up_arrow_is_csi_a() {
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(key_event_to_bytes(key), Some(b"\x1b[A".to_vec()));
    }
}

