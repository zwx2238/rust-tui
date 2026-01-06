pub(crate) fn inject_requirements(code: &str) -> String {
    let (requirements, code_lines) = parse_requirements(code);
    if requirements.is_empty() {
        return code.to_string();
    }
    let mut out = build_pip_prelude(&requirements);
    append_code_lines(&mut out, code_lines);
    out
}

pub(crate) fn filter_pip_output(stdout: &str, exit_code: Option<i32>) -> String {
    if !pip_filter_allowed(exit_code) {
        return stdout.to_string();
    }
    let mut out = strip_pip_markers(stdout);
    trim_trailing_newline(&mut out);
    out
}

fn pip_filter_allowed(exit_code: Option<i32>) -> bool {
    matches!(exit_code, Some(0))
}

fn strip_pip_markers(stdout: &str) -> String {
    let mut out = String::new();
    let mut in_pip = false;
    for line in stdout.lines() {
        if line.trim() == "DEEPCHAT_PIP_BEGIN" {
            in_pip = true;
            continue;
        }
        if line.trim() == "DEEPCHAT_PIP_END" {
            in_pip = false;
            continue;
        }
        if !in_pip {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn trim_trailing_newline(out: &mut String) {
    if out.ends_with('\n') {
        out.pop();
        if out.ends_with('\r') {
            out.pop();
        }
    }
}

fn parse_requirements(code: &str) -> (Vec<String>, Vec<&str>) {
    let mut requirements = Vec::new();
    let mut code_lines = Vec::new();
    let mut in_header = true;
    for line in code.lines() {
        let trimmed = line.trim();
        if in_header && trimmed.is_empty() {
            code_lines.push(line);
            continue;
        }
        if in_header {
            if let Some(rest) = trimmed.strip_prefix("# requirements:") {
                requirements.extend(parse_requirement_list(rest));
                continue;
            }
            in_header = false;
        }
        code_lines.push(line);
    }
    (requirements, code_lines)
}

fn parse_requirement_list(rest: &str) -> Vec<String> {
    rest.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn build_pip_prelude(requirements: &[String]) -> String {
    let mut out = String::new();
    out.push_str("import subprocess, sys, os\n");
    out.push_str("work_dir = os.environ.get(\"DEEPCHAT_WORKDIR\", \"/opt/deepchat\")\n");
    out.push_str("tmp_dir = os.path.join(work_dir, \"tmp\")\n");
    out.push_str("site_dir = os.path.join(work_dir, \"site-packages\")\n");
    out.push_str(
        "cache_dir = os.environ.get(\"PIP_CACHE_DIR\") or os.path.join(tmp_dir, \"pip-cache\")\n",
    );
    out.push_str("os.makedirs(tmp_dir, exist_ok=True)\n");
    out.push_str("os.makedirs(site_dir, exist_ok=True)\n");
    out.push_str("os.makedirs(cache_dir, exist_ok=True)\n");
    out.push_str("os.environ[\"PIP_CACHE_DIR\"] = cache_dir\n");
    out.push_str("print(\"DEEPCHAT_PIP_BEGIN\")\n");
    append_pip_install(&mut out, requirements);
    out.push_str("print(\"DEEPCHAT_PIP_END\")\n");
    out.push_str("if site_dir not in sys.path:\n");
    out.push_str("    sys.path.insert(0, site_dir)\n");
    out
}

fn append_pip_install(out: &mut String, requirements: &[String]) {
    out.push_str("subprocess.check_call([sys.executable, \"-m\", \"pip\", \"install\", \"--target\", site_dir, \"--cache-dir\", cache_dir");
    for req in requirements {
        out.push_str(", \"");
        out.push_str(req);
        out.push('"');
    }
    out.push_str("])\n");
}

fn append_code_lines(out: &mut String, code_lines: Vec<&str>) {
    for line in code_lines {
        out.push_str(line);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_pip_output, inject_requirements};

    #[test]
    fn inject_requirements_adds_pip_block() {
        let code = "# requirements: requests\nprint('hi')";
        let out = inject_requirements(code);
        assert!(out.contains("pip"));
        assert!(out.contains("requests"));
    }

    #[test]
    fn filter_pip_output_strips_on_success() {
        let stdout = "DEEPCHAT_PIP_BEGIN\npip stuff\nDEEPCHAT_PIP_END\nok\n";
        let out = filter_pip_output(stdout, Some(0));
        assert_eq!(out.trim(), "ok");
    }
}
