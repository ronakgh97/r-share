use anyhow::Result;
use clap::Parser;
use rshare::cli::{Args, Commands, TrustAction};
use rshare::prelude::message::show_welcome;
use rshare::prelude::{init, listen, serve, trust};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Init { keys, force }) => {
            init::run(keys, force).await?;
        }
        Some(Commands::Listen {
            path,
            from,
            max_size,
            quiet,
        }) => {
            listen::run(path, from, max_size, quiet).await?;
        }
        Some(Commands::Trust { action }) => match action {
            TrustAction::Add { name, key } => {
                trust::add(name, key).await?;
            }
            TrustAction::List { verbose } => {
                trust::list(verbose).await?;
            }
            TrustAction::Remove { name } => {
                trust::remove(name).await?;
            }
        },
        Some(Commands::Serve { file, to, quiet }) => {
            serve::run(file, to, quiet).await?;
        }
        None => {
            show_welcome();
        }
        _ => {
            show_welcome();
        }
    }

    Ok(())
}
