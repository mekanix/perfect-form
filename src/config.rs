use std::{env, fs, path::Path};

use clap::ValueEnum;
use serde::Deserialize;

const APP_PREFIX: &str = "PERFECT_FORM";
const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl Environment {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Testing => "testing",
            Environment::Production => "production",
        }
    }

    #[must_use]
    pub fn default_sqlite_file(self) -> &'static str {
        match self {
            Environment::Development => ".dev.sqlite",
            Environment::Testing => ".test.sqlite",
            Environment::Production => ".prod.sqlite",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
struct DatabaseConfig {
    url: Option<String>,
    driver: Option<String>,
    user: Option<String>,
    password: Option<String>,
    database: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct JwtConfig {
    secret: Option<String>,
}

impl DatabaseConfig {
    fn to_url(&self) -> Option<String> {
        if let Some(url) = &self.url {
            return Some(url.clone());
        }

        let driver = self
            .driver
            .as_deref()
            .map(|d| d.to_lowercase())
            .unwrap_or_else(|| "postgres".to_owned());

        if driver == "sqlite" {
            let path = self.database.as_deref()?;
            if path == ":memory:" {
                return Some("sqlite::memory:".to_owned());
            }
            return Some(format!("sqlite://{path}"));
        }

        let host = self.host.as_deref()?;
        let database = self.database.as_deref()?;
        let user = self.user.as_deref().unwrap_or("");
        let password = self
            .password
            .as_deref()
            .map(|p| format!(":{p}"))
            .unwrap_or_default();
        let port = self.port.map(|p| format!(":{p}")).unwrap_or_default();

        let scheme = match driver.as_str() {
            "mysql" | "mariadb" => "mysql",
            _ => "postgres",
        };

        Some(format!(
            "{scheme}://{user}{password}@{host}{port}/{database}"
        ))
    }
}

#[derive(Debug, Default, Deserialize)]
struct EnvironmentConfig {
    database: Option<DatabaseConfig>,
    jwt: Option<JwtConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct AppConfigFile {
    database: Option<DatabaseConfig>,
    jwt: Option<JwtConfig>,
    development: Option<EnvironmentConfig>,
    testing: Option<EnvironmentConfig>,
    production: Option<EnvironmentConfig>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_environment(environment: Environment, config_path: Option<String>) -> Self {
        let config_file = load_config_file(config_path);
        let database_url = resolve_database_url(environment, &config_file);
        let jwt_secret = resolve_jwt_secret(environment, &config_file);

        Self {
            environment,
            database_url,
            jwt_secret,
        }
    }

    #[must_use]
    pub fn cookie_secure(&self) -> bool {
        self.environment == Environment::Production
    }
}

fn load_config_file(config_path: Option<String>) -> AppConfigFile {
    let path = config_path
        .or_else(|| env::var(format!("{APP_PREFIX}_CONFIG")).ok())
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_owned());

    if !Path::new(&path).exists() {
        return AppConfigFile::default();
    }

    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read config file {path}: {e}"));

    toml::from_str(&contents).unwrap_or_else(|e| panic!("Failed to parse config file {path}: {e}"))
}

fn env_database_config(environment: Environment, config: &AppConfigFile) -> Option<DatabaseConfig> {
    let env_config = match environment {
        Environment::Development => config.development.as_ref(),
        Environment::Testing => config.testing.as_ref(),
        Environment::Production => config.production.as_ref(),
    }?;

    env_config.database.clone()
}

fn resolve_database_url(environment: Environment, config: &AppConfigFile) -> String {
    let env_upper = environment.as_str().to_uppercase();

    // 1. Environment variables take highest priority.
    for var_name in [
        format!("{APP_PREFIX}_DATABASE_URL"),
        "DATABASE_URL".to_owned(),
        format!("{APP_PREFIX}_{env_upper}_DATABASE_URL"),
    ] {
        if let Ok(url) = env::var(&var_name)
            && !url.is_empty()
        {
            return url;
        }
    }

    // 2. Environment-specific config file section.
    if let Some(env_cfg) = env_database_config(environment, config)
        && let Some(url) = env_cfg.to_url()
        && !url.is_empty()
    {
        return url;
    }

    // 3. Global config file database settings.
    if let Some(global_cfg) = &config.database
        && let Some(url) = global_cfg.to_url()
        && !url.is_empty()
    {
        return url;
    }

    // 4. Built-in defaults for non-production environments.
    if environment == Environment::Production {
        panic!(
            "No database URL configured for production. \
             Set DATABASE_URL, {APP_PREFIX}_DATABASE_URL, \
             {APP_PREFIX}_PRODUCTION_DATABASE_URL, \
             or provide a [production.database] section in the config file."
        );
    }

    let file = environment.default_sqlite_file();
    let path = env::current_dir()
        .expect("Failed to get current directory")
        .join(file);

    if !path.exists() {
        fs::File::create(&path).expect("Failed to create SQLite database file");
    }

    format!("sqlite://{}", path.display())
}

fn resolve_jwt_secret(environment: Environment, config: &AppConfigFile) -> String {
    // 1. Environment variable takes highest priority.
    if let Ok(secret) = env::var(format!("{APP_PREFIX}_JWT_SECRET"))
        && !secret.is_empty()
    {
        return secret;
    }

    // 2. Environment-specific config file section.
    let env_secret = match environment {
        Environment::Development => config
            .development
            .as_ref()
            .and_then(|e| e.jwt.as_ref())
            .and_then(|j| j.secret.clone()),
        Environment::Testing => config
            .testing
            .as_ref()
            .and_then(|e| e.jwt.as_ref())
            .and_then(|j| j.secret.clone()),
        Environment::Production => config
            .production
            .as_ref()
            .and_then(|e| e.jwt.as_ref())
            .and_then(|j| j.secret.clone()),
    };
    if let Some(secret) = env_secret
        && !secret.is_empty()
    {
        return secret;
    }

    // 3. Global config file setting.
    if let Some(secret) = config.jwt.as_ref().and_then(|j| j.secret.clone())
        && !secret.is_empty()
    {
        return secret;
    }

    if environment == Environment::Production {
        panic!(
            "No JWT secret configured for production. \
             Set {APP_PREFIX}_JWT_SECRET or provide jwt.secret in the config file."
        );
    }

    // Insecure default for local development only.
    "dev-secret-change-me".to_owned()
}
