use crate::config::Config;

#[derive(Clone)]
pub struct ModelProfile {
    pub key: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
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
        })
        .collect::<Vec<_>>();
    let default_key = cfg.default_model.clone();
    ModelRegistry {
        default_key,
        models,
    }
}
