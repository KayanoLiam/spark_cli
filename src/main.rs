mod cli;
mod api;
mod config;
mod session;
mod utils;

use anyhow::Result;
use clap::{Parser, CommandFactory};
use cli::args::{Cli, CodeAction, Commands, ConfigAction, SessionAction};
use config::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    // Resolve explicit config path if provided
    let explicit_path = cli.runtime.config.as_deref().map(std::path::Path::new);
    // Project root: current working dir for project-scoped config
    let project_root = std::env::current_dir().ok();
    let mut settings = Settings::load_with(project_root.as_deref(), explicit_path)?;

    match &cli.command {
        Some(Commands::Interactive) => {
            cli::commands::handle_interactive(&settings).await?
        }
        Some(Commands::Chat) => {
            let prompt = if !cli.prompt.is_empty() {
                Some(cli.prompt.join(" "))
            } else {
                None
            };
            cli::commands::handle_chat(&settings, prompt, &cli.runtime).await?
        }
        Some(Commands::Config { action }) => match action {
            ConfigAction::Init { force, scope } => {
                match scope.as_deref() {
                    Some("project") => {
                        let root = std::env::current_dir()?;
                        Settings::init_scoped(*force, Some(&root))?;
                        println!("Initialized project config at {}/{}", root.display(), config::settings::CONFIG_FILE_NAME);
                    }
                    _ => {
                        Settings::init(*force)?;
                        println!("Initialized user config at ~/.spark_cli/{}", config::settings::CONFIG_FILE_NAME);
                    }
                }
            }
            ConfigAction::List => cli::commands::handle_config_list(&settings).await?,
            ConfigAction::Set { key, value } => {
                cli::commands::handle_config_set(&mut settings, key, value).await?
            }
        },
        Some(Commands::Session { action }) => match action {
            SessionAction::New { name } => {
                cli::commands::handle_session_new(&settings, name).await?
            }
            SessionAction::List => cli::commands::handle_session_list(&settings).await?,
            SessionAction::Load { id } => {
                cli::commands::handle_session_load(&settings, id).await?
            }
            SessionAction::Delete { id } => {
                cli::commands::handle_session_delete(&settings, id).await?
            }
        },
        Some(Commands::Code { action }) => match action {
            CodeAction::Generate { lang, r#type } => {
                cli::commands::handle_code_generate(&settings, lang, r#type).await?
            }
            CodeAction::Review { file } => {
                cli::commands::handle_code_review(&settings, file).await?
            }
            CodeAction::Optimize { file } => {
                cli::commands::handle_code_optimize(&settings, file).await?
            }
        },
        None => {
            if !cli.prompt.is_empty() {
                let prompt = cli.prompt.join(" ");
                cli::commands::handle_chat(&settings, Some(prompt), &cli.runtime).await?
            } else {
                // No command and no prompt: show help
                Cli::command().print_help()?;
                println!();
            }
        }
    }

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}
