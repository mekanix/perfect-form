use clap::Parser;
use sea_orm::Database;
use sea_orm_cli::{MigrateSubcommands, run_migrate_generate, run_migrate_init};
use sea_orm_migration::cli::run_migrate;

#[path = "../config.rs"]
mod config;

#[derive(Debug, Parser)]
#[command(name = "migrate")]
struct Args {
    #[arg(
        long,
        value_enum,
        default_value_t = config::Environment::Production,
        help = "Runtime environment"
    )]
    environment: config::Environment,

    #[arg(long, help = "Path to the configuration file")]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<MigrateSubcommands>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cfg = config::Config::from_environment(args.environment, args.config);

    // Only the database URL is needed here; silence dead-code warnings for the
    // other config fields in this binary.
    let _ = cfg.environment;
    let _ = cfg.jwt_secret.clone();
    let _ = cfg.cookie_secure();

    match args.command {
        Some(MigrateSubcommands::Init) => {
            run_migrate_init("migration")?;
        }
        Some(MigrateSubcommands::Generate {
            migration_name,
            local_time,
            universal_time: _,
        }) => {
            run_migrate_generate("migration", &migration_name, !local_time)?;
        }
        _ => {
            let db = Database::connect(&cfg.database_url).await?;
            run_migrate(migration::Migrator, &db, args.command, false).await?;
        }
    }

    Ok(())
}
