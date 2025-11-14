use crate::config::KEY_FINGERPRINT_DISPLAY_LEN;
use crate::dirs::config::Config;
use crate::dirs::{config, contacts, keys};
use crate::utils::error::Result;
use colored::Colorize;
use std::path::PathBuf;

pub async fn run(key_path: Option<PathBuf>, force: bool) -> Result<()> {
    println!("{}", "Initializing rshare...\n".bright_cyan().bold());

    // Determine keys path (CLI arg or default)
    let keys_path = key_path.unwrap_or_else(|| keys::get_default_keys_dir().unwrap());

    // Check if config exists
    let config_path = config::get_config_path()?;

    if config::exists_config_at(&config_path) {
        println!("{} Found config file", "✓".bright_green());

        let loaded_config = config::load_config()?;

        // Check if keys exist
        if keys::keys_exist_at(&loaded_config.path.keys_path)
            && loaded_config.path.download_path.exists()
        {
            println!("{} Found keys and downloads", "✓".bright_green());

            // Load and validate keys
            match keys::load_keys_from(&loaded_config.path.keys_path) {
                Ok((private_key, public_key)) => {
                    if keys::validate_keypair(&private_key, &public_key).is_ok() {
                        println!("{} Keys are valid", "✓".bright_green());

                        if !force {
                            // Exit early if not forcing
                            println!(
                                "\n{}",
                                " rshare is already initialized!".bright_green().bold()
                            );
                            println!("   Keys: {}", loaded_config.path.keys_path.display());
                            println!("   Config: {}", config_path.display());
                            return Ok(());
                        } else {
                            println!("\n{}", "  Forcing regeneration...".bright_yellow());
                        }
                    } else {
                        println!("{} Keys invalid, regenerating...", "✗".bright_red());
                    }
                }
                Err(_) => {
                    println!("{} Failed to load keys, regenerating...", "✗".bright_red());
                }
            }
        } else {
            println!("{} No keys found", "✗".bright_yellow());
        }
    } else {
        println!("{} No config found", "✗".bright_yellow());
    }

    // Generate new keys
    println!("\n{}", " Generating new keypair".bright_cyan());
    let (private_key, public_key) = keys::generate_keys()?;
    keys::validate_keypair(&private_key, &public_key)?;

    // Save keys
    println!("{}", " Saving keys".bright_cyan());
    keys::save_keys_to(&private_key, &public_key, keys_path.clone())?;

    // Add self to trust
    println!("{}", " Adding self to trust".bright_cyan());
    let mut contacts = contacts::load_contacts()?;

    // Prevent duplicate "self" entry
    if force && contacts.contacts.contains_key("self") {
        contacts.remove("self")?;
    }

    contacts.add("self".to_string(), hex::encode(&public_key.to_bytes()))?;
    contacts::save_contacts(&contacts)?;

    // Create/update config
    println!("{}", " Saving config and downloads dirs".bright_cyan());
    let new_config = Config::create_config(keys_path.clone());
    config::save_download_path(&new_config)?;
    config::save_config(&new_config)?;

    println!("\n{}", " Locations:".bright_cyan());
    println!("   Keys:   {}", keys_path.display());
    println!("   Config: {}", config_path.display());

    println!("\n{}", " Fingerprints:".bright_cyan());
    println!(
        "   Private: {}...",
        hex::encode(&private_key.to_bytes()[..KEY_FINGERPRINT_DISPLAY_LEN]).bright_white()
    );
    println!(
        "   Public:  {}",
        hex::encode(&public_key.to_bytes()).bright_white()
    );

    println!(
        "\n{}",
        "Configure your config and setup your local server"
            .bright_red()
            .bold()
    );

    Ok(())
}
