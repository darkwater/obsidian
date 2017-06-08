extern crate config as rsconfig;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default)]
pub struct Color(pub f64, pub f64, pub f64, pub f64);

impl FromStr for Color {
    type Err = &'static str;

    /// Parse a string of format #rrggbb
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err_msg = "invalid color";
        let mut iter = s.chars().into_iter().peekable();

        // Optional leading #
        if let Some(&'#') = iter.peek() {
            let _ = iter.next();
        }

        let red = iter.next().and_then(|d| d.to_digit(16)).ok_or(err_msg)? * 16
                + iter.next().and_then(|d| d.to_digit(16)).ok_or(err_msg)?;

        let green = iter.next().and_then(|d| d.to_digit(16)).ok_or(err_msg)? * 16
                  + iter.next().and_then(|d| d.to_digit(16)).ok_or(err_msg)?;

        let blue = iter.next().and_then(|d| d.to_digit(16)).ok_or(err_msg)? * 16
                 + iter.next().and_then(|d| d.to_digit(16)).ok_or(err_msg)?;

        if let Some(_) = iter.next() {
            return Err(err_msg);
        }

        let red   = red   as f64 / 255.0;
        let green = green as f64 / 255.0;
        let blue  = blue  as f64 / 255.0;

        Ok(Color(red, green, blue, 1.0))
    }
}

#[derive(Debug)]
pub struct Config {
    pub colors: HashMap<String, Color>,
}

impl Config {
    pub fn default() -> Self {
        let mut config_path = PathBuf::from(env::var("HOME").unwrap_or(".".to_string()));
        config_path.push(".config/obsidian/config.toml");

        let mut config = rsconfig::Config::new();
        let _ = config.merge(rsconfig::File::new(config_path.to_str().unwrap(), rsconfig::FileFormat::Toml).required(true));

        Self::parse_rsconfig(config)
    }

    pub fn parse_rsconfig(config: rsconfig::Config) -> Self {
        let colors = config.get_table("colors").expect("no colors defined").into_iter().map(|(name, color)| {
            let color = color.into_str().expect("invalid color");
            (name, color.parse().expect("invalid color"))
        }).collect();

        Self {
            colors: colors
        }
    }

    pub fn get_color(&self, name: &str) -> Color {
        self.colors.get(name).expect(&format!("missing color: {}", name)).clone()
    }
}
