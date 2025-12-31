use crate::args::Args;
use crate::config::{Config, ModelItem};

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

pub fn build_model_registry(
    cfg: Option<&Config>,
    args: &Args,
    fallback_api_key: Option<&str>,
) -> ModelRegistry {
    let cfg_base_url = cfg.and_then(|c| c.base_url.clone());
    let cfg_model = cfg.and_then(|c| c.model.clone());
    let cfg_api_key = cfg
        .and_then(|c| c.api_key.clone())
        .or_else(|| fallback_api_key.map(|v| v.to_string()));
    let models_cfg = cfg.and_then(|c| c.models.clone()).unwrap_or_default();
    let default_key = cfg
        .and_then(|c| c.default_model.clone())
        .or_else(|| models_cfg.first().map(|m| m.key.clone()))
        .unwrap_or_else(|| "deepseek".to_string());

    let models = if models_cfg.is_empty() {
        vec![ModelProfile {
            key: default_key.clone(),
            base_url: cfg_base_url
                .unwrap_or_else(|| args.base_url.trim_end_matches('/').to_string()),
            api_key: cfg_api_key.unwrap_or_default(),
            model: cfg_model.unwrap_or_else(|| args.model.clone()),
        }]
    } else {
        models_cfg
            .into_iter()
            .map(|m| fill_model_item(m, &cfg_base_url, &cfg_api_key, &cfg_model, args))
            .collect()
    };

    let default_key = if models.iter().any(|m| m.key == default_key) {
        default_key
    } else {
        models
            .first()
            .map(|m| m.key.clone())
            .unwrap_or(default_key)
    };
    ModelRegistry { default_key, models }
}

fn fill_model_item(
    item: ModelItem,
    cfg_base_url: &Option<String>,
    cfg_api_key: &Option<String>,
    cfg_model: &Option<String>,
    args: &Args,
) -> ModelProfile {
    ModelProfile {
        key: item.key,
        base_url: item
            .base_url
            .or_else(|| cfg_base_url.clone())
            .unwrap_or_else(|| args.base_url.trim_end_matches('/').to_string()),
        api_key: item.api_key.or_else(|| cfg_api_key.clone()).unwrap_or_default(),
        model: item
            .model
            .or_else(|| cfg_model.clone())
            .unwrap_or_else(|| args.model.clone()),
    }
}
