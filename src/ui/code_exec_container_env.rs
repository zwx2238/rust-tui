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

pub(crate) fn pip_target_dir() -> &'static str {
    "/tmp/deepchat/site-packages"
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
