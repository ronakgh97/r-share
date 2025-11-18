use crate::config::APP_VERSION;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "rshare",
    version = APP_VERSION,
    about = "R-Share - A Rust-based cli server tool for sharing files over a local/public network.",
    long_about = "R-Share is lightweight and secure file sharing tool built in Rust. \
    It allows users to easily share files over local or public networks with encryption and access controls."
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Me {
        /// Show your public key
        #[arg(short, long, default_value = "false")]
        verbose: bool,
    },

    Health {
        /// Check relay server health
        #[arg(short, long)]
        server: Option<String>,
    },

    /// Initialize and generate a public/private key
    Init {
        /// Save keys to custom path, or default to ~/.rshare/keys/
        #[arg(short, long)]
        keys: Option<PathBuf>,

        /// Overwrite existing keys and config file if they exist
        #[arg(short, long, default_value = "false")]
        force: bool,
    },

    Listen {
        /// Directory to save received files
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Only accept files from trusted contact
        #[arg(short, long, required = true)]
        from: String,

        /// Use relays server from config file
        #[arg(short, long)]
        relay: Option<String>,

        /// Enable progress bars
        #[arg(short, long, default_value = "false")]
        quiet: bool,
    },

    Serve {
        /// File(s) or directory to send
        #[arg(short, long, required = true)]
        file: PathBuf,

        /// Send to trusted contact by name
        #[arg(short, long, required = true)]
        to: String,

        /// Use relays server from config file
        #[arg(short, long)]
        relay: Option<String>,

        /// Enable progress bars
        #[arg(short, long, default_value = "false")]
        quiet: bool,
    },

    /// Manage relay servers
    Relay {
        #[command(subcommand)]
        action: ServerAction,
    },

    /// Manage trusted contacts
    Trust {
        #[command(subcommand)]
        action: TrustAction,
    },

    /// View transfer history
    History {
        /// Show last N transfers
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum ServerAction {
    /// Add a relay server
    Add {
        /// Server name
        #[arg(short, long, required = true)]
        name: String,

        /// Server IP address or domain
        #[arg(short, long, required = true)]
        ip: String,

        /// HTTP port (default: 8080)
        #[arg(short, long)]
        http_port: Option<u16>,

        /// Socket port (default: 10000)
        #[arg(short, long)]
        socket_port: Option<u16>,
    },

    /// List all relay servers
    List {
        /// Show full server details
        #[arg(short, long)]
        verbose: bool,
    },

    /// Remove a relay server
    Remove {
        /// Server name
        #[arg(short, long, required = true)]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum TrustAction {
    /// Add a trusted contact
    Add {
        /// Contact name
        #[arg(short, long, required = true)]
        name: String,

        /// Public key (hex string)
        #[arg(short, long, required = true)]
        key: String,
    },

    /// List all trusted contacts
    List {
        /// Show full public keys
        #[arg(short, long)]
        verbose: bool,
    },

    /// Remove a trusted contact
    Remove {
        /// Contact name
        name: String,
    },
}
