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
            quiet,
            local,
        }) => {
            listen::run(path, from, quiet, local).await?;
        }
        Some(Commands::Serve {
            file,
            to,
            quiet,
            local,
        }) => {
            serve::run(file, to, quiet, local).await?;
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
        None => {
            show_welcome();
        }
        _ => {
            show_welcome();
        }
    }

    Ok(())
}
