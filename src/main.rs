use anyhow::Result;
use clap::Parser;

mod browser;
mod chat;
mod cli;
mod commands;
mod config;
mod docker;
mod health;
mod output;
mod port;
mod provider;
#[allow(dead_code)]
mod sandbox;
#[cfg(unix)]
#[allow(dead_code)]
mod sandbox_unix;
#[cfg(windows)]
#[allow(dead_code)]
mod sandbox_windows;
mod state;
mod template;
mod token;
mod validate;

use cli::{Cli, Commands, PairCommands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set global flags
    output::set_verbose(cli.verbose);
    output::set_no_color(cli.no_color);

    let result = run(cli).await;

    if let Err(e) = result {
        output::error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Create {
            name,
            provider,
            api_key,
            model,
            port,
            image,
            mem,
            cpus,
            pids,
            docker: _,
            process,
        } => {
            commands::create::run(
                &name,
                provider.as_deref(),
                api_key.as_deref(),
                model.as_deref(),
                port,
                image.as_deref(),
                mem.as_deref(),
                cpus.as_deref(),
                pids.as_deref(),
                process,
            )
            .await
        }

        Commands::Destroy { name, force } => commands::destroy::run(&name, force, cli.force),

        Commands::List => commands::list::run(),

        Commands::Status { name } => commands::status::run(&name),

        Commands::Start { name } => commands::lifecycle::start(&name),

        Commands::Stop { name } => commands::lifecycle::stop(&name),

        Commands::Restart { name } => commands::lifecycle::restart(&name),

        Commands::Logs { name, follow, tail } => commands::logs::run(&name, follow, &tail),

        Commands::Token { name, regenerate } => commands::token_cmd::run(&name, regenerate),

        Commands::Open { name } => commands::open::run(&name),

        Commands::Pair { subcmd } => match subcmd {
            PairCommands::Approve { name, label } => {
                commands::pair::approve(&name, label.as_deref())
            }
            PairCommands::List { name } => commands::pair::list(&name),
            PairCommands::Revoke { name, label } => commands::pair::revoke(&name, &label),
        },

        Commands::Update {
            name,
            model,
            mem,
            cpus,
            pids,
        } => commands::update::run(
            &name,
            model.as_deref(),
            mem.as_deref(),
            cpus.as_deref(),
            pids.as_deref(),
        ),

        Commands::Chat { name } => commands::chat_cmd::run(&name).await,

        Commands::Doctor => {
            let issues = commands::doctor::run();
            if issues > 0 {
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Version => {
            println!("moltctrl v{}", config::VERSION);
            Ok(())
        }
    }
}
