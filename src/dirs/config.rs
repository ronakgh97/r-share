use crate::config::*;
use crate::dirs::keys;
use crate::utils::error::{Error, Result};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub path: PathConfig,
    pub server: Vec<ServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub keys_path: PathBuf,
    pub download_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server_name: String,
    pub default: bool,
    pub server_ip: String,
    pub http_port: u16,
    pub socket_port: u16,
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
            server: vec![get_default_server_config().unwrap()],
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
            server: vec![get_default_server_config().unwrap()],
        }
    }

    pub fn select_server(&self, name: Option<String>) -> Result<ServerConfig> {
        if let Some(server_name) = name {
            self.server
                .iter()
                .find(|s| s.server_name == server_name)
                .cloned()
                .ok_or_else(|| {
                    Error::InvalidInput(format!(
                        "Relay server '{}' not found in config",
                        server_name
                    ))
                })
        } else {
            self.server
                .iter()
                .find(|s| s.default)
                .cloned()
                .ok_or_else(|| Error::InvalidInput("No default server in config".into()))
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

fn get_default_server_config() -> Result<ServerConfig> {
    Ok(ServerConfig {
        server_name: "my_server".into(),
        default: true,
        server_ip: local_ip()
            .unwrap_or(DEFAULT_PRIVATE_IP.parse().unwrap())
            .to_string(),
        http_port: DEFAULT_HTTP_PORT,
        socket_port: DEFAULT_SOCKET_PORT,
    })
}

pub fn get_default_server(config: &Config) -> Result<ServerConfig> {
    config
        .server
        .iter()
        .find(|s| s.default)
        .cloned()
        .ok_or_else(|| Error::InvalidInput("No default server found".into()))
}

pub fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::FileNotFound("Could not find home directory".into()))?;
    Ok(home.join(".rshare").join("config.toml"))
}

pub fn exists_config_at(config_path: &PathBuf) -> bool {
    config_path.exists() && config_path.is_file()
}

pub fn save_download_path(config: &Config) -> Result<()> {
    std::fs::create_dir_all(&config.path.download_path)?;
    Ok(())
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

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| Error::FileRead(format!("Failed to read config: {}", e)))?;

    toml::from_str(&content).map_err(|e| Error::InvalidInput(format!("Invalid config file: {}", e)))
}

pub fn add_server(config: &mut Config, server: &ServerConfig) -> Result<()> {
    // Check for name conflicts
    if config
        .server
        .iter()
        .any(|s| s.server_name == server.server_name)
    {
        return Err(Error::InvalidInput(format!(
            "Server '{}' already exists",
            server.server_name
        )));
    }

    // Check for port conflicts
    if config
        .server
        .iter()
        .any(|s| s.server_ip == server.server_ip && s.server_name == server.server_name)
    {
        return Err(Error::InvalidInput(
            "Server with same IP or name already exists".into(),
        ));
    }

    // If new server is default, clear existing defaults
    if server.default {
        for s in &mut config.server {
            s.default = false;
        }
    }

    config.server.push(server.clone());
    save_config(config)?;
    Ok(())
}

pub fn list_servers(config: &Config) -> Result<Vec<ServerConfig>> {
    Ok(config.server.clone())
}
pub fn remove_server(config: &mut Config, target: String) -> Result<ServerConfig> {
    if let Some(server) = config.server.iter().find(|s| s.server_name == target) {
        if server.default {
            return Err(Error::InvalidInput("Cannot remove default server".into()));
        }
    }

    let before = config.server.len();
    config.server.retain(|s| s.server_name != target);

    if config.server.len() == before {
        // No matching server found
        return Err(Error::InvalidInput("Server not found".into()));
    }

    let server = config
        .server
        .iter()
        .find(|s| s.server_name == target)
        .cloned()
        .unwrap();

    save_config(config)?;
    Ok(server)
}
