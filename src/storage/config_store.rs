use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::theme::ThemeName;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeName,
    #[serde(default)]
    pub deck_positions: HashMap<String, usize>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeName::Terminal,
            deck_positions: HashMap::new(),
        }
    }
}

pub struct ConfigStore {
    pub config: AppConfig,
    pub path: PathBuf,
}

impl ConfigStore {
    pub fn load() -> Result<Self, String> {
        let path = config_path()?;

        if path.exists() {
            let raw =
                fs::read_to_string(&path).map_err(|e| format!("failed to read config: {e}"))?;
            let config: AppConfig =
                serde_json::from_str(&raw).map_err(|e| format!("failed to parse config: {e}"))?;
            Ok(Self { config, path })
        } else {
            let store = Self {
                config: AppConfig::default(),
                path,
            };
            store.save()?;
            Ok(store)
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create config dir: {e}"))?;
        }
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("failed to serialize config: {e}"))?;
        fs::write(&self.path, json).map_err(|e| format!("failed to write config: {e}"))
    }

    pub fn set_theme(&mut self, theme: ThemeName) -> Result<(), String> {
        self.config.theme = theme;
        self.save()
    }
}

fn config_path() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("com", "fcard", "fcard")
        .ok_or_else(|| "could not determine config directory".to_string())?;
    Ok(dirs.config_dir().join("config.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_theme_is_terminal() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.theme, ThemeName::Terminal)
    }

    #[test]
    fn config_roundtrips_json() {
        let original = AppConfig {
            theme: ThemeName::Dracula,
            deck_positions: HashMap::new(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.theme, original.theme);
    }

    #[test]
    fn all_theme_names_roundtrip_json() {
        for &name in ThemeName::ALL {
            let cfg = AppConfig {
                theme: name,
                deck_positions: HashMap::new(),
            };
            let json = serde_json::to_string(&cfg).unwrap();
            let restored: AppConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.theme, name);
        }
    }
}
