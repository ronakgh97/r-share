use crate::dirs::config::{Config, load_config};
use crate::server::RelayClient;
use crate::utils::error::Result;
use colored::Colorize;

pub async fn run(server: Option<String>) -> Result<()> {
    match load_config() {
        Ok(loaded_config) => {
            println!("{} Found config file", "✓".bright_green());
            println!();

            // Select relay server from config
            let server_config = Config::select_server(&loaded_config, server)?;

            // Create relay client
            let relay_client = RelayClient::new(
                server_config.server_ip,
                server_config.http_port,
                server_config.socket_port,
            );

            // Check server health
            relay_client.health_check().await?;

            print!(
                "  Server: {} is healthy",
                server_config.server_name.bright_green()
            );
            println!();
        }
        Err(_) => {
            println!();
            println!("{} No config file found", "✗".bright_red());
            println!(" rs init   Initialize rshare");
        }
    }

    Ok(())
}
