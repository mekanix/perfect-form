use clap::Parser;
use sea_orm::{Database, EntityTrait};

use topcoat::router::Router;

mod app;
mod auth;
mod cli;
mod components;
mod config;
pub mod entity;

fn default_environment() -> config::Environment {
    // When launched by `topcoat dev`, default to development so the local
    // SQLite file is used. Direct `cargo run` still defaults to production.
    if std::env::var("TOPCOAT_DEV_URL").is_ok() {
        config::Environment::Development
    } else {
        config::Environment::Production
    }
}

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let environment = cli.environment.unwrap_or_else(default_environment);
    let cfg = config::Config::from_environment(environment, cli.config);

    println!(
        "Environment: {}, Database: {}",
        cfg.environment.as_str(),
        cfg.database_url
    );

    ensure_sqlite_file(&cfg.database_url);

    let db = Database::connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    match cli.command {
        Some(cli::Command::User(cmd)) => {
            run_user_command(&db, cmd).await;
        }
        None => {
            let router: Router = app::router(db, cfg);
            topcoat::start(router).await.unwrap();
        }
    }
}

async fn run_user_command(db: &sea_orm::DatabaseConnection, cmd: cli::UserCommand) {
    use sea_orm::ActiveValue::Set;

    match cmd {
        cli::UserCommand::Create { email, admin } => {
            let password = read_password_or_stdin();

            let password_hash = auth::hash_password(&password).expect("Failed to hash password");

            let user = crate::entity::user::ActiveModel {
                email: Set(email),
                password_hash: Set(password_hash),
                admin: Set(admin),
                ..Default::default()
            };

            crate::entity::user::Entity::insert(user)
                .exec(db)
                .await
                .expect("Failed to create user");

            println!("User created (admin: {admin})");
        }
    }
}

fn read_password_or_stdin() -> String {
    match rpassword::prompt_password("Password: ") {
        Ok(password) => password,
        Err(_) => {
            use std::io::{self, BufRead};

            let stdin = io::stdin();
            let mut line = String::new();
            stdin
                .lock()
                .read_line(&mut line)
                .expect("Failed to read password from stdin");
            line.trim_end().to_owned()
        }
    }
}

fn ensure_sqlite_file(url: &str) {
    let path = url
        .strip_prefix("sqlite://")
        .or_else(|| url.strip_prefix("sqlite:"))
        .unwrap_or(url);

    if path == ":memory:" || path.starts_with("?mode=memory") {
        return;
    }

    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    if !path.exists() {
        std::fs::File::create(path).expect("Failed to create SQLite database file");
    }
}

#[cfg(test)]
mod tests;
