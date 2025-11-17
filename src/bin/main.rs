use anyhow::Result;
use clap::Parser;
use rshare::args::{health, init, listen, serve, trust};
use rshare::cli::{Args, Commands, TrustAction};
use rshare::utils::message::show_welcome;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Me { .. }) => {
            // TODO: Implement "me" command
            //trust::show_me(verbose).await?;
        }

        Some(Commands::Health { server }) => {
            health::run(server).await?;
        }

        Some(Commands::Init { keys, force }) => {
            init::run(keys, force).await?;
        }
        Some(Commands::Listen {
            path,
            from,
            quiet,
            relay,
        }) => {
            listen::run(path, from, quiet, relay).await?;
        }
        Some(Commands::Serve {
            file,
            to,
            quiet,
            relay,
        }) => {
            serve::run(file, to, quiet, relay).await?;
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
            show_welcome().await?;
        }
        _ => {
            show_welcome().await?;
        }
    }

    Ok(())
}
