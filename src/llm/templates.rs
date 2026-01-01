use minijinja::{Environment, context};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

pub struct RigTemplates {
    env: Environment<'static>,
    root: PathBuf,
}

impl RigTemplates {
    pub fn load(prompts_dir: &str) -> Result<Self, String> {
        let root = Path::new(prompts_dir).join("rig");
        if !root.exists() {
            return Err(format!("缺少工具模板目录：{}", root.display()));
        }
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(root.clone()));
        Ok(Self { env, root })
    }

    pub fn tool_defs(&self) -> Result<Vec<ToolSchema>, String> {
        let path = self.root.join("tools.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取工具定义失败：{} ({e})", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("解析工具定义失败：{e}"))
    }

    pub fn render_preamble(&self, base_system: &str, tools: &[ToolSchema]) -> Result<String, String> {
        let tmpl = self
            .env
            .get_template("tool_preamble.jinja")
            .map_err(|e| format!("加载模板失败：{e}"))?;
        tmpl.render(context! { base_system => base_system, tools => tools })
            .map_err(|e| format!("渲染模板失败：{e}"))
    }

    pub fn render_tool_result(
        &self,
        name: &str,
        args: &serde_json::Value,
        output: &str,
    ) -> Result<String, String> {
        let tmpl = self
            .env
            .get_template("tool_result.jinja")
            .map_err(|e| format!("加载模板失败：{e}"))?;
        tmpl.render(context! { name => name, args => args, output => output })
            .map_err(|e| format!("渲染模板失败：{e}"))
    }

    pub fn render_followup(&self) -> Result<String, String> {
        let tmpl = self
            .env
            .get_template("tool_followup.jinja")
            .map_err(|e| format!("加载模板失败：{e}"))?;
        tmpl.render(context! {})
            .map_err(|e| format!("渲染模板失败：{e}"))
    }
}
