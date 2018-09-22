use std::str::FromStr;
use ::serde::{self, Deserialize, Deserializer};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color(pub f64, pub f64, pub f64, pub f64);

impl Color {
    pub fn white() -> Self {
        Color(1.0, 1.0, 1.0, 1.0)
    }
}

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

#[cfg(tests)]
mod test {
    use super::Color;

    #[test]
    fn deserialize_colors() {
        for (s,          c) in vec![
            ( "#000000", Color(0.0,          0.0,           0.0,          1.0) ),
            ( "#1d1f21", Color(29.0 / 255.0, 31.0 / 255.0,  33.0 / 255.0, 1.0) ),
            ( "#ffaf00", Color(1.0,          175.0 / 255.0, 0.0,          1.0) ),
            ( "#ffffff", Color(1.0,          1.0,           1.0,          1.0) ),
        ] {
            assert_eq!(s.parse::<Color>().unwrap(), c);
        }
    }
}
