use config::{ConfigError, Source, Value, ValueKind};
use serde::Deserialize;
use std::{collections::HashMap, env, net::IpAddr};

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub bind: ServerBindSettings,
    pub database: DatabaseSettings,
}

#[derive(Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub port: u16,
    pub name: String,
}

#[derive(Clone, Deserialize)]
pub struct ServerBindSettings {
    pub addr: IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, Default)]
pub struct CustomEnvironment {
    /// Maps environment variable to a key
    custom_mappings: HashMap<String, String>,
}

impl CustomEnvironment {
    pub fn with_custom<K, V>(map: HashMap<K, V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        let map: HashMap<String, String> = map
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        Self {
            custom_mappings: map,
        }
    }

    pub fn add_custom<K, V>(mut self, env_var: &str, config_key: &str) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.custom_mappings
            .insert(env_var.into(), config_key.into());
        self
    }
}

impl Source for CustomEnvironment {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, ConfigError> {
        let mut m = HashMap::new();
        let uri = "custom environment".to_owned();

        let collector = |(key, value): (String, String)| {
            let key = match self.custom_mappings.get(&key) {
                Some(key) => key.clone(),
                None => {
                    return;
                }
            };

            let value = ValueKind::String(value);

            m.insert(key, Value::new(Some(&uri), value));
        };

        env::vars().for_each(collector);

        Ok(m)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let custom_env = HashMap::from([
        ("MYSQL_ID", "database.username"),
        ("MYSQL_PW", "database.password"),
        ("MYSQL_HOSTNAME", "database.hostname"),
        ("MYSQL_PORT", "database.port"),
        ("MYSQL_DB_NAME", "database.name"),
    ]);

    let settings = config::Config::builder()
        .set_default("database.hostname", "127.0.0.1")?
        .set_default("bind.addr", "127.0.0.1")?
        .set_default("bind.port", 8000_u16)?
        .add_source(config::File::new("config.toml", config::FileFormat::Toml))
        .add_source(CustomEnvironment::with_custom(custom_env))
        .build()?;
    settings.try_deserialize::<Settings>()
}
