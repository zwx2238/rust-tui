use crate::ui::workspace::WorkspaceConfig;
use std::sync::{Mutex, OnceLock};

mod container_start;
mod files;
mod stream;

use container_start::{is_container_running, start_container};

pub(crate) use stream::{run_bash_in_container_stream, run_python_in_container_stream};

#[cfg(test)]
pub(crate) fn ensure_container(
    container_id: &mut Option<String>,
    workspace: &WorkspaceConfig,
) -> Result<String, String> {
    if let Some(id) = container_id.as_ref()
        && is_container_running(id)
    {
        return Ok(id.clone());
    }
    let id = start_container(Some(workspace))?;
    *container_id = Some(id.clone());
    Ok(id)
}

static CONTAINER_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn cached_container() -> &'static Mutex<Option<String>> {
    CONTAINER_CACHE.get_or_init(|| Mutex::new(None))
}

pub(crate) fn ensure_container_cached(workspace: &WorkspaceConfig) -> Result<String, String> {
    let mut guard = cached_container()
        .lock()
        .map_err(|_| "Docker 启动失败：容器缓存锁异常".to_string())?;
    if let Some(id) = guard.as_ref()
        && is_container_running(id)
    {
        return Ok(id.clone());
    }
    let id = start_container(Some(workspace))?;
    *guard = Some(id.clone());
    Ok(id)
}

#[cfg(test)]
pub(crate) fn stop_exec(container_id: &str, run_id: &str) -> bool {
    stream::stop_exec(container_id, run_id)
}
