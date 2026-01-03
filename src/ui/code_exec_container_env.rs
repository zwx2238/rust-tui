pub(crate) enum CodeExecNetwork {
    None,
    Host,
    Bridge,
}

pub(crate) fn code_exec_network_mode() -> CodeExecNetwork {
    match std::env::var("DEEPCHAT_CODE_EXEC_NETWORK") {
        Ok(value) => {
            let v = value.trim().to_ascii_lowercase();
            if v.is_empty() {
                CodeExecNetwork::Host
            } else if v == "0" || v == "false" || v == "off" || v == "no" || v == "none" {
                CodeExecNetwork::None
            } else if v == "bridge" {
                CodeExecNetwork::Bridge
            } else {
                CodeExecNetwork::Host
            }
        }
        Err(_) => CodeExecNetwork::Host,
    }
}

pub(crate) fn read_only_enabled() -> bool {
    match std::env::var("DEEPCHAT_READ_ONLY") {
        Ok(value) => {
            let v = value.trim().to_ascii_lowercase();
            !(v.is_empty() || v == "0" || v == "false" || v == "off" || v == "no")
        }
        Err(_) => false,
    }
}

pub(crate) fn work_dir() -> String {
    if read_only_enabled() {
        "/opt/deepchat/work".to_string()
    } else {
        "/opt/deepchat".to_string()
    }
}

pub(crate) fn tmp_dir() -> String {
    format!("{}/tmp", work_dir())
}

pub(crate) fn run_dir() -> String {
    format!("{}/run", work_dir())
}

pub(crate) fn pip_target_dir() -> String {
    format!("{}/site-packages", work_dir())
}

pub(crate) fn pip_cache_dir() -> String {
    match std::env::var("DEEPCHAT_CODE_EXEC_PIP_CACHE_DIR") {
        Ok(value) => {
            let v = value.trim();
            if v.is_empty() {
                std::env::temp_dir()
                    .join("deepchat")
                    .join("pip-cache")
                    .to_string_lossy()
                    .to_string()
            } else {
                v.to_string()
            }
        }
        Err(_) => std::env::temp_dir()
            .join("deepchat")
            .join("pip-cache")
            .to_string_lossy()
            .to_string(),
    }
}

pub(crate) fn prepare_pip_cache_dir() {
    let dir = pip_cache_dir();
    let _ = std::fs::create_dir_all(dir);
}

pub(crate) fn site_tmpfs_mb() -> u32 {
    match std::env::var("DEEPCHAT_CODE_EXEC_SITE_SIZE_MB") {
        Ok(value) => value.trim().parse::<u32>().unwrap_or(2048),
        Err(_) => 2048,
    }
}

pub(crate) fn pip_index_url() -> Option<String> {
    match std::env::var("DEEPCHAT_CODE_EXEC_PIP_INDEX_URL") {
        Ok(value) => {
            let v = value.trim();
            if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            }
        }
        Err(_) => None,
    }
}

pub(crate) fn pip_extra_index_url() -> Option<String> {
    match std::env::var("DEEPCHAT_CODE_EXEC_PIP_EXTRA_INDEX_URL") {
        Ok(value) => {
            let v = value.trim();
            if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{read_only_enabled, work_dir};
    use crate::test_support::{env_lock, restore_env, set_env};

    #[test]
    fn read_only_changes_work_dir() {
        let _guard = env_lock().lock().unwrap();
        let prev = set_env("DEEPCHAT_READ_ONLY", "1");
        assert!(read_only_enabled());
        assert_eq!(work_dir(), "/opt/deepchat/work");
        restore_env("DEEPCHAT_READ_ONLY", prev);
        assert_eq!(work_dir(), "/opt/deepchat");
    }
}
