use clap::{Parser, Subcommand};

use crate::config::Environment;

#[derive(Debug, Parser)]
#[command(name = "perfect-form")]
pub struct Cli {
    #[arg(long, value_enum, global = true, help = "Runtime environment")]
    pub environment: Option<Environment>,

    #[arg(long, global = true, help = "Path to the configuration file")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand)]
    User(UserCommand),
}

#[derive(Debug, Subcommand)]
pub enum UserCommand {
    /// Create a new user
    Create {
        /// User email
        email: String,

        /// Grant administrator privileges
        #[arg(long)]
        admin: bool,
    },
}
