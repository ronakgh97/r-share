use crate::config::APP_VERSION;
use crate::dirs::keys::keys_exist_at;
use crate::dirs::{config, keys};
use colored::Colorize;
use figlet_rs::FIGfont;

pub fn show_welcome() {
    // Load the standard font
    let standard_font = FIGfont::standard().unwrap();

    // Render the ASCII art
    let figure = standard_font.convert("r-share");

    if let Some(figure) = figure {
        println!("{figure}");
    } else {
        // Fallback if figlet fails
        println!("╔════════════════════════════╗");
        println!("║          R-SHARE           ║");
        println!("╚════════════════════════════╝");
    }

    println!("\n A Rust-based CLI File Share Tool");
    println!("   Version: {}\n", APP_VERSION.bright_green().bold());

    // Try to load config
    match config::load_config() {
        Ok(loaded_config) => {
            // Config exists, check keys
            if keys_exist_at(&loaded_config.path.keys_path) {
                match keys::load_keys_from(&loaded_config.path.keys_path) {
                    Ok((private, public)) => {
                        println!("{} Keys loaded", "✓".bright_green());
                        println!(
                            "   Private: {}....",
                            hex::encode(&private.to_bytes()[..4]).dimmed()
                        );
                        println!(
                            "   Public:  {}",
                            hex::encode(&public.to_bytes()).bright_white()
                        );
                    }
                    Err(_) => {
                        println!("{} Failed to load keys", "✗".bright_red());
                        println!(" rs init  --force   Reinitialize your keys");
                    }
                }
            } else {
                println!("{} Keys not found", "✗".bright_yellow());
                println!("\n   {}", "rshare init".bright_cyan().bold());
                println!(" rs init        Reinitialize your keys");
            }
        }
        Err(_) => {
            // Config doesn't exist yet
            println!("{} Not initialized", "✗".bright_yellow());
        }
    }

    println!("\n📖 Docs: https://github.com/ronakgh97/r-share");
}
