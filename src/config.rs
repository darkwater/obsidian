extern crate config as rsconfig;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use color::Color;

#[derive(Deserialize)]
pub struct Config {
    pub dpi:    f64,
    pub colors: HashMap<String, Color>,
    pub mpd:    MpdConfig,
}

#[derive(Deserialize)]
pub struct MpdConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn default() -> Config {
        let mut config_path = PathBuf::from(env::var("HOME").unwrap_or(".".to_string()));
        config_path.push(".config/obsidian/config.toml");

        let mut config = rsconfig::Config::default();
        config
            .merge(rsconfig::File::from_str(include_str!("default_config.toml"), rsconfig::FileFormat::Toml)).unwrap()
            .merge(rsconfig::File::new(config_path.to_str().unwrap(),            rsconfig::FileFormat::Toml)).unwrap()
            .merge(rsconfig::Environment::with_prefix("OBSIDIAN")).unwrap();

        config.try_into().unwrap()
    }

    pub fn get_color(&self, name: &str) -> Color {
        self.colors.get(name).expect(&format!("missing color: {}", name)).clone()
    }

    pub fn dpi_scale<In>(&self, i: In) -> i32
    where
        In: Into<f64>,
    {
        (i.into() * self.dpi) as i32
    }
}
