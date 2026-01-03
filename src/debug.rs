//! 调试工具模块
//!
//! 提供用于调试的辅助功能，如等待调试器 attach。

use std::fs;
use std::thread;
use std::time::Duration;

/// 等待 gdb 或其他调试器 attach 到当前进程
///
/// 通过检查 `/proc/self/status` 文件中的 `TracerPid` 字段来判断是否有调试器 attach。
/// 如果 `TracerPid` 不为 0，说明有调试器 attach，函数返回。
///
/// # 错误处理
/// 如果读取文件失败，函数会继续等待，不会返回错误。
///
/// # 示例
/// ```no_run
/// use rust_tui::debug::wait_for_gdb_attach;
///
/// // 在程序启动时调用
/// wait_for_gdb_attach().unwrap();
/// ```
pub fn wait_for_gdb_attach() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("等待 gdb attach... (PID: {})", std::process::id());
    eprintln!("使用以下命令 attach: gdb -p {}", std::process::id());

    loop {
        match check_tracer_pid() {
            Ok(Some(pid)) if pid != 0 => {
                eprintln!("检测到调试器 attach (TracerPid: {})，继续执行...", pid);
                return Ok(());
            }
            Ok(Some(_)) => {
                // TracerPid 为 0，继续等待
            }
            Ok(None) => {
                // 未找到 TracerPid 行，继续等待
            }
            Err(_) => {
                // 读取失败，继续等待
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// 检查当前进程的 TracerPid
///
/// 返回 `Ok(Some(pid))` 如果找到 TracerPid，`Ok(None)` 如果未找到，`Err` 如果读取失败。
fn check_tracer_pid() -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let status = fs::read_to_string("/proc/self/status")?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("TracerPid:") {
            let pid_str = rest.trim();
            let pid = pid_str.parse::<u32>()?;
            return Ok(Some(pid));
        }
    }
    Ok(None)
}
