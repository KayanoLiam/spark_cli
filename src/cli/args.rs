use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "spark", version, about = "AI-assisted CLI tool", propagate_version = true)]
pub struct Cli {
    /// One-shot prompt input
    pub prompt: Vec<String>,

    #[command(flatten)]
    pub io: IoArgs,

    #[command(flatten)]
    pub runtime: RuntimeArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args, Debug, Default)]
pub struct IoArgs {
    /// Input file path
    #[arg(short = 'f', long = "file", global = true)]
    pub input_file: Option<String>,

    /// Output file path
    #[arg(short = 'o', long = "output", global = true)]
    pub output_file: Option<String>,
}

#[derive(Args, Debug, Default)]
pub struct RuntimeArgs {
    /// Override provider for this run
    #[arg(long = "provider")]
    pub provider: Option<String>,

    /// Override model for this run
    #[arg(long = "model")]
    pub model: Option<String>,

    /// Stream output if provider supports
    #[arg(long = "stream")]
    pub stream: bool,

    /// Explicit config file path
    #[arg(long = "config")]
    pub config: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interactive TUI/chat
    Interactive,
    Chat,

    /// Config management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Session management
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Code-related commands
    Code {
        #[command(subcommand)]
        action: CodeAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Initialize default config file (~/.spark_cli/config.toml)
    Init {
        /// Overwrite if exists
        #[arg(long)]
        force: bool,
        /// Scope to create config: user or project (default: user)
        #[arg(long, value_parser = ["user", "project"])]
        scope: Option<String>,
    },
    Set { key: String, value: String },
    List,
}

#[derive(Subcommand, Debug)]
pub enum SessionAction {
    New { name: String },
    List,
    Load { id: String },
    Delete { id: String },
}

#[derive(Subcommand, Debug)]
pub enum CodeAction {
    Generate { 
        #[arg(long)]
        lang: String, 
        #[arg(long = "type")]
        r#type: String 
    },
    Review { file: String },
    Optimize { file: String },
}
