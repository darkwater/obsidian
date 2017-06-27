extern crate config as rsconfig;

use self::rsconfig::Value;
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

pub struct Config {
    pub colors: HashMap<String, Color>,
    pub mpd:    MpdConfig,
    pub launch: LaunchConfig,
}

pub struct MpdConfig {
    pub host: String,
    pub port: u16,
}

pub struct LaunchConfig {
    pub left:   Option<String>,
    pub middle: Option<String>,
    pub right:  Option<String>,
}

impl Config {
    pub fn default() -> Self {
        let mut config = rsconfig::Config::new();
        let mut config_path = PathBuf::from(env::var("HOME").unwrap_or(".".to_string()));
        config_path.push(".config/obsidian/config.toml");

        let _ = config.merge(rsconfig::File::from_str(include_str!("default_config.toml"), rsconfig::FileFormat::Toml)
                             .required(true));

        let _ = config.merge(rsconfig::File::new(config_path.to_str().unwrap(), rsconfig::FileFormat::Toml)
                             .required(false));

        Self::parse_rsconfig(config)
    }

    pub fn parse_rsconfig(config: rsconfig::Config) -> Self {
        let colors = config.get_table("colors").unwrap().into_iter().map(|(name, color)| {
            let errmsg = format!("invalid color {:?}", color);

            let color = color.into_str().unwrap();
            (name, color.parse().expect(&errmsg))
        }).collect();

        let mut mpd    = config.get_table("mpd").unwrap();
        let mut launch = config.get_table("launch").unwrap();

        Self {
            mpd: MpdConfig {
                host: mpd.remove("host").unwrap().try_into().unwrap(),
                port: mpd.remove("port").unwrap().try_into().unwrap(),
            },
            launch: LaunchConfig {
                left:   launch.remove("left").map(|c| c.try_into().unwrap()),
                middle: launch.remove("middle").map(|c| c.try_into().unwrap()),
                right:  launch.remove("right").map(|c| c.try_into().unwrap()),
            },
            colors: colors,
        }
    }

    pub fn get_color(&self, name: &str) -> Color {
        self.colors.get(name).expect(&format!("missing color: {}", name)).clone()
    }
}
