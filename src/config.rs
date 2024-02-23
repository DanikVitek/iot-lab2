use std::str::FromStr;

use color_eyre::eyre::{eyre, Context};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    database: Database,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    host: String,
    port: u16,
    username: SecretString,
    password: SecretString,
    #[serde(rename = "dbname")]
    name: SecretString,
}

impl Configuration {
    pub fn try_read() -> color_eyre::Result<Self> {
        let base_path =
            std::env::current_dir().context("Failed to determine the current directory")?;
        let config_dir = base_path.join("configuration");

        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base")).required(true))
            .add_source({
                let environment: Environment = std::env::var("APP_ENVIRONMENT")?.parse()?;
                config::File::from(config_dir.join(environment.as_str())).required(true)
            })
            .build()?;

        config.try_deserialize().map_err(Into::into)
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}

impl Database {
    pub fn connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(self.username.expose_secret())
            .password(self.password.expose_secret())
            .database(self.name.expose_secret())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl FromStr for Environment {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(eyre!("Unknown environment: {}", s)),
        }
    }
}
