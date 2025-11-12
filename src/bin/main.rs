use anyhow::Result;
use clap::Parser;
use rshare::args::{init, listen, serve, trust};
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
            show_welcome().await?;
        }
        _ => {
            show_welcome().await?;
        }
    }

    Ok(())
}
