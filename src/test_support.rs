#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
pub(crate) fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
pub(crate) fn set_env(key: &str, value: &str) -> Option<String> {
    let prev = std::env::var(key).ok();
    unsafe { std::env::set_var(key, value) };
    prev
}

#[cfg(test)]
pub(crate) fn restore_env(key: &str, prev: Option<String>) {
    if let Some(val) = prev {
        unsafe { std::env::set_var(key, val) };
    } else {
        unsafe { std::env::remove_var(key) };
    }
}
