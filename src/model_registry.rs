use crate::config::Config;

#[derive(Clone)]
pub struct ModelProfile {
    pub key: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: Option<u64>,
}

#[derive(Clone)]
pub struct ModelRegistry {
    pub default_key: String,
    pub models: Vec<ModelProfile>,
}

impl ModelRegistry {
    pub fn get(&self, key: &str) -> Option<&ModelProfile> {
        self.models.iter().find(|m| m.key == key)
    }

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.models.iter().position(|m| m.key == key)
    }
}

pub fn build_model_registry(cfg: &Config) -> ModelRegistry {
    let models = cfg
        .models
        .iter()
        .cloned()
        .map(|m| ModelProfile {
            key: m.key,
            base_url: m.base_url.trim_end_matches('/').to_string(),
            api_key: m.api_key,
            model: m.model,
            max_tokens: m.max_tokens,
        })
        .collect::<Vec<_>>();
    let default_key = cfg.default_model.clone();
    ModelRegistry {
        default_key,
        models,
    }
}

#[cfg(test)]
mod tests {
    use super::build_model_registry;
    use crate::config::{Config, ModelItem};

    #[test]
    fn trims_base_url_and_indexes() {
        let cfg = Config {
            theme: "dark".to_string(),
            models: vec![ModelItem {
                key: "m1".to_string(),
                base_url: "https://api.test/".to_string(),
                api_key: "k".to_string(),
                model: "m".to_string(),
                max_tokens: None,
            }],
            default_model: "m1".to_string(),
            prompts_dir: "/tmp/prompts".to_string(),
            tavily_api_key: "key".to_string(),
        };
        let registry = build_model_registry(&cfg);
        assert_eq!(registry.default_key, "m1");
        assert_eq!(registry.index_of("m1"), Some(0));
        assert_eq!(registry.get("m1").unwrap().base_url, "https://api.test");
    }
}
