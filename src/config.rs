extern crate config as rsconfig;

use std::cell::Cell;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use ::serde::{self, Deserialize, Deserializer};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(de: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(de)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub dpi:    Cell<f64>,
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
        (i.into() * self.dpi.get()) as i32
    }
}

mod test {
    use super::rsconfig;
    use super::Color;

    #[test]
    fn deserialize_colors() {
        for (s,          c) in vec![
            ( "#000000", Color(0.0,          0.0,           0.0,          1.0) ),
            ( "#1d1f21", Color(29.0 / 255.0, 31.0 / 255.0,  33.0 / 255.0, 1.0) ),
            ( "#ffaf00", Color(1.0,          175.0 / 255.0, 0.0,          1.0) ),
            ( "#ffffff", Color(1.0,          1.0,           1.0,          1.0) ),
        ] {
            assert_eq!(rsconfig::Value::from(s).try_into::<Color>().unwrap(), c);
        }
    }
}
