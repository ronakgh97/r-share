use crate::dirs::config;
use crate::dirs::config::{ServerConfig, load_config};
use crate::utils::error::Result;
use colored::Colorize;

pub async fn add(
    name: String,
    ip: String,
    http_port: Option<u16>,
    socket_port: Option<u16>,
) -> Result<()> {
    match load_config() {
        Ok(mut config) => {
            println!("{} Found config file", "✓".bright_green());
            println!();

            // Create server_config from params
            let server_config = ServerConfig {
                server_name: name,
                default: false,
                server_ip: ip,
                http_port: http_port.unwrap_or(8000),
                socket_port: socket_port.unwrap_or(10000),
            };

            config::add_server(&mut config, &server_config)?;

            println!(" {} Server added", "✓".bright_green());
            println!();
            pretty_print(&server_config);
        }
        Err(_) => {
            println!();
            println!("{} No config file found", "✗".bright_red());
            println!(" rs init   Initialize rshare");
        }
    }
    Ok(())
}

pub async fn list(_verbose: bool) -> Result<()> {
    match load_config() {
        Ok(config) => {
            println!("{} Found config file", "✓".bright_green());
            println!();

            let server_config = config::list_servers(&config)?;

            if server_config.is_empty() {
                println!();
                println!("{} No servers found", "✗".bright_yellow());
                return Ok(());
            }

            println!("My servers:\n");
            for server in server_config.iter() {
                pretty_print(server);
            }
        }

        Err(_) => {
            println!();
            println!("{} No config file found", "✗".bright_red());
            println!(" rs init   Initialize rshare");
        }
    }
    Ok(())
}
pub async fn remove(name: String) -> Result<()> {
    match load_config() {
        Ok(mut config) => {
            println!("{} Found config file", "✓".bright_green());
            println!();

            let server_config = config::remove_server(&mut config, name)?;

            println!("{} Server removed", "✓".bright_green());
            pretty_print(&server_config);
        }

        Err(_) => {
            println!();
            println!("{} No config file found", "✗".bright_red());
            println!(" rs init   Initialize rshare");
        }
    }
    Ok(())
}

fn pretty_print(server_config: &ServerConfig) {
    println!(
        "{}",
        format!("  • {}", server_config.server_name)
            .bright_white()
            .bold()
    );
    println!(
        "    IP:           {}",
        server_config.server_ip.bright_blue().underline()
    );
    println!(
        "    HTTP Port:    {}",
        server_config.http_port.to_string().bright_yellow()
    );
    println!(
        "    Socket Port:  {}",
        server_config.socket_port.to_string().bright_yellow()
    );
    println!(
        "    Default:      {}",
        server_config.default.to_string().bright_magenta()
    );
    println!();
}
