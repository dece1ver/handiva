use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) always_on_top: bool,
    #[serde(default = "default_interval")]
    pub(crate) default_interval: u64,
    #[serde(default)]
    pub(crate) hotkeys: HotKeyConfig,
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HotKeyConfig {
    #[serde(default = "default_set_position_key")]
    pub(crate) set_position: String,
    #[serde(default = "default_toggle_key")]
    pub(crate) toggle: String,
}

impl Default for HotKeyConfig {
    fn default() -> Self {
        Self {
            set_position: default_set_position_key(),
            toggle: default_toggle_key(),
        }
    }
}

fn default_interval() -> u64 {
    500
}

fn default_set_position_key() -> String {
    "F6".to_string()
}

fn default_toggle_key() -> String {
    "F7".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            always_on_top: false,
            default_interval: default_interval(),
            hotkeys: HotKeyConfig::default(),
            config_path: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            info!(
                "Файл конфигурации не найден, создаётся новый: {:?}",
                config_path
            );
            let config = Self::default();
            config.save_to(&config_path)?;
            return Ok(config);
        }

        debug!("Загрузка конфигурации из: {:?}", config_path);

        let content = fs::read_to_string(&config_path).with_context(|| {
            format!("Не удалось прочитать файл конфигурации: {:?}", config_path)
        })?;

        let mut config: Config =
            toml::from_str(&content).with_context(|| "Не удалось распарсить конфигурацию")?;

        config.config_path = Some(config_path);

        info!("Конфигурация успешно загружена");
        debug!("Настройки: {:?}", config);

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            self.save_to(path)
        } else {
            let path = Self::get_config_path()?;
            self.save_to(&path)
        }
    }

    fn save_to(&self, path: &PathBuf) -> Result<()> {
        debug!("Сохранение конфигурации в: {:?}", path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Не удалось создать директорию: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "Не удалось сериализовать конфигурацию")?;

        fs::write(path, content)
            .with_context(|| format!("Не удалось записать файл конфигурации: {:?}", path))?;

        info!("Конфигурация успешно сохранена");
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("com", "handiva", "Handiva")
            .context("Не удалось определить директорию конфигурации")?
            .config_dir()
            .to_path_buf();

        Ok(config_dir.join("config.toml"))
    }

    pub fn update_and_save<F>(&mut self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        update_fn(self);
        self.save().map_err(|e| {
            error!("Ошибка при сохранении конфигурации: {}", e);
            e
        })
    }
}
