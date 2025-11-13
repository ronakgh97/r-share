use crate::config::*;
use crate::dirs::keys;
use crate::utils::error::{Error, Result};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub path: PathConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub keys_path: PathBuf,
    pub download_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub public_ip: String,
    pub private_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: {
                PathConfig {
                    keys_path: keys::get_default_keys_dir().unwrap(),
                    download_path: dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("rshare")
                        .join("downloads"),
                }
            },
            server: ServerConfig {
                public_ip: DEFAULT_PUBLIC_IP.to_string(),
                private_ip: local_ip()
                    .unwrap_or(DEFAULT_PRIVATE_IP.parse().unwrap())
                    .to_string(),
            },
        }
    }
}

impl Config {
    pub fn create_config(key_path: PathBuf) -> Self {
        Config {
            path: {
                PathConfig {
                    keys_path: key_path,
                    download_path: dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("rshare")
                        .join("downloads"),
                }
            },
            server: ServerConfig {
                // Use available public server by IP address or domain name
                public_ip: DEFAULT_PUBLIC_IP.to_string(),
                // Use your own available self-host or private server by IP address or domain name
                private_ip: local_ip()
                    .unwrap_or(DEFAULT_PRIVATE_IP.parse().unwrap())
                    .to_string(),
            },
        }
    }
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| {
            Error::FileWrite(format!(
                "Failed to serialize config to TOML: {}",
                e.to_string()
            ))
        })
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::FileNotFound("Could not find home directory".into()))?;
    Ok(home.join(".rshare").join("config.toml"))
}

pub fn save_download_path(config: &Config) -> Result<()> {
    std::fs::create_dir_all(&config.path.download_path)?;
    Ok(())
}

pub fn exists_config_at(config_path: &PathBuf) -> bool {
    config_path.exists() && config_path.is_file()
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| Error::FileRead(format!("Failed to read config: {}", e)))?;

    toml::from_str(&content).map_err(|e| Error::InvalidInput(format!("Invalid config file: {}", e)))
}

/// Save config to default location
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    // Create parent directory
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::FileWrite(format!("Failed to create config directory: {}", e)))?;
    }

    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| Error::FileWrite(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(&config_path, toml_string)
        .map_err(|e| Error::FileWrite(format!("Failed to write config: {}", e)))?;

    Ok(())
}
